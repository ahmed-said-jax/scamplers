use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

// TODO: the repetition here can be improved by writing a custom deserializer
#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PipelineMetrics {
    CellrangerArcMetrics {
        some_metric: String,
    },
    CellrangerAtacMetrics {
        some_other_metric: String,
    },
    CellrangerCountMetrics {
        #[serde_as(deserialize_as = "DisplayFromStr")]
        estimated_number_of_cells: u64,

        #[serde_as(deserialize_as = "DisplayFromStr")]
        mean_reads_per_cell: u64,

        #[serde_as(deserialize_as = "DisplayFromStr")]
        median_genes_per_cell: u64,

        #[serde_as(deserialize_as = "DisplayFromStr")]
        number_of_reads: u64,

        #[serde_as(deserialize_as = "DisplayFromStr")]
        valid_barcodes: f32,

        #[serde_as(deserialize_as = "DisplayFromStr")]
        sequencing_saturation: f32,

        #[serde_as(deserialize_as = "DisplayFromStr")]
        q30_bases_in_barcode: f32,

        #[serde_as(deserialize_as = "DisplayFromStr")]
        q30_bases_in_rna_read: f32,

        #[serde_as(deserialize_as = "DisplayFromStr")]
        q30_bases_in_umi: f32,

        #[serde_as(deserialize_as = "DisplayFromStr")]
        reads_mapped_to_genome: f32,

        #[serde_as(deserialize_as = "DisplayFromStr")]
        reads_mapped_confidently_to_genome: f32,

        #[serde_as(deserialize_as = "DisplayFromStr")]
        reads_mapped_confidently_to_intergenic_regions: f32,

        #[serde_as(deserialize_as = "DisplayFromStr")]
        reads_mapped_confidently_to_intronic_regions: f32,

        #[serde_as(deserialize_as = "DisplayFromStr")]
        reads_mapped_confidently_to_exonic_regions: f32,

        #[serde_as(deserialize_as = "DisplayFromStr")]
        reads_mapped_confidently_to_transcriptome: f32,

        #[serde_as(deserialize_as = "DisplayFromStr")]
        reads_mapped_antisense_to_gene: f32,

        #[serde_as(deserialize_as = "DisplayFromStr")]
        fraction_reads_in_cells: f32,

        #[serde_as(deserialize_as = "DisplayFromStr")]
        total_genes_detected: u64,

        #[serde_as(deserialize_as = "DisplayFromStr")]
        median_umi_counts_per_cell: u64,
    },
}
