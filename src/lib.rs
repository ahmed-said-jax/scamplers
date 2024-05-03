mod config;
mod models;
mod mongo;
mod nf_tenx;
use anyhow::{bail, Context, Result};
use camino::Utf8PathBuf;
use glob::glob;
use models::DataSet;
use mongo::{get_db, upsert_data_sets, upsert_labs};
use nf_tenx::pipeline_metadata_to_data_set;
use std::fs;

pub fn sync_files(config_dir: Utf8PathBuf, files: Vec<Utf8PathBuf>) -> Result<()> {
    let config_path = config_dir.join("scamplers.json");
    let scamplers_config = config::load_scamplers_config(&config_path)
        .with_context(|| format!("could not load configuration from {config_path}"))?;

    let db = get_db(&scamplers_config.db_uri, &scamplers_config.db_name)
        .with_context(|| "could not connect to database")?;

    let collection_names = ["lab", "data_set"];

    for f in files {
        //TODO wrap the below actions in a function that simply takes the file
        let file_stem = f.file_stem().unwrap_or_default();

        if !collection_names.contains(&file_stem) {
            bail!("filename must be one of {:?}, not {f}", collection_names)
        };

        let contents = fs::read_to_string(&f).with_context(|| format!("could not read {f}"))?;

        let load_error_message = format!("could not load data from {f}");
        let upsert_error_message = format!("could not insert data from {f}");

        if file_stem == "data_set" {
            let data_sets: Vec<models::DataSet> =
                serde_json::from_str(&contents).with_context(|| load_error_message)?;
            let collection = db.collection("data_set");

            upsert_data_sets(&collection, data_sets).with_context(|| upsert_error_message)?;
        } else {
            let labs: Vec<models::Lab> =
                serde_json::from_str(&contents).with_context(|| load_error_message)?;
            let collection = db.collection("lab");

            upsert_labs(&collection, labs).with_context(|| upsert_error_message)?;
        };
    }

    Ok(())
}

pub fn sync_nf_tenx(config_dir: Utf8PathBuf) -> Result<()> {
    let config_path = config_dir.join("scamplers.json");
    let scamplers_config = config::load_scamplers_config(&config_path)
        .with_context(|| format!("could not load configuration from {config_path}"))?;

    let db = get_db(&scamplers_config.db_uri, &scamplers_config.db_name)
        .with_context(|| "could not connect to database")?;

    // let pattern = format!("{}/**", scamplers_config.nf_10x_data_dir.to_string());
    let error_message = "could not load pipeline metadata for";

    let mut data_sets: Vec<DataSet> = Vec::new();

    for f in glob(&scamplers_config.nf_10x_pipeline_metadata_pattern).with_context(|| {
        format!(
            "bad glob pattern: {}",
            scamplers_config.nf_10x_pipeline_metadata_pattern
        )
    })? {
        let f = f.with_context(|| error_message)?;

        let data_set = pipeline_metadata_to_data_set(&f).with_context(|| format!("{error_message} {:?}", &f))?;

        data_sets.push(data_set);
    }

    upsert_data_sets(&db.collection("data_set"), data_sets)
}
