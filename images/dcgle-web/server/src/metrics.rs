use model::{State, Error};
use actix_web::{
    error::ErrorInternalServerError, get, web, App, HttpServer, Responder,
    web::ServiceConfig,
};
use log::{error, info};
use prometheus::{Encoder, TextEncoder};

use prometheus::{
    register_counter_vec, register_histogram, register_histogram_vec, register_int_counter, register_int_gauge,
    CounterVec, Histogram, HistogramVec, IntCounter, IntGauge
};

#[get("/healthz")]
async fn healthz(state: web::Data<State>) -> impl Responder {
    state.health().await.map(|_| "ok")
        .map_err(|err| {
            error!("healthz error: {:?}", err);
            ErrorInternalServerError(format!("{:?}", err))
        })
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
) -> Result<(), anyhow::Error> {
    let state = State::connect(db_url).await?;
    info!("start metrics service at {}:{}", host, port);
    Ok(HttpServer::new(move || App::new().configure(config_app(state.clone())))
        .bind(format!("{}:{}", host, port).as_str())?
        .run()
        .await?)
}
