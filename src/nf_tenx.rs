use crate::models::{DataSet, Library, Sample};
use anyhow::{Context, Error, Result};
use camino::Utf8PathBuf;
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::{collections::HashMap, path::PathBuf};

#[derive(Deserialize)]
struct Record {
    #[serde(rename = "libraries")]
    library_ids: Vec<String>,
    command: String,     // TODO: add validation with an enum
    sample_name: String, // TODO: add metrics as enum to accommodate different metrics
}

#[derive(Deserialize)]
struct PipelineMetadata {
    record: Record,
    metrics: Vec<HashMap<String, Value>>,
}

fn cellranger_multi_to_samples(cellranger_multi_dir: &Utf8PathBuf) -> Result<Vec<Sample>> {
    let per_sample_outs = cellranger_multi_dir.join("per_sample_outs").read_dir()?;

    let mut samples: Vec<Sample> = Vec::new();

    for sample_dir in per_sample_outs {
        let sample_dir = sample_dir?;

        let metrics_summary_file = sample_dir.path().join("metrics_summary.csv");

        let mut reader = csv::Reader::from_path(&metrics_summary_file)
            .with_context(|| format!("could not read {:?}", metrics_summary_file))?;

        let first_row = reader.records().next().ok_or(Error::msg(format!(
            "no rows found in {:?}",
            metrics_summary_file
        )))??;

        let n_cells_column = 5;

        let n_cells = first_row.get(n_cells_column).ok_or(Error::msg(format!(
            "could not get column {n_cells_column} for {:?}",
            metrics_summary_file
        )))?;

        let n_cells: u32 = n_cells.replace(",", "").parse()?;

        let sample_name = Utf8PathBuf::try_from(sample_dir.path())?
            .file_name()
            .unwrap_or_default()
            .to_string();

        samples.push(Sample {
            name: sample_name,
            date_received: None,
            targeted_cell_recovery: None,
            estimated_number_of_cells: Some(n_cells),
        })
    }

    Ok(samples)
}

// TODO: actually unwrap pipeline metadata into datasets in a more controlled fashion
pub fn pipeline_metadata_to_data_set(pipeline_metadata_file: &PathBuf) -> Result<DataSet> {
    let contents = fs::read_to_string(&pipeline_metadata_file)?;
    let pipeline_metadata: PipelineMetadata = serde_json::from_str(&contents)?;

    let mut updated_samples: Vec<Sample> = Vec::new();

    if pipeline_metadata.record.command == "multi" {
        let parent_dir = pipeline_metadata_file
            .parent()
            .ok_or(Error::msg("parent directory cannot be root"))?;

        let cellranger_multi_dir = parent_dir.join("cellranger-multi");
        let cellranger_multi_dir = Utf8PathBuf::try_from(cellranger_multi_dir)?;

        updated_samples = cellranger_multi_to_samples(&cellranger_multi_dir)?;
    } else {
        for (i, metrics) in pipeline_metadata.metrics.iter().enumerate() {
            let metrics_source = metrics.get("file").ok_or(Error::msg(format!(
                "key 'file' not found in 'metrics' object {i}"
            )))?;
            let metrics_source = metrics_source.as_str().ok_or(Error::msg(format!(
                "could not convert {} to str",
                metrics_source
            )))?;

            if metrics_source == "metrics_summary.csv" {
                let n_cells =
                    metrics
                        .get("Estimated Number of Cells")
                        .ok_or(Error::msg(format!(
                            "estimated number of cells not found in 'metrics' object {i}"
                        )))?;

                let n_cells = n_cells.as_str().ok_or(Error::msg(format!(
                    "could not convert {metrics_source} to str"
                )))?;
                let n_cells: u32 = n_cells.replace(",", "").parse()?;

                let sample = Sample {
                    name: pipeline_metadata.record.sample_name.clone(),
                    date_received: None,
                    targeted_cell_recovery: None,
                    estimated_number_of_cells: Some(n_cells),
                };
                println!("{:#?}", &sample);

                updated_samples.push(sample);

                // println!("{:#?}", &sample);
            }
        }
    }

    let mut libraries: Vec<Library> = Vec::new();

    for lib_id in pipeline_metadata.record.library_ids {
        let library = Library {
            _id: lib_id,
            status: None,
            date_cdna_prepared: None,
            date_submitted_to_gt: None,
            date_sequencing_data_returned: None,
        };
        libraries.push(library);
    }

    let ds = DataSet {
        libraries,
        samples: updated_samples,
        lab_name: None,
        date_delivered: None,
    };

    Ok(ds)
}
