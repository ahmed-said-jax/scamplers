use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, JsonSchema, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MassUnit {
    Ng,
    Pg,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, Clone)]
#[serde(rename_all = "lowercase")]
pub enum VolumeUnit {
    #[serde(rename = "µl")]
    Mcl,
    Ml,
}

#[derive(Deserialize, Debug, Serialize, JsonSchema, Clone)]
#[serde(rename_all = "lowercase")]
pub enum LengthUnit {
    #[serde(rename = "µl")]
    Mcm,
}
