use std::{fs, str::FromStr};

use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use scamplers::{
    db::institution::{Institution, NewInstitution, UpdatedInstitution},
    serve_app,
};
use schemars::schema_for;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Cli { command } = Cli::parse();

    match command {
        Command::Serve { config_path, log_dir } => serve_app(config_path, log_dir).await,
        Command::Schema { output_dir } => {
            let schema = [
                ("new_institution", schema_for!(NewInstitution)),
                ("updated_institution", schema_for!(UpdatedInstitution)),
                ("institution", schema_for!(Institution)),
            ];
            for (filename, s) in schema {
                let s = serde_json::to_string_pretty(&s)?;
                let output_path = output_dir.join(format!("{filename}.json"));
                fs::write(output_path, s)?;
            }

            Ok(())
        }
    }
}

#[derive(Subcommand)]
enum Command {
    Serve {
        #[arg(short, long)]
        config_path: Option<Utf8PathBuf>,
        #[arg(short, long)]
        log_dir: Option<Utf8PathBuf>,
    },
    Schema {
        #[arg(short, long)]
        output_dir: Utf8PathBuf,
    },
}

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
