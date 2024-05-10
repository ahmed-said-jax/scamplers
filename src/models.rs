use crate::tenx::PipelineMetrics;
use anyhow::{Context, Error, Result};
use camino::Utf8PathBuf;
use chrono::NaiveDate;
use csv::StringRecord;
use glob::glob;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};
use std::{collections::HashMap, fmt::Debug};

// TODO: add validation to all these models

// TODO: we can make this more flexible by accepting a file that is a list of DataSet/Lab, or a file that is just one DataSet/Lab. That will enable parallelization and easier command-line usage
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

// TODO: should this be an enum for different types of libraries?
#[derive(Debug, Deserialize, Serialize)]
pub struct Library {
    pub _id: String,

    #[serde(rename = "type")]
    pub type_: String, // TODO: make this sophisticated and limited

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>, // Create a controlled vocabulary for this

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_cdna_prepared: Option<NaiveDate>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_submitted_to_gt: Option<NaiveDate>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_sequencing_data_returned: Option<NaiveDate>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub atac_confidently_mapped_read_pairs: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub gex_reads_mapped_confidently_to_genome: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reads_mapped_confidently_to_genome: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Sample {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_received: Option<NaiveDate>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub targeted_cell_recovery: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_number_of_cells: Option<f64>,
}

impl DataSet {
    pub fn metrics_summary_files(&self) -> Result<Vec<Utf8PathBuf>> {
        let data_set_path = self.path.to_string();
        let pattern = format!("{data_set_path}/**/*summary.csv");

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
            .map(|column| column.replace(" ", "_").replace("-", "_").to_lowercase())
            .collect();
        reader.set_headers(header);

        let mut metrics: Vec<PipelineMetrics> = Vec::new();
        // TODO: the below processing should be wrapped into a function
        for result in reader.deserialize() {
            let record: HashMap<String, String> = result?;
            let mut formatted_record = Map::new();

            // This loop is such a hack
            for (key, raw_value) in record.iter() {
                let raw_value = raw_value.replace(",", "");
                let mut value: f64;

                if raw_value.contains("%") {
                    value = raw_value.replace("%", "").parse()?;
                    value = value / 100.0;
                } else {
                    value = raw_value.parse()?;
                }

                let value = Number::from_f64(value).unwrap();
                formatted_record.insert(key.to_string(), Value::Number(value));
            }
            let as_json_value = serde_json::to_value(formatted_record)?;
            let metric = serde_json::from_value(as_json_value)?;

            metrics.push(metric)
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
                self.libraries[0].reads_mapped_confidently_to_genome =
                    Some(*reads_mapped_confidently_to_genome); // TODO: should metrics that are actually the same be called the same thing across different types of libraries?

                Ok(self)
            }
            PipelineMetrics::CellrangerarcCountMetrics {
                estimated_number_of_cells,
                feature_linkages_detected,
                linked_genes,
                linked_peaks,
                atac_confidently_mapped_read_pairs,
                atac_fraction_of_genome_in_peaks,
                atac_fraction_of_high_quality_fragments_in_cells,
                atac_fraction_of_high_quality_fragments_overlapping_tss,
                atac_fraction_of_high_quality_fragments_overlapping_peaks,
                atac_fraction_of_transposition_events_in_peaks_in_cells,
                atac_mean_raw_read_pairs_per_cell,
                atac_median_high_quality_fragments_per_cell,
                atac_non_nuclear_read_pairs,
                atac_number_of_peaks,
                atac_percent_duplicates,
                atac_q30_bases_in_barcode,
                atac_q30_bases_in_read_1,
                atac_q30_bases_in_read_2,
                atac_q30_bases_in_sample_index_i1,
                atac_sequenced_read_pairs,
                atac_tss_enrichment_score,
                atac_unmapped_read_pairs,
                atac_valid_barcodes,
                gex_fraction_of_transcriptomic_reads_in_cells,
                gex_mean_raw_reads_per_cell,
                gex_median_umi_counts_per_cell,
                gex_median_genes_per_cell,
                gex_percent_duplicates,
                gex_q30_bases_in_umi,
                gex_q30_bases_in_barcode,
                gex_q30_bases_in_read_2,
                gex_reads_mapped_antisense_to_gene,
                gex_reads_mapped_confidently_to_exonic_regions,
                gex_reads_mapped_confidently_to_genome,
                gex_reads_mapped_confidently_to_intergenic_regions,
                gex_reads_mapped_confidently_to_intronic_regions,
                gex_reads_mapped_confidently_to_transcriptome,
                gex_reads_mapped_to_genome,
                gex_reads_with_tso,
                gex_sequenced_read_pairs,
                gex_total_genes_detected,
                gex_valid_umis,
                gex_valid_barcodes,
            } => {
                self.samples[0].estimated_number_of_cells = Some(*estimated_number_of_cells);

                for lib in &mut self.libraries {
                    if lib.type_ == "Gene Expression" {
                        lib.gex_reads_mapped_confidently_to_genome =
                            Some(*gex_reads_mapped_confidently_to_genome);
                    } else if lib.type_ == "Chromatin Accessibility" {
                        lib.atac_confidently_mapped_read_pairs =
                            Some(*atac_confidently_mapped_read_pairs);
                    }
                }

                Ok(self)
            }
            _ => Err(Error::msg("not implemented other metrics types")),
        }
    }
}
