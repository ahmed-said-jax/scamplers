use camino::Utf8PathBuf;
use config::Config;

mod config;
pub mod db;

mod server;
mod util;

pub async fn serve_dev_app(host: String, port: u16) -> anyhow::Result<()> {
    server::serve(None, None, Some((host, port))).await
}

pub async fn serve_prod_app(config: Config, log_dir: Option<Utf8PathBuf>) -> anyhow::Result<()> {
    server::serve(log_dir, Some(config), None).await
}
