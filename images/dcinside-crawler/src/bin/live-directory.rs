#![feature(never_type)]

use actix_web::{
    error::ResponseError, get, http::StatusCode, post, web, App, HttpResponse, HttpServer,
    Responder,
};
use chrono::Utc;
use std::time::Duration;

use dcinside_crawler::error::*;
use err_derive::Error;
use std::convert::TryInto;

use dcinside_crawler::crawler::Crawler;
use dcinside_crawler::model::*;

use serde::Deserialize;

use actix_web_prom::PrometheusMetrics;
use prometheus::{labels, opts, Histogram, HistogramOpts, IntCounterVec, IntGauge};

use log::{error, info, warn};

#[derive(Error, Debug)]
pub enum LiveDirectoryError {
    #[error(display = "crawler error")]
    Crawler(#[source] CrawlerError),
    #[error(display = "sled")]
    Sled(#[source] sled::Error),
    #[error(display = "not found")]
    NotFound,
}
impl ResponseError for LiveDirectoryError {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::NotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        HttpResponse::build(status_code).body(self.to_string())
    }
}

use std::hash::{Hash, Hasher};
fn hash<T>(obj: T) -> u64
where
    T: Hash,
{
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    obj.hash(&mut hasher);
    hasher.finish()
}

#[derive(Clone)]
struct State {
    crawler: Crawler,
    gallery_db: sled::Tree,
    gallery_kind: GalleryKind,
    metrics: Metrics,
    target_docs_count_per_crawl: usize,
    min_wait_seconds_per_gallery: usize,
    publish_duration_estimate_weight1: f64,
    publish_duration_estimate_weight2: f64,
}

impl State {
    fn new(gallery_kind: GalleryKind, metrics: Metrics) -> Self {
        let config = sled::Config::new().temporary(true);
        let db = config.open().unwrap().open_tree("galleries").unwrap();
        State {
            crawler: Crawler::new(),
            gallery_db: db,
            gallery_kind,
            metrics,
            target_docs_count_per_crawl: 1,
            min_wait_seconds_per_gallery: 3600 * 3,
            publish_duration_estimate_weight1: 0.0999,
            publish_duration_estimate_weight2: 0.0001,
        }
    }
    fn docs_per_crawl(mut self, v: usize) -> Self {
        self.target_docs_count_per_crawl = v;
        self
    }
    fn min_wait_seconds(mut self, v: usize) -> Self {
        self.min_wait_seconds_per_gallery = v;
        self
    }
    fn pub_dur_estimate_weight1(mut self, v: f64) -> Self {
        self.publish_duration_estimate_weight1 = v;
        self
    }
    fn pub_dur_estimate_weight2(mut self, v: f64) -> Self {
        self.publish_duration_estimate_weight2 = v;
        self
    }
    fn with_db(db: sled::Tree, gallery_kind: GalleryKind, metrics: Metrics) -> Self {
        State {
            crawler: Crawler::new(),
            gallery_db: db,
            gallery_kind,
            metrics,
            target_docs_count_per_crawl: 1,
            min_wait_seconds_per_gallery: 3600 * 3,
            publish_duration_estimate_weight1: 0.0999,
            publish_duration_estimate_weight2: 0.0001,
        }
    }
    async fn update(&self) -> Result<(), LiveDirectoryError> {
        let now = Utc::now();
        let hot_galleries = match self.gallery_kind {
            GalleryKind::Major => self.crawler.realtime_hot_galleries().await?,
            GalleryKind::Minor => self.crawler.realtime_hot_minor_galleries().await?,
            GalleryKind::Mini => panic!("mini gallery kind not supported yet"),
        };
        for index in hot_galleries {
            let new_state = GalleryState {
                index,
                last_ranked: now,
                last_crawled_at: None,
                last_published_at: None,
                last_crawled_document_id: None,
                visible: true,
                last_error: None,
                publish_duration_in_seconds: Some(0.0),
                registered_at: Some(Utc::now()),
            };
            self.gallery_db.fetch_and_update(
                new_state.index.id.clone().as_bytes(),
                move |old| {
                    Some(match old {
                        Some(bytes) => {
                            let new_index = new_state.index.clone();
                            let mut old_state = serde_json::from_slice::<GalleryState>(bytes)
                                .unwrap_or_else(|e| {
                                    error!(
                                        "fail to parse sled tree data {} with error {}",
                                        &new_index.id, e
                                    );
                                    new_state.clone()
                                });
                            old_state.last_ranked = now;
                            old_state.index = new_index;
                            old_state.visible = true;
                            old_state.last_published_at = None;
                            old_state.publish_duration_in_seconds = None;
                            serde_json::to_vec(&old_state).unwrap()
                        }
                        None => serde_json::to_vec(&new_state).unwrap(),
                    })
                },
            )?;
        }
        let weekly_hot_galleries = match self.gallery_kind {
            GalleryKind::Major => self.crawler.weekly_hot_galleries().await?,
            GalleryKind::Minor => Vec::new(),
            GalleryKind::Mini => panic!("mini gallery kind not supported yet"),
        };
        for index in weekly_hot_galleries {
            self.gallery_db
                .fetch_and_update(index.id.clone().as_bytes(), move |old| {
                    Some(match old {
                        Some(bytes) => bytes.to_vec(),
                        None => {
                            let new_state = GalleryState {
                                index: index.clone(),
                                last_ranked: now,
                                last_crawled_at: None,
                                last_published_at: None,
                                last_crawled_document_id: None,
                                visible: true,
                                last_error: None,
                                publish_duration_in_seconds: Some(0.0),
                                registered_at: Some(Utc::now()),
                            };
                            serde_json::to_vec(&new_state).unwrap()
                        }
                    })
                })?;
        }
        self.metrics
            .gallery_total
            .set(self.gallery_db.len().try_into().unwrap());
        Ok(())
    }
    fn upgrade_db(gallery_db: sled::Tree) -> Result<(), LiveDirectoryError> {
        let keys: Vec<_> = gallery_db
            .iter()
            .filter_map(|res| {
                if res.is_err() {
                    error!("fail to iterate over sled");
                }
                res.ok()
            })
            .map(|(k, _)| k)
            .collect();
        for k in keys {
            gallery_db.fetch_and_update(k, |old| {
                old.map(|bytes| {
                    serde_json::from_slice::<GalleryState>(bytes)
                        .map(|mut old_state| {
                            old_state.registered_at =
                                Some(old_state.registered_at.unwrap_or_else(Utc::now));
                            serde_json::to_vec(&old_state).unwrap()
                        })
                        .unwrap()
                })
            })?;
        }
        info!("db upgrade done");
        Ok(())
    }
    fn estimate_publish_duration(
        &self,
        last_crawled_at: Option<chrono::DateTime<Utc>>,
        crawled_document_count: usize,
        state: &GalleryState,
    ) -> f64 {
        (1.0 - self.publish_duration_estimate_weight1 - self.publish_duration_estimate_weight2)
            * state.publish_duration_in_seconds.unwrap_or(0.0)
            + match (
                last_crawled_at,
                state.last_published_at.or(state.registered_at),
            ) {
                (Some(n), Some(o)) => {
                    self.publish_duration_estimate_weight1
                        * ((n.signed_duration_since(o).num_seconds() as f64)
                            / (crawled_document_count as f64))
                            .min(3600.0)
                        + self.publish_duration_estimate_weight2
                            * ((n.signed_duration_since(o).num_seconds() as f64)
                                / (crawled_document_count as f64))
                                .min(3600.0 * 24.0)
                }
                (Some(_n), _) => 0.0f64,
                _ => 0.0f64,
            }
    }
    fn report(&self, form: GalleryCrawlReportForm) -> Result<(), LiveDirectoryError> {
        let mut found = false;
        self.metrics
            .worker_report_success_total
            .with_label_values(&[
                self.gallery_kind.into(),
                form.worker_part.to_string().as_str(),
            ])
            .inc();
        self.metrics
            .crawled_document_count_histogram
            .observe(form.crawled_document_count as f64);
        if form.crawled_document_count >= 500 {
            match self.gallery_db.get(form.id.as_bytes()) {
                Ok(Some(bytes)) => match serde_json::from_slice::<GalleryState>(&bytes) {
                    Ok(state) => {
                        warn!(
                                "[{} gallery] too many crawled documents: `{}` documents crawled. last published at `{}`, publish duration: `{}` secs", 
                                form.id,
                                form.crawled_document_count,
                                state.last_published_at.map(|t| t.to_string()).unwrap_or_else(|| "None".to_string()),
                                state.publish_duration_in_seconds.unwrap_or(0.0),
                                );
                    }
                    _ => {
                        warn!(
                                "[{} gallery] too many crawled documents: `{}` documents crawled. Fail to parse saved state(It's wiered!)", 
                                form.id,
                                form.crawled_document_count,
                                );
                    }
                },
                _ => {
                    warn!(
                        "[{} gallery] too many crawled documents: `{}` documents crawled. No saved states(It's wiered!)", 
                        form.id,
                        form.crawled_document_count,
                        );
                }
            }
        }
        self.gallery_db
            .fetch_and_update(form.id.as_bytes(), |old| match old {
                Some(bytes) => {
                    found = true;
                    serde_json::from_slice::<GalleryState>(bytes)
                        .map(|mut old_state| {
                            old_state.publish_duration_in_seconds =
                                Some(self.estimate_publish_duration(
                                    form.last_crawled_at,
                                    form.crawled_document_count,
                                    &old_state,
                                ));
                            if form.crawled_document_count > 0 {
                                old_state.last_published_at = form.last_crawled_at;
                            }
                            old_state.last_crawled_at = form.last_crawled_at;
                            old_state.last_crawled_document_id = form.last_crawled_document_id;
                            serde_json::to_vec(&old_state).unwrap()
                        })
                        .ok()
                }
                None => None,
            })?;
        if found {
            Ok(())
        } else {
            Err(LiveDirectoryError::NotFound)
        }
    }
    fn error_report(&self, form: GalleryCrawlErrorReportForm) -> Result<(), LiveDirectoryError> {
        let mut found = false;
        self.metrics
            .worker_report_error_total
            .with_label_values(&[
                self.gallery_kind.into(),
                form.worker_part.to_string().as_str(),
            ])
            .inc();
        if let CrawlerErrorReport::Unknown = form.error {
            warn!(
                "Unknown error reported from `{}` gallery at worker `{}`",
                form.id, form.worker_part
            )
        };
        self.gallery_db
            .fetch_and_update(form.id.as_bytes(), |old| match old {
                Some(bytes) => {
                    found = true;
                    serde_json::from_slice::<GalleryState>(bytes)
                        .map(|mut old_state| {
                            old_state.last_error = Some(form.error.clone());
                            old_state.last_crawled_at = form.last_crawled_at;
                            old_state.publish_duration_in_seconds = Some(
                                self.estimate_publish_duration(form.last_crawled_at, 0, &old_state),
                            );
                            old_state.visible = !matches!(
                                form.error,
                                CrawlerErrorReport::PageNotFound
                                    | CrawlerErrorReport::MinorGalleryClosed
                                    | CrawlerErrorReport::MinorGalleryPromoted
                                    | CrawlerErrorReport::AdultPage
                            );
                            serde_json::to_vec(&old_state).unwrap()
                        })
                        .ok()
                }
                None => None,
            })?;
        if found {
            Ok(())
        } else {
            Err(LiveDirectoryError::NotFound)
        }
    }

