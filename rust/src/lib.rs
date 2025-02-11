#![allow(async_fn_in_trait)]
use std::{
    fs,
    sync::Arc,
};

use anyhow::Context;
use axum::Router;
use camino::Utf8Path;
use db::index_sets::IndexSetFileUrl;
use diesel_async::{
    AsyncConnection, AsyncPgConnection, RunQueryDsl,
    async_connection_wrapper::AsyncConnectionWrapper,
    pooled_connection::{
        AsyncDieselConnectionManager,
        deadpool::{Pool},
    },
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use garde::Validate;
use seed_data::download_and_insert_index_sets;
use serde::{Deserialize, Serialize};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner},
};
use tokio::net::TcpListener;
use url::Url;
use uuid::Uuid;

mod api;
pub mod db;
pub mod schema;
mod seed_data;
mod web;

const TIMEZONE: &str = "America/New_York";
const LOGIN_USER: &str = "login_user";
const DOCKER_COMPOSE: &[u8] = include_bytes!("../../compose.yaml");

pub async fn serve_app(config_path: Option<&Utf8Path>) -> anyhow::Result<()> {
    let app_config = match config_path {
        Some(path) => AppConfig2::from_path(path)
            .context("failed to parse and validate configuration file")?,
        None => AppConfig2::default(),
    };

    let app_state = AppState2::new(&app_config)
        .await
        .context("failed to create app state")?;
    tracing::info!("ran database migrations");

    insert_seed_data(app_state.clone(), &app_config)
        .await
        .context("failed to insert seed data")?;
    tracing::info!("inserted seed data");

    let app = app(app_state);

    let addr = app_config.server_address();

    let listener = TcpListener::bind(addr)
        .await
        .context(format!("failed to listen on {addr}"))?;

    axum::serve(listener, app)
        .await
        .context("failed to serve app")?;
    tracing::info!("scamplers server listening on {addr}");

    Ok(())
}

#[derive(Deserialize, Validate, Default)]
#[garde(allow_unvalidated)]
#[serde(tag = "build", rename_all = "snake_case")]
pub enum AppConfig2 {
    #[default]
    Dev,
    Test {
        auth: AuthConfig,
        address: Option<String>,
    },
    Prod {
        db_name: String,
        db_host: String,
        db_port: u16,
        db_login_user_password: String,
        #[garde(dive)]
        index_set_file_urls: Vec<IndexSetFileUrl>,
        auth: AuthConfig,
        address: String,
    },
}

impl AppConfig2 {
    fn from_path(path: &Utf8Path) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        config.validate()?;

        Ok(config)
    }

    fn server_address(&self) -> &str {
        use AppConfig2::*;

        let dev_address = "localhost:8000";

        match self {
            Dev => dev_address,
            Test { address, .. } => address.as_ref().map(|s| s.as_str()).unwrap_or(dev_address),
            Prod { address, .. } => address,
        }
    }
}

#[derive(Deserialize, Validate, Serialize)]
#[garde(allow_unvalidated)]
pub struct AuthConfig {
    url: Url,
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
        _pg_container: Arc<ContainerAsync<Postgres>>,
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

trait DevTestContainer: Sized {
    async fn from_docker_compose() -> anyhow::Result<Self>;
    async fn host_spec(&self) -> anyhow::Result<String>;
}

impl DevTestContainer for ContainerAsync<Postgres> {
    async fn from_docker_compose() -> anyhow::Result<Self> {
        use anyhow::Error;

        let docker_compose: DockerCompose = serde_json::from_slice(DOCKER_COMPOSE)?;

        let e = "failed to parse postgres image tag specifier";

        let postgres_version = docker_compose
            .services
            .db
            .image
            .split(":")
            .nth(1)
            .ok_or(Error::msg(e))?;

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

        match app_config {
            Dev => {
                let pg_container: ContainerAsync<Postgres> = ContainerAsync::from_docker_compose()
                    .await
                    .context(container_err)?;
                let db_url = format!(
                    "postgres://postgres@{}/postgres",
                    pg_container.host_spec().await?
                );

                run_migrations(&db_url).await.context(migrations_err)?;

                let db_config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&db_url);
                let db_pool = Pool::builder(db_config).build()?;

                let mut db_conn = db_pool.get().await?;
                let user_id = Uuid::new_v4();
                diesel::sql_query(format!(r#"create user "{user_id}" with superuser"#))
                    .execute(&mut db_conn)
                    .await
                    .context("failed to create dev superuser")?;

                Ok(Self::Dev {
                    db_pool,
                    _pg_container: Arc::new(pg_container),
                    user_id,
                })
            }
            Test { auth, .. } => {
                let pg_container = ContainerAsync::from_docker_compose()
                    .await
                    .context(container_err)?;
                let db_host_spec = pg_container.host_spec().await?;

                run_migrations(&format!("postgres://postgres@{db_host_spec}/postgres"))
                    .await
                    .context(migrations_err)?;

                let db_config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(format!(
                    "postgres://login_user@{db_host_spec}/postgres"
                ));
                let db_pool = Pool::builder(db_config).build()?;

                Ok(Self::Test {
                    db_pool,
                    _pg_container: Arc::new(pg_container),
                    auth_url: auth.url.clone(),
                })
            }
            Prod {
                db_name,
                db_host,
                db_port,
                db_login_user_password,
                auth,
                ..
            } => {
                let docker_compose: DockerCompose = serde_json::from_slice(DOCKER_COMPOSE)?;
                let secrets: Result<Vec<_>, _> = docker_compose.services.scamplers.secrets[1..3]
                    .iter()
                    .map(|path| fs::read_to_string(format!("/run/secrets/{path}")))
                    .collect();
                let secrets = secrets?;

                let (db_root_username, db_root_password) = (&secrets[0], &secrets[1]);
                run_migrations(&format!(
                    "postgres://{db_root_username}:{db_root_password}@{db_host}:{db_port}/\
                     {db_name}"
                ))
                .await
                .context(migrations_err)?;

                let db_config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(format!(
                    "postgres://login_user:{db_login_user_password}@{db_host}:{db_port}/{db_name}"
                ));
                let db_pool = Pool::builder(db_config).build()?;

                Ok(Self::Prod {
                    db_pool,
                    http_client: reqwest::Client::new(),
                    auth_url: auth.url.clone(),
                })
            }
        }
    }

    async fn db_conn(
        &self,
    ) -> db::Result<diesel_async::pooled_connection::deadpool::Object<AsyncPgConnection>> {
        use AppState2::*;

        match self {
            Dev { db_pool, .. } | Test { db_pool, .. } | Prod { db_pool, .. } => {
                Ok(db_pool.get().await?)
            }
        }
    }
}

async fn run_migrations(db_url: &str) -> anyhow::Result<()> {
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

    let db_conn = AsyncPgConnection::establish(db_url).await?;

    let mut wrapper: AsyncConnectionWrapper<AsyncPgConnection> =
        AsyncConnectionWrapper::from(db_conn);

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
            index_set_file_urls,
            ..
        } => {
            download_and_insert_index_sets(app_state, index_set_file_urls).await?;
        }
        _ => (),
    }

    Ok(())
}

fn app(app_state: AppState2) -> Router {
    Router::new()
        .nest("/api", api::router())
        .with_state(app_state)
}
