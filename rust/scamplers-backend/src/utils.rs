use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner},
};

pub trait ToAddress {
    fn to_address(&self) -> String;
}

impl ToAddress for (String, u16) {
    fn to_address(&self) -> String {
        let (host, port) = self;
        format!("{host}:{port}")
    }
}

pub trait DevContainer: Sized {
    async fn new() -> anyhow::Result<Self>;
    async fn db_url(&self) -> anyhow::Result<String>;
}

impl DevContainer for ContainerAsync<Postgres> {
    async fn new() -> anyhow::Result<Self> {
        let postgres_version = option_env!("SCAMPLERS_POSTGRES_VERSION").unwrap_or("17");
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
