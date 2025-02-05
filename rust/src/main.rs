use std::str::FromStr;

use camino::Utf8PathBuf;
use clap::Parser;
use scamplers::serve_app;
use tracing::Level;
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Cli {
        config_path,
        log_dir,
    } = Cli::parse();

    let log_writer = tracing_appender::rolling::daily(log_dir, "scamplers.log");

    let filter = tracing_subscriber::filter::Targets::new().with_target("scamplers", Level::INFO);

    let layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(log_writer)
        .with_filter(filter);

    tracing_subscriber::registry().with(layer).init();

    serve_app(&config_path).await
}

#[derive(Parser)]
#[command(version)]
struct Cli {
    config_path: Utf8PathBuf,
    #[arg(short, long, default_value_t = Utf8PathBuf::from_str("scamplers_logs").unwrap())]
    log_dir: Utf8PathBuf,
}
