use crate::error::{Result, Error};
use log::{debug, error};
use serde::{Serialize, Deserialize};
use lazy_static::lazy_static;
use prometheus::{
    register_counter_vec, register_histogram, register_histogram_vec, register_int_counter, register_int_gauge,
    CounterVec, Histogram, HistogramVec, IntCounter, IntGauge
};

type Pool = sqlx::postgres::PgPool;
type PoolOptions = sqlx::postgres::PgPoolOptions;

type DateTime = chrono::DateTime<chrono::Utc>;

lazy_static! {
    static ref REQUEST_COUNTER: CounterVec = register_counter_vec!(
        "tantivyrest_request_total",
        "Total number of requests",
        &["func"]
        ).unwrap();
    static ref REQUEST_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "tantivyrest_request_duration_seconds",
        "The request latencies in seconds.",
        &["func"]
        ).unwrap();
    static ref COMMIT_HISTOGRAM: Histogram = register_histogram!(
        "tantivyrest_commit_duration_seconds",
        "The commit latencies in seconds."
        ).unwrap();
    static ref COMMIT_COUNTER: IntCounter = register_int_counter!(
        "tantivyrest_commit_total", 
        "Total number of commits"
        ).unwrap();
    static ref DOCUMENT_GAUGE: IntGauge = register_int_gauge!(
        "tantivyrest_document_number", 
        "Total number of commits"
        ).unwrap();
    static ref SEGMENT_GAUGE: IntGauge = register_int_gauge!(
        "tantivyrest_segment_number", 
        "Total number of commits"
        ).unwrap();
}


#[derive(sqlx::FromRow, Debug, Deserialize, Serialize, PartialEq)]
pub struct Document {
    pub gallery_id: String,
    pub id: i32,
    pub title: String,
    pub subject: Option<String>,
    pub author_nickname: String,
    pub author_ip: Option<String>,
    pub author_id: Option<String>,
    pub comment_count: i32,
    pub like_count: i32,
    pub view_count: i32,
    pub kind: String,
    pub is_recommend: bool,
    pub created_at: DateTime,
}

#[derive(Clone)]
pub struct State {
    pool: Pool,
}

impl State {
    pub async fn migrate(db_url: &str) -> Result<()> {
        let pool = PoolOptions::new().connect(db_url).await?;
        let mut migrator = sqlx::migrate!();
        migrator.migrations.to_mut().retain(|migration| !migration.description.ends_with(".down"));
        migrator.run(&pool).await?;
        Ok(())
    }
    pub async fn connect(db_url: &str) -> Result<Self> {
        let pool = PoolOptions::new().connect(db_url).await?;
        Ok(State{
            pool,
            })
    }
    pub async fn health(&self) -> Result<()> {
        let _ = sqlx::query("SELECT 1").fetch_one(&self.pool).await?;
        Ok(())
    }
    pub async fn search_docs_by_title(&self, title: &str, last_created_at: Option<DateTime>) -> Result<Vec<Document>> {
        debug!("search_by_title: title={:?}", title);
        REQUEST_COUNTER.with_label_values(&["search_by_title"]).inc();
        let timer = REQUEST_HISTOGRAM
            .with_label_values(&["search_docs_by_title"])
            .start_timer();
        let res = sqlx::query_as::<_, Document>("SELECT * FROM users WHERE _title @@ ? ORDER BY created_at <=| ? LIMIT 100")
            .bind(title)
            .bind(last_created_at.unwrap_or(chrono::Utc::now()))
            .fetch_all(&self.pool)
            .await?;
        timer.observe_duration();
        Ok(res)
    }
    pub async fn search_docs_by_gallery_and_title(&self, gallery: &str, title: &str, last_created_at: Option<DateTime>) -> Result<Vec<Document>> {
        debug!("search_by_gallery_and_title: gallery={:?} title={:?}", gallery, title);
        REQUEST_COUNTER.with_label_values(&["search_by_gallery_and_title"]).inc();
        let timer = REQUEST_HISTOGRAM
            .with_label_values(&["search_docs_by_title"])
            .start_timer();
        let res = sqlx::query_as::<_, Document>("SELECT * FROM users WHERE _title @@ ? ORDER BY _gallery_id_and_created_at <=| cat_and_time(?, ?) LIMIT 100")
            .bind(title)
            .bind(gallery)
            .bind(chrono::Utc::now())
            .fetch_all(&self.pool)
            .await?;
        timer.observe_duration();
        Ok(res)
    }
}
