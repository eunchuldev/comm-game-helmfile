use crate::error::{Result, Error};
use crate::model::State;
use serde::{Deserialize};
use actix_web::{
    error::ErrorInternalServerError, get, post, web, App, HttpResponse, HttpServer, Responder,
    web::ServiceConfig,
};
use actix_files::{Files, NamedFile};
use futures::TryStreamExt;
use log::{debug, error};
use prometheus::{Encoder, TextEncoder};
use std::time::Duration;
use std::thread;

type DateTime = chrono::DateTime<chrono::Utc>;

#[derive(Debug, Deserialize, PartialEq)]
pub enum SearchTarget {
    Document,
    Comment
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct SearchQuery {
    pub gallery_id: Option<String>,
    pub title: Option<String>,
    pub author_nickname: Option<String>,
    pub author_ip: Option<String>,
    pub author_id: Option<String>,
    pub last_created_at: Option<DateTime>,
    pub target: SearchTarget,
}

#[get("/api/search")]
async fn search(state: web::Data<State>, query: web::Query<SearchQuery>) -> Result<HttpResponse> {
    let res = match query.into_inner() {
        SearchQuery { target: SearchTarget::Document, gallery_id: None, title: Some(title), last_created_at, .. } => 
            Ok(state.search_docs_by_title(&title, last_created_at).await?),
        SearchQuery { target: SearchTarget::Document, gallery_id: Some(gallery_id), title: Some(title), last_created_at, .. } => 
            Ok(state.search_docs_by_gallery_and_title(&gallery_id, &title, last_created_at).await?),
        _ =>  
            Err(Error::SearchRequestBadRequest),
    }?;
    Ok(HttpResponse::Ok().json(res))
}

fn config_app(state: State) -> Box<dyn FnOnce(&mut ServiceConfig)> {
    Box::new(move |cfg: &mut ServiceConfig| {
        cfg
            .app_data(web::Data::new(state))
            .service(search)
            .service(Files::new("/", "./svelte-app/public/").index_file("index.html"));
    })
}

pub async fn serve(
    host: &str,
    port: usize,
    db_url: &str,
) -> Result<()> {
    let state = State::connect(db_url).await?;
    println!("start web service at {}:{}", host, port);
    Ok(HttpServer::new(move || App::new().configure(config_app(state.clone())))
        .bind(format!("{}:{}", host, port).as_str())?
        .run()
        .await?)
}
