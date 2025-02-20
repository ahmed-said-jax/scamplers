use chrono::NaiveDateTime;
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Validate)]
#[garde(allow_unvalidated)]
pub (super) struct MeasurementMetadata {
    measured_by: Uuid,
    measured_at: NaiveDateTime
}