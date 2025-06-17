use crate::model::{
    sample_metadata::NewSampleMetadata,
    specimen::{EmbeddingMatrix, NewMeasurementData},
};
#[cfg(feature = "typescript")]
use scamplers_macros::frontend_enum;
#[cfg(feature = "backend")]
use {
    scamplers_macros::{backend_db_enum, backend_insertion, backend_with_getters},
    scamplers_schema::specimen,
};

#[cfg_attr(feature = "backend", backend_db_enum)]
#[cfg_attr(feature = "typescript", frontend_enum)]
pub enum BlockPreservationMethod {
    FormaldehydeDerivativeFixation,
}

#[cfg_attr(feature = "backend", backend_db_enum)]
#[cfg_attr(feature = "typescript", frontend_enum)]
pub enum BlockType {
    Block,
}

#[cfg_attr(feature = "backend", backend_insertion(specimen))]
pub struct NewBlock {
    #[cfg_attr(feature = "backend", diesel(skip_insertion), serde(flatten))]
    pub metadata: NewSampleMetadata,
    pub embedded_in: EmbeddingMatrix,
    pub preserved_with: BlockPreservationMethod,
    #[cfg_attr(feature = "backend", garde(dive), diesel(skip_insertion))]
    pub measurements: Vec<NewMeasurementData>,
    #[cfg_attr(feature = "backend", serde(rename = "type"))]
    pub type_: BlockType,
}

#[cfg_attr(feature = "backend", backend_with_getters)]
mod read {
    use crate::model::{
        sample_metadata::SampleMetadataSummary,
        specimen::{
            EmbeddingMatrix,
            block::{BlockPreservationMethod, BlockType},
        },
    };
    use uuid::Uuid;

    #[cfg(feature = "backend")]
    use {scamplers_macros::backend_selection, scamplers_schema::specimen};

    #[cfg_attr(feature = "backend", backend_selection(specimen))]
    pub struct BlockReference {
        id: Uuid,
        link: String,
    }

    #[cfg_attr(feature = "backend", backend_selection(specimen))]
    pub struct BlockSummary {
        #[cfg_attr(feature = "backend", diesel(embed), serde(flatten))]
        reference: BlockReference,
        #[cfg_attr(feature = "backend", diesel(embed), serde(flatten))]
        metadata: SampleMetadataSummary,
        embedded_in: Option<EmbeddingMatrix>,
        preserved_with: Option<BlockPreservationMethod>,
        type_: BlockType,
    }
}
pub use read::*;
