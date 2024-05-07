use camino::Utf8PathBuf;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::tenx::PipelineMetrics;
use anyhow::Result;

// TODO: add validation to all these models
// TODO: add defaults and new methods
#[derive(Debug, Deserialize, Serialize)]
pub struct Institution {
    pub name: String,
    pub ror_id: Option<String>,
    pub country: String,
    pub state: Option<String>,
    pub city: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub orcid: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Lab {
    pub name: String,
    pub pi: Person,
    pub institution: Institution,
    pub members: Vec<Person>,
    pub delivery_dir: Utf8PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DataSet {
    pub delivery_dir: Option<String>,
    pub libraries: Vec<Library>,
    pub samples: Vec<Sample>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub lab_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_delivered: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Library {
    pub _id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_cdna_prepared: Option<NaiveDate>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_submitted_to_gt: Option<NaiveDate>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_sequencing_data_returned: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Sample {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_received: Option<NaiveDate>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub targeted_cell_recovery: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_number_of_cells: Option<u64>,
}


impl DataSet {
    pub fn with_metrics(&mut self, metrics: PipelineMetrics, sample_name: Option<String>) -> Result<&DataSet> {
        match metrics {
            PipelineMetrics::CellRangerCount(cellranger_count_metrics) => {
                self.samples[0].estimated_number_of_cells = Some(cellranger_count_metrics.estimated_number_of_cells);

                Ok(self)
            }

            _ => Ok(self)
        }
    }
}