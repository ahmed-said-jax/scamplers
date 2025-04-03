use std::{collections::HashMap, str::FromStr};

use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types::{self, SqlType},
};
use diesel_async::RunQueryDsl;
use garde::Validate;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::NewDatasetMetadata;
use crate::{
    db::{
        Create,
        utils::{BelongsToExt, DbJson},
    },
    schema::{self, chromium_dataset},
};

#[derive(Deserialize, Validate, Serialize, Debug)]
struct SingleRowCsv {
    #[garde(pattern(r#"^\w*summary\.csv$"#))]
    filename: String,
    #[garde(skip)]
    #[serde(deserialize_with = "deserialize_10x_single_row_csv")]
    contents: HashMap<String, Value>,
}

#[derive(Deserialize, Validate, Serialize, Debug)]
struct MultiRowCsv {
    #[garde(pattern(r#"^\w+/\w+summary\.csv$"#))]
    filename: String,
    #[garde(skip)]
    #[serde(deserialize_with = "deserialize_10x_multi_row_csv")]
    contents: Vec<HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize, AsExpression, SqlType, Debug, FromSqlRow, Default)]
#[serde(untagged)]
#[diesel(sql_type = sql_types::Jsonb)]
enum ParsedMetricsFile {
    SingleRowCsv(SingleRowCsv),
    MultiRowCsv(MultiRowCsv),
    Json(Value),
    #[default]
    Unknown,
}
impl DbJson for ParsedMetricsFile {}
// For convenience, the trait DbJson requires both `FromSql` and `ToSql` implementations, but we don't actually want to
// use the `FromSql` implmentation as it will give us an untagged enum, which is slow to deserialize
impl FromSql<sql_types::Jsonb, Pg> for ParsedMetricsFile {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Self::from_sql_inner(bytes)
    }
}
impl ToSql<sql_types::Jsonb, Pg> for ParsedMetricsFile {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        self.to_sql_inner(out)
    }
}

#[derive(Deserialize, Validate, Insertable)]
#[diesel(table_name = schema::chromium_dataset, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
struct NewChromiumDatasetCore {
    #[serde(skip)]
    id: Uuid,
    #[garde(dive)]
    #[diesel(skip_insertion)]
    metadata: NewDatasetMetadata,
    #[garde(skip)]
    gems_id: Uuid,
    #[garde(custom(validate_html))]
    web_summary: String,
}
impl BelongsToExt<NewDatasetMetadata> for NewChromiumDatasetCore {
    fn set_parent_id(&mut self, parent_id: Uuid) {
        self.id = parent_id;
    }
}

#[derive(Deserialize, Validate, strum::IntoStaticStr)]
#[serde(tag = "cmdline")]
#[garde(allow_unvalidated)]
enum NewChromiumDataset {
    #[serde(rename = "cellranger-arc count")]
    #[strum(serialize = "cellranger-arc count")]
    CellrangerarcCount {
        core: NewChromiumDatasetCore,
        metrics: SingleRowCsv,
    },
    #[serde(rename = "cellranger-atac count")]
    #[strum(serialize = "cellranger-atac count")]
    CellrangeratacCount {
        core: NewChromiumDatasetCore,
        #[serde(deserialize_with = "deserialize_10x_json")]
        metrics: Value,
    },
    #[serde(rename = "cellranger count")]
    #[strum(serialize = "cellranger count")]
    CellrangerCount {
        core: NewChromiumDatasetCore,
        metrics: SingleRowCsv,
    },
    #[serde(rename = "cellranger multi")]
    #[strum(serialize = "cellranger multi")]
    CellrangerMulti {
        core: NewChromiumDatasetCore,
        metrics: Vec<MultiRowCsv>,
    },
    #[serde(rename = "cellranger vdj")]
    #[strum(serialize = "cellranger vdj")]
    CellrangerVdj {
        core: NewChromiumDatasetCore,
        metrics: SingleRowCsv,
    },
}
impl NewChromiumDataset {
    async fn validate_chemistry_and_n_samples(
        &self,
        conn: &mut diesel_async::AsyncPgConnection,
    ) -> crate::db::Result<()> {
        use crate::schema::{chemistry, gems};

        let (gems_id, n_metrics_files) = match self {
            Self::CellrangerarcCount { core, .. }
            | Self::CellrangeratacCount { core, .. }
            | Self::CellrangerCount { core, .. }
            | Self::CellrangerVdj { core, .. } => (core.gems_id, 1),
            Self::CellrangerMulti { core, metrics } => (core.gems_id, metrics.len()),
        };

        let (n_samples, chemistry_name, expected_cmdline): (i32, Option<String>, Option<String>) = gems::table
            .left_join(chemistry::table)
            .filter(gems::id.eq(gems_id))
            .select((
                gems::n_samples,
                chemistry::name.nullable(),
                chemistry::cmdline.nullable(),
            ))
            .first(conn)
            .await?;

        if n_samples as usize != n_metrics_files {
            return Err(super::Error::NMetricsFiles {
                expected_n_metrics_files: n_samples,
                found_n_metrics_files: n_metrics_files as i32,
            }
            .into());
        }

        let found_cmdline: &str = self.into();
        let Some(expected_cmdline) = expected_cmdline else {
            if found_cmdline != "cellranger atac" {
                return Err(super::Error::InvalidCmdline {
                    chemistry: chemistry_name,
                    expected_cmdline: "cellranger atac".to_string(),
                    found_cmdline: found_cmdline.to_string(),
                }
                .into());
            }
            return Ok(());
        };
        if expected_cmdline != found_cmdline {
            return Err(super::Error::InvalidCmdline {
                chemistry: chemistry_name,
                expected_cmdline,
                found_cmdline: found_cmdline.to_string(),
            }
            .into());
        }

        Ok(())
    }
}
impl Create for Vec<NewChromiumDataset> {
    type Returns = ();

