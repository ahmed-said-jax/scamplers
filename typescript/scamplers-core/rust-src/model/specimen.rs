#[cfg(feature = "typescript")]
use scamplers_macros::frontend_enum;
use time::OffsetDateTime;
use uuid::Uuid;
#[cfg(feature = "backend")]
use {
    scamplers_macros::{backend_db_enum, backend_db_json, backend_insertion},
    scamplers_schema::specimen_measurement,
};
pub mod block;

use crate::string::NonEmptyString;

#[cfg_attr(feature = "backend", backend_db_enum)]
#[cfg_attr(feature = "typescript", frontend_enum)]
pub enum EmbeddingMatrix {
    CarboxymethylCellulose,
    OptimalCuttingTemperatureCompound,
    Paraffin,
}

#[cfg_attr(feature = "backend", backend_db_json, serde(rename_all = "UPPERCASE"))]
pub enum NewMeasurementData {
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

#[cfg_attr(feature = "backend", backend_insertion(specimen_measurement))]
pub struct NewSpecimenMeasurement {
    pub specimen_id: Uuid,
    pub measured_by: Uuid,
    #[cfg_attr(
        feature = "backend",
        garde(dive),
        diesel(skip_insertion),
        serde(flatten)
    )]
    pub data: NewMeasurementData,
}
