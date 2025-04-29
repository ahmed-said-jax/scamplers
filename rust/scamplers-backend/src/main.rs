use clap::Parser;
use scamplers_backend::{config::Cli, server::serve};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().unwrap_or_default();
    let Cli { config, log_dir } = Cli::parse();

    serve(config, log_dir).await?;

    Ok(())
}