    fn list_part(&self, total: u64, part: u64) -> Vec<GalleryState> {
        let now = Utc::now();
        self.gallery_db
            .iter()
            .filter_map(|res| {
                if res.is_err() {
                    error!("fail to iterate over sled");
                }
                res.ok()
            })
            .filter(|(id, _)| hash(id) % total == part)
            .filter_map(|(_, state)| {
                let res = serde_json::from_slice::<GalleryState>(&state);
                if res.is_err() {
                    error!("fail to parse value during iterate over sled");
                }
                match res {
                    Ok(v) if v.visible => match v.last_published_at.or(v.registered_at) {
                        Some(t) => {
                            let duration_from_last_publish =
                                now.signed_duration_since(t).num_seconds() as f64;
                            let wait_time = (v.publish_duration_in_seconds.unwrap_or(0.0)
                                * self.target_docs_count_per_crawl as f64)
                                .min(self.min_wait_seconds_per_gallery as f64);
                            self.metrics.crawl_waittime_histogram.observe(wait_time);
                            if duration_from_last_publish >= wait_time {
                                Some(v)
                            } else {
                                None
                            }
                        }
                        None => {
                            self.metrics.crawl_waittime_histogram.observe(0.0);
                            Some(v)
                        }
                    },
                    _ => None,
                }
            })
            .collect()
    }
}

async fn update_forever(state: State, delay: Duration) -> Result<!, LiveDirectoryError> {
    info!("start update live directory");
    loop {
        state.update().await?;
        info!(
            "update live directory done. wait {} seconds..",
            delay.as_secs()
        );
        actix::clock::delay_for(delay).await;
    }
}

#[get("/health")]
async fn health() -> impl Responder {
    "ok"
}

#[derive(Deserialize)]
pub struct ListPartQuery {
    part: u64,
    total: u64,
}
#[get("/list")]
async fn list_part(
    web::Query(query): web::Query<ListPartQuery>,
    state: web::Data<State>,
) -> impl Responder {
    web::Json(state.list_part(query.total, query.part))
}

#[post("/report")]
async fn report(
    web::Json(form): web::Json<GalleryCrawlReportForm>,
    state: web::Data<State>,
) -> Result<HttpResponse, LiveDirectoryError> {
    let state = state;
    state.report(form)?;
    Ok(HttpResponse::Ok().finish())
}

#[post("/error-report")]
async fn error_report(
    web::Json(form): web::Json<GalleryCrawlErrorReportForm>,
    state: web::Data<State>,
) -> Result<HttpResponse, LiveDirectoryError> {
    let state = state;
    state.error_report(form)?;
    Ok(HttpResponse::Ok().finish())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(health)
        .service(list_part)
        .service(report)
        .service(error_report);
}

#[derive(Clone)]
struct Metrics {
    gallery_total: IntGauge,
    worker_report_success_total: IntCounterVec,
    worker_report_error_total: IntCounterVec,
    crawl_waittime_histogram: Histogram,
    crawled_document_count_histogram: Histogram,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            gallery_total: IntGauge::new("dccrawler_gallery_total", "dccrawler_gallery_total")
                .unwrap(),
            worker_report_success_total: IntCounterVec::new(
                opts!(
                    "dccrawler_worker_report_success_total",
                    "dccrawler_worker_report_success_total"
                ),
                &["gallery_kind", "part"],
            )
            .unwrap(),
            worker_report_error_total: IntCounterVec::new(
                opts!(
                    "dccrawler_worker_report_error_total",
                    "dccrawler_worker_report_error_total"
                ),
                &["gallery_kind", "part"],
            )
            .unwrap(),
            crawl_waittime_histogram: Histogram::with_opts(HistogramOpts::new(
                "dccrawler_crawl_waittime_histogram",
                "dccrawler_crawl_waittime_histogram",
            ))
            .unwrap(),
            crawled_document_count_histogram: Histogram::with_opts(HistogramOpts::new(
                "dccrawler_crawled_document_count_histogram",
                "dccrawler_crawled_document_count_histogram",
            ))
            .unwrap(),
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let store_path = std::env::var("STORE_PATH").unwrap_or_else(|_| "".to_string());
    let _total_worker_count: u64 = std::env::var("TOTAL_WORKER_COUNT")
        .unwrap_or_else(|_| "30".to_string())
        .parse()
        .unwrap();
    let gallery_kind: GalleryKind = std::env::var("GALLERY_KIND")
        .unwrap_or_else(|_| "major".to_string())
        .into();
    let docs_per_crawl: usize = std::env::var("DOCS_PER_CRAWL")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .unwrap();
    let min_wait_seconds: usize = std::env::var("MIN_WAIT_SECONDS")
        .unwrap_or_else(|_| "10800".to_string())
        .parse()
        .unwrap();
    let pub_dur_estimate_weight1: f64 = std::env::var("PUB_DUR_ESTIMATE_WEIGHT1")
        .unwrap_or_else(|_| "0.0999".to_string())
        .parse()
        .unwrap();
    let pub_dur_estimate_weight2: f64 = std::env::var("PUB_DUR_ESTIMATE_WEIGHT2")
        .unwrap_or_else(|_| "0.0001".to_string())
        .parse()
        .unwrap();

