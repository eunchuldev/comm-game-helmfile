mod error;

use serde::{Serialize, Deserialize};
use juniper::{graphql_object, EmptyMutation, EmptySubscription, GraphQLObject, RootNode};

pub use error::Error;

type Pool = sqlx::postgres::PgPool;
type PoolOptions = sqlx::postgres::PgPoolOptions;

type DateTime = chrono::DateTime<chrono::Utc>;

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize, PartialEq, GraphQLObject)]
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
    pub async fn migrate(db_url: &str) -> Result<(), Error> {
        let pool = PoolOptions::new().connect(db_url).await?;
        let mut migrator = sqlx::migrate!();
        migrator.migrations.to_mut().retain(|migration| !migration.description.ends_with(".down"));
        migrator.run(&pool).await?;
        Ok(())
    }
    pub async fn connect(db_url: &str) -> Result<Self, Error> {
        let pool = PoolOptions::new().connect(db_url).await?;
        Ok(State{
            pool,
            })
    }
    pub async fn health(&self) -> Result<(), Error> {
        let _ = sqlx::query("SELECT 1").fetch_one(&self.pool).await?;
        Ok(())
    }
    pub async fn search_docs_by_title(&self, title: &str, last_created_at: Option<DateTime>) -> Result<Vec<Document>, Error> {
        let res = sqlx::query_as::<_, Document>("SELECT * FROM users WHERE _title @@ ? ORDER BY created_at <=| ? LIMIT 100")
            .bind(title)
            .bind(last_created_at.unwrap_or(chrono::Utc::now()))
            .fetch_all(&self.pool)
            .await?;
        Ok(res)
    }
    pub async fn search_docs_by_gallery_and_title(&self, gallery: &str, title: &str, last_created_at: Option<DateTime>) -> Result<Vec<Document>, Error> {
        let res = sqlx::query_as::<_, Document>("SELECT * FROM users WHERE _title @@ ? ORDER BY _gallery_id_and_created_at <=| cat_and_time(?, ?) LIMIT 100")
            .bind(title)
            .bind(gallery)
            .bind(last_created_at.unwrap_or(chrono::Utc::now()))
            .fetch_all(&self.pool)
            .await?;
        Ok(res)
    }
}

impl juniper::Context for State {}


pub struct Query;

#[graphql_object(context = State)]
impl Query {
    fn apiVersion() -> String {
        "0.1".to_string()
    }
    async fn documents(
        state: &State, 
        gallery_id: Option<String>,
        title: Option<String>,
        author_nickname: Option<String>,
        author_ip: Option<String>,
        author_id: Option<String>,
        last_created_at: Option<DateTime>,
        ) -> Result<Vec<Document>, Error> {
        Ok(match (gallery_id, title) {
            (Some(gallery_id), Some(title)) => state.search_docs_by_gallery_and_title(&gallery_id, &title, last_created_at).await?,
            (None, Some(title)) => state.search_docs_by_title(&title, last_created_at).await?,
            _ => return Err(Error::BadRequest("documents", "valid form: { [gallery_id,] [last_created_at,] ONE_OF(title, author_nickname, authir_ip, author_id)")),
        })
    }
}

pub type Schema = RootNode<'static, Query, EmptyMutation<State>, EmptySubscription<State>>;

pub fn schema() -> Schema {
    Schema::new(
        Query,
        EmptyMutation::<State>::new(),
        EmptySubscription::<State>::new(),
    )
}
