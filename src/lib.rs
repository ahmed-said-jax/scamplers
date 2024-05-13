mod models;
pub mod mongo;
mod tenx;
use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use config::Config;
use models::{DataSet, InsertableCollection};
use mongo::get_delivered_data_sets;
use mongo::{upsert_data_sets, upsert_labs};
use mongodb::{bson::doc, sync::{Collection, Database}};
use serde::{Deserialize, Serialize};
use std::{env, fs};

// TODO: the commands here could be parallelized

#[derive(Deserialize, Serialize)]
pub struct ScamplersConfig {
    pub db_name: String,
    pub db_uri: String,
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
    db: Database,
    files: impl IntoIterator<Item = Utf8PathBuf>,
    overwrite_data_sets: bool
) -> Result<()> {
    for f in files {
        let contents = fs::read_to_string(&f).with_context(|| format!("could not read {f}"))?;

        let data: InsertableCollection = serde_json::from_str(&contents)
            .with_context(|| format!("could not load data from {f}"))?;
        let insertion_error = format!("could not insert data from {f}");

        match data {
            InsertableCollection::DataSets(data_sets) => {
                let collection = db.collection("data_set");

                if !overwrite_data_sets {
                    let data_sets: Vec<DataSet> = data_sets.into_iter().filter(|ds| ds.date_delivered.is_none()).collect();
                    upsert_data_sets(&collection, data_sets).with_context(|| insertion_error)?;
                }

                else {
                    upsert_data_sets(&collection, data_sets).with_context(|| insertion_error)?;
                }
            }

            InsertableCollection::Labs(labs) => {
                upsert_labs(&db.collection("lab"), labs).with_context(|| insertion_error)?;
            }
        }
    }

    Ok(())
}

pub fn sync_10x(db: Database) -> Result<()> {
    let collection: Collection<DataSet> = db.collection("data_set");
    let data_sets = get_delivered_data_sets(&collection)?;

    let mut updated_data_sets: Vec<DataSet> = Vec::new();

    for ds in data_sets {
        updated_data_sets.push(ds.with_metrics(None)?);
    }

    upsert_data_sets(&collection, updated_data_sets)?;

    Ok(())
}

// TODO: is rstest worth looking into rstest for pytest-style fixtures?
// I think the way to answer the question is to rewrite these tests using rstest and see which are more ergonomic/easy to read
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
        sync::Collection,
    };

    fn data_files() -> [Utf8PathBuf; 2] {
        let current_file = file!();
        let test_data_dir = Utf8PathBuf::from(current_file)
            .parent()
            .unwrap()
            .join("test_data");

        let files = [
            test_data_dir.join("data_sets.json"),
            test_data_dir.join("labs.json"),
        ];

        files
    }

    #[test]
    fn test_sync_files() -> Result<()> {
        let scamplers_config = ScamplersConfig::load().unwrap();
        let db = get_db(&scamplers_config.db_uri, &"test_sync-files".to_string()).unwrap();
        let files = data_files();

        sync_files(db.clone(), files, true)?;

        let ds_collection: Collection<DataSet> = db.collection("data_set");
        assert_eq!(ds_collection.estimated_document_count(None)?, 2);

        let lab_collection: Collection<Lab> = db.collection("lab");
        assert_eq!(lab_collection.estimated_document_count(None)?, 1);

        db.drop(None)?;

        Ok(())
    }

    #[test]
    fn test_sync_10x() -> Result<()> {
        let scamplers_config = ScamplersConfig::load().unwrap();
        let db = get_db(&scamplers_config.db_uri, &"test_sync-10x".to_string()).unwrap();
        let files = data_files();

        sync_files(db.clone(), files, true)?;
        sync_10x(db.clone())?;

        let collection: Collection<DataSet> = db.collection("data_set");
        let filter = doc! { "samples": { "$all": [ { "$elemMatch": { "estimated_number_of_cells": { "$exists": true } } } ] } };

        let count = collection.count_documents(filter, None)?;
        assert_eq!(count, 2);

        db.drop(None)?;

        Ok(())
    }

    #[test]
    fn test_sync_10x_then_sync_files() -> Result<()> {
        let scamplers_config = ScamplersConfig::load().unwrap();
        let db = get_db(&scamplers_config.db_uri, &"test_sync-10x_then_sync-files".to_string()).unwrap();
        let files = data_files();

        sync_files(db.clone(), files.clone(), true)?;
        sync_10x(db.clone())?;
        sync_files(db.clone(), files, false)?;

        let collection: Collection<DataSet> = db.collection("data_set");
        let filter = doc! { "samples": { "$all": [ { "$elemMatch": { "estimated_number_of_cells": { "$exists": true } } } ] } };

        let count = collection.count_documents(filter, None)?;
        assert_eq!(count, 2);

        db.drop(None)?;

        Ok(())
    }
}
