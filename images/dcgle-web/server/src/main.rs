use clap::Clap;
use thiserror::Error;
use futures::TryFutureExt;
use model::Error as ModelError;

pub mod web;
pub mod metrics;

#[derive(Error, Debug)]
enum Error {
    #[error("web err: {0}")]
    WebError(#[from] web::WebError),
    #[error("metrics err: {0}")]
    MetricsError(#[from] metrics::MetricsError),
    #[error("model err: {0}")]
    ModelError(#[from] ModelError),
}

#[derive(Clap, Debug)]
#[clap(author, about, version)]
struct Opts {
    #[clap(short, long, default_value = "0.0.0.0", env = "HOST")]
    host: String,
    #[clap(short, long, default_value = "8000", env = "PORT")]
    port: u16,
    #[clap(short, long, default_value = "9213", env = "METRICS_PORT")]
    metrics_port: u16,
    #[clap(short, long, env = "DATABASE_URL")]
    db_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init();
    let opts: Opts = Opts::parse();
    model::State::migrate(&opts.db_url).await?;
    let s1 = web::serve(
        &opts.db_url,
        &opts.host,
        opts.port,
    );
    let s2 =metrics::serve(
        &opts.db_url,
        &opts.host,
        opts.metrics_port,
    );
    futures::future::try_join(s1.map_err(Error::from), s2.map_err(Error::from)).await?;
    Ok(())
}
