use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner},
};

pub(super) trait DevContainer: Sized {
    async fn new() -> anyhow::Result<Self>;
    async fn db_url(&self) -> anyhow::Result<String>;
}

impl DevContainer for ContainerAsync<Postgres> {
    async fn new() -> anyhow::Result<Self> {
        let postgres_version = "17-alpine3.21";

        Ok(Postgres::default()
            .with_host_auth()
            .with_tag(postgres_version)
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
