use std::{
    env::{self, current_dir},
    fs,
    process::Command,
    str::FromStr,
};

use camino::Utf8PathBuf;
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{Container, ImageExt, runners::SyncRunner},
};

fn main() {
    println!("cargo::rerun-if-changed=../../db/migrations");

    dotenvy::dotenv().unwrap_or_default();

    let in_docker = option_env!("IN_DOCKER").unwrap_or_default();
    let in_docker = bool::from_str(in_docker).unwrap_or_default();

    if in_docker {
        return;
    }
    let postgres_version = option_env!("SCAMPLERS_POSTGRES_VERSION").unwrap_or("17-alpine");

    let postgres_instance = Postgres::default()
        .with_host_auth()
        .with_tag(postgres_version)
        .start()
        .expect("unable to start postgres in a container - is the docker daemon running?");
    let connection_string = postgres_instance.connection_string();

    // Run migrations and generate schema.rs file
    generate_schema(&connection_string);

    // The container is not always cleaned up
    drop(postgres_instance);
}

trait BuildDbContainer {
    fn connection_string(&self) -> String;
}
impl BuildDbContainer for Container<Postgres> {
    fn connection_string(&self) -> String {
        format!(
            "postgres://postgres@{}:{}/postgres",
            self.get_host().unwrap(),
            self.get_host_port_ipv4(5432).unwrap()
        )
    }
}

fn generate_schema(connection_string: &str) {
    let mut diesel_cmd = Command::new("diesel");

    let args = [
        "migration",
        "run",
        "--database-url",
        connection_string,
        "--migration-dir",
        "../../db/migrations",
    ];
    diesel_cmd.args(args);

    let output = diesel_cmd.output().unwrap();

    if !output.stderr.is_empty() {
        panic!("{}", String::from_utf8(output.stderr).unwrap());
    }
}
