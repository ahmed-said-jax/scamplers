use mongodb::sync::Database;
use anyhow::{Error, Result, Context};
use regex::Regex;
use serde::{Deserialize, Serialize};
use crate::models::DataSet;
use camino::Utf8PathBuf;
use glob::{glob, GlobError};
use std::{collections::HashMap, path::PathBuf};
use csv::{Reader, StringRecord};

// TODO: is this the idiomatic way of doing things? Should I be returning an iterator of &str instead of a Vec<String>?
fn get_string_vars(string: &String) -> Result<Vec<String>> {
    let re = Regex::new(r"{(A-Za-z_])}").unwrap();
    let matches = re.find_iter(string).map(|m| m.as_str().to_string()).collect();

    Ok(matches)
}

// TODO: need extra string sanitization
pub fn validate_glob_pattern(db: &Database, pattern: &String) -> Result<()> {
    let vars = get_string_vars(pattern)?;

    for var in vars {
        let (collection, field) = var.split_once('.').ok_or(Error::msg(format!("pattern must contain at least one '.', but {pattern} does not")))?;

        if !db.list_collection_names(None)?.contains(&collection.to_string()) {
            return Err(Error::msg(format!("pattern {pattern} does not contain a valid database collection")))
        }

        // TODO: implement some kind of sophisticated recursive checking to check that the fields are actually fields, as deep as needed
        // TODO: alternatively, make this rigid. The pattern must have a data_set._id in it, and that's that.
    }

    Ok(())
}


pub fn get_metrics_file(data_set: &DataSet, pattern: &str) -> Result<PathBuf> {
    let delivery_dir = data_set.delivery_dir.clone().with_context(|| "no delivery dir set")?;
    let pattern = pattern.replace("{data_set.delivery_dir}", &delivery_dir);

    let matching_paths: Vec<PathBuf> = glob(&pattern).with_context(|| format!("invalid pattern {pattern}"))?.map(|p| p.unwrap()).collect();
    let n_paths = matching_paths.len();

    if n_paths != 1 {
        return Err(Error::msg(format!("{pattern} must match exactly one file, but {n_paths} files were found")))
    }

    Ok(matching_paths[0].clone())
}

// TODO: add validation for the fields here?
#[derive(Debug, Deserialize, Serialize)]
struct CellRangerCountMetrics {
    pub estimated_number_of_cells: u64,
    mean_reads_per_cell: u64,
    median_genes_per_cell: u64,
    number_of_reads: u64,
    valid_barcodes: u8,
    sequencing_saturation: u8,
    q30_bases_in_barcode: u8,
    q30_bases_in_rna_read: u8,
    q30_bases_in_umi: u8,
    reads_mapped_to_genome: u8,
    reads_mapped_confidently_to_genome: u8,
    reads_mapped_confidently_to_intergenic_regions: u8,
    reads_mapped_confidently_to_intronic_regions: u8,
    reads_mapped_confidently_to_exonic_regions: u8,
    reads_mapped_confidently_to_transcriptome: u8,
    reads_mapped_antisense_to_gene: u8,
    fraction_reads_in_cells: u8, 
    total_genes_detected: u64,
    median_umi_counts_per_cell: u64
}

pub enum PipelineMetrics {
    CellRangerCount(CellRangerCountMetrics),
    CellRangerMulti,
    CellRangerVdj,
    CellRangerAtac,
    CellRangerArc,
    SpaceRanger
}

impl PipelineMetrics {
    pub fn from_metrics<T: std::io::Read>(mut csv_reader: Reader<T>) -> Result<PipelineMetrics> {
        let cellranger_count_header = StringRecord::from_iter(["Estimated Number of Cells", "Mean Reads per Cell", "Median Genes per Cell", "Number of Reads", "Valid Barcodes", "Sequencing Saturation", "Q30 Bases in Barcode", "Q30 Bases in RNA Read", "Q30 Bases in UMI", "Reads Mapped to Genome", "Reads Mapped Confidently to Genome", "Reads Mapped Confidently to Intergenic Regions", "Reads Mapped Confidently to Intronic Regions", "Reads Mapped Confidently to Exonic Regions", "Reads Mapped Confidently to Transcriptome", "Reads Mapped Antisense to Gene", "Fraction Reads in Cells", "Total Genes Detected", "Median UMI Counts per Cell"]);
        
        let pipeline: Result<PipelineMetrics> = match csv_reader.headers() {
            cellranger_count_header => {
                let cellranger_count_metrics: Vec<CellRangerCountMetrics> = csv_reader.deserialize().map(|record| record.unwrap()).collect();
                Ok(PipelineMetrics::CellRangerCount(cellranger_count_metrics[0]))
            },
            _ => Ok(PipelineMetrics::CellRangerAtac)
        };

        pipeline
    }
}