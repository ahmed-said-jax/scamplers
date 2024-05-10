use anyhow::Result;
use camino::Utf8PathBuf;
use clap::{command, value_parser, Parser, Subcommand};
use scamplers::{sync_10x, sync_files, ScamplersConfig};

// LONG-TERM TODOS:
// review all function parameters and ensure that they receive references when they don't need to own data, rather than to copy stuff
// figure out how to parallelize things
// add logging (some kind of structured, machine-readable output?)
// related to above - do not fail blindly on one bad record, instead, skip it and log
// develop some kind of methodology for where to put error context - does it go in calling functions, or should it go in called functions, or both?
// related to above - add context for all errors
// Utf8PathBufs should probably just be Utf8Path. Since this is not an owned type, everything needs to become a reference to that

// SHORT-TERM TODOS:
// finish metrics ingestion from all *ranger pipelines
// write some tests to ensure that the internal API developed makes sense
// modularize more things

#[derive(Debug, Parser)]
#[command(version, arg_required_else_help = true)]
struct Cli {
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

// is it a good design to have main load the config and then pass it into these functions? or should the functions just load the config on their own?
fn main() -> Result<()> {
    let scamplers_config = ScamplersConfig::load()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::SyncFiles { files } => sync_files(&scamplers_config, files),
        Commands::SyncGoogleSheets {} => Ok(()),
        Commands::Sync10X {} => sync_10x(&scamplers_config),
    }
}
