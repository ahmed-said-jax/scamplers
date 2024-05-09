mod models;
mod mongo;
mod tenx;
use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use models::{DataSet, InsertableCollection};
use mongo::get_delivered_data_sets;
use mongo::{get_db, upsert_data_sets, upsert_labs};
use mongodb::{bson::doc, sync::Collection};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Deserialize, Serialize)]
pub struct ScamplersConfig {
    db_name: String,
    db_uri: String,
}

pub fn sync_files(scamplers_config: ScamplersConfig, files: Vec<Utf8PathBuf>) -> Result<()> {
    let db = get_db(&scamplers_config.db_uri, &scamplers_config.db_name)?;

    for f in files {
        let contents = fs::read_to_string(&f).with_context(|| format!("could not read {f}"))?;

        let data: InsertableCollection = serde_json::from_str(&contents).with_context(|| format!("could not load data from {f}"))?;
        let insertion_error = format!("could not insert data from {f}");

        match data {
            InsertableCollection::DataSet(data_sets) => upsert_data_sets(&db.collection("data_set"), data_sets).with_context(|| insertion_error)?,
            InsertableCollection::Lab(labs) => upsert_labs(&db.collection("lab"), labs).with_context(|| insertion_error)?
        }
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
