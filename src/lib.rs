mod models;
mod mongo;
mod tenx;
use anyhow::{bail, Context, Result};
use camino::Utf8PathBuf;
use models::DataSet;
use mongo::get_delivered_data_sets;
use mongo::{get_db, upsert_data_sets, upsert_labs};
use mongodb::{bson::doc, sync::Collection};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct ScamplersConfig {
    db_name: String,
    db_uri: String,
    nf_10x_pipeline_metadata_pattern: String,

    #[serde(rename = "10x_metrics_summary_patterns")]
    tenx_metrix_summary_patterns: Vec<String>,
}

impl ScamplersConfig {
    pub fn from_file(path: &Utf8PathBuf) -> Result<ScamplersConfig> {
        let contents = fs::read_to_string(path)?;

        Ok(serde_json::from_str(&contents)?)
    }
}

pub fn sync_files(scamplers_config: ScamplersConfig, files: Vec<Utf8PathBuf>) -> Result<()> {
    let db = get_db(&scamplers_config.db_uri, &scamplers_config.db_name)?;

    let collection_names = ["lab", "data_set"];

    for f in files {
        //TODO wrap the below actions in a function that simply takes the file as input
        let file_stem = f.file_stem().unwrap_or_default();

        if !collection_names.contains(&file_stem) {
            bail!("filename must be one of {:?}, not {f}", collection_names)
        };

        let contents = fs::read_to_string(&f).with_context(|| format!("could not read {f}"))?;

        let load_error_message = format!("could not load data from {f}");
        let upsert_error_message = format!("could not insert data from {f}");

        // TODO: should we change the design such that each input file actually has to be an instance of a data_set or lab,
        // rather than a list of those? That enables very parallel processing

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

pub fn sync_10x(scamplers_config: ScamplersConfig) -> Result<()> {
    let db = get_db(&scamplers_config.db_uri, &scamplers_config.db_name)?;
    let collection: Collection<DataSet> = db.collection("data_set");
    let data_sets = get_delivered_data_sets(&collection)?;

    let mut updated_data_sets: Vec<DataSet> = Vec::new();

    for ds in data_sets {
        updated_data_sets.push(ds.with_metrics(None)?);
    }

    upsert_data_sets(&collection, updated_data_sets)?;

    Ok(())
}
