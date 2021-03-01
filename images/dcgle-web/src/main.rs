use clap::Clap;
use error::Result;

pub mod error;
pub mod model;
pub mod web;
pub mod metrics;

#[derive(Clap, Debug)]
#[clap(author, about, version)]
struct Opts {
    #[clap(short, long, default_value = "0.0.0.0", env = "HOST")]
    host: String,
    #[clap(short, long, default_value = "8000", env = "PORT")]
    port: usize,
    #[clap(short, long, default_value = "9213", env = "METRICS_PORT")]
    metrics_port: usize,
    #[clap(short, long, env = "DATABASE_URL")]
    db_url: String,
}

#[actix_web::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let opts: Opts = Opts::parse();
    model::State::migrate(&opts.db_url).await?;
    let s1 = web::serve(
        &opts.host,
        opts.port,
        &opts.db_url,
    );
    let s2 =metrics::serve(
        &opts.host,
        opts.metrics_port,
        &opts.db_url
    );
    futures::future::try_join(s1, s2).await?;
    Ok(())
}
