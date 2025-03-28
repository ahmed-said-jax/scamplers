use std::{collections::HashMap, fs, str::FromStr};

use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};
use clap::{Args, Parser, Subcommand};

use crate::seed_data::SeedData;

#[derive(Args)]
pub struct Config {
    #[arg(long, env)]
    db_root_user: String,
    #[arg(long, env)]
    db_root_password: String,
    #[arg(long, env)]
    db_login_user_password: String,
    #[arg(long, env)]
    db_host: String,
    #[arg(long, env)]
    db_port: u16,
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
    pub fn from_secrets_dir(dir: &Utf8Path) -> anyhow::Result<Self> {
        // This is a bit of AI genius
        let read_secret =
            |name: &str| fs::read_to_string(dir.join(name)).context(format!("failed to read secret {name}"));

        let config = Self {
            db_root_user: read_secret("db_root_user")?,
            db_root_password: read_secret("db_root_password")?,
            db_login_user_password: read_secret("db_login_user_password")?,
            db_host: read_secret("db_host")?,
            db_port: read_secret("db_port")?,
            db_name: read_secret("db_name")?,
            auth_host: read_secret("auth_host")?,
            auth_port: read_secret("auth_port")?.parse()?,
            app_host: read_secret("app_host")?,
            app_port: read_secret("app_port")?.parse()?,
            seed_data: serde_json::from_str(&read_secret("seed_data")?)?,
        };

        Ok(config)
    }
    pub fn auth_url(&self) -> String {
        let Self {
            auth_host, auth_port, ..
        } = self;

        format!("{auth_host}:{auth_port}")
    }

    pub fn app_url(&self) -> String {
        let Self { app_host, app_port, .. } = self;

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
            format!("{base}{db_root_user}:{db_root_password}}@{db_spec}")
        } else {
            format!("{base}{LOGIN_USER}:{db_login_user_password}}@{db_spec}")
        }
    }

    pub fn db_root_url(&self) -> String {
        self.db_url(true)
    }

    pub fn db_login_url(&self) -> String {
        self.db_url(false)
    }
}

#[derive(Subcommand)]
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
