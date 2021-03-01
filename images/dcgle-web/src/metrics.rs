use crate::error::Result;
use crate::model::State;
use actix_web::{
    error::ErrorInternalServerError, get, web, App, HttpServer, Responder,
    web::ServiceConfig,
};
use log::{error, info};
use prometheus::{Encoder, TextEncoder};

#[get("/healthz")]
async fn healthz(state: web::Data<State>) -> Result<&'static str> {
    Ok(state.health().await.map(|_| "ok")?)
}

#[get("/metrics")]
async fn metrics(_data: web::Data<State>) -> impl Responder {
    let mut buffer = Vec::<u8>::new();
    TextEncoder::new()
        .encode(&prometheus::gather(), &mut buffer)
        .map_err(|err| {
            error!("metrics error: {:?}", err);
            ErrorInternalServerError(format!("{:?}", err))
        })?;
    String::from_utf8(buffer).map_err(|err| {
        error!("metrics error: {:?}", err);
        ErrorInternalServerError(format!("{:?}", err))
    })
}

fn config_app(state: State) -> Box<dyn FnOnce(&mut ServiceConfig)> {
    Box::new(move |cfg: &mut ServiceConfig| {
        cfg
            .app_data(web::Data::new(state))
            .service(healthz)
            .service(metrics);
    })
}

pub async fn serve(
    host: &str,
    port: usize,
    db_url: &str,
) -> Result<()> {
    let state = State::connect(db_url).await?;
    info!("start metrics service at {}:{}", host, port);
    Ok(HttpServer::new(move || App::new().configure(config_app(state.clone())))
        .bind(format!("{}:{}", host, port).as_str())?
        .run()
        .await?)
}