    let prometheus = PrometheusMetrics::new(
        "dccrawler",
        Some("/metrics"),
        Some(labels! { "gallery_kind".to_string() => <&str>::from(gallery_kind).to_string() }),
    );
    let metrics = Metrics {
        gallery_total: IntGauge::with_opts(opts!(
            "dccrawler_gallery_total",
            "dccrawler_gallery_total",
            labels! { "gallery_kind" => <&str>::from(gallery_kind) }
        ))
        .unwrap(),
        worker_report_success_total: IntCounterVec::new(
            opts!(
                "dccrawler_worker_report_success_total",
                "dccrawler_worker_report_success_total"
            ),
            &["gallery_kind", "part"],
        )
        .unwrap(),
        worker_report_error_total: IntCounterVec::new(
            opts!(
                "dccrawler_worker_report_error_total",
                "dccrawler_worker_report_error_total"
            ),
            &["gallery_kind", "part"],
        )
        .unwrap(),
        crawl_waittime_histogram: Histogram::with_opts(
            HistogramOpts::new(
                "dccrawler_crawl_waittime_histogram",
                "dccrawler_crawl_waittime_histogram",
            )
            .const_label("gallery_kind", <&str>::from(gallery_kind))
            .buckets(vec![
                60.0,
                300.0,
                1800.0,
                3600.0,
                3600.0 * 12.0,
                3600.0 * 24.0,
            ]),
        )
        .unwrap(),
        crawled_document_count_histogram: Histogram::with_opts(
            HistogramOpts::new(
                "dccrawler_crawled_document_count_histogram",
                "dccrawler_crawled_document_count_histogram",
            )
            .const_label("gallery_kind", <&str>::from(gallery_kind))
            .buckets(vec![0.0, 5.0, 10.0, 30.0, 100.0, 500.0, 1000.0, 10000.0]),
        )
        .unwrap(),
    };

