use std::fs;

use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use scamplers::{
    cli::{self, Cli, Config},
    db::{
        institution::{Institution, NewInstitution, UpdatedInstitution},
        sample::specimen::Specimen,
    },
    serve_app, serve_app2,
};
use schemars::schema_for;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().unwrap_or_default();
    let Cli { command } = Cli::parse();

    match command {
        cli::Command::Dev => serve_app2(None, None)?,
        cli::Command::Test { config, log_dir } => serve_app2(Some(config), log_dir.as_ref())?,
        cli::Command::Prod { secrets_dir, log_dir } => {
            serve_app2(Some(Config::from_secrets_dir(&secrets_dir)?), Some(&log_dir))
        }
        _ => todo!(),
    }

    Ok(())
}
