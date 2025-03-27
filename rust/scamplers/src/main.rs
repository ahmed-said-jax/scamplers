use std::fs;

use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use scamplers::{
    cli::{self, Cli},
    db::{
        institution::{Institution, NewInstitution, UpdatedInstitution},
        sample::specimen::Specimen,
    },
    serve_app,
};
use schemars::schema_for;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().unwrap_or_default();
    let Cli { command } = Cli::parse();

    match command {
        cli::Command::Dev => serve_app(),
    }
}
