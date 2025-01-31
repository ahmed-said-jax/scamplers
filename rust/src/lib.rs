#![allow(async_fn_in_trait)]
#![feature(associated_type_defaults)]

use anyhow::Context;
use axum::{extract::State, Router};
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
mod static_files;

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

    tokio::spawn(sync_with_static_files(
        app_config.static_files.clone(),
        app_state.db_pool.clone(),
    ));

    let app = Router::new()
        .nest("/api", api::router())
        .with_state(app_state);

    let listener = TcpListener::bind(&app_config.server_addr)
        .await
        .context(format!("failed to listen on {}", app_config.server_addr))?;

    axum::serve(listener, app)
        .await
        .context("failed to serve app")?;
    tracing::info!("scamplers server listening on {}", app_config.server_addr);

    Ok(())
}

#[derive(Deserialize)]
struct AppConfig {
    db_url: String, // this url should allow the `scamplers` db user to connect, not root
    static_files: Vec<Utf8PathBuf>,
    server_addr: String,
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

async fn sync_with_static_files(
    files: Vec<Utf8PathBuf>,
    db_pool: Pool<AsyncPgConnection>,
) -> anyhow::Result<()> {
    const THIRTY_MINUTES: u64 = 30 * 60;

    let mut db_conn = db_pool.get().await?;
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(THIRTY_MINUTES));

    loop {
        interval.tick().await;
        tracing::info!("synchronizing database with static files");

        static_files::synchronize(&files, &mut db_conn).await;

        tracing::info!("completed synchronization with static files");
    }
}
