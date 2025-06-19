use crate::model::specimen::{block::NewBlock, core::MeasurementData, tissue::NewTissue};
use uuid::Uuid;
#[cfg(feature = "backend")]
use {
    scamplers_macros::{backend_insertion, backend_with_getters},
    scamplers_schema::specimen_measurement,
};

pub mod block;
mod core;
pub mod tissue;

#[cfg_attr(feature = "backend", derive(serde::Deserialize))]
#[cfg_attr(feature = "backend", serde(rename_all = "lowercase", tag = "type"))]
pub enum NewSpecimen {
    Block(NewBlock),
    Tissue(NewTissue),
}

#[cfg_attr(
    feature = "backend",
    backend_insertion(specimen_measurement),
    derive(bon::Builder)
)]
pub struct NewSpecimenMeasurement {
    specimen_id: Uuid,
    measured_by: Uuid,
    #[cfg_attr(
        feature = "backend",
        garde(dive),
        diesel(skip_insertion),
        serde(flatten)
    )]
    data: MeasurementData,
}

#[cfg_attr(feature = "backend", backend_with_getters)]
mod with_getters {
    use crate::model::{
        person::PersonHandle,
        sample_metadata::{SampleMetadata, SampleMetadataSummary},
        specimen::MeasurementData,
    };
    use uuid::Uuid;

    #[cfg(feature = "backend")]
    use {
        scamplers_macros::backend_selection,
        scamplers_schema::{specimen, specimen_measurement},
    };

    #[cfg_attr(feature = "backend", backend_selection(specimen))]
    pub struct SpecimenHandle {
        id: Uuid,
        link: String,
    }

    #[cfg_attr(feature = "backend", backend_selection(specimen))]
    pub struct SpecimenCore {
        #[cfg_attr(feature = "backend", diesel(embed), serde(flatten))]
        handle: SpecimenHandle,
        type_: String,
        embedded_in: Option<String>,
        fixative: Option<String>,
        frozen: bool,
        cryopreserved: bool,
        storage_buffer: Option<String>,
    }

    #[cfg_attr(feature = "backend", backend_selection(specimen))]
    pub struct SpecimenSummary {
        #[cfg_attr(feature = "backend", diesel(embed), serde(flatten))]
        core: SpecimenCore,
        #[cfg_attr(feature = "backend", diesel(embed), serde(flatten))]
        metadata: SampleMetadataSummary,
    }

    #[cfg_attr(feature = "backend", backend_selection(specimen_measurement))]
    struct SpecimenMeasurement {
        #[cfg_attr(feature = "backend", diesel(embed))]
        measured_by: PersonHandle,
        data: MeasurementData,
    }

    #[cfg_attr(feature = "backend", derive(serde::Serialize, bon::Builder))]
    pub struct Specimen {
        #[cfg_attr(feature = "backend", serde(flatten))]
        core: SpecimenCore,
        #[cfg_attr(feature = "backend", serde(flatten))]
        metadata: SampleMetadata,
        measurements: Vec<SpecimenMeasurement>,
    }
}
pub use with_getters::*;
