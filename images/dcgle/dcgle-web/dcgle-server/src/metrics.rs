use warp::{Filter, reject::Reject};
use log::{error, info};
use dcgle_model::{State, Error as ModelError};
use prometheus::{Encoder, TextEncoder};
use std::str::FromStr;
use thiserror::Error;
use http::StatusCode;
use async_graphql_warp::BadRequest;
use std::net::{IpAddr, SocketAddr, AddrParseError};

#[derive(Error, Debug)]
pub enum MetricsError {
    #[error("io err: {0}")]
    IoError(#[from] std::io::Error),
    #[error("addr parse err: {0}")]
    AddrParseError(#[from] AddrParseError),
    #[error("model err: {0}")]
    ModelError(#[from] ModelError),
    #[error("prometheus err: {0}")]
    PrometheusError(#[from] prometheus::Error),
    #[error("fromutf8 err: {0}:")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
}

impl Reject for MetricsError {} 


use prometheus::{
    register_counter_vec, register_histogram, register_histogram_vec, register_int_counter, register_int_gauge,
    CounterVec, Histogram, HistogramVec, IntCounter, IntGauge
};

pub async fn serve(db_url: &str, host: &str, port: u16) -> Result<(), MetricsError> {
    let state = State::connect(db_url).await?;
    warp::serve(routes(state))
        .run(SocketAddr::new(IpAddr::from_str(host)?, port))
        .await;
    Ok(())
}

pub fn routes(state: State) 
    -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    healthz(state)
        .or(metrics())
}


pub fn healthz(state: State) 
    -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("healthz")
        .and(warp::get())
        .and(warp::any().map(move || state.clone()))
        .and_then(|state: State| async move {
            state.health().await.map_err(MetricsError::from)?;//.map_err(warp::reject::custom)?;
            Ok::<_, warp::Rejection>(warp::reply::reply())
        })
}

pub fn metrics() 
    -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("metrics")
        .and(warp::get())
        .and_then(|| async move {
            let mut buffer = Vec::<u8>::new();
            TextEncoder::new()
                .encode(&prometheus::gather(), &mut buffer).map_err(MetricsError::from).map_err(warp::reject::custom)?;
                /*.map_err(|err| {
                    error!("metrics error: {:?}", err);
                    ErrorInternalServerError(format!("{:?}", err))
                })?;*/
            Ok::<_, warp::Rejection>(String::from_utf8(buffer).map_err(MetricsError::from).map_err(warp::reject::custom)?)/*.map_err(|err| {
                error!("metrics error: {:?}", err);
                ErrorInternalServerError(format!("{:?}", err))
            })*/
        })
}


