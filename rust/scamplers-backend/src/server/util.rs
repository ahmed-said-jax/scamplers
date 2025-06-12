use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner},
};
use uuid::Uuid;

pub struct DevContainer {
    container: ContainerAsync<Postgres>,
    db_root_password: Option<String>,
}

impl DevContainer {
    /// # Errors
    pub async fn new(container_name: &str, with_password: bool) -> anyhow::Result<Self> {
        let postgres_version = "18beta1-alpine";

        let container = Postgres::default();
        let (container, db_root_password) = if with_password {
            let pass = Uuid::now_v7().to_string();
            (container.with_password(&pass), Some(pass))
        } else {
            (container.with_host_auth(), None)
        };

        let container = container
            .with_tag(postgres_version)
            .with_container_name(container_name)
            .start()
            .await?;

        Ok(Self {
            container,
            db_root_password,
        })
    }

    #[must_use]
    pub fn password(&self) -> Option<&str> {
        self.db_root_password.as_deref()
    }

    /// # Errors
    pub async fn db_host(&self) -> anyhow::Result<String> {
        Ok(self.container.get_host().await?.to_string())
    }

    /// # Errors
    pub async fn db_port(&self) -> anyhow::Result<u16> {
        Ok(self.container.get_host_port_ipv4(5432).await?)
    }

    /// # Errors
    pub async fn db_url(&self) -> anyhow::Result<String> {
        let Self {
            container,
            db_root_password,
        } = self;

        let base = "postgres://postgres";
        let db_spec = format!(
            "{}:{}/postgres",
            container.get_host().await?,
            container.get_host_port_ipv4(5432).await?
        );

        if let Some(pass) = db_root_password {
            Ok(format!("{base}:{pass}@{db_spec}"))
        } else {
            Ok(format!("{base}@{db_spec}"))
        }
    }
}
