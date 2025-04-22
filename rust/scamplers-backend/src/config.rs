use std::{fs, str::FromStr};

use anyhow::{Context, anyhow};
use camino::{Utf8Path, Utf8PathBuf};
use clap::{Args, Parser, Subcommand};

use crate::db::seed_data::SeedData;

#[derive(Args)]
pub struct Config {
    #[arg(long)]
    secrets_dir: Option<Utf8PathBuf>,
    #[arg(long, env = "SCAMPLERS_DB_ROOT_USER", default_value_t)]
    db_root_user: String,
    #[arg(long, env = "SCAMPLERS_DB_ROOT_PASSWORD", default_value_t)]
    db_root_password: String,
    #[arg(long, env = "SCAMPLERS_DB_LOGIN_USER_PASSWORD", default_value_t)]
    db_login_user_password: String,
    #[arg(long, env = "SCAMPLERS_AUTH_SECRET", default_value_t)]
    auth_secret: String,
    #[arg(long, env = "SCAMPLERS_DB_HOST")]
    db_host: String,
    #[arg(long, env = "SCAMPLERS_DB_PORT")]
    db_port: u16,
    #[arg(long, env = "SCAMPLERS_DB_NAME", default_value_t)]
    db_name: String,
    #[arg(long, env = "SCAMPLERS_BACKEND_HOST", default_value_t = String::from("localhost"))]
    host: String,
    #[arg(long, env = "SCAMPLERS_BACKEND_PORT", default_value_t = 8000)]
    port: u16,
    #[arg(skip)]
    seed_data: Option<SeedData>,
    #[arg(long, env = "SCAMPLERS_SEED_DATA_PATH")]
    seed_data_path: Option<Utf8PathBuf>,
}
impl Config {
    pub fn read_secrets(&mut self) -> anyhow::Result<()> {
        let Self {
            secrets_dir,
            db_root_user,
            db_root_password,
            db_login_user_password,
            auth_secret,
            db_name,
            seed_data,
            seed_data_path,
            ..
        } = self;

        let Some(secrets_dir) = secrets_dir else {
            return Ok(());
        };

        // This is a bit of AI genius
        let read_secret = |name: &str| {
            fs::read_to_string(secrets_dir.join(name))
                .context(format!("failed to read secret {name}"))
        };

        *db_root_user = read_secret("db_root_user")?;
        *db_root_password = read_secret("db_root_password")?;
        *db_login_user_password = read_secret("db_login_user_password")?;
        *auth_secret = read_secret("auth_secret")?;
        *db_name = read_secret("db_name")?;
        *seed_data= serde_json::from_str(&read_secret("seed_data")?)?;
        *seed_data_path = None;

        Ok(())
    }

    pub fn app_address(&self) -> String {
        let Self {
            host: app_host,
            port: app_port,
            ..
        } = self;

        format!("{app_host}:{app_port}")
    }

    pub fn db_login_user_password(&self) -> &str {
        &self.db_login_user_password
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

    pub fn auth_secret(&self) -> &str {
        &self.auth_secret
    }

    pub fn seed_data(&mut self) -> anyhow::Result<Option<SeedData>> {
        let Self {
            seed_data,
            seed_data_path,
            ..
        } = self;

        match (seed_data, seed_data_path) {
            (seed_data, None) => Ok(seed_data.take()),
            (None, Some(seed_data_path)) => Ok(Some(serde_json::from_str(&fs::read_to_string(
                seed_data_path,
            )?)?)),
            (Some(_), Some(_)) => Err(anyhow!(
                "`seed_data` and `seed_data_path` are mutually exclusive"
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
    Prod {
        #[command(flatten)]
        config: Config,
        #[arg(short, long)]
        log_dir: Utf8PathBuf,
    },
}

#[derive(Parser)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}
