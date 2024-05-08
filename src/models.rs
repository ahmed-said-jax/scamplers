use crate::tenx::{format_metrics_summary_file, CellrangerCountMetrics, Pipeline};
use anyhow::{Context, Error, Result};
use camino::Utf8PathBuf;
use chrono::NaiveDate;
use glob::glob;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

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
    pub path: Utf8PathBuf,
    pub libraries: Vec<Library>,
    pub samples: Vec<Sample>,
    pub lab: Lab,

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
    fn metrics_summary_files(&self) -> Result<Vec<Utf8PathBuf>> {
        let data_set_path = self.path.to_string();
        let pattern = format!("{data_set_path}/**/metrics_summary.csv");

        let matches = glob(&pattern)?;
        let mut metrics_summary_files = Vec::new();

        for path in matches {
            let path = Utf8PathBuf::try_from(path?)?;
            metrics_summary_files.push(path);
        }

        Ok(metrics_summary_files)
    }

    fn pipeline(&self) -> Result<Pipeline> {
        let metrics_summary_files = self.metrics_summary_files()?;

        if metrics_summary_files.len() > 1 {
            return Err(Error::msg("not implemented yet"));
        }

        let metrics_summary_file = metrics_summary_files[0].to_owned();
        let mut reader = csv::Reader::from_path(metrics_summary_file)?;

        let cellranger_count_header = csv::StringRecord::from_iter([
            "Estimated Number of Cells",
            "Mean Reads per Cell",
            "Median Genes per Cell",
            "Number of Reads",
            "Valid Barcodes",
            "Sequencing Saturation",
            "Q30 Bases in Barcode",
            "Q30 Bases in RNA Read",
            "Q30 Bases in UMI",
            "Reads Mapped to Genome",
            "Reads Mapped Confidently to Genome",
            "Reads Mapped Confidently to Intergenic Regions",
            "Reads Mapped Confidently to Intronic Regions",
            "Reads Mapped Confidently to Exonic Regions",
            "Reads Mapped Confidently to Transcriptome",
            "Reads Mapped Antisense to Gene",
            "Fraction Reads in Cells",
            "Total Genes Detected",
            "Median UMI Counts per Cell",
        ]);

        if reader.headers()?.to_owned() == cellranger_count_header {
            return Ok(Pipeline::CellrangerCount);
        } else {
            return Err(Error::msg("not supported"));
        }
    }

    fn metrics_summaries<T: DeserializeOwned + Copy>(&self) -> Result<T> {
        let metrics_summary_files = self.metrics_summary_files()?;

        if metrics_summary_files.len() > 1 {
            return Err(Error::msg("not implemented yet"));
        }

        // These 3 lines of code are a hack
        let metrics_summary_file = metrics_summary_files[0].to_owned();
        let (header, records) = format_metrics_summary_file(metrics_summary_file)
            .with_context(|| "something else ba")?;

        let mut rows: Vec<T> = Vec::new();
        for record in records {
            let line: T = record.deserialize(Some(&header))?;
            rows.push(line);
        }

        if rows.len() > 1 {
            return Err(Error::msg("expected just one row"));
        }

        Ok(rows[0])
    }

    pub fn with_metrics(mut self) -> Result<DataSet> {
        let pipeline = self.pipeline()?;

        match pipeline {
            Pipeline::CellrangerCount => {
                let metrics_summaries: CellrangerCountMetrics = self.metrics_summaries()?;
                self.samples[0].estimated_number_of_cells =
                    Some(metrics_summaries.estimated_number_of_cells);
            }
            _ => {
                self.samples[0].estimated_number_of_cells = Some(0);
            }
        };

        Ok(self)
    }
}

// #[cfg(test)]
// mod tests {
//     use tempfile::tempdir;
//     use camino::Utf8PathBuf;

//     use crate::tenx::CellrangerCountMetrics;

//     #[test]
//     fn data_set_with_metrics() {
//         let data_set_path = tempdir().unwrap();
//         let data_set_path = data_set_path.path();

//         let metrics_summary_file = Utf8PathBuf::try_from(data_set_path.join("metrics_summary.csv")).unwrap();
//     }
// }
