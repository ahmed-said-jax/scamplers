mod models;
mod mongo;
mod tenx;
use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use config::Config;
use models::{DataSet, InsertableCollection};
use mongo::get_delivered_data_sets;
use mongo::{get_db, upsert_data_sets, upsert_labs};
use mongodb::{bson::doc, sync::Collection};
use serde::{Deserialize, Serialize};
use std::{env, fs};

// TODO: the commands here could be parallelized

#[derive(Deserialize, Serialize)]
pub struct ScamplersConfig {
    db_name: String,
    db_uri: String,
}

impl ScamplersConfig {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().unwrap_or_default();
        let config_path = env::var("SCAMPLERS_CONFIG_PATH")
            .unwrap_or("/sc/service/etc/.config/scamplers".to_string());

        let config = Config::builder()
            .set_default("db_name", "test")?
            .add_source(config::File::with_name(&config_path).required(false))
            .add_source(config::Environment::with_prefix("SCAMPLERS"))
            .build()?;

        config.try_deserialize().with_context(|| format!("could not load configuration from environment and/or file. Fix the fields in {config_path} or fix the corresponding environment variables prefixed by 'SCAMPLERS'"))
    }
}

pub fn sync_files(
    scamplers_config: &ScamplersConfig,
    files: impl IntoIterator<Item = Utf8PathBuf>,
) -> Result<()> {
    let db = get_db(&scamplers_config.db_uri, &scamplers_config.db_name)?;

    for f in files {
        let contents = fs::read_to_string(&f).with_context(|| format!("could not read {f}"))?;

        let data: InsertableCollection = serde_json::from_str(&contents)
            .with_context(|| format!("could not load data from {f}"))?;
        let insertion_error = format!("could not insert data from {f}");

        match data {
            InsertableCollection::DataSets(data_sets) => {
                upsert_data_sets(&db.collection("data_set"), data_sets.into_iter().filter(|ds| ds.date_delivered.is_some()).collect())
                    .with_context(|| insertion_error)?
            }
            InsertableCollection::Labs(labs) => {
                upsert_labs(&db.collection("lab"), labs).with_context(|| insertion_error)?
            }
        }
    }

    Ok(())
}

pub fn sync_10x(scamplers_config: &ScamplersConfig) -> Result<()> {
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


#[cfg(test)]
mod tests {
    use crate::{
        models::{DataSet, Lab},
        mongo::get_db,
        sync_10x, sync_files, ScamplersConfig,
    };
    use anyhow::Result;
    use camino::Utf8PathBuf;
    use mongodb::{
        bson::doc,
        sync::{Collection, Database},
    };

    fn data_dir() -> Utf8PathBuf {
        let current_file = file!();
        Utf8PathBuf::from(current_file)
            .parent()
            .unwrap()
            .join("test_data")
    }

    fn fill_db(db_name: &str) -> (Database, ScamplersConfig) {
        let mut scamplers_config = ScamplersConfig::load().unwrap();
        scamplers_config.db_name = db_name.to_string();

        let test_data_dir = data_dir();
        let files = [
            test_data_dir.join("data_sets.json"),
            test_data_dir.join("labs.json"),
        ];

        sync_files(&scamplers_config, files).unwrap();

        (
            get_db(&scamplers_config.db_uri, &scamplers_config.db_name).unwrap(),
            scamplers_config,
        )
    }

    #[test]
    fn test_sync_files() -> Result<()> {
        let (db, _) = fill_db("test_sync-files");

        let ds_collection: Collection<DataSet> = db.collection("data_set");
        assert_eq!(ds_collection.estimated_document_count(None)?, 2);

        let lab_collection: Collection<Lab> = db.collection("lab");
        assert_eq!(lab_collection.estimated_document_count(None)?, 1);

        db.drop(None)?;

        Ok(())
    }

    #[test]
    fn test_sync_10x() -> Result<()> {
        let (db, scamplers_config) = fill_db("test_sync-10x");
        sync_10x(&scamplers_config)?;

        let collection: Collection<DataSet> = db.collection("data_set");
        let filter = doc! { "samples": { "$all": [ { "$elemMatch": { "estimated_number_of_cells": { "$exists": true } } } ] } };

        let count = collection.count_documents(filter, None)?;
        assert_eq!(count, 2);

        db.drop(None)?;

        Ok(())
    }

    #[test]
    fn test_sync_10x_then_sync_files() -> Result<()> {
        let (_, scamplers_config) = fill_db("test_sync-10x");
        sync_10x(&scamplers_config)?;

        fill_db("test_sync-10x");
        
        test_sync_files()
    }
}
