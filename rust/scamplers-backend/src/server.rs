#![allow(async_fn_in_trait)]
use std::sync::Arc;

use anyhow::Context;
// use auth::{authenticate_api_request, authenticate_browser_request};
use crate::{config::Config, db};
use axum::{Router, routing::get};
use camino::Utf8PathBuf;
use diesel_async::{
    AsyncConnection, AsyncPgConnection, RunQueryDsl,
    async_connection_wrapper::AsyncConnectionWrapper,
    pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use testcontainers_modules::{postgres::Postgres, testcontainers::ContainerAsync};
use tokio::{net::TcpListener, signal, sync::Mutex};
use tower_http::trace::TraceLayer;
use util::DevContainer;
use uuid::Uuid;
mod api;
pub mod auth;
mod util;

/// # Errors
pub async fn serve(mut config: Config, log_dir: Option<Utf8PathBuf>) -> anyhow::Result<()> {
    initialize_logging(log_dir);

    config
        .read_secrets()
        .context("failed to read secrets directory")?;
    let app_addr = config.app_address();

    let mut app_state = AppState::new(config)
        .await
        .context("failed to initialize app state")?;
    tracing::info!("initialized app state");

    let db_root_conn = app_state
        .db_root_conn()
        .await
        .context("failed to connect to database as root")?;

    run_migrations(db_root_conn)
        .await
        .context("failed to run database migrations")?;
    tracing::info!("ran database migrations");

    app_state
        .set_login_user_password()
        .await
        .context("failed to set password for login_user and/or auth_user")?;

    app_state
        .write_seed_data()
        .await
        .context("failed to insert seed data")?;
    tracing::info!("inserted seed data");

    app_state.drop_db_root_pool();

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

    match log_dir {
        None => {
            let dev_test_log_filter = Targets::new()
                .with_target("scamplers_backend", Level::DEBUG)
                .with_target("tower_http", Level::TRACE);
            let log_layer = log_layer.pretty().with_filter(dev_test_log_filter);

            tracing_subscriber::registry().with(log_layer).init();
        }
        Some(path) => {
            let log_writer = tracing_appender::rolling::daily(path, "scamplers.log");
            let prod_log_filter = Targets::new().with_target("scamplers", Level::INFO);
            let log_layer = log_layer
                .json()
                .with_writer(log_writer)
                .with_filter(prod_log_filter);

            tracing_subscriber::registry().with(log_layer).init();
        }
    }
}

#[derive(Clone)]
enum AppState {
    Dev {
        db_pool: Pool<AsyncPgConnection>,
        _pg_container: Arc<ContainerAsync<Postgres>>,
        user_id: Uuid,
        http_client: reqwest::Client,
        config: Arc<Mutex<Config>>,
    },
    Prod {
        db_pool: Pool<AsyncPgConnection>,
        db_root_pool: Option<Pool<AsyncPgConnection>>,
        http_client: reqwest::Client,
        config: Arc<Mutex<Config>>,
    },
}
impl AppState {
    async fn new(config: Config) -> anyhow::Result<Self> {
        let container_err = "failed to start postgres container instance";

        let state = if config.is_dev() {
            let pg_container: ContainerAsync<Postgres> =
                ContainerAsync::new().await.context(container_err)?;
            let db_root_url = pg_container.db_url().await?;

            let mut db_conn = AsyncPgConnection::establish(&db_root_url).await?;
            let user_id = Uuid::now_v7();
            diesel::sql_query(format!(r#"create user "{user_id}" with superuser"#))
                .execute(&mut db_conn)
                .await
                .context("failed to create dev superuser")?;

            let db_config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&db_root_url);
            let db_pool = Pool::builder(db_config).build()?;

            Self::Dev {
                db_pool,
                _pg_container: Arc::new(pg_container),
                user_id,
                http_client: reqwest::Client::new(),
                config: Arc::new(Mutex::new(config)),
            }
        } else {
            let db_config =
                AsyncDieselConnectionManager::<AsyncPgConnection>::new(config.db_login_url());
            let db_pool = Pool::builder(db_config).build()?;

            let db_root_config =
                AsyncDieselConnectionManager::<AsyncPgConnection>::new(config.db_root_url());
            let db_root_pool = Some(Pool::builder(db_root_config).max_size(1).build()?);

            Self::Prod {
                db_pool,
                db_root_pool,
                http_client: reqwest::Client::new(),
                config: Arc::new(Mutex::new(config)),
            }
        };

        Ok(state)
    }

    pub async fn db_conn(
        &self,
    ) -> db::error::Result<diesel_async::pooled_connection::deadpool::Object<AsyncPgConnection>>
    {
        use AppState::{Dev, Prod};

        match self {
            Dev { db_pool, .. } | Prod { db_pool, .. } => Ok(db_pool.get().await?),
        }
    }

    async fn db_root_conn(
        &self,
    ) -> db::error::Result<diesel_async::pooled_connection::deadpool::Object<AsyncPgConnection>>
    {
        use AppState::Prod;

        let Prod { db_root_pool, .. } = self else {
            return self.db_conn().await;
        };

        let Some(db_root_pool) = db_root_pool else {
            return Err(db::error::Error::Other {
                message: "root user connection to database should not be required at this stage"
                    .to_string(),
            });
        };

        Ok(db_root_pool.get().await?)
    }

    // In theory, this should be two separate functions - one that actually does the password setting, and one that
    // constructs the arguments. This is the only time this sequence of events happens, so we can keep it as is.
    // Also, this shouldn't be a method of `AppState`
    async fn set_login_user_password(&self) -> anyhow::Result<()> {
        const LOGIN_USER: &str = "login_user";

        let password = match self {
            AppState::Dev { .. } => Uuid::now_v7().to_string(),
            AppState::Prod { config, .. } => {
                let config = config.lock().await;
                config.db_login_user_password().to_string()
            }
        };

        let mut db_conn = self.db_root_conn().await?;
        diesel::sql_query(format!(
            r#"alter user "{LOGIN_USER}" with password '{password}'"#
        ))
        .execute(&mut db_conn)
        .await?;

        Ok(())
    }

    // TODO: This also shouldn't be a method of `AppState`
    async fn write_seed_data(&self) -> anyhow::Result<()> {
        use AppState::{Dev, Prod};

        let mut db_conn = self.db_root_conn().await?;

        match self {
            Dev {
                http_client,
                config,
                ..
            }
            | Prod {
                http_client,
                config,
                ..
            } => {
                let config = config.lock().await;

                let seed_data = config.seed_data()?;
                seed_data.write(&mut db_conn, http_client.clone()).await
            }
        }
    }

    fn drop_db_root_pool(&mut self) {
        use AppState::{Dev, Prod};

        match self {
            Dev { .. } => (),
            Prod { db_root_pool, .. } => {
                *db_root_pool = None;
            }
        }
    }
}

async fn run_migrations(
    db_conn: diesel_async::pooled_connection::deadpool::Object<AsyncPgConnection>,
) -> anyhow::Result<()> {
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../../db/migrations");

    let mut wrapper: AsyncConnectionWrapper<
        diesel_async::pooled_connection::deadpool::Object<AsyncPgConnection>,
    > = AsyncConnectionWrapper::from(db_conn);

    tokio::task::spawn_blocking(move || {
        wrapper.run_pending_migrations(MIGRATIONS).unwrap();
    })
    .await?;

    Ok(())
}

fn app(app_state: AppState) -> Router {
    api::router()
        .layer(TraceLayer::new_for_http())
        .route("/health", get(async || ()))
        .with_state(app_state)
}

// I don't entirely understand why I need to manually call `drop` here
async fn shutdown_signal(app_state: AppState) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        () = ctrl_c => {drop(app_state);},
        () = terminate => {drop(app_state)},
    }
}
