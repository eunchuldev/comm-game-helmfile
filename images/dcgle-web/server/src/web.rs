use model::{State, Schema, schema};
use serde::{Deserialize};
use actix_web::{
    get, post, web, App, HttpResponse, HttpServer,
    web::ServiceConfig,
};
use actix_files::Files;
use log::{info};
use crate::juniper_actix::{graphiql_handler, graphql_handler, playground_handler};
use anyhow::Error;

#[get("/graphiql")]
async fn graphiql() -> Result<HttpResponse, actix_web::Error> {
    graphiql_handler("/graphql", None).await
}

#[get("/playground")]
async fn playground() -> Result<HttpResponse, actix_web::Error> {
    playground_handler("/graphql", None).await
}

#[post("/graphql")]
async fn graphql(
    req: actix_web::HttpRequest,
    payload: actix_web::web::Payload,
    state: web::Data<State>,
    schema: web::Data<Schema>,
) -> Result<HttpResponse, actix_web::Error> {
    graphql_handler(&schema, &state, req, payload).await
}

fn config_app(state: State) -> Box<dyn FnOnce(&mut ServiceConfig)> {
    let schema = schema();
    Box::new(move |cfg: &mut ServiceConfig| {
        cfg
            .app_data(web::Data::new(state))
            .app_data(web::Data::new(schema))
            .service(graphql)
            .service(graphiql)
            .service(playground)
            .service(Files::new("/", "./svelte-app/public/").index_file("index.html"));
    })
}

pub async fn serve(
    host: &str,
    port: usize,
    db_url: &str,
) -> Result<(), Error> {
    let state = State::connect(db_url).await?;
    info!("start web service at {}:{}", host, port);
    Ok(HttpServer::new(move || App::new().configure(config_app(state.clone())))
        .bind(format!("{}:{}", host, port).as_str())?
        .run()
        .await?)
}
