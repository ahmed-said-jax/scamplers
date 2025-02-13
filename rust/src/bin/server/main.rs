use camino::Utf8PathBuf;
use clap::Parser;
use app::serve_app;

mod app;
mod api;
mod seed_data;
mod web;
mod db;

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