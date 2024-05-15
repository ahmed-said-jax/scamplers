use crate::tenx::{CellrangerMultiMetrics, CellrangerMultiMetricsCategory, PipelineMetrics};
use anyhow::{Context, Error, Result};
use camino::Utf8PathBuf;
use chrono::NaiveDate;
use glob::glob;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};

// TODO: add validation to all these models
// especially a data_set, we want to confirm that the number of samples matches the number of metrics files

// TODO: we can make this more flexible by accepting a file that is a list of DataSet/Lab, or a file that is just one DataSet/Lab. That will enable parallelization and easier command-line usage
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum InsertableCollection {
    DataSets(Vec<DataSet>),
    Labs(Vec<Lab>),
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DataSet {
    pub path: Utf8PathBuf,
    pub libraries: Vec<Library>,
    pub samples: Vec<Sample>,
    pub lab_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_delivered: Option<NaiveDate>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_metrics: Option<HashMap<String, Vec<HashMap<String, String>>>>
}

// TODO: should this be an enum for different types of libraries?
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Library {
    pub _id: String,

    #[serde(rename = "type")]
    pub type_: LibraryType,

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
    pub reads_mapped_confidently_to_genome: Option<f64>,
}

// TODO: this doesn't really follow the right design pattern. Instead, it would be better to define an enum that contains each `Library`, with each variant of the enum containing those fields relating to each library type. That is probably more robust than having a shit-ton of fields defined for a generic `Library`
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LibraryType {
    #[serde(rename = "Chromatin Accessibility")]
    ChromatinAccessibility,

    #[serde(rename = "Gene Expression")]
    GeneExpression,

    #[serde(rename = "Multiplexing Capture")]
    MultiplexingCapture,

    #[serde(rename = "Antibody Capture")]
    AntibodyCapture,

    #[serde(rename = "CRISPR Guide Capture")]
    CrisprGuideCapture,

    #[serde(rename = "Multipexing Capture for 3' Cell Multiplexing")]
    MultiplexingCaptureFor3PCellMultiplexing,

    #[serde(rename = "VDJ")]
    Vdj,

    #[serde(rename = "VDJ-T")]
    VdjT,

    #[serde(rename = "VDJ-T-GD")]
    VdjTGd,

    #[serde(rename = "VDJ-B")]
    VdjB,

    #[serde(rename = "Antigen Capture")]
    AntigenCapture,
    
    Custom
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Sample {
    pub name: String,
    pub sanitized_name: String,

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

        if metrics_summary_files.len() < 1 {
            let error = Error::msg(format!("no metrics files matching {pattern} found in path {}", data_set_path));

            return Err(error);
        }

        Ok(metrics_summary_files)
    }

    pub fn metrics_summary(
        &self,
        metrics_summary_files: Option<Vec<Utf8PathBuf>>,
    ) -> Result<PipelineMetrics> {
        let metrics_summary_files = match metrics_summary_files {
            Some(files) => files,
            None => self.metrics_summary_files()?,
        };

        let metrics_summary_file = &metrics_summary_files[0];
        let reader = csv::Reader::from_path(metrics_summary_file)?;
        PipelineMetrics::from_csv_reader(reader)
    }

    pub fn cellranger_multi_metrics_summaries(&self, metrics_summary_files: Option<Vec<Utf8PathBuf>>) -> Result<HashMap<String, Vec<CellrangerMultiMetrics>>> {
        let metrics_summary_files = match metrics_summary_files {
            Some(files) => files,
            None => self.metrics_summary_files()?,
        };

        let mut sample_name_to_metrics = HashMap::new();

        for f in metrics_summary_files.iter() {
            let reader = csv::Reader::from_path(f)?;
            let metrics = CellrangerMultiMetrics::from_csv_reader(reader)?;

            let sample_name = f.parent().unwrap().file_name().unwrap().to_string(); // TODO: Handle these cases explictly, though they are likely to never happen

            sample_name_to_metrics.insert(sample_name, metrics);
        }

        Ok(sample_name_to_metrics)
    }

    pub fn with_metrics(mut self, metrics_summary: Option<PipelineMetrics>) -> Result<Self> {
        let metrics_summary = match metrics_summary {
            Some(metrics) => metrics,
            None => self.metrics_summary(None)?
        };

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
                self.samples[0].estimated_number_of_cells = Some(estimated_number_of_cells);
                self.libraries[0].reads_mapped_confidently_to_genome =
                    Some(reads_mapped_confidently_to_genome); // TODO: should metrics that are actually the same be called the same thing across different types of libraries?

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
                self.samples[0].estimated_number_of_cells = Some(estimated_number_of_cells);

                for lib in &mut self.libraries {
                    match lib.type_ {
                        LibraryType::GeneExpression => {
                            lib.reads_mapped_confidently_to_genome =
                            Some(gex_reads_mapped_confidently_to_genome);
                        }
                        LibraryType::ChromatinAccessibility => {
                            lib.atac_confidently_mapped_read_pairs =
                            Some(atac_confidently_mapped_read_pairs);
                        }
                        _ => () // TODO: is this implicit skipping bad?
                    }
                }
                Ok(self)
            }
            _ => Ok(self)
        }
    }

    pub fn with_cellranger_multi_metrics(mut self, cellranger_multi_metrics_summaries: Option<HashMap<String, Vec<CellrangerMultiMetrics>>>) -> Result<Self> {
        let cellranger_multi_metrics_summaries = match cellranger_multi_metrics_summaries {
            Some(metrics) => metrics,
            None => self.cellranger_multi_metrics_summaries(None)?
        };

        for (sample_name, metrics_summary) in cellranger_multi_metrics_summaries.into_iter() {
            for sample in &mut self.samples {
                if sample.sanitized_name != sample_name {
                    continue
                }

                for row in &metrics_summary {
                    match row.category {
                        CellrangerMultiMetricsCategory::Cells => {
                            if row.metric_name == "Cells" {
                                if let Some(n_cells) = row.metric_value {
                                    sample.estimated_number_of_cells = Some(n_cells);
                                }
                            }
                        }
                        CellrangerMultiMetricsCategory::Library => ()
                    }
                }
            }
        }
        Ok(self)
    }

    pub fn raw_metrics(&self, metrics_summary_files: Option<Vec<Utf8PathBuf>>) -> Result<HashMap<String, Vec<HashMap<String, String>>>> {
        let metrics_summary_files = match metrics_summary_files {
            Some(files) => files,
            None => self.metrics_summary_files()?,
        };

        let mut all_raw_metrics = HashMap::new();

        if metrics_summary_files.len() == 1 {
            let f = &metrics_summary_files[0];
            let mut reader = csv::Reader::from_path(f)?;
            all_raw_metrics.insert(f.canonicalize_utf8()?.to_string(), reader.deserialize().map(|val| val.unwrap()).collect());

            return Ok(all_raw_metrics)
        }

        for f in &metrics_summary_files {
            let mut reader = csv::Reader::from_path(f)?;
            let mut raw_metrics = Vec::new();

            for result in reader.deserialize() {
                raw_metrics.push(result?);
            }

            all_raw_metrics.insert(f.canonicalize_utf8()?.to_string(), raw_metrics);
        }
        
        Ok(all_raw_metrics)
    }

    pub fn with_raw_metrics(mut self, raw_metrics: Option<HashMap<String, Vec<HashMap<String, String>>>>) -> Result<Self> {
        let raw_metrics = match raw_metrics {
            Some(metrics) => metrics,
            None => self.raw_metrics(None)?
        };

        self.raw_metrics = Some(raw_metrics);

        Ok(self)
    }
}
