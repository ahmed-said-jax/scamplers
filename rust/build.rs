use std::{fs, process::Command, str::FromStr};

use camino::Utf8PathBuf;
use regex::Regex;
use serde::Serialize;
use similar::TextDiff;
use testcontainers_modules::{postgres::Postgres, testcontainers::{runners::SyncRunner, Container, ImageExt}};

fn main() {
    println!("cargo::rerun-if-changed=../migrations");
    
    // Start a temporary container for diesel to run migrations against
    let postgres_instance = postgres_container();
    let connection_string = format!("postgres://postgres@{}:{}/postgres",postgres_instance.get_host().unwrap(),postgres_instance.get_host_port_ipv4(5432).unwrap());

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

    // Run migrations one more time, but this time diesel will apply the just-generated patch
    generate_schema(&connection_string);

    // The container is not always cleaned up
    drop(postgres_instance)
}

fn postgres_container() -> Container<Postgres> {
    let docker_compose = include_bytes!("../compose.yaml");
    let docker_compose: serde_json::Value = serde_json::from_slice(docker_compose).unwrap();

    let postgres_version = docker_compose["services"]["db"]["image"].as_str().unwrap().split(":").nth(1).unwrap();

    Postgres::default().with_host_auth().with_tag(postgres_version).start().unwrap()
}

fn generate_schema(connection_string: &str) {
    let mut diesel_setup = Command::new("diesel");
    let args = ["migration", "run", "--database-url", &format!("{connection_string}")];
    diesel_setup.args(args);
    diesel_setup.output().unwrap();
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

        let fixed_line = array_nullable_regex.replace(line, format!(r#"Array<{inner_type}>"#)).to_string();

        schema_patch.push(fixed_line);
    }
    
    let schema_patch = schema_patch.join("\n");
    fs::write(schema_patch_file, &schema_patch).unwrap();

    let diff = TextDiff::from_lines(&schema_str, &schema_patch);
    let diff = diff.unified_diff().context_radius(6).header("src/schema.rs", "src/schema.patch").to_string();

    fs::write("src/schema.patch", &diff).unwrap();
}

// These structs represent just the options we need from the `diesel.toml` file
#[derive(Serialize)]
struct DieselConfig<'a> {
    print_schema: PrintSchema<'a>,
    migrations_directory: MigrationsDirectory<'a>
}

impl<'a> DieselConfig<'a> {
    fn new() -> Self {
        Self {
            print_schema: PrintSchema {
                file: "src/schema.rs",
                custom_type_derives: ["diesel::query_builder::QueryId", "Clone"],
                patch_file: None
            },
            migrations_directory: MigrationsDirectory {dir: "../migrations"}
        }
    }
    fn with_patch_file(mut self) -> Self {
        self.print_schema.patch_file = Some("src/schema.patch");
        self
    }
    fn write(&self) {
        fs::write("diesel.toml", toml::to_string(self).unwrap()).unwrap();
    }
}

#[derive(Serialize)]
struct PrintSchema<'a> {
    file: &'a str,
    custom_type_derives: [&'a str; 2],
    patch_file: Option<&'a str>
}

#[derive(Serialize)]
struct MigrationsDirectory<'a> {
    dir: &'a str
}