use anyhow::Result;
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct ScamplersConfig {
    pub db_name: String,
    pub db_uri: String,
    pub nf_10x_pipeline_metadata_pattern: String,
}

pub fn load_scamplers_config(path: &Utf8PathBuf) -> Result<ScamplersConfig> {
    let contents = fs::read_to_string(path)?;

    let scamplers_config = serde_json::from_str(&contents)?;

    Ok(scamplers_config)
}
