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

    serve_app(config_path, log_dir).await
}

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[arg(short, long)]
    config_path: Option<Utf8PathBuf>,
    #[arg(short, long)]
    log_dir: Option<Utf8PathBuf>,
}
