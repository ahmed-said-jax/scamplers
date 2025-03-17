use chrono::NaiveDateTime;
use diesel::{backend::Backend, deserialize::{FromSql, FromSqlRow}, expression::AsExpression, pg::Pg, prelude::*, serialize::ToSql, sql_types};
use garde::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{suspension::NewSuspension, suspension_measurement::MeasurementData};
use crate::{db::{DbEnum, DbJson}, schema};

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

#[derive(Deserialize, Serialize, Validate, FromSqlRow, Default, Debug, AsExpression, JsonSchema)]
#[diesel(sql_type = sql_types::Jsonb)]
#[garde(allow_unvalidated)]
struct MultiplexedSuspensionMeasurement {
    #[serde(flatten)]
    data: MeasurementData,
    taken_after_storage: bool
}
impl DbJson for MultiplexedSuspensionMeasurement {}
impl FromSql<sql_types::Jsonb, Pg> for MultiplexedSuspensionMeasurement {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Self::from_sql_inner(bytes)
    }
}
impl ToSql<sql_types::Jsonb, Pg> for MultiplexedSuspensionMeasurement {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        self.to_sql_inner(out)
    }
}


#[derive(Deserialize, Insertable)]
#[diesel(table_name = schema::multiplexed_suspension_measurement, check_for_backend(Pg))]
struct NewMultiplexedSuspensionMeasurement {
    measured_by: Uuid,
    #[serde(flatten)]
    data: MultiplexedSuspensionMeasurement,
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
    measurements: Vec<NewMultiplexedSuspensionMeasurement>,
}
