use serde::{Deserialize, Serialize};

// TODO: add validation for the fields here?
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct CellrangerCountMetrics {
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
    median_umi_counts_per_cell: u64,
}

pub enum Pipeline {
    CellrangerArc,
    CellrangerAtac,
    CellrangerCount,
    CellrangerVdj,
    CellrangerMulti,
    SpacerangerCount
}