use std::{fs, str::FromStr};

use anyhow::{Context, anyhow};
use camino::{Utf8Path, Utf8PathBuf};
use clap::{Args, Parser, Subcommand};

use crate::{auth, seed_data::SeedData};

#[derive(Args)]
pub struct Config {
    #[arg(long, env = "SCAMPLERS_DB_ROOT_USER")]
    db_root_user: String,
    #[arg(long, env = "SCAMPLERS_DB_ROOT_PASSWORD")]
    db_root_password: String,
    #[arg(long, env = "SCAMPLERS_DB_LOGIN_USER_PASSWORD")]
    db_login_user_password: String,
    #[arg(long, env = "SCAMPLERS_DB_AUTH_USER_PASSWORD")]
    db_auth_user_password: String,
    #[arg(long, env = "SCAMPLERS_DB_HOST")]
    db_host: String,
    #[arg(long, env = "SCAMPLERS_DB_PORT")]
    db_port: u16,
    #[arg(long, env = "SCAMPLERS_DB_NAME")]
    db_name: String,
    #[arg(long, env = "SCAMPLERS_APP_HOST")]
    app_host: String,
    #[arg(long, env = "SCAMPLERS_APP_PORT")]
    app_port: u16,
    #[arg(skip)]
    seed_data: Option<SeedData>,
    #[arg(long, env = "SCAMPLERS_SEED_DATA_PATH")]
    seed_data_path: Option<Utf8PathBuf>,
}
impl Config {
    pub fn from_secrets_dir(dir: &Utf8Path) -> anyhow::Result<Self> {
        // This is a bit of AI genius
        let read_secret =
            |name: &str| fs::read_to_string(dir.join(name)).context(format!("failed to read secret {name}"));

        let config = Self {
            db_root_user: read_secret("db_root_user")?,
            db_root_password: read_secret("db_root_password")?,
            db_login_user_password: read_secret("db_login_user_password")?,
            db_auth_user_password: read_secret("db_auth_user_password")?,
            db_host: read_secret("db_host")?,
            db_port: read_secret("db_port")?.parse()?,
            db_name: read_secret("db_name")?,
            app_host: "0.0.0.0".to_string(), // Slightly hacky
            app_port: read_secret("app_port")?.parse()?,
            seed_data: serde_json::from_str(&read_secret("seed_data")?)?,
            seed_data_path: None,
        };

        Ok(config)
    }

    pub fn app_address(&self) -> String {
        let Self {
            app_host,
            app_port,
            ..
        } = self;

        format!("{app_host}:{app_port}")
    }

    pub fn db_login_user_password(&self) -> &str {
        &self.db_login_user_password
    }

    pub fn db_auth_user_password(&self) -> &str {
        &self.db_auth_user_password
    }

    fn db_url(&self, root: bool) -> String {
        const LOGIN_USER: &str = "login_user";
        let Self {
            db_root_user,
            db_root_password,
            db_login_user_password,
            db_host,
            db_port,
            db_name,
            ..
        } = self;

        let base = "postgres://";
        let db_spec = format!("{db_host}:{db_port}/{db_name}");

        if root {
            format!("{base}{db_root_user}:{db_root_password}@{db_spec}")
        } else {
            format!("{base}{LOGIN_USER}:{db_login_user_password}@{db_spec}")
        }
    }

    pub fn db_root_url(&self) -> String {
        self.db_url(true)
    }

    pub fn db_login_url(&self) -> String {
        self.db_url(false)
    }

    pub fn seed_data(&self) -> anyhow::Result<Option<SeedData>> {
        let Self {
            seed_data,
            seed_data_path,
            ..
        } = self;

        match (seed_data, seed_data_path) {
            (None, None) => Ok(None),
            (Some(seed_data), None) => Ok(Some(seed_data.clone())),
            (None, Some(seed_data_path)) => Ok(Some(serde_json::from_str(&fs::read_to_string(seed_data_path)?)?)),
            (Some(_), Some(_)) => Err(anyhow!(
                "seed_data_path should not be set alongside seed data read from secrets dir"
            )),
        }
    }
}

#[derive(Subcommand)]
pub enum Command {
    Dev {
        #[arg(default_value_t = String::from("localhost"))]
        host: String,
        #[arg(default_value_t = 8000)]
        port: u16,
    },
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
        log_dir: Utf8PathBuf,
    },
    Schema {
        #[arg(short, long)]
        output_dir: Utf8PathBuf,
    },
}

#[derive(Parser)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}
