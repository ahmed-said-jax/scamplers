use time::OffsetDateTime;

use crate::{model::sample_metadata::NewSampleMetadata, string::NonEmptyString};
#[cfg(feature = "backend")]
use scamplers_macros::backend_db_json;

#[cfg_attr(feature = "backend", backend_db_json, serde(rename_all = "UPPERCASE"))]
pub(super) enum MeasurementData {
    Rin {
        #[cfg_attr(feature = "backend", valuable(skip))]
        measured_at: OffsetDateTime,
        #[cfg_attr(feature = "backend", garde(dive))]
        instrument_name: NonEmptyString, // This should be an enum
        #[cfg_attr(feature = "backend", garde(range(min = 1.0, max = 10.0)))]
        value: f32,
    },
    Dv200 {
        #[cfg_attr(feature = "backend", valuable(skip))]
        measured_at: OffsetDateTime,
        #[cfg_attr(feature = "backend", garde(dive))]
        instrument_name: NonEmptyString, // This should be a different enum
        #[cfg_attr(feature = "backend", garde(range(min = 0.0, max = 1.0)))]
        value: f32,
    },
}

#[cfg_attr(
    feature = "backend",
    derive(
        serde::Deserialize,
        garde::Validate,
        valuable::Valuable,
        Debug,
        bon::Builder
    )
)]
pub(super) struct NewSpecimenCore {
    #[cfg_attr(feature = "backend", garde(dive), serde(flatten))]
    pub(super) metadata: NewSampleMetadata,
    #[cfg_attr(feature = "backend", garde(dive), serde(default), builder(default))]
    pub(super) measurements: Vec<MeasurementData>,
}
