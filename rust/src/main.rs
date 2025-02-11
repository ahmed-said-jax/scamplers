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

    let log_filter = tracing_subscriber::filter::Targets::new().with_target("scamplers", Level::INFO);

    match log_dir {
        Some(log_dir) => {
            let log_writer = tracing_appender::rolling::daily(log_dir, "scamplers.log");
        
            let log_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_writer(log_writer)
                .with_filter(log_filter);

            tracing_subscriber::registry().with(log_layer).init();
        }
        None => {
            let log_layer = tracing_subscriber::fmt::layer().pretty().with_filter(log_filter);

            tracing_subscriber::registry().with(log_layer).init();
        }
    };

    serve_app(config_path.as_ref().map(|p| p.as_path())).await
}

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[arg(short, long)]
    config_path: Option<Utf8PathBuf>,
    #[arg(short, long)]
    log_dir: Option<Utf8PathBuf>,
}
