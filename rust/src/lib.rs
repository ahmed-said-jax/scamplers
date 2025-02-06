#![allow(async_fn_in_trait)]

use std::{fs, io};

use anyhow::Context;
use api::ApiUser;
use axum::{middleware::from_extractor_with_state, Router};
use camino::Utf8Path;
use db::{index_sets::IndexSetFileUrl, Create};
use diesel_async::{
    AsyncConnection, AsyncPgConnection,
    async_connection_wrapper::AsyncConnectionWrapper,
    pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use garde::Validate;
use seed_data::download_and_insert_index_sets;
use serde::Deserialize;
use tokio::net::TcpListener;

mod api;
pub mod db;
pub mod schema;
mod seed_data;
mod web;

const TIMEZONE: &str = "America/New_York";
const LOGIN_USER: &str = "login_user";

pub async fn serve_app(config_path: &Utf8Path) -> anyhow::Result<()> {
    let app_config =
        AppConfig::from_path(config_path).context("failed to parse and validate configuration file")?;

    run_migrations(&app_config)
        .await
        .context("failed to run database migrations")?;
    tracing::info!("ran database migrations");

    let app_state = AppState::from_config(&app_config).context("failed to create app state")?;

    insert_seed_data(app_state.clone(), &app_config)
        .await
        .context("failed to insert seed data")?;
    tracing::info!("inserted seed data");

    let app = app(app_state);

    let listener = TcpListener::bind(&app_config.server_address)
        .await
        .context(format!("failed to listen on {}", app_config.server_address))?;

    axum::serve(listener, app)
        .await
        .context("failed to serve app")?;
    tracing::info!(
        "scamplers server listening on {}",
        app_config.server_address
    );

    Ok(())
}

#[derive(Deserialize, Validate)]
#[garde(allow_unvalidated)]
struct AppConfig {
    db_url: String,
    #[garde(dive)]
    index_set_file_urls: Vec<IndexSetFileUrl>,
    server_address: String,
    #[garde(custom(|auth_url: &Option<String>, _| production_has_auth_url(auth_url, self.production)))]
    auth_url: Option<String>,
    #[serde(default)]
    production: bool,
}

fn production_has_auth_url(auth_url: &Option<String>, production: bool) -> garde::Result {
    if auth_url.is_none() && production {
        return Err(garde::Error::new("auth_url must be supplied for production"));
    }

    Ok(())
}

impl AppConfig {
    fn from_path(path: &Utf8Path) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(path)?;

        Ok(toml::from_str(&contents)?)
    }
}

async fn run_migrations(AppConfig { db_url, .. }: &AppConfig) -> anyhow::Result<()> {
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

    let conn = AsyncPgConnection::establish(db_url)
        .await
        .context("failed to connect to database")?;
    let mut wrapper: AsyncConnectionWrapper<AsyncPgConnection> = AsyncConnectionWrapper::from(conn);

    tokio::task::spawn_blocking(move || {
        wrapper.run_pending_migrations(MIGRATIONS).unwrap();
    })
    .await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    db_pool: Pool<AsyncPgConnection>,
    http_client: reqwest::Client,
    auth_url: Option<String>,
    production: bool,
}

impl AppState {
    fn from_config(AppConfig {db_url, production, auth_url, ..}: &AppConfig) -> anyhow::Result<Self> {
        let db_config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(db_url);
        let db_pool = Pool::builder(db_config).build()?;

        let app_state = Self {
            db_pool,
            http_client: reqwest::Client::new(),
            auth_url: auth_url.clone(),
            production: *production,
        };

        Ok(app_state)
    }
}

// Right now, the only seed data we're inserting is the sample index sets
async fn insert_seed_data(
    app_state: AppState,
    AppConfig { index_set_file_urls: index_set_urls, .. }: &AppConfig,
) -> anyhow::Result<()> {
    download_and_insert_index_sets(app_state, &index_set_urls).await?;

    Ok(())
}

fn app(app_state: AppState) -> Router {
    Router::new().nest("/api", api::router().route_layer(from_extractor_with_state::<ApiUser, AppState>(app_state.clone()))).with_state(app_state)
}
