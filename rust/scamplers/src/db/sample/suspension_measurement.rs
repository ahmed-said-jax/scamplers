use chrono::NaiveDateTime;
use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    serialize::ToSql,
    sql_types,
};
use garde::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::suspension::BiologicalMaterial;
use crate::db::{
    units::{LengthUnit, VolumeUnit},
    utils::DbJson,
};

#[derive(Deserialize, Debug, Serialize, JsonSchema, Clone)]
#[serde(rename_all = "snake_case")]
pub enum CellCountingMethod {
    BrightField,
    Aopi,
    TrypanBlue,
}

#[derive(Deserialize, Serialize, Validate, Debug, FromSqlRow, Default, AsExpression, JsonSchema, Clone)]
#[diesel(sql_type = sql_types::Jsonb)]
#[serde(rename_all = "snake_case", tag = "quantity")]
#[garde(allow_unvalidated)]
pub enum MeasurementData {
    Concentration {
        measured_at: NaiveDateTime,
        instrument_name: String,
        counting_method: CellCountingMethod,
        #[garde(range(min = 0.0))]
        value: f32,
        unit: (BiologicalMaterial, VolumeUnit),
    },
    Volume {
        measured_at: NaiveDateTime,
        #[garde(range(min = 0.0))]
        value: f32,
        unit: VolumeUnit,
    },
    Viability {
        measured_at: NaiveDateTime,
        instrument_name: String,
        #[garde(range(min = 0.0, max = 1.0))]
        value: f32,
    },
    MeanDiameter {
        measured_at: NaiveDateTime,
        instrument_name: String,
        #[garde(range(min = 0.0))]
        value: f32,
        unit: (BiologicalMaterial, LengthUnit),
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
