use crate::tenx::PipelineMetrics;
use anyhow::{Context, Error, Result};
use camino::Utf8PathBuf;
use chrono::NaiveDate;
use csv::StringRecord;
use glob::glob;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum InsertableCollection {
    DataSet(Vec<DataSet>),
    Lab(Vec<Lab>),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Lab {
    pub name: String,
    pub pi: Person,
    pub institution: Institution,
    pub members: Vec<Person>,
    pub delivery_dir: Utf8PathBuf,
}

// TODO: add validation to all these models
// TODO: add defaults and new methods
#[derive(Debug, Deserialize, Serialize)]
pub struct Institution {
    pub name: String,
    pub ror_id: Option<String>,
    pub country: String,
    pub state: Option<String>,
    pub city: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub orcid: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DataSet {
    pub path: Utf8PathBuf,
    pub libraries: Vec<Library>,
    pub samples: Vec<Sample>,
    pub lab_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_delivered: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Library {
    pub _id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_cdna_prepared: Option<NaiveDate>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_submitted_to_gt: Option<NaiveDate>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_sequencing_data_returned: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Sample {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_received: Option<NaiveDate>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub targeted_cell_recovery: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_number_of_cells: Option<u64>,
}

impl DataSet {
    pub fn metrics_summary_files(&self) -> Result<Vec<Utf8PathBuf>> {
        let data_set_path = self.path.to_string();
        let pattern = format!("{data_set_path}/**/metrics_summary.csv");

        let matches =
            glob(&pattern).with_context(|| format!("glob pattern {pattern} is malformed"))?;
        let mut metrics_summary_files = Vec::new();

        for path in matches {
            let path = path?;
            let path = Utf8PathBuf::try_from(path).with_context(|| {
                format!("non-unicode characters found in a path yielded by {pattern}")
            })?;

            metrics_summary_files.push(path);
        }

        Ok(metrics_summary_files)
    }

    pub fn metrics_summaries(
        &self,
        metrics_summary_files: Option<Vec<Utf8PathBuf>>,
    ) -> Result<Vec<PipelineMetrics>> {
        let metrics_summary_files = match metrics_summary_files {
            Some(files) => files,
            None => self.metrics_summary_files()?,
        };

        if metrics_summary_files.len() != 1 {
            return Err(Error::msg(
                "not implemented yet - too many files or no file (this error message will be improved)",
            ));
        }

        let metrics_summary_file = &metrics_summary_files[0];
        let mut reader = csv::Reader::from_path(metrics_summary_file)?;
        let header: StringRecord = reader
            .headers()?
            .iter()
            .map(|column| column.replace(" ", "_").to_lowercase())
            .collect();

        let mut metrics: Vec<PipelineMetrics> = Vec::new();
        for result in reader.records() {
            // TODO: the following three lines are a bit of a hack
            let record = result?;
            let record: StringRecord = record
                .iter()
                .map(|value| value.replace(",", "").replace("%", ""))
                .collect();
            let record: HashMap<String, String> = record.deserialize(Some(&header))?;

            let as_json_value = serde_json::to_value(record)?;
            let metric = serde_json::from_value(as_json_value)?;

            metrics.push(metric);
        }

        Ok(metrics)
    }

    pub fn with_metrics(mut self, metrics_summaries: Option<Vec<PipelineMetrics>>) -> Result<Self> {
        let metrics_summaries = match metrics_summaries {
            Some(metrics) => metrics,
            None => self.metrics_summaries(None)?,
        };

        if metrics_summaries.len() > 1 {
            return Err(Error::msg("not implemented for many summaries"));
        }

        let metrics_summary = &metrics_summaries[0];

        match metrics_summary {
            PipelineMetrics::CellrangerCountMetrics {
                estimated_number_of_cells,
                mean_reads_per_cell,
                median_genes_per_cell,
                number_of_reads,
                valid_barcodes,
                sequencing_saturation,
                q30_bases_in_barcode,
                q30_bases_in_rna_read,
                q30_bases_in_umi,
                reads_mapped_to_genome,
                reads_mapped_confidently_to_genome,
                reads_mapped_confidently_to_intergenic_regions,
                reads_mapped_confidently_to_intronic_regions,
                reads_mapped_confidently_to_exonic_regions,
                reads_mapped_confidently_to_transcriptome,
                reads_mapped_antisense_to_gene,
                fraction_reads_in_cells,
                total_genes_detected,
                median_umi_counts_per_cell,
            } => {
                self.samples[0].estimated_number_of_cells = Some(*estimated_number_of_cells);

                Ok(self)
            }
            _ => Err(Error::msg("not implemented other metrics types")),
        }
    }
}
