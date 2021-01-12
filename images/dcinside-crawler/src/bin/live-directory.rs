use chrono::Utc;
use actix_web::{get, post, web, App, HttpServer, HttpResponse, Responder, error::ResponseError, http::StatusCode};
use std::time::Duration;

use err_derive::Error;
use dcinside_crawler::error::*;
use std::sync::Arc;
use std::convert::TryInto;


use dcinside_crawler::crawler::Crawler;
use dcinside_crawler::model::*;

use serde::Deserialize;

use actix_web_prom::PrometheusMetrics;
use prometheus::{IntGauge};

use log::{info, error};

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
            Self::NotFound  => StatusCode::NOT_FOUND,
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
where T: Hash,
{
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    obj.hash(&mut hasher);
    hasher.finish()
}

#[derive(Clone)]
struct State {
    crawler: Crawler,
    gallery_db: sled::Tree,
}

impl State {
    fn new() -> Self {
        let config = sled::Config::new().temporary(true);
        let db = config.open().unwrap().open_tree("galleries").unwrap();
        State {
            crawler: Crawler::new(),
            gallery_db: db,
        }
    }
    fn with_db(db: sled::Tree) -> Self {
        State {
            crawler: Crawler::new(),
            gallery_db: db,
        }
    }
    async fn update(&self) -> Result<(), LiveDirectoryError> {
        let now = Utc::now();
        let hot_galleries = self.crawler.realtime_hot_galleries().await?;
        for index in hot_galleries {
            let new_state = GalleryState {
                index,
                last_ranked: now.clone(),
                last_crawled_at: None,
                last_crawled_document_id: None,
            };
            self.gallery_db.fetch_and_update(new_state.index.id.clone().as_bytes(), move |old| Some(match old {
                Some(bytes) => {
                    let new_index = new_state.index.clone();
                    let mut old_state = serde_json::from_slice::<GalleryState>(bytes).unwrap_or_else(|e| {
                        error!("fail to parse sled tree data {} with error {}", &new_index.id, e);
                        new_state.clone()
                    });
                    old_state.last_ranked = now.clone();
                    old_state.index = new_index;
                    serde_json::to_vec(&old_state).unwrap()
                },
                None => serde_json::to_vec(&new_state).unwrap(),
            }))?;
        }
        Ok(())
    }
    fn report(&self, form: GalleryCrawlReportForm) -> Result<(), LiveDirectoryError> {
        let mut found = false;
        self.gallery_db.fetch_and_update(form.id.as_bytes(), |old| match old {
            Some(bytes) => {
                found = true;
                serde_json::from_slice::<GalleryState>(bytes).map(|mut old_state| {
                    old_state.last_crawled_at = form.last_crawled_at;
                    old_state.last_crawled_document_id = form.last_crawled_document_id;
                    serde_json::to_vec(&old_state).unwrap()
                }).ok()
            }
            None => None
        })?;
        if found {
            Ok(())
        } else {
            Err(LiveDirectoryError::NotFound)
        }
    }
    fn list_part(&self, total: u64, part: u64) -> Vec<GalleryState> {
        self.gallery_db.iter().filter_map(|res| {
            if res.is_err() {
                error!("fail to iterate over sled");
            }
            res.ok()
        }).filter(|(id, _)| {
            hash(id) % total == part
        }).filter_map(|(_, state)| { 
            let res = serde_json::from_slice::<GalleryState>(&state);
            if res.is_err() {
                error!("fail to parse value during iterate over sled");
            }
            res.ok()
        }).collect()
    }
}

async fn update_forever(state: State, delay: Duration, db_size_gauge: IntGauge) -> Result<(), LiveDirectoryError> {
    info!("start update live directory");
    loop {
        state.update().await?;
        db_size_gauge.set(state.gallery_db.len().try_into().unwrap());
        info!("update live directory done. wait {} seconds..", delay.as_secs());
        actix::clock::delay_for(delay).await;
    }
    Ok(())
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
async fn list_part(web::Query(query): web::Query<ListPartQuery>, state: web::Data<State>) -> impl Responder {
    web::Json(state.list_part(query.total, query.part))
}

#[post("/report")]
async fn report(web::Json(form): web::Json<GalleryCrawlReportForm>, state: web::Data<State>) -> Result<HttpResponse, LiveDirectoryError> {
    let state = state.clone();
    state.report(form)?;
    Ok(HttpResponse::Ok().finish())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(health).service(list_part).service(report);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let store_path = std::env::var("STORE_PATH").unwrap_or("".to_string());

    let prometheus = PrometheusMetrics::new("api", Some("/metrics"), None);
    let counter = IntGauge::new("major_gallery_count", "major_gallery_count").unwrap();
    prometheus
        .registry
        .register(Box::new(counter.clone()))
        .unwrap();

    let db = if store_path.is_empty() {
        let config = sled::Config::new().temporary(true);
        config.open().unwrap()
    } else {
        sled::open(store_path).unwrap()
    };
    let db = db.open_tree("galleries").unwrap();

    let state = State::with_db(db.clone());
    actix_rt::spawn(async move { update_forever(state, Duration::from_secs(60), counter.clone()).await; });
    HttpServer::new(move || {
        let state = State::with_db(db.clone());
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
    use actix_web::{body::Body, test, web, App, http};

    #[actix_rt::test]
    async fn state_update_list_part() {
        let state = State::new();
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
        let state = State::new();
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

}
