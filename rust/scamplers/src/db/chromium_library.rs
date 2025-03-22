use chrono::NaiveDateTime;
use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types::{self, SqlType},
};
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::utils::DbJson;
use crate::{
    db::units::{MassUnit, VolumeUnit},
    schema,
};

#[derive(Deserialize, Serialize, SqlType, AsExpression, Debug, FromSqlRow, Default, Validate)]
#[serde(rename_all = "snake_case", tag = "quantity")]
#[diesel(sql_type = sql_types::Jsonb)]
#[garde(allow_unvalidated)]
enum MeasurementData {
    Concentration {
        measured_at: NaiveDateTime,
        instrument_name: String,
        #[garde(range(min = 1.0))]
        value: f32,
        unit: (MassUnit, VolumeUnit),
    },
    #[default]
    Unknown,
}
impl DbJson for MeasurementData {}
impl FromSql<sql_types::Jsonb, Pg> for MeasurementData {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Self::from_sql_inner(bytes)
    }
}
impl ToSql<sql_types::Jsonb, Pg> for MeasurementData {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        self.to_sql_inner(out)
    }
}

#[derive(Deserialize, Insertable, Validate)]
#[diesel(table_name = schema::chromium_library_measurement, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
struct LibraryMeasurement {
    #[serde(default)]
    library_id: Uuid,
    measured_by: Uuid,
    data: MeasurementData,
}

#[derive(Deserialize, Insertable, Validate)]
#[diesel(table_name = schema::chromium_library, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
pub struct NewChromiumLibrary {
    legacy_id: String,
    cdna_id: Uuid,
    single_index_set_name: Option<String>,
    dual_index_set_name: Option<String>,
    #[garde(range(min = 1))]
    number_of_sample_index_pcr_cycles: i32,
    prepared_at: NaiveDateTime,
    notes: Option<Vec<String>>,
    #[diesel(skip_insertion)]
    measurements: Vec<LibraryMeasurement>,
}
