use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::{command, value_parser, Parser, Subcommand};
use dotenvy;
use scamplers::{sync_10x, sync_files, ScamplersConfig};
use std::env;
// TODO: change all String in parameter definitions to &str

#[derive(Debug, Parser)]
#[command(version, arg_required_else_help = true)]
struct CLI {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// TODO
    #[command(arg_required_else_help = true)]
    SyncFiles {
        #[arg(value_parser = value_parser!(Utf8PathBuf))]
        files: Vec<Utf8PathBuf>,
    },

    /// TODO
    #[command()]
    SyncGoogleSheets {},

    /// TODO
    #[command(name = "sync-10x")]
    Sync10X {},
}

fn main() -> Result<()> {
    dotenvy::dotenv().unwrap_or_default();

    // TODO: explore confy to make this cleaner
    let config_dir = Utf8PathBuf::from(
        env::var("SCAMPLERS_CONFIG_DIR").unwrap_or("/sc/service/etc/.config/scamplers".into()),
    );

    let scamplers_config_path = config_dir.join("scamplers.json");
    let scamplers_config = ScamplersConfig::from_file(&scamplers_config_path)
        .with_context(|| format!("could not read config file from {scamplers_config_path}"))?;

    let cli = CLI::parse();

    match cli.command {
        Commands::SyncFiles { files } => sync_files(scamplers_config, files),
        Commands::SyncGoogleSheets {} => Ok(()),
        Commands::Sync10X {} => sync_10x(scamplers_config)
    }
}
