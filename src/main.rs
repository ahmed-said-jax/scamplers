use anyhow::Result;
use camino::Utf8PathBuf;
use clap::{command, value_parser, Parser, Subcommand};
use dotenvy;
use scamplers::{sync_files, sync_nf_tenx};
use std::env;

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
    #[command()]
    SyncNfTenx {},
}

fn main() -> Result<()> {
    dotenvy::dotenv().unwrap_or_default();

    // TODO: explore confy to make this cleaner
    let config_dir = Utf8PathBuf::from(
        env::var("SCAMPLERS_CONFIG_DIR").unwrap_or("/sc/service/etc/.config/scamplers".into()),
    );

    let cli = CLI::parse();

    match cli.command {
        Commands::SyncFiles { files } => sync_files(config_dir, files),
        Commands::SyncGoogleSheets {} => Ok(()),
        Commands::SyncNfTenx {} => sync_nf_tenx(config_dir),
    }
}
