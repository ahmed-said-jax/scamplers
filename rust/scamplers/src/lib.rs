#![allow(async_fn_in_trait)]
use std::{fs, sync::Arc};

use anyhow::Context;
use axum::Router;
use camino::{Utf8Path, Utf8PathBuf};
use db::index_sets::IndexSetFileUrl;
use diesel::sql_query;
use diesel_async::{
    AsyncConnection, AsyncPgConnection, RunQueryDsl,
    async_connection_wrapper::AsyncConnectionWrapper,
    pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use garde::Validate;
use seed_data::{download_and_insert_index_sets, insert_test_data};
use serde::{Deserialize, Serialize};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner},
};
use tokio::{net::TcpListener, signal};
use url::Url;
use uuid::Uuid;

mod api;
pub mod db;
pub mod schema;
mod seed_data;
mod web;

const LOGIN_USER: &str = "login_user";
const DOCKER_COMPOSE: &[u8] = include_bytes!("../../../compose.yaml");

pub async fn serve_app(config_path: Option<Utf8PathBuf>, log_dir: Option<Utf8PathBuf>) -> anyhow::Result<()> {
    let app_config = match config_path {
        Some(path) => AppConfig2::from_path(&path).context("failed to parse and validate configuration file")?,
        None => AppConfig2::default(),
    };

    initialize_logging(&app_config, &log_dir).context("failed to initialize logging")?;

    let app_state = AppState2::new(&app_config)
        .await
        .context("failed to create app state")?;
    tracing::info!("ran database migrations");

    insert_seed_data(app_state.clone(), &app_config)
        .await
        .context("failed to insert seed data")?;
    tracing::info!("inserted seed data");

    let app = app(app_state.clone());

    let addr = app_config.server_address();

    let listener = TcpListener::bind(addr)
        .await
        .context(format!("failed to listen on {addr}"))?;

    tracing::info!("scamplers server listening on {addr}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(app_state))
        .await
        .context("failed to serve app")?;

    Ok(())
}

#[derive(Deserialize, Validate)]
#[garde(allow_unvalidated)]
struct DbConfig {
    name: String,
    host: String,
    port: u16,
    login_user_password: String,
}

#[derive(Deserialize, Validate)]
#[garde(allow_unvalidated)]
#[serde(tag = "build", rename_all = "snake_case")]
pub enum AppConfig2 {
    Dev {
        #[serde(default = "AppConfig2::dev_server_address")]
        address: String,
    },
    Test {
        db: DbConfig,
        auth_url: Url,
        #[serde(default = "AppConfig2::dev_server_address")]
        address: String,
    },
    Prod {
        db: DbConfig,
        auth_url: Url,
        #[garde(dive)]
        index_set_file_urls: Vec<IndexSetFileUrl>,
        address: String,
    },
}
impl Default for AppConfig2 {
    fn default() -> Self {
        Self::Dev {
            address: Self::dev_server_address(),
        }
    }
}

impl AppConfig2 {
    fn dev_server_address() -> String {
        "localhost:8000".to_string()
    }

    fn from_path(path: &Utf8Path) -> anyhow::Result<Self> {
        let iter = path.read_dir_utf8();
        let contents = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        config.validate()?;

        Ok(config)
    }

    fn server_address(&self) -> &str {
        use AppConfig2::*;

        match self {
            Dev { address, .. } | Test { address, .. } | Prod { address, .. } => address,
        }
    }
}

fn initialize_logging(app_config: &AppConfig2, log_dir: &Option<Utf8PathBuf>) -> anyhow::Result<()> {
    use AppConfig2::*;
    use tracing::Level;
    use tracing_subscriber::{filter::Targets, prelude::*};

    let log_layer = tracing_subscriber::fmt::layer();
    let dev_test_log_filter = Targets::new()
        .with_target(env!("CARGO_PKG_NAME"), Level::DEBUG)
        .with_target("tower_http", Level::DEBUG);

    match (app_config, log_dir) {
        (Dev { .. } | Test { .. }, None) => {
            let log_layer = log_layer.pretty().with_filter(dev_test_log_filter);

            tracing_subscriber::registry().with(log_layer).init();
        }
        (Test { .. }, Some(path)) => {
            let log_writer = tracing_appender::rolling::daily(path, "scamplers.log");
            let log_layer = log_layer
                .json()
                .with_writer(log_writer)
                .with_filter(dev_test_log_filter);

            tracing_subscriber::registry().with(log_layer).init();
        }
        (Prod { .. }, Some(path)) => {
            let log_writer = tracing_appender::rolling::daily(path, "scamplers.log");
            let prod_log_filter = Targets::new().with_target("scamplers", Level::INFO);
            let log_layer = log_layer.json().with_writer(log_writer).with_filter(prod_log_filter);

            tracing_subscriber::registry().with(log_layer).init();
        }
        _ => {
            return Err(anyhow::Error::msg(
                "this combination of configuration and 'log_dir' is not supported",
            ));
        }
    };

    Ok(())
}

#[derive(Clone)]
enum AppState2 {
    Dev {
        db_pool: Pool<AsyncPgConnection>,
        _pg_container: Arc<ContainerAsync<Postgres>>,
        user_id: Uuid,
    },
    Test {
        db_pool: Pool<AsyncPgConnection>,
        auth_url: Url,
    },
    Prod {
        db_pool: Pool<AsyncPgConnection>,
        http_client: reqwest::Client,
        auth_url: Url,
    },
}

#[derive(Deserialize)]
struct DockerCompose {
    services: Services,
}
impl DockerCompose {
    fn read() -> anyhow::Result<Self> {
        Ok(serde_json::from_slice(DOCKER_COMPOSE)?)
    }
    fn db_root(&self) -> anyhow::Result<(String, String)> {
        let Self {
            services:
                Services {
                    scamplers: ScamplersService { secrets, .. },
                    ..
                },
            ..
        } = self;

        let mut db_root = secrets[1..3].into_iter().map(|filename| fs::read_to_string(format!("/run/secrets/{filename}")));

        Ok((db_root.nth(0).unwrap()?, db_root.nth(1).unwrap()?))
    }
}

#[derive(Deserialize)]
struct Services {
    db: DbService,
    scamplers: ScamplersService,
}

#[derive(Deserialize)]
struct DbService {
    image: String,
}

#[derive(Deserialize)]
struct ScamplersService {
    secrets: [String; 4],
}

trait DevContainer: Sized {
    async fn from_docker_compose() -> anyhow::Result<Self>;
    async fn host_spec(&self) -> anyhow::Result<String>;
}

impl DevContainer for ContainerAsync<Postgres> {
    async fn from_docker_compose() -> anyhow::Result<Self> {
        use anyhow::Error;

        let DockerCompose {
            services: Services {
                db: DbService { image },
                ..
            },
            ..
        } = DockerCompose::read()?;

        let err = "failed to parse postgres image tag specifier";

        let postgres_version = image.split(":").nth(1).ok_or(Error::msg(err))?;

        Ok(Postgres::default()
            .with_host_auth()
            .with_tag(postgres_version)
            .start()
            .await?)
    }

    async fn host_spec(&self) -> anyhow::Result<String> {
        Ok(format!(
            "{}:{}",
            self.get_host().await?,
            self.get_host_port_ipv4(5432).await?
        ))
    }
}

impl AppState2 {
    async fn new(app_config: &AppConfig2) -> anyhow::Result<Self> {
        use AppConfig2::*;

        let container_err = "failed to start postgres container instance";
        let migrations_err = "failed to run database migrations";

        let state = match app_config {
            Dev { .. } => {
                let pg_container: ContainerAsync<Postgres> =
                    ContainerAsync::from_docker_compose().await.context(container_err)?;
                let db_root_user_url = format!("postgres://postgres@{}/postgres", pg_container.host_spec().await?);

                run_migrations(&db_root_user_url).await.context(migrations_err)?;

                let mut db_conn = AsyncPgConnection::establish(&db_root_user_url).await?;
                let user_id = Uuid::new_v4();
                diesel::sql_query(format!(r#"create user "{user_id}" with superuser"#))
                    .execute(&mut db_conn)
                    .await
                    .context("failed to create dev superuser")?;

                let db_config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&db_root_user_url);
                let db_pool = Pool::builder(db_config).build()?;

                Self::Dev {
                    db_pool,
                    _pg_container: Arc::new(pg_container),
                    user_id,
                }
            }
            Prod {
                db:
                    DbConfig {
                        name: db_name,
                        host: db_host,
                        port: db_port,
                        login_user_password: db_login_user_password,
                    },
                auth_url,
                ..
            }
            | Test {
                db:
                    DbConfig {
                        name: db_name,
                        host: db_host,
                        port: db_port,
                        login_user_password: db_login_user_password,
                    },
                auth_url,
                ..
            } => {
                let docker_compose = DockerCompose::read()?;

                let (db_root_user, db_root_password) = docker_compose.db_root()?;

                let db_root_user_url =
                    format!("postgres://{db_root_user}:{db_root_password}@{db_host}:{db_port}/{db_name}");
                run_migrations(&db_root_user_url).await.context(migrations_err)?;

                let db_config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(format!(
                    "postgres://{LOGIN_USER}:{db_login_user_password}@{db_host}:{db_port}/{db_name}"
                ));
                let db_pool = Pool::builder(db_config).build()?;

                match app_config {
                    Prod { .. } => Self::Prod {
                        db_pool,
                        http_client: reqwest::Client::new(),
                        auth_url: auth_url.clone(),
                    },
                    Test { .. } => Self::Test { db_pool, auth_url: auth_url.clone() },
                    _ => unreachable!("we already checked for the Dev configuration"),
                }
            }
        };

        Ok(state)
    }

    async fn db_conn(&self) -> db::Result<diesel_async::pooled_connection::deadpool::Object<AsyncPgConnection>> {
        use AppState2::*;

        match self {
            Dev { db_pool, .. } | Test { db_pool, .. } | Prod { db_pool, .. } => Ok(db_pool.get().await?),
        }
    }
}

async fn run_migrations(db_url: &str) -> anyhow::Result<()> {
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

    let db_conn = AsyncPgConnection::establish(db_url).await?;

    let mut wrapper: AsyncConnectionWrapper<AsyncPgConnection> = AsyncConnectionWrapper::from(db_conn);

    tokio::task::spawn_blocking(move || {
        wrapper.run_pending_migrations(MIGRATIONS).unwrap();
    })
    .await?;

    Ok(())
}

// Right now, the only seed data we're inserting is the sample index sets
async fn insert_seed_data(app_state: AppState2, app_config: &AppConfig2) -> anyhow::Result<()> {
    match app_config {
        AppConfig2::Prod {
            index_set_file_urls, ..
        } => download_and_insert_index_sets(app_state, index_set_file_urls).await,
        _ => insert_test_data(app_state)
            .await
            .context("failed to populate database with test data"),
    }
}

fn app(app_state: AppState2) -> Router {
    Router::new().nest("/api", api::router()).with_state(app_state)
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
