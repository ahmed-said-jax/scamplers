use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use super::{suspension::NewSuspension, suspension_measurement::MeasurementData};
use crate::schema;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum MultiplexingTagType {
    FlexBarcode,
    TotalSeqA,
    TotalSeqB,
    TotalSeqC,
    #[serde(rename = "ocm")]
    OCM,
}

#[derive(Deserialize)]
struct MultiplexedSuspensionMeasurement {
    #[serde(flatten)]
    data: MeasurementData,
}

#[derive(Deserialize, Insertable)]
#[diesel(table_name = schema::multiplexed_suspension, check_for_backend(Pg))]
struct NewMultiplexedSuspension {
    legacy_id: String,
    pooled_at: NaiveDateTime,
    notes: Option<Vec<String>>,
    #[diesel(skip_insertion)]
    tag_type: MultiplexingTagType,
    #[diesel(skip_insertion)]
    suspensions: Vec<NewSuspension>,
    #[diesel(skip_insertion)]
    preparer_ids: Vec<Uuid>,
    #[diesel(skip_insertion)]
    measurements: Vec<MeasurementData>,
}
