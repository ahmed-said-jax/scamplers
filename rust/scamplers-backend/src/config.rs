use std::fs;

use anyhow::{Context, bail};
use camino::Utf8PathBuf;
use clap::{Args, Parser};

use crate::db::seed_data::SeedData;

pub const LOGIN_USER: &str = "login_user";

#[derive(Args, serde::Deserialize, Clone)]
pub struct Config {
    #[arg(long, default_value_t)]
    dev: bool,
    #[arg(long)]
    secrets_dir: Option<Utf8PathBuf>,
    #[arg(long, env = "SCAMPLERS_DB_ROOT_USER", default_value_t)]
    db_root_user: String,
    #[arg(long, env = "SCAMPLERS_DB_ROOT_PASSWORD", default_value_t)]
    db_root_password: String,
    #[arg(long, env = "SCAMPLERS_DB_LOGIN_USER_PASSWORD", default_value_t)]
    db_login_user_password: String,
    #[arg(long, env = "SCAMPLERS_DB_HOST", default_value_t = String::from("localhost"))]
    db_host: String,
    #[arg(long, env = "SCAMPLERS_DB_PORT", default_value_t = 5432)]
    db_port: u16,
    #[arg(long, env = "SCAMPLERS_DB_NAME", default_value_t)]
    db_name: String,
    #[arg(long, env = "SCAMPLERS_FRONTEND_TOKEN", default_value_t)]
    frontend_token: String,
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
    #[must_use]
    pub fn is_dev(&self) -> bool {
        self.dev
    }

    /// # Errors
    pub fn read_secrets(&mut self) -> anyhow::Result<()> {
        let Self {
            secrets_dir,
            db_root_user,
            db_root_password,
            db_login_user_password,
            db_name,
            frontend_token,
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
        *frontend_token = read_secret("frontend_token")?;
        *db_name = read_secret("db_name")?;
        *seed_data = serde_json::from_str(&read_secret("seed_data")?)?;
        *seed_data_path = None;

        Ok(())
    }

    #[must_use]
    pub fn app_address(&self) -> String {
        let Self {
            host: app_host,
            port: app_port,
            ..
        } = self;

        format!("{app_host}:{app_port}")
    }

    #[must_use]
    pub fn db_login_user_password(&self) -> &str {
        &self.db_login_user_password
    }

    fn db_url(&self, root: bool) -> String {
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

    #[must_use]
    pub fn db_root_url(&self) -> String {
        self.db_url(true)
    }

    #[must_use]
    pub fn db_login_url(&self) -> String {
        self.db_url(false)
    }

    #[must_use]
    pub fn frontend_token(&self) -> &str {
        &self.frontend_token
    }

    /// # Errors
    pub fn seed_data(&self) -> anyhow::Result<SeedData> {
        let Self {
            seed_data,
            seed_data_path,
            ..
        } = self;

        match (seed_data, seed_data_path) {
            (Some(seed_data), None) => Ok(seed_data.clone()),
            (None, Some(seed_data_path)) => {
                Ok(serde_json::from_str(&fs::read_to_string(seed_data_path)?)?)
            }
            (Some(_), Some(_)) => bail!("`seed_data` and `seed_data_path` are mutually exclusive"),
            (None, None) => bail!("neither `seed_data` nor `seed_data_path` was supplied"),
        }
    }
}

#[derive(Parser)]
#[command(version)]
pub struct Cli {
    #[command(flatten)]
    pub config: Config,
    #[arg(long, env = "SCAMPLERS_LOG_DIR")]
    pub log_dir: Option<Utf8PathBuf>,
}