    let reg = prometheus.clone().registry;
    reg.register(Box::new(metrics.gallery_total.clone()))
        .unwrap();
    reg.register(Box::new(metrics.worker_report_error_total.clone()))
        .unwrap();
    reg.register(Box::new(metrics.worker_report_success_total.clone()))
        .unwrap();
    reg.register(Box::new(metrics.crawl_waittime_histogram.clone()))
        .unwrap();
    reg.register(Box::new(metrics.crawled_document_count_histogram.clone()))
        .unwrap();

    let db = if store_path.is_empty() {
        let config = sled::Config::new().temporary(true);
        config.open().unwrap()
    } else {
        sled::open(store_path).unwrap()
    };
    let db = db.open_tree("galleries").unwrap();

    let _metrics = metrics.clone();
    let db2 = db.clone();
    State::upgrade_db(db2.clone()).unwrap();
    actix_rt::spawn(async move {
        loop {
            let state = State::with_db(db2.clone(), gallery_kind, _metrics.clone())
                .docs_per_crawl(docs_per_crawl)
                .min_wait_seconds(min_wait_seconds)
                .pub_dur_estimate_weight1(pub_dur_estimate_weight1)
                .pub_dur_estimate_weight2(pub_dur_estimate_weight2);
            let res = update_forever(state, Duration::from_secs(60)).await;
            if let Err(e) = res {
                error!("updator restart due to: {}", e.to_string());
            }
        }
    });
    HttpServer::new(move || {
        let state = State::with_db(db.clone(), gallery_kind, metrics.clone())
            .docs_per_crawl(docs_per_crawl)
            .min_wait_seconds(min_wait_seconds)
            .pub_dur_estimate_weight1(pub_dur_estimate_weight1)
            .pub_dur_estimate_weight2(pub_dur_estimate_weight2);
        App::new()
            .wrap(prometheus.clone())
            .app_data(web::Data::new(state))
            .configure(config)
    })
    .bind(format!("0.0.0.0:{}", port))?
    .workers(1)
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http, test, App};

