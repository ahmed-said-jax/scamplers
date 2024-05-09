use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::{command, value_parser, Parser, Subcommand};
use config::Config;
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
    let config_path = env::var("SCAMPLERS_CONFIG_PATH")
        .unwrap_or("/sc/service/etc/.config/scamplers".to_string());

    let config = Config::builder()
        .set_default("db_name", "test")?
        .add_source(config::File::with_name(&config_path).required(false))
        .add_source(config::Environment::with_prefix("SCAMPLERS"))
        .build()?;
    let scamplers_config: ScamplersConfig = config.try_deserialize().with_context(|| format!("could not load configuration from of environment and file. Fix the fields in {config_path} or set environment variables prefixed by 'SCAMPLERS'"))?;

    let cli = CLI::parse();

    match cli.command {
        Commands::SyncFiles { files } => sync_files(scamplers_config, files),
        Commands::SyncGoogleSheets {} => Ok(()),
        Commands::Sync10X {} => sync_10x(scamplers_config),
    }
}
