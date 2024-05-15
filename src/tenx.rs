use crate::models::LibraryType;
use anyhow::{Context, Error, Result};
use csv::{Reader, StringRecord};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use std::collections::HashMap;
use std::io::Read;
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PipelineMetrics {
    CellrangerarcCountMetrics {
        estimated_number_of_cells: f64,
        feature_linkages_detected: f64,
        linked_genes: f64,
        linked_peaks: f64,
        atac_confidently_mapped_read_pairs: f64,
        atac_fraction_of_genome_in_peaks: f64,
        atac_fraction_of_high_quality_fragments_in_cells: f64,
        atac_fraction_of_high_quality_fragments_overlapping_tss: f64,
        atac_fraction_of_high_quality_fragments_overlapping_peaks: f64,
        atac_fraction_of_transposition_events_in_peaks_in_cells: f64,
        atac_mean_raw_read_pairs_per_cell: f64,
        atac_median_high_quality_fragments_per_cell: f64,
        atac_non_nuclear_read_pairs: f64,
        atac_number_of_peaks: f64,
        atac_percent_duplicates: f64,
        atac_q30_bases_in_barcode: f64,
        atac_q30_bases_in_read_1: f64,
        atac_q30_bases_in_read_2: f64,
        atac_q30_bases_in_sample_index_i1: f64,
        atac_sequenced_read_pairs: f64,
        atac_tss_enrichment_score: f64,
        atac_unmapped_read_pairs: f64,
        atac_valid_barcodes: f64,
        gex_fraction_of_transcriptomic_reads_in_cells: f64,
        gex_mean_raw_reads_per_cell: f64,
        gex_median_umi_counts_per_cell: f64,
        gex_median_genes_per_cell: f64,
        gex_percent_duplicates: f64,
        gex_q30_bases_in_umi: f64,
        gex_q30_bases_in_barcode: f64,
        gex_q30_bases_in_read_2: f64,
        gex_reads_mapped_antisense_to_gene: f64,
        gex_reads_mapped_confidently_to_exonic_regions: f64,
        gex_reads_mapped_confidently_to_genome: f64,
        gex_reads_mapped_confidently_to_intergenic_regions: f64,
        gex_reads_mapped_confidently_to_intronic_regions: f64,
        gex_reads_mapped_confidently_to_transcriptome: f64,
        gex_reads_mapped_to_genome: f64,
        gex_reads_with_tso: f64,
        gex_sequenced_read_pairs: f64,
        gex_total_genes_detected: f64,
        gex_valid_umis: f64,
        gex_valid_barcodes: f64,
    },
    CellrangeratacCountMetrics {
        some_other_metric: String,
    },
    CellrangerCountMetrics {
        estimated_number_of_cells: f64,
        mean_reads_per_cell: f64,
        median_genes_per_cell: f64,
        number_of_reads: f64,
        valid_barcodes: f64,
        sequencing_saturation: f64,
        q30_bases_in_barcode: f64,
        q30_bases_in_rna_read: f64,
        q30_bases_in_umi: f64,
        reads_mapped_to_genome: f64,
        reads_mapped_confidently_to_genome: f64,
        reads_mapped_confidently_to_intergenic_regions: f64,
        reads_mapped_confidently_to_intronic_regions: f64,
        reads_mapped_confidently_to_exonic_regions: f64,
        reads_mapped_confidently_to_transcriptome: f64,
        reads_mapped_antisense_to_gene: f64,
        fraction_reads_in_cells: f64,
        total_genes_detected: f64,
        median_umi_counts_per_cell: f64,
    },
    CellrangerVdjMetrics {
        some_vdj_metrics: f64,
    },
    SpacerangerCountMetrics {
        some_spatial_metric: f64,
    },
}

impl PipelineMetrics {
    // TODO: this needs better error-handling
    // TODO: this needs to be modularized significantly so we can test each part of the code
    pub fn from_csv_reader(mut reader: Reader<impl Read>) -> Result<Self> {
        let header = reader.headers()?;
        let header = format_csv_header(header);
        reader.set_headers(header);

        let mut reader = reader.deserialize();

        let no_data_found_error = Error::msg("no data found");

        let record: HashMap<String, String> = reader.next().ok_or(no_data_found_error)??;
        let mut typecast_record: HashMap<String, f64> = HashMap::new();

        for (key, raw_value) in record.into_iter() {
            let value = raw_value_to_f64(&raw_value)?;
            typecast_record.insert(key, value);
        }

        let too_many_rows_found_error =
            Error::msg(format!("expected only one row, found multiple"));
        let next_row = reader.next();
        if next_row.is_some() {
            return Err(too_many_rows_found_error);
        }

        let as_json_value = serde_json::to_value(typecast_record)?;
        Ok(serde_json::from_value(as_json_value)?)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CellrangerMultiMetrics {
    pub category: CellrangerMultiMetricsCategory,
    pub library_type: LibraryType,
    pub grouped_by: String,
    pub group_name: String,
    pub metric_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_value: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum CellrangerMultiMetricsCategory {
    Cells,
    Library,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum CellrangerMultiMetricValueType {
    Number(f64),
    String(String),
}

impl CellrangerMultiMetrics {
    // TODO: this is some of the ugliest code I've ever seen
    pub fn from_csv_reader(mut reader: Reader<impl Read>) -> Result<Vec<Self>> {
        let header = reader.headers()?;
        let header = format_csv_header(header);
        reader.set_headers(header);

        let mut typecast_data: Vec<HashMap<String, Value>> = Vec::new();

        for result in reader.deserialize() {
            let record: HashMap<String, String> = result?;
            let mut typecast_record = HashMap::new();

            for (key, raw_value) in record.into_iter() {
                if key == "metric_value" {
                    let conversion_result = raw_value_to_f64(&raw_value);

                    if let Ok(number) = conversion_result {
                        typecast_record
                            .insert(key, Value::Number(Number::from_f64(number).unwrap()));
                    }
                } else {
                    typecast_record.insert(key, Value::String(raw_value));
                }
            }

            typecast_data.push(typecast_record);
        }

        let as_json_value = serde_json::to_value(typecast_data)?;
        Ok(serde_json::from_value(as_json_value)?)
    }
}

fn format_csv_header(header: &StringRecord) -> StringRecord {
    header
        .iter()
        .map(|column| column.replace(" ", "_").replace("-", "_").to_lowercase())
        .collect()
}

fn raw_value_to_f64(raw_value: &String) -> Result<f64> {
    let parse_error_context = format!("could not coerce {raw_value} into float");

    let mut no_comma = raw_value.replace(",", "");
    let mut value: f64;

    let re = Regex::new(r"^(\d+)\s\(\d{1,2}\.\d+%\)$").unwrap();
    let matches = re.captures(&no_comma);

    if let Some(number) = matches {
        let (_, [value]) = number.extract();
        no_comma = value.to_string();
    }

    if raw_value.contains("%") {
        value = no_comma
            .replace("%", "")
            .parse()
            .context(parse_error_context)?;
        value = value / 100.0;
    } else {
        value = no_comma.parse().context(parse_error_context)?;
    }

    Ok(value)
}