    #[actix_rt::test]
    async fn state_update_minor_list_part() {
        let state = State::new(GalleryKind::Minor, Metrics::default());
        state.update().await.unwrap();
        let res1 = state.list_part(2, 0);
        let res2 = state.list_part(2, 1);
        assert!(!res1.is_empty());
        assert!(!res2.is_empty());
        let mut h = std::collections::HashSet::new();
        for t in res1.iter() {
            h.insert(t.index.id.clone());
        }
        for t in res2.iter() {
            h.insert(t.index.id.clone());
        }
        assert_eq!(h.len(), res1.len() + res2.len());
        assert_eq!(
            state.metrics.gallery_total.get() as usize,
            res1.len() + res2.len()
        );
    }
    #[actix_rt::test]
    async fn state_update_list_part() {
        let state = State::new(GalleryKind::Major, Metrics::default());
        state.update().await.unwrap();
        let res1 = state.list_part(2, 0);
        let res2 = state.list_part(2, 1);
        assert!(!res1.is_empty());
        assert!(!res2.is_empty());
        let mut h = std::collections::HashSet::new();
        for t in res1.iter() {
            h.insert(t.index.id.clone());
        }
        for t in res2.iter() {
            h.insert(t.index.id.clone());
        }
        assert!(h.len() > 70);
        assert_eq!(h.len(), res1.len() + res2.len());
        assert_eq!(
            state.metrics.gallery_total.get() as usize,
            res1.len() + res2.len()
        );
    }
    #[actix_rt::test]
    async fn state_report() {
        let state = State::new(GalleryKind::Major, Metrics::default());
        state.update().await.unwrap();
        let res1 = state.list_part(2, 0);
        assert!(res1[1].last_crawled_at.is_none());
        let now = Utc::now();
        state
            .report(GalleryCrawlReportForm {
                worker_part: 1u64,
                id: res1[0].index.id.clone(),
                last_crawled_at: Some(now),
                last_crawled_document_id: Some(1),
                crawled_document_count: 1usize,
            })
            .unwrap();
        let res2 = state.list_part(2, 0);
        assert_eq!(res2[0].last_crawled_at, Some(now));
        assert_eq!(res2[0].last_crawled_document_id, Some(1));
        assert_eq!(
            state
                .metrics
                .worker_report_success_total
                .with_label_values(&["major", "1"])
                .get(),
            1
        );
    }
    #[actix_rt::test]
    async fn state_error_report() {
        let state = State::new(GalleryKind::Major, Metrics::default());
        state.update().await.unwrap();
        let res1 = state.list_part(2, 1);
        assert!(res1[0].last_crawled_at.is_none());
        let now = Utc::now();
        state
            .error_report(GalleryCrawlErrorReportForm {
                worker_part: 1u64,
                id: res1[0].index.id.clone(),
                last_crawled_at: Some(now),
                error: CrawlerErrorReport::MinorGalleryClosed,
            })
            .unwrap();
        let res2 = state.list_part(2, 1);
        assert_ne!(res1.len(), res2.len());
        assert_ne!(res1[0].index.id, res2[0].index.id);
        assert_eq!(
            state
                .metrics
                .worker_report_error_total
                .with_label_values(&["major", "1"])
                .get(),
            1
        );
    }

    #[actix_rt::test]
    async fn test_health() {
        let mut app = test::init_service(App::new().configure(config)).await;
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
    }
}
