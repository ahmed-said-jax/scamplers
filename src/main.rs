use anyhow::Result;
use camino::Utf8PathBuf;
use clap::{command, value_parser, Parser, Subcommand};
use scamplers::{mongo::get_db, sync_10x, sync_files, ScamplersConfig};

// LONG-TERM TODOS:
// review all function parameters and ensure that functions receive references when they don't need to own data (performance)
// figure out how to parallelize things (performance)
// add logging (some kind of structured, machine-readable output?)
// related to above - do not fail blindly on one bad record, instead, skip it and log
// develop some kind of methodology for where to put error context - does it go in calling functions, or should it go in called functions, or both?
// related to above - add context for all errors
// Utf8PathBufs should probably just be Utf8Path. Since this is not an owned type, everything needs to become a reference to that

// SHORT-TERM TODOS:
// finish metrics ingestion from all *ranger pipelines
// write some tests to ensure that the internal API developed makes sense
// modularize

#[derive(Debug, Parser)]
#[command(version, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// TODO: there should be an init-db com
#[derive(Debug, Subcommand)]
enum Commands {
    /// TODO
    #[command(arg_required_else_help = true)]
    SyncFiles {
        #[arg(value_parser = value_parser!(Utf8PathBuf))]
        files: Vec<Utf8PathBuf>,

        #[arg(long, short, value_parser = value_parser!(bool))]
        overwrite_data_sets: bool
    },

    /// TODO
    #[command()]
    SyncGoogleSheets {},

    /// TODO
    #[command(name = "sync-10x")]
    Sync10X {},
}

fn main() -> Result<()> {
    let scamplers_config = ScamplersConfig::load()?;
    let db = get_db(&scamplers_config.db_uri, &scamplers_config.db_name)?;

    let cli = Cli::parse();

    match cli.command {
        Commands::SyncFiles { files , overwrite_data_sets} => sync_files(db, files, overwrite_data_sets),
        Commands::SyncGoogleSheets {} => Ok(()),
        Commands::Sync10X {} => sync_10x(db),
    }
}