    async fn create(self, conn: &mut diesel_async::AsyncPgConnection) -> crate::db::Result<Self::Returns> {
        use NewChromiumDataset::*;
        #[derive(Insertable)]
        #[diesel(table_name = schema::chromium_dataset)]
        struct InsertDataset {
            #[diesel(embed)]
            core: NewChromiumDatasetCore,
            metrics: Vec<ParsedMetricsFile>,
        }

        let mut insertions = Vec::with_capacity(self.len());

        for ds in self {
            // TODO: We shouldn't be making calls to the database in a loop. We should actually fetch all these records
            // at once, transform them into a `HashMap` from `gems_id` to the data, and then validate.
            ds.validate_chemistry_and_n_samples(conn).await?;
            let (core, metrics) = match ds {
                CellrangerarcCount { core, metrics }
                | CellrangerCount { core, metrics }
                | CellrangerVdj { core, metrics } => (core, vec![ParsedMetricsFile::SingleRowCsv(metrics)]),
                CellrangeratacCount { core, metrics } => (core, vec![ParsedMetricsFile::Json(metrics)]),
                CellrangerMulti { core, metrics } => (
                    core,
                    metrics.into_iter().map(|m| ParsedMetricsFile::MultiRowCsv(m)).collect(),
                ),
            };
            insertions.push(InsertDataset { core, metrics });
        }
        // Unfortunately, another iteration is necessary, unless we get a bit more clever
        let metadatas: Vec<_> = insertions
            .iter()
            .map(
                |InsertDataset {
                     core: NewChromiumDatasetCore { metadata, .. },
                     ..
                 }| metadata,
            )
            .collect();

        // Yet another iteration
        let ids = metadatas.create(conn).await?;
        for (ds, metadata_id) in insertions.iter_mut().zip(ids) {
            ds.core.set_parent_id(metadata_id);
        }

        diesel::insert_into(chromium_dataset::table)
            .values(insertions)
            .execute(conn)
            .await?;

        Ok(())
    }
}

fn deserialize_csv<'de, D>(deserializer: D) -> Result<Vec<HashMap<String, Value>>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    let rdr = csv::Reader::from_reader(raw.as_bytes());
    let records = rdr.into_deserialize();
    let records: csv::Result<Vec<HashMap<String, Value>>> = records.collect();

    records.map_err(|e| serde::de::Error::custom(e))
}

fn deserialize_10x_single_row_csv<'de, D>(deserializer: D) -> Result<HashMap<String, Value>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut records = deserialize_csv(deserializer)?;

    if records.len() != 1 {
        return Err(serde::de::Error::custom("expected CSV with exactly one row"));
    }

    let map = records.remove(0);

    Ok(map.parse())
}

fn deserialize_10x_multi_row_csv<'de, D>(deserializer: D) -> Result<Vec<HashMap<String, Value>>, D::Error>
where
    D: Deserializer<'de>,
{
    let records = deserialize_csv(deserializer)?;
    Ok(records.into_iter().map(|r| r.parse()).collect())
}

fn deserialize_10x_json<'de, D>(deserializer: D) -> Result<Value, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    Ok(serde_json::from_str(&raw).map_err(|e| serde::de::Error::custom(e))?)
}

trait TenxCsv {
    fn parse(self) -> HashMap<String, Value>;
}
impl TenxCsv for HashMap<String, Value> {
    fn parse(self) -> HashMap<String, Value> {
        let mut new_map = HashMap::with_capacity(self.len());

        for (key, mut value) in self {
            let key = heck::AsSnakeCase(key).to_string();

            // if we were able to parse it as a non-string, return that
            if !value.is_string() {
                new_map.insert(key, value);
                continue;
            }

            // if not, convert it to a string and remove the comma
            let value_as_string = value.to_string();
            let formatted = value_as_string.replace([',', '%', '"'], "");

            // some strings have the form of "1000 (10.00%)". We want to extract the 1000
            let re = Regex::new(r"^(\d+)\s\(\d{1,3}\.\d+\)$").unwrap();
            let matches = re.captures(&formatted);

            let extracted_number = match matches {
                Some(capture_group) => {
                    let (_, [number]) = capture_group.extract();
                    number
                }
                None => &formatted,
            };

            if let Ok(n) = serde_json::Number::from_str(extracted_number) {
                // if the original string had a '%' in it, we want to divide by 100
                if value_as_string.contains('%') && extracted_number == formatted {
                    value = Value::from(n.as_f64().unwrap() / 100.0);
                } else {
                    value = Value::from(n.as_f64().unwrap());
                }
            }

            new_map.insert(key, value);
        }

        new_map
    }
}

fn validate_html(document: &str, _: &()) -> garde::Result {
    let result = scraper::Html::parse_document(document);
    if !result.errors.is_empty() {
        return Err(garde::Error::new("invalid HTML"));
    }

    Ok(())
}
