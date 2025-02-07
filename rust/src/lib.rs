#![allow(async_fn_in_trait)]

use std::{default, fs};

use anyhow::Context;
use axum::Router;
use camino::Utf8Path;
use db::{
    index_sets::IndexSetFileUrl,
    person::{UserRole, create_user_if_not_exists, grant_roles_to_user},
};
use diesel_async::{
    AsyncConnection, AsyncPgConnection, RunQueryDsl,
    async_connection_wrapper::AsyncConnectionWrapper,
    pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use garde::Validate;
use seed_data::download_and_insert_index_sets;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use uuid::Uuid;

mod api;
pub mod db;
pub mod schema;
mod seed_data;
mod web;

const TIMEZONE: &str = "America/New_York";
const LOGIN_USER: &str = "login_user";

pub async fn serve_app(config_path: Option<&Utf8Path>) -> anyhow::Result<()> {
    let app_config = match (config_path) {
        Some(path) => AppConfig2::from_path(path).context("failed to parse and validate configuration file")?,
        None => AppConfig2::default()
    };

    match (app_config.is_prod(), cfg!(feature = "dev-or-test")) {
        (true, true) => return Err(anyhow::Error::msg("production build must not be built with 'dev-or-test' feature flag")),
        (false, false) => return Err(anyhow::Error::msg("dev or test builds must be built with 'dev-or-test' feature flag")),
        (false, false) => todo!("build a test database and return a connection to it"),
        _ => ()
    };

    let app_config = AppConfig::from_path(config_path)
        .context("failed to parse and validate configuration file")?;

    run_migrations(&app_config)
        .await
        .context("failed to run database migrations")?;
    tracing::info!("ran database migrations");

    let app_state = AppState::from_config(&app_config)
        .await
        .context("failed to create app state")?;

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

#[derive(Deserialize, Validate, Serialize, Default)]
#[garde(allow_unvalidated)]
#[serde(tag = "build", rename_all = "snake_case")]
enum AppConfig2 {
    #[default]
    Dev,
    Test {
        auth_config: AuthConfig,
        server_address: Option<String>
    },
    Prod {
        db_url: String,
        #[garde(dive)]
        index_set_file_urls: Vec<IndexSetFileUrl>,
        auth_config: AuthConfig,
        server_address: String
    }
}

impl AppConfig2 {
    fn from_path(path: &Utf8Path) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        config.validate()?;

        Ok(config)
    }

    fn is_prod(&self) -> bool {
        matches!(self, Self::Prod{..})
    }
}

// Liable to change
#[derive(Deserialize, Validate, Serialize, Default)]
#[garde(allow_unvalidated)]
struct AuthConfig {
    ms_client_id: String,
}

#[derive(Deserialize, Validate)]
#[garde(allow_unvalidated)]
struct AppConfig {
    db_url: String,
    #[garde(dive)]
    index_set_file_urls: Vec<IndexSetFileUrl>,
    server_address: String,
    auth_url: Option<String>,
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
}

impl AppState {
    async fn from_config(
        AppConfig {
            db_url, auth_url, ..
        }: &AppConfig,
    ) -> anyhow::Result<Self> {
        let db_config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(db_url);
        let db_pool = Pool::builder(db_config).build()?;

        // If there's no authentication mechanism, we're testing, so all users are admin
        if auth_url.is_none() {
            let mut conn = db_pool.get().await?;
            diesel::select(create_user_if_not_exists(Uuid::nil()))
                .execute(&mut conn)
                .await?;
            diesel::select(grant_roles_to_user(Uuid::nil(), vec![UserRole::AppAdmin]))
                .execute(&mut conn)
                .await?;
        }

        let app_state = Self {
            db_pool,
            http_client: reqwest::Client::new(),
            auth_url: auth_url.clone(),
        };

        Ok(app_state)
    }
}

// Right now, the only seed data we're inserting is the sample index sets
async fn insert_seed_data(
    app_state: AppState,
    AppConfig {
        index_set_file_urls: index_set_urls,
        ..
    }: &AppConfig,
) -> anyhow::Result<()> {
    download_and_insert_index_sets(app_state, &index_set_urls).await?;

    Ok(())
}

fn app(app_state: AppState) -> Router {
    Router::new()
        .nest("/api", api::router())
        .with_state(app_state)
}
