use actix_web::{get, http::StatusCode, web, App, HttpServer, Responder};
use std::time::Duration;

use dcinside_crawler::error::*;
use err_derive::Error;

use std::convert::TryInto;

use dcinside_crawler::crawler::Crawler;
use dcinside_crawler::model::*;

use serde::Serialize;

use actix_web_prom::PrometheusMetrics;
use prometheus::IntGauge;

use log::{error, info};

use actix_web::client::{PayloadError, SendRequestError};

#[derive(Error, Debug)]
pub enum WorkerError {
    #[error(display = "crawler error")]
    Crawler(#[source] CrawlerError),
    #[error(display = "actix client send")]
    SendRequest(#[source] SendRequestError),
    #[error(display = "acitx client payload")]
    Payload(#[source] PayloadError),
    #[error(display = "serde")]
    Serde(#[source] serde_json::Error),
    #[error(display = "err http response: {}", _0)]
    Response(StatusCode),
    #[error(display = "nats connect error: {}", _0)]
    NatsConnect(std::io::Error),
    #[error(display = "nats publish error: {}", _0)]
    NatsPublish(std::io::Error),
    #[error(display = "bincode: {}", _0)]
    Bincode(#[source] bincode::Error),
}

#[derive(Serialize)]
pub struct ListPartQuery {
    part: u64,
    total: u64,
}

#[derive(Clone)]
struct State {
    crawler: Crawler,
    nats_conn: nats::Connection,
    nats_subject: String,
    live_directory_url: String,
    data_broker_url: String,
    part: u64,
    total: u64,
    start_page: usize,
}

#[derive(Default)]
struct ResultMetric {
    gallery_success: usize,
    document_success: usize,
    comment_success: usize,
    gallery_error: usize,
    document_error: usize,
    comment_error: usize,
}
impl State {
    fn new(
        live_directory_url: &str,
        data_broker_url: &str,
        nats_url: &str,
        nats_subject: String,
        total: u64,
        part: u64,
    ) -> Result<Self, WorkerError> {
        Ok(State {
            crawler: Crawler::new(),
            live_directory_url: live_directory_url.to_string(),
            nats_subject,
            nats_conn: nats::connect(nats_url).map_err(WorkerError::NatsConnect)?,
            data_broker_url: data_broker_url.to_string(),
            total,
            part,
            start_page: 2,
        })
    }
    fn with_crawler_delay(mut self, v: u64) -> Self {
        self.crawler = self.crawler.delay(v);
        self
    }
    async fn fetch_gallery_list(&self) -> Result<Vec<GalleryState>, WorkerError> {
        let bytes = self
            .crawler
            .client
            .get(format!("{}/list", self.live_directory_url))
            .query(&ListPartQuery {
                total: self.total,
                part: self.part,
            })
            .unwrap()
            .send()
            .await?
            .body()
            .limit(1024 * 1024 * 8)
            .await?;
        Ok(serde_json::from_slice(&bytes)?)
    }
    async fn error_report(&self, form: GalleryCrawlErrorReportForm) -> Result<(), WorkerError> {
        let res = self
            .crawler
            .client
            .post(format!("{}/error-report", self.live_directory_url))
            .send_json(&form)
            .await?;
        if res.status() == StatusCode::OK {
            Ok(())
        } else {
            Err(WorkerError::Response(res.status()))
        }
    }
    async fn report_success(&self, form: GalleryCrawlReportForm) -> Result<(), WorkerError> {
        let res = self
            .crawler
            .client
            .post(format!("{}/report", self.live_directory_url))
            .send_json(&form)
            .await?;
        if res.status() == StatusCode::OK {
            Ok(())
        } else {
            Err(WorkerError::Response(res.status()))
        }
    }
    async fn send_data(&self, data: &Document) -> Result<(), WorkerError> {
        let res = self
            .crawler
            .client
            .post(&self.data_broker_url)
            .send_json(data)
            .await?;
        let nats_res = self
            .nats_conn
            .publish(&self.nats_subject, &bincode::serialize(&data)?);
        if let Err(e) = nats_res {
            error!("nats publish fail due to: {}", e.to_string());
        }
        if res.status() == StatusCode::OK {
            Ok(())
        } else {
            Err(WorkerError::Response(res.status()))
        }
    }
    async fn run(&mut self) -> Result<ResultMetric, WorkerError> {
        let mut gallery_states = self.fetch_gallery_list().await?;
        let mut metric = ResultMetric::default();
        let len = gallery_states.len();
        gallery_states.sort_by(|a, b| match (a.last_crawled_at, b.last_crawled_at) {
            (Some(a), Some(b)) => a.cmp(&b),
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
        for (i, gallery_state) in gallery_states.into_iter().enumerate() {
            info!(
                "{}/{} start | {}(last crawled at {:?})",
                i, len, gallery_state.index.id, gallery_state.last_crawled_at
            );
            let now = chrono::Utc::now();
            let res = match gallery_state.last_crawled_document_id {
                Some(last_crawled_document_id) if last_crawled_document_id > 0 => {
                    self.crawler
                        .documents_after(
                            &gallery_state.index,
                            last_crawled_document_id,
                            self.start_page,
                        )
                        .await
                }
                _ => {
                    self.crawler
                        .documents(&gallery_state.index, self.start_page)
                        .await
                }
            };
            match &res {
                Ok(res) => {
                    info!("crawled documents: {}", res.len());
                    metric.gallery_success += 1;
                    let mut last_document_id =
                        gallery_state.last_crawled_document_id.unwrap_or(0usize);
                    for r in res {
                        match r {
                            Ok(doc) => {
                                metric.document_success += 1;
                                metric.comment_success += 1;
                                if last_document_id < doc.id {
                                    last_document_id = doc.id;
                                }
                                if let Err(e) = self.send_data(doc).await {
                                    error!("error while send data: {}", e.to_string());
                                }
                            }
                            Err(CrawlerError::DocumentParseError(err)) => {
                                error!(
                                    "document parse error of {}: {}",
                                    &gallery_state.index.id,
                                    err.to_string()
                                );
                                metric.document_error += 1;
                            }
                            Err(CrawlerError::CommentParseError(err)) => {
                                error!(
                                    "coments parse error of {}: {}",
                                    &gallery_state.index.id,
                                    err.to_string()
                                );
                                metric.comment_error += 1;
                            }
                            Err(err) => {
                                error!(
                                    "document crawl of {}: {}",
                                    &gallery_state.index.id,
                                    err.to_string()
                                );
                                metric.document_error += 1;
                            }
                        };
                    }
                    if let Err(e) = self
                        .report_success(GalleryCrawlReportForm {
                            id: gallery_state.index.id.clone(),
                            worker_part: self.part,
                            last_crawled_at: Some(now),
                            last_crawled_document_id: if last_document_id > 0 {
                                Some(last_document_id)
                            } else {
                                None
                            },
                            crawled_document_count: res.len(),
                        })
                        .await
                    {
                        error!("error while report: {}", e.to_string());
                    };
                }
                Err(err) => {
                    error!(
                        "get index of {} fail: {}",
                        &gallery_state.index.id,
                        err.to_string()
                    );
                    metric.gallery_error += 1;
                    info!("report error");
                    if let Err(e) = self
                        .error_report(GalleryCrawlErrorReportForm {
                            worker_part: self.part,
                            id: gallery_state.index.id.clone(),
                            error: err.into(),
                            last_crawled_at: Some(now),
                        })
                        .await
                    {
                        error!("error while error report: {}", e.to_string());
                    };
                }
            };
        }
        Ok(metric)
    }
}

#[derive(Clone)]
struct ResultMetricGauges {
    gallery_success: IntGauge,
    document_success: IntGauge,
    comment_success: IntGauge,
    gallery_error: IntGauge,
    document_error: IntGauge,
    comment_error: IntGauge,
}
async fn crawl_forever(
    mut state: State,
    delay: Duration,
    gauges: ResultMetricGauges,
) -> Result<std::convert::Infallible, WorkerError> {
    loop {
        let metric = state.run().await?;
        gauges
            .gallery_success
            .set(metric.gallery_success.try_into().unwrap());
        gauges
            .document_success
            .set(metric.document_success.try_into().unwrap());
        gauges
            .comment_success
            .set(metric.comment_success.try_into().unwrap());
        gauges
            .gallery_error
            .set(metric.gallery_error.try_into().unwrap());
        gauges
            .document_error
            .set(metric.document_error.try_into().unwrap());
        gauges
            .comment_error
            .set(metric.comment_error.try_into().unwrap());
        info!("crawl done. wait {} milli seconds..", delay.as_millis());
        actix::clock::delay_for(delay).await;
    }
}

#[get("/health")]
async fn health() -> impl Responder {
    "ok"
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(health);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let live_directory_url = std::env::var("LIVE_DIRECTORY_URL").expect("LIVE_DIRECTORY_URL");
    let data_broker_url = std::env::var("DATA_BROKER_URL").expect("DATA_BROKER_URL");
    let nats_url = std::env::var("NATS_URL").expect("NATS_URL");
    let nats_subject =
        std::env::var("NATS_SUBJECT").unwrap_or_else(|_| "crawled.dcinside.documents".to_string());

    let part: u64 = std::env::var("PART").expect("PART").parse().expect("PART");
    let total: u64 = std::env::var("TOTAL")
        .expect("TOTAL")
        .parse()
        .expect("TOTAL");
    let delay: u64 = std::env::var("DELAY")
        .unwrap_or_else(|_| "100".to_string())
        .parse()
        .expect("DELAY");
    let sleep_duration: u64 = std::env::var("SLEEP_DURATION")
        .unwrap_or_else(|_| "6000".to_string())
        .parse()
        .expect("SLEEP_DURATION");

    let prometheus = PrometheusMetrics::new("api", Some("/metrics"), None);
    let metrics = ResultMetricGauges {
        gallery_success: IntGauge::new("dccrawler_gallery_success", "gallery_success").unwrap(),
        gallery_error: IntGauge::new("dccrawler_gallery_error", "gallery_error").unwrap(),
        document_success: IntGauge::new("dccrawler_document_success", "document_success").unwrap(),
        document_error: IntGauge::new("dccrawler_document_error", "document_error").unwrap(),
        comment_success: IntGauge::new("dccrawler_comment_success", "comment_success").unwrap(),
        comment_error: IntGauge::new("dccrawler_comment_error", "comment_error").unwrap(),
    };

    let reg = prometheus.clone().registry;
    reg.register(Box::new(metrics.gallery_success.clone()))
        .unwrap();
    reg.register(Box::new(metrics.gallery_error.clone()))
        .unwrap();
    reg.register(Box::new(metrics.document_success.clone()))
        .unwrap();
    reg.register(Box::new(metrics.document_error.clone()))
        .unwrap();
    reg.register(Box::new(metrics.comment_success.clone()))
        .unwrap();
    reg.register(Box::new(metrics.comment_error.clone()))
        .unwrap();

    actix_rt::spawn(async move {
        loop {
            let state = State::new(
                &live_directory_url,
                &data_broker_url,
                &nats_url,
                nats_subject.clone(),
                total,
                part,
            )
            .unwrap()
            .with_crawler_delay(delay);
            let res = crawl_forever(
                state,
                Duration::from_millis(sleep_duration),
                metrics.clone(),
            )
            .await;
            if let Err(e) = res {
                error!("crawler restart due to: {}", e.to_string());
            }
        }
    });
    HttpServer::new(move || App::new().wrap(prometheus.clone()).configure(config))
        .bind(format!("0.0.0.0:{}", port))?
        .workers(1)
        .run()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    fn s() -> String {
        "a".to_string()
    }
    #[test]
    fn bincode_serialize() {
        let doc = Document {
            gallery: Gallery {
                id: s(),
                name: s(),
                kind: GalleryKind::Major,
            },
            gallery_id: s(),
            id: 1,
            title: s(),
            subject: Some(s()),
            author: User {
                ip: Some(s()),
                nickname: s(),
                id: None,
                kind: UserKind::Dynamic,
            },
            comment_count: 1,
            like_count: 2,
            view_count: 3,
            kind: DocumentKind::Text,
            is_recommend: true,
            created_at: chrono::Utc::now(),

            comments: Some(vec![
                Comment {
                    id: 1,
                    author: User {
                        ip: None,
                        nickname: s(),
                        id: Some(s()),
                        kind: UserKind::Static,
                    },
                    depth: 0,
                    kind: CommentKind::Text,
                    contents: s(),
                    parent_id: None,
                    created_at: Some(chrono::Utc::now()),
                }]),
            body: None
        };
        let bytes = bincode::serialize(&doc).unwrap();
    }

    /*
    #[actix_rt::test]
    async fn state_update_list_part() {
        let mut state = State::new(
            live_directory_url: "",
            data_broker_url: "",
            total: 1u64,
            part: 1u64);
        state.update().await.unwrap();
        let res1 = state.list_part(2, 0);
        let res2 = state.list_part(2, 1);
        assert!(res1.len() > 0);
        assert!(res2.len() > 0);
        let mut h = std::collections::HashSet::new();
        for t in res1.iter() { h.insert(t.index.id.clone()); }
        for t in res2.iter() { h.insert(t.index.id.clone()); }
        assert_eq!(h.len(), res1.len() + res2.len());
    }
    #[actix_rt::test]
    async fn state_report() {
        let mut state = State::new("");
        state.update().await.unwrap();
        let res1 = state.list_part(2, 0);
        assert!(res1[0].last_crawled_at.is_none());
        let now = Utc::now();
        state.report(GalleryCrawlReportForm{
            id: res1[0].index.id.clone(),
            last_crawled_at: Some(now.clone()),
            last_crawled_document_id: Some(1),
        }).unwrap();
        let res1 = state.list_part(2, 0);
        assert_eq!(res1[0].last_crawled_at, Some(now));
        assert_eq!(res1[0].last_crawled_document_id, Some(1));
    }

    #[actix_rt::test]
    async fn test_health() {
        let mut app = test::init_service(App::new().configure(config)).await;
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
    }
    */
}
