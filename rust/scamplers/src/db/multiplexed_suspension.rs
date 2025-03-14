use chrono::NaiveDateTime;
use serde::Deserialize;
use diesel::{prelude::*, pg::Pg};

use crate::schema;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum MultiplexingTagType {
    FlexBarcode,
    TotalSeqA,
    TotalSeqB,
    TotalSeqC,
    #[serde(rename = "ocm")]
    OCM
}

#[derive(Deserialize, Insertable)]
#[diesel(table_name = schema::multiplexed_suspension, check_for_backend(Pg))]
struct NewMultiplexedSuspension {
    legacy_id: String,
    pooled_at: NaiveDateTime,
    notes: Option<Vec<String>>,
}
