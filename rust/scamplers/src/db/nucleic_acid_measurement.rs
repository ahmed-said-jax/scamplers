use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    serialize::ToSql,
    sql_types::{self, SqlType},
};
use garde::Validate;
use serde::{Deserialize, Serialize};

use super::{
    units::{MassUnit, VolumeUnit},
    utils::{DbJson, DefaultNowNaiveDateTime},
};

#[derive(Deserialize, Serialize, Default, Validate, Debug)]
pub struct Concentration {
    #[garde(range(min = 0.0))]
    value: f32,
    #[garde(skip)]
    unit: (MassUnit, VolumeUnit),
}

#[derive(Deserialize, Serialize, Default, Validate, Debug)]
pub struct ElectrophoreticSizingRange(
    #[garde(range(min = 0, max = self.1))] i32,
    #[garde(range(min = self.0, max = 10_000))] i32,
);

#[derive(Deserialize, Serialize, SqlType, AsExpression, Debug, FromSqlRow, Default, Validate)]
#[serde(rename_all = "snake_case", tag = "type")]
#[diesel(sql_type = sql_types::Jsonb)]
#[garde(allow_unvalidated)]
pub enum MeasurementData {
    Electrophoretic {
        #[serde(default)]
        measured_at: DefaultNowNaiveDateTime,
        instrument_name: String,
        #[garde(range(min = 0.0))]
        mean_library_size_bp: f32,
        #[garde(dive)]
        sizing_range: ElectrophoreticSizingRange,
        #[garde(dive)]
        concentration: Concentration,
    },
    Fluorometric {
        #[serde(default)]
        measured_at: DefaultNowNaiveDateTime,
        instrument_name: String,
        #[garde(dive)]
        concentration: Concentration,
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
