
use clap::Parser;
use scamplers::{
    cli::{self, Cli, Config},
    serve_dev_app, serve_prod_app,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().unwrap_or_default();
    let Cli { command } = Cli::parse();

    match command {
        cli::Command::Dev { host, port } => serve_dev_app(host, port).await?,
        cli::Command::Test { config, log_dir } => serve_prod_app(config, log_dir).await?,
        cli::Command::Prod { secrets_dir, log_dir } => {
            serve_prod_app(Config::from_secrets_dir(&secrets_dir)?, Some(log_dir)).await?
        }
        _ => todo!(),
    }

    Ok(())
}
