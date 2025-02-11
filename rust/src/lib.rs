#![allow(async_fn_in_trait)]
use std::{borrow::Cow, default, fs, path, sync::Arc};

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
use url::Url;
use uuid::Uuid;
use testcontainers_modules::{testcontainers::{Container, ImageExt, runners::SyncRunner}, postgres::Postgres};

mod api;
pub mod db;
pub mod schema;
mod seed_data;
mod web;

const TIMEZONE: &str = "America/New_York";
const LOGIN_USER: &str = "login_user";

pub async fn serve_app(config_path: Option<&Utf8Path>) -> anyhow::Result<()> {
    let app_config = match config_path {
        Some(path) => AppConfig2::from_path(path).context("failed to parse and validate configuration file")?,
        None => AppConfig2::default()
    };

    let app_state = AppState2::from_config(&app_config);

    let app_config = AppConfig::from_path(config_path.unwrap())
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
pub enum AppConfig2 {
    #[default]
    Dev,
    Test {
        auth_config: AuthConfig,
        server_address: Option<String>
    },
    Prod {
        #[garde(pattern("postgres://login_user@.*"))] // Just make sure that the user is the `login_user`
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
    url: Url
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

async fn run_migrations2() {}

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

#[derive(Clone)]
enum AppState2 {
    Dev {
        container: Arc<Container<Postgres>>,
        user_id: Uuid
    },
    Test {
        container: Arc<Container<Postgres>>
    },
    Prod {
        db_pool: Pool<AsyncPgConnection>,
        http_client: reqwest::Client,
        auth_url: Url,
    }
}


fn postgres_container_instance() -> anyhow::Result<Container<Postgres>> {
    use anyhow::Error;

    let docker_compose = include_bytes!("../../compose.yaml");
    let docker_compose: serde_json::Value = serde_json::from_slice(docker_compose)?;

    let e = "failed to parse postgres image tag specifier";

    let postgres_version = docker_compose["services"]["db"]["image"].as_str().ok_or(Error::msg(e))?.split(":").nth(1).ok_or(Error::msg(e))?;

    Ok(Postgres::default().with_host_auth().with_tag(postgres_version).start()?)
}

trait PostgresContainerExt {
    fn database_url(&self) -> anyhow::Result<String>;
}

impl PostgresContainerExt for Container<Postgres> {
    fn database_url(&self) -> anyhow::Result<String> {
        Ok(format!("postgres://postgres@{}:{}/postgres", self.get_host()?, self.get_host_port_ipv4(5432)?))
    }
}


impl AppState2 {
    async fn from_config(app_config: &AppConfig2) -> anyhow::Result<Self> {
        use AppConfig2::*;

        let e = "failed to start postgres container instance";

        match app_config {
            Dev => {
                let postgres_instance = postgres_container_instance().context(e)?;
                let mut conn = AsyncPgConnection::establish(&postgres_instance.database_url()?).await?;
                let user_id = Uuid::new_v4();
                diesel::sql_query(format!("create user {user_id} with superuser")).execute(&mut conn).await.context("failed to create dev superuser")?;

                Ok(Self::Dev { container: Arc::new(postgres_instance), user_id})
            },
            Test {..} => {
                let postgres_instance = postgres_container_instance().context(e)?;
                Ok(Self::Test { container: Arc::new(postgres_instance) })
            },
            Prod { db_url, auth_config, .. } => {
                let db_config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(db_url);
                let db_pool = Pool::builder(db_config).build()?;

                let auth_url = auth_config.url.clone();

                Ok(Self::Prod { db_pool, http_client: reqwest::Client::new(), auth_url })
            }
        }
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
