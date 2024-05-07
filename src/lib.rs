mod models;
mod mongo;
mod nf_tenx;
mod tenx;
use anyhow::{bail, Context, Result};
use camino::Utf8PathBuf;
use glob::{glob, GlobError};
use models::DataSet;
use mongo::{get_db, upsert_data_sets, upsert_labs};
use nf_tenx::pipeline_metadata_to_data_set;
use std::{fs, path::{Path, PathBuf}};
use serde::{Deserialize, Serialize};
use tenx::{get_metrics_file};
use mongodb::{bson::doc, sync::Collection};
use camino::Utf8Path;
use csv::Reader;

use crate::tenx::PipelineMetrics;


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
    let db = get_db(&scamplers_config.db_uri, &scamplers_config.db_name)
        .with_context(|| "could not connect to database")?;

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
        }

        else {
            let labs: Vec<models::Lab> =
                serde_json::from_str(&contents).with_context(|| load_error_message)?;
            let collection = db.collection("lab");

            upsert_labs(&collection, labs).with_context(|| upsert_error_message)?;
        };
    }

    Ok(())
}

pub fn sync_nf_tenx(scamplers_config: ScamplersConfig) -> Result<()> {
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

        let data_set = pipeline_metadata_to_data_set(&f)
            .with_context(|| format!("{error_message} {:?}", &f))?;

        data_sets.push(data_set);
    }

    upsert_data_sets(&db.collection("data_set"), data_sets)
}

pub fn sync_10x(scamplers_config: ScamplersConfig) -> Result<()> {
    let db = get_db(&scamplers_config.db_uri, &scamplers_config.db_name).with_context(|| format!("could not connect to database {} at {}", scamplers_config.db_name, scamplers_config.db_uri))?;

    for pattern in scamplers_config.tenx_metrix_summary_patterns {
        // TODO: factor out data_set-getting into a separate function
        let collection: Collection<DataSet> = db.collection("data_set");
        let mut data_sets: Vec<DataSet> = collection.find(doc! { "date_delivered": "$exists"}, None).with_context(|| "could not get data_sets")?.map(|ds| ds.unwrap()).collect();

        // TODO: probably separate out various concerns and package into modular functions that can be easily tested
        for mut ds in &data_sets {
            let metrics_summary_file = match get_metrics_file(&ds, &pattern) {
                Ok(path) => path,
                Err(error) => {
                    // TODO: proper logging
                    println!("\n{}", error.to_string());
                    continue;
                }
            };



            let mut reader = Reader::from_path(metrics_summary_file).with_context(|| "a usefule error message")?;
            let metrics = PipelineMetrics::from_metrics(reader)?;

            ds = ds.with_metrics(metrics, None)?;
        }

        upsert_data_sets(&collection, data_sets)?;
    }
    
    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use std::{env, fs};

//     use camino::Utf8PathBuf;
//     use dotenvy::dotenv;
//     use tempfile::tempdir;
//     use anyhow::Result;
//     use super::{sync_files, config::ScamplersConfig, models::{DataSet, Lab, Library, Sample}};

//     fn config_dir() -> Result<Utf8PathBuf> {
//         dotenv().unwrap_or_default();

//         let db_name = "test".to_string();
//         let db_uri = env::var("SCAMPLERS_TEST_DB_URI")?;
//         let nf_10x_pipeline_metadata_pattern = "/Users/saida/work/10x_data/*/10x-genomics/*/pipeline-metadata.json".to_string(); // TODO: change this to something in the tests directory

//         let dir = Utf8PathBuf::from_path_buf(tempdir()?.path().to_path_buf()).unwrap();
//         let scamplers_config_path = dir.join("scamplers.json");

//         let scamplers_config = ScamplersConfig {db_name, db_uri, nf_10x_pipeline_metadata_pattern};

//         fs::write(scamplers_config_path, serde_json::to_string(&scamplers_config)?)?;

//         Ok(dir)
//     }

//     fn data_sets() -> Vec<DataSet> {
//         let lib0 = Library { _id: "SC9900000".to_string(), status: None, date_cdna_prepared: None, date_sequencing_data_returned: None, date_submitted_to_gt: None};
//         let lib1 = Library { _id: "SC9900001".to_string(), status: None, date_cdna_prepared: None, date_sequencing_data_returned: None, date_submitted_to_gt: None};

//         let sample0 = Sample { name: "sample0".to_string(), date_received: None, targeted_cell_recovery: None, estimated_number_of_cells: None};
//         let sample1 = Sample { name: "sample1".to_string(), date_received: None, targeted_cell_recovery: None, estimated_number_of_cells: None};
        
//         let ds0 = DataSet {libraries: vec![lib0], samples: vec![sample0], lab_name: Some("Ahmed Said Lab".to_string()), date_delivered: None};
//         let ds1 = DataSet {libraries: vec![lib1], samples: vec![sample1], lab_name: Some("Ahmed Said Lab".to_string()), date_delivered: None};

//         vec![ds0, ds1]
//     }

//     #[test]
//     fn test_sync_files() {
//         let data_sets 
//         let ds1 = 
//         let result = sync_files(config_dir(), files);
        

//     }

// }