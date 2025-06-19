use crate::model::{sample_metadata::NewSampleMetadata, specimen::NewSpecimenMeasurement};
use uuid::Uuid;
#[cfg(feature = "backend")]
use {scamplers_macros::backend_insertion, scamplers_schema::specimen};

#[cfg_attr(feature = "backend", backend_insertion(specimen))]
pub struct NewSpecimenCommon {
    #[cfg_attr(
        feature = "backend",
        diesel(skip_insertion),
        serde(flatten),
        garde(dive)
    )]
    pub(super) metadata: NewSampleMetadata,
    #[cfg_attr(feature = "backend", serde(skip))]
    pub(super) metadata_id: Uuid,
    #[cfg_attr(
        feature = "backend",
        diesel(skip_insertion),
        garde(dive),
        serde(default)
    )]
    pub(super) measurements: Vec<NewSpecimenMeasurement>,
}
