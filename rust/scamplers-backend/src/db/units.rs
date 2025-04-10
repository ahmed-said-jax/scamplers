use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, JsonSchema, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum MassUnit {
    Ng,
    Pg,
    #[default]
    Unknown,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum VolumeUnit {
    #[serde(rename = "µl")]
    Mcl,
    Ml,
    #[default]
    Unknown,
}

#[derive(Deserialize, Debug, Serialize, JsonSchema, Clone)]
#[serde(rename_all = "lowercase")]
pub enum LengthUnit {
    #[serde(rename = "µl")]
    Mcm,
}
