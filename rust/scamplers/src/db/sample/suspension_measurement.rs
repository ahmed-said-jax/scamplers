use chrono::NaiveDateTime;
use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types};
use garde::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::suspension::BiologicalMaterial;

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum VolumeUnit {
    µl,
    Ml,
}

#[derive(Deserialize, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LengthUnit {
    µl,
}

#[derive(Deserialize, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CellCountingMethod {
    BrightField,
    Aopi,
    TrypanBlue,
}

#[derive(Deserialize, Serialize, Validate, Debug, FromSqlRow, Default, AsExpression, JsonSchema)]
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
        instrument_name: String,
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
