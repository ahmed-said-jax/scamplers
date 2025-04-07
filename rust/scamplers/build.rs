use std::{
    env::{self, current_dir},
    fs,
    process::Command,
    str::FromStr,
};

use camino::Utf8PathBuf;
use regex::Regex;
use serde::Serialize;
use similar::TextDiff;
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{Container, ImageExt, runners::SyncRunner},
};

fn main() {
    println!("cargo::rerun-if-changed=migrations");

    let in_docker = option_env!("IN_DOCKER").unwrap_or_default();
    let in_docker = bool::from_str(in_docker).unwrap_or_default();

    // The purpose of this script is to ensure that the db migrations and the rust code are synchronized and compatible
    // with one another, as well as to apply a custom patch to the code generated by the diesel CLI. If we're in docker,
    // it means this is a production build, and we can trust that the person compiling this app has run `compose.sh` at
    // the root of this repo, which runs this script outside of a docker container
    if in_docker {
        return;
    }

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    if manifest_dir.contains("build-scamplers") {
        let path = Utf8PathBuf::from_str(manifest_dir).unwrap().join("../scamplers");
        env::set_current_dir(&path).unwrap();
    }

    dotenvy::dotenv().unwrap_or_default();

    // Since we're not in a docker container, this environment variable should be set
    let postgres_version = option_env!("SCAMPLERS_POSTGRES_VERSION").unwrap_or("17");

    let postgres_instance = Postgres::default()
        .with_host_auth()
        .with_tag(postgres_version)
        .start()
        .expect("unable to start postgres in a container - is the docker daemon running?");
    let connection_string = postgres_instance.connection_string();

    // Create a config with no patch file
    let mut diesel_config = DieselConfig::new();
    diesel_config.write();

    // Run migrations and generate schema.rs file
    generate_schema(&connection_string);

    // Generate a schema patch if needed
    patch_schema();

    // Put the patch file in the config
    diesel_config = diesel_config.with_patch_file();
    diesel_config.write();

    // Run migrations one more time, but this time diesel will apply the
    // just-generated patch
    generate_schema(&connection_string);

    // The container is not always cleaned up
    drop(postgres_instance);

    diesel_config.rm();
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

    let args = ["migration", "run", "--database-url", connection_string];
    diesel_cmd.args(args);

    let output = diesel_cmd.output().unwrap();

    if !output.stderr.is_empty() {
        panic!("{}", String::from_utf8(output.stderr).unwrap());
    }
}

fn patch_schema() {
    let schema = Utf8PathBuf::from_str("src/schema.rs").unwrap();
    let schema_str = fs::read_to_string(schema).unwrap();
    let schema_lines: Vec<&str> = schema_str.split("\n").collect();

    let schema_patch_file = Utf8PathBuf::from_str("src/schema_patch.rs").unwrap();
    let schema_patch = fs::read_to_string(&schema_patch_file).unwrap_or_default();
    let mut schema_patch: Vec<String> = schema_patch.split("\n").map(|s| s.to_string()).collect();

    if schema_lines == schema_patch {
        return;
    }

    schema_patch.clear();

    let array_nullable_regex = Regex::new(r#"Array<Nullable<(\w+)>>"#).unwrap();

    for line in schema_lines {
        let Some(m) = array_nullable_regex.captures(line) else {
            schema_patch.push(line.to_string());
            continue;
        };

        let inner_type = m.get(1).unwrap().as_str();

        let fixed_line = array_nullable_regex
            .replace(line, format!(r#"Array<{inner_type}>"#))
            .to_string();

        schema_patch.push(fixed_line);
    }

    let schema_patch = schema_patch.join("\n");
    fs::write(schema_patch_file, &schema_patch).unwrap();

    let diff = TextDiff::from_lines(&schema_str, &schema_patch);
    let diff = diff
        .unified_diff()
        .context_radius(6)
        .header("src/schema.rs", "src/schema.patch")
        .to_string();

    fs::write("src/schema.patch", &diff).unwrap();
}

// These structs represent just the options we need from the `diesel.toml` file
#[derive(Serialize)]
struct DieselConfig<'a> {
    print_schema: PrintSchema<'a>,
    migrations_directory: MigrationsDirectory<'a>,
}

impl<'a> DieselConfig<'a> {
    fn new() -> Self {
        Self {
            print_schema: PrintSchema {
                file: "src/schema.rs",
                custom_type_derives: ["diesel::query_builder::QueryId", "Clone"],
                patch_file: None,
            },
            migrations_directory: MigrationsDirectory {
                dir: "../../db/migrations",
            },
        }
    }

    fn with_patch_file(mut self) -> Self {
        self.print_schema.patch_file = Some("src/schema.patch");
        self
    }

    fn write(&self) {
        fs::write("diesel.toml", toml::to_string(self).unwrap()).unwrap();
    }

    fn rm(self) {
        fs::remove_file("diesel.toml").unwrap();
    }
}

#[derive(Serialize)]
struct PrintSchema<'a> {
    file: &'a str,
    custom_type_derives: [&'a str; 2],
    patch_file: Option<&'a str>,
}

#[derive(Serialize)]
struct MigrationsDirectory<'a> {
    dir: &'a str,
}
