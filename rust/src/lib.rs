#![allow(async_fn_in_trait)]

use anyhow::Context;
use axum::Router;
use camino::{Utf8Path, Utf8PathBuf};
use diesel_async::{
    async_connection_wrapper::AsyncConnectionWrapper,
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncConnection, AsyncPgConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use serde::Deserialize;
use tokio::net::TcpListener;

mod api;
pub mod db;
pub mod schema;
mod seed_data;

const TIMEZONE: &str = "America/New_York";
const LOGIN_USER: &str = "login_user";

pub async fn serve_app(config_path: &Utf8Path) -> anyhow::Result<()> {
    let mut app_config =
        AppConfig::from_path(config_path).context("failed to parse configuration file")?;

    run_migrations(&app_config)
        .await
        .context("failed to run database migrations")?;
    tracing::info!("ran database migrations");

    let app_state = AppState::from_config(&mut app_config).context("failed to create app state")?;

    insert_seed_data(&app_config.seed_data_paths, app_state.db_pool.clone()).await.context("failed to insert seed data")?;
    tracing::info!("inserted seed data");

    let app = Router::new()
        .nest("/api", api::router())
        .with_state(app_state);

    let listener = TcpListener::bind(&app_config.server_address)
        .await
        .context(format!("failed to listen on {}", app_config.server_address))?;

    axum::serve(listener, app)
        .await
        .context("failed to serve app")?;
    tracing::info!("scamplers server listening on {}", app_config.server_address);

    Ok(())
}

#[derive(Deserialize)]
struct AppConfig {
    db_url: String, // this url should allow the `scamplers` db user to connect, not root
    seed_data_paths: Vec<Utf8PathBuf>,
    server_address: String,
    #[serde(default)]
    production: bool,
}

impl AppConfig {
    fn from_path(path: &Utf8Path) -> anyhow::Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name(path.as_str()))
            .build()?;

        Ok(config.try_deserialize()?)
    }
}

async fn run_migrations(app_config: &AppConfig) -> anyhow::Result<()> {
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

    let conn = AsyncPgConnection::establish(&app_config.db_url)
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
    production: bool,
}

impl AppState {
    fn from_config(app_config: &mut AppConfig) -> anyhow::Result<Self> {
        let db_config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&app_config.db_url);
        let db_pool = Pool::builder(db_config).build()?;

        let app_state = Self {
            db_pool,
            http_client: reqwest::Client::new(),
            production: app_config.production,
        };

        Ok(app_state)
    }
}

async fn insert_seed_data(
    files: &[Utf8PathBuf],
    db_pool: Pool<AsyncPgConnection>,
) -> anyhow::Result<()> {
    let mut conn = db_pool.get().await?;

    for file in files {
        seed_data::insert(file, &mut conn).await?;
    }

    Ok(())
}
