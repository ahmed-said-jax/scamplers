use clap::Parser;
use scamplers_backend::{
    config::{Cli, Config, Command},
    serve_dev_app, serve_prod_app
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().unwrap_or_default();
    let Cli { command } = Cli::parse();

    match command {
        Command::Dev { host, port } => serve_dev_app(host, port).await?,
        Command::Prod { config, log_dir } => {
            serve_prod_app(config, Some(log_dir)).await?
        }
        _ => todo!(),
    }

    Ok(())
}
