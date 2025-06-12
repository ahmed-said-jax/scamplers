use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner},
};

pub trait DevContainer: Sized {
    async fn new(container_name: &str) -> anyhow::Result<Self>;
    async fn db_url(&self) -> anyhow::Result<String>;
}

impl DevContainer for ContainerAsync<Postgres> {
    async fn new(container_name: &str) -> anyhow::Result<Self> {
        let postgres_version = "18beta1-alpine";

        Ok(Postgres::default()
            .with_host_auth()
            .with_tag(postgres_version)
            .with_container_name(container_name)
            .start()
            .await?)
    }

    async fn db_url(&self) -> anyhow::Result<String> {
        Ok(format!(
            "postgres://postgres@{}:{}",
            self.get_host().await?,
            self.get_host_port_ipv4(5432).await?
        ))
    }
}
