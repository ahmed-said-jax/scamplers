#![allow(async_fn_in_trait)]
use std::sync::Arc;

use anyhow::{Context, anyhow};
use axum::Router;
use camino::Utf8PathBuf;
use cli::Config;
use diesel_async::{
    AsyncConnection, AsyncPgConnection, RunQueryDsl,
    async_connection_wrapper::AsyncConnectionWrapper,
    pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use seed_data::SeedData;
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner},
};
use tokio::{net::TcpListener, signal};
use tower_http::{services::fs::ServeDir, trace::TraceLayer};
use utils::{DevContainer, ToAddress};
use uuid::Uuid;

mod api;
mod auth;
pub mod cli;
pub mod db;
pub mod schema;
mod seed_data;
mod utils;
mod web;

pub async fn serve_dev_app(host: String, port: u16) -> anyhow::Result<()> {
    serve(None, None, Some((host, port))).await
}

pub async fn serve_prod_app(config: Config, log_dir: Option<Utf8PathBuf>) -> anyhow::Result<()> {
    serve(log_dir, Some(config), None).await
}

async fn serve(
    log_dir: Option<Utf8PathBuf>,
    config: Option<Config>,
    app_addr: Option<(String, u16)>,
) -> anyhow::Result<()> {
    initialize_logging(log_dir);

    let app_addr = match (&config, app_addr) {
        (Some(config), None) => config.app_address(),
        (None, Some(host_port)) => host_port.to_address(),
        _ => {
            return Err(anyhow!("exactly one of `config` or `app_addr` must be supplied"));
        }
    };

    let app_state = AppState2::new(config).await.context("failed to initialize app state")?;
    tracing::info!("initialized app state");

    let db_conn = AsyncPgConnection::establish(&app_state.db_root_url().await?)
        .await
        .context("failed to log into database as root user")?;

    run_migrations(db_conn)
        .await
        .context("failed to run database migrations")?;
    tracing::info!("ran database migrations");

    app_state
        .insert_seed_data()
        .await
        .context("failed to insert seed data")?;
    tracing::info!("inserted seed data");

    let app = app(app_state.clone());

    let listener = TcpListener::bind(&app_addr)
        .await
        .context(format!("failed ot listen on {app_addr}"))?;
    tracing::info!("scamplers listening on {app_addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(app_state))
        .await
        .context("failed to serve app")?;

    Ok(())
}

fn initialize_logging(log_dir: Option<Utf8PathBuf>) {
    use tracing::Level;
    use tracing_subscriber::{filter::Targets, prelude::*};

    let log_layer = tracing_subscriber::fmt::layer();
    let dev_test_log_filter = Targets::new()
        .with_target(env!("CARGO_PKG_NAME"), Level::DEBUG)
        .with_target("tower_http", Level::DEBUG);

    match log_dir {
        None => {
            let log_layer = log_layer.pretty().with_filter(dev_test_log_filter);

            tracing_subscriber::registry().with(log_layer).init();
        }
        Some(path) => {
            let log_writer = tracing_appender::rolling::daily(path, "scamplers.log");
            let prod_log_filter = Targets::new().with_target("scamplers", Level::INFO);
            let log_layer = log_layer.json().with_writer(log_writer).with_filter(prod_log_filter);

            tracing_subscriber::registry().with(log_layer).init();
        }
    }
}

#[derive(Clone)]
enum AppState2 {
    Dev {
        db_pool: Pool<AsyncPgConnection>,
        pg_container: Arc<ContainerAsync<Postgres>>,
        user_id: Uuid,
    },
    Prod {
        db_pool: Pool<AsyncPgConnection>,
        http_client: reqwest::Client,
        config: Arc<Config>,
    },
}
impl AppState2 {
    async fn new(config: Option<Config>) -> anyhow::Result<Self> {
        let container_err = "failed to start postgres container instance";

        let state = match config {
            None => {
                let pg_container: ContainerAsync<Postgres> = ContainerAsync::new().await.context(container_err)?;
                let db_root_url = pg_container.db_url().await?;

                let mut db_conn = AsyncPgConnection::establish(&db_root_url).await?;
                let user_id = Uuid::new_v4();
                diesel::sql_query(format!(r#"create user "{user_id}" with superuser"#))
                    .execute(&mut db_conn)
                    .await
                    .context("failed to create dev superuser")?;

                let db_config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&db_root_url);
                let db_pool = Pool::builder(db_config).build()?;

                Self::Dev {
                    db_pool,
                    pg_container: Arc::new(pg_container),
                    user_id,
                }
            }
            Some(config) => {
                let db_config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&config.db_login_url());
                let db_pool = Pool::builder(db_config).build()?;

                Self::Prod {
                    db_pool,
                    http_client: reqwest::Client::new(),
                    config: Arc::new(config),
                }
            }
        };

        Ok(state)
    }

    async fn db_root_url(&self) -> anyhow::Result<String> {
        use AppState2::*;

        let db_root_url = match self {
            Dev { pg_container, .. } => pg_container.db_url().await?,
            Prod { config, .. } => config.db_root_url(),
        };

        Ok(db_root_url)
    }

    async fn insert_seed_data(&self) -> anyhow::Result<()> {
        use AppState2::*;
        let mut db_conn = self.db_conn().await?;
        let db_conn = &mut db_conn;

        match self {
            Dev { .. } => SeedData::Dev.insert(db_conn, reqwest::Client::new()).await,
            Prod {
                http_client, config, ..
            } => {
                let seed_data = config.seed_data()?;
                match seed_data {
                    Some(seed_data) => seed_data.insert(db_conn, http_client.clone()).await,
                    None => Ok(()),
                }
            }
        }
    }

    async fn db_conn(&self) -> db::Result<diesel_async::pooled_connection::deadpool::Object<AsyncPgConnection>> {
        use AppState2::*;

        match self {
            Dev { db_pool, .. } | Prod { db_pool, .. } => Ok(db_pool.get().await?),
        }
    }
}

async fn run_migrations(db_conn: AsyncPgConnection) -> anyhow::Result<()> {
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

    let mut wrapper: AsyncConnectionWrapper<AsyncPgConnection> = AsyncConnectionWrapper::from(db_conn);

    tokio::task::spawn_blocking(move || {
        wrapper.run_pending_migrations(MIGRATIONS).unwrap();
    })
    .await?;

    Ok(())
}

fn app(app_state: AppState2) -> Router {
    let router = Router::new()
        .nest("/api", api::router())
        .with_state(app_state)
        .layer(TraceLayer::new_for_http());

    match &app_state {
        AppState2::Dev { .. } => router,
        AppState2::Prod { .. } => router.nest_service("/", ServeDir::new("/opt/scamplers-web")),
    }
}

// I don't entirely understand why I need to manually call `drop` here
async fn shutdown_signal(app_state: AppState2) {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {drop(app_state);},
        _ = terminate => {drop(app_state)},
    }
}
