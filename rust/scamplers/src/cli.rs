use std::{collections::HashMap, fs, str::FromStr};

use camino::{Utf8Path, Utf8PathBuf};
use clap::{Args, Parser, Subcommand};

use crate::seed_data::SeedData;

#[derive(Args, Debug)]
pub struct Config {
    #[arg(long, env)]
    db_root_user: String,
    #[arg(long, env)]
    db_root_password: String,
    #[arg(long, env)]
    db_name: String,
    #[arg(long, env)]
    auth_host: String,
    #[arg(long, env)]
    auth_port: u16,
    #[arg(long, env)]
    app_host: String,
    #[arg(long, env)]
    app_port: u16,
    #[arg(skip)]
    seed_data: SeedData,
}
impl Config {
    fn from_secrets_dir(dir: &Utf8Path) -> anyhow::Result<Self> {
        let secrets = [
            "db_root_user",
            "db_root_password",
            "db_name",
            "auth_host",
            "auth_port",
            "app_host",
            "app_port",
            "seed_data",
        ];
        let secrets: Result<Vec<_>, _> = secrets.iter().map(|f| fs::read_to_string(dir.join(f))).collect();
        let secrets = secrets?;

        let config = Self {
            db_root_user: secrets[0].clone(),
            db_root_password: secrets[1].clone(),
            db_name: secrets[2].clone(),
            auth_host: secrets[3].clone(),
            auth_port: secrets[4].parse()?,
            app_host: secrets[5].clone(),
            app_port: secrets[6].parse()?,
            seed_data: serde_json::from_str(&secrets[7])?,
        };

        Ok(config)
    }
}
impl Default for Config {
    fn default() -> Self {
        let db = "postgres".to_string();

        Self {
            db_root_user: db.clone(),
            db_name: db.clone(),
            app_host: "localhost".to_string(),
            app_port: 8000,
            seed_data:
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Dev,
    Test {
        #[command(flatten)]
        config: Config,
        #[arg(short, long)]
        log_dir: Option<Utf8PathBuf>,
    },
    Prod {
        #[arg(long, env, default_value_t = Utf8PathBuf::from_str("/run/secrets").unwrap())]
        secrets_dir: Utf8PathBuf,
        #[arg(short, long)]
        log_dir: Option<Utf8PathBuf>,
    },
    Schema {
        #[arg(short, long)]
        output_dir: Utf8PathBuf,
    },
}

// #[derive(Subcommand, Debug)]
// pub enum Command {
//     Serve {
//         secrets_dir: Option<Utf8PathBuf>,
//         #[command(flatten)]
//         config: Option<Config>,
//     },
//     Schema {
//         #[arg(short, long)]
//         output_dir: Utf8PathBuf,
//     },
// }

#[derive(Parser, Debug)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}
