use anyhow::Result;
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

// TODO: add validation for the fields here?
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct CellrangerCountMetrics {
    pub estimated_number_of_cells: u64,
    mean_reads_per_cell: u64,
    median_genes_per_cell: u64,
    number_of_reads: u64,
    valid_barcodes: f32,
    sequencing_saturation: f32,
    q30_bases_in_barcode: f32,
    q30_bases_in_rna_read: f32,
    q30_bases_in_umi: f32,
    reads_mapped_to_genome: f32,
    reads_mapped_confidently_to_genome: f32,
    reads_mapped_confidently_to_intergenic_regions: f32,
    reads_mapped_confidently_to_intronic_regions: f32,
    reads_mapped_confidently_to_exonic_regions: f32,
    reads_mapped_confidently_to_transcriptome: f32,
    reads_mapped_antisense_to_gene: f32,
    fraction_reads_in_cells: f32,
    total_genes_detected: u64,
    median_umi_counts_per_cell: u64,
}

pub enum Pipeline {
    CellrangerArc,
    CellrangerAtac,
    CellrangerCount,
    CellrangerVdj,
    CellrangerMulti,
    SpacerangerCount,
}

pub fn format_metrics_summary_file(
    path: Utf8PathBuf,
) -> Result<(csv::StringRecord, Vec<csv::StringRecord>)> {
    let mut reader = csv::Reader::from_path(path)?;

    let header = reader
        .headers()?
        .iter()
        .map(|col| col.replace(" ", "_").to_lowercase())
        .collect();
    let mut formatted_data: Vec<csv::StringRecord> = Vec::new();

    for result in reader.records() {
        let row = result?;
        let record = row.iter().map(|i| i.replace(",", "").replace("%", ""));

        formatted_data.push(record.collect());
    }

    Ok((header, formatted_data))
}
