use chrono::NaiveDateTime;
use diesel::{
    dsl::{AssumeNotNull, InnerJoin},
    pg::Pg,
    prelude::*,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use garde::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use valuable::Valuable;

use super::{
    AsDieselExpression, BoxedDieselExpression, Create,
    lab::LabStub,
    utils::{AsIlike, DefaultNowNaiveDateTime},
};
use crate::schema::{
    self,
    dataset_metadata::{self, delivered_at, name as name_col},
    lab,
};
mod chromium;

#[derive(Insertable, Deserialize, Validate)]
#[diesel(table_name = schema::dataset_metadata, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
struct NewDatasetMetadata {
    name: String,
    lab_id: Uuid,
    data_path: String, // TODO: There should be some nice validation for this
    #[serde(default)]
    delivered_at: Option<DefaultNowNaiveDateTime>,
}

impl Create for Vec<NewDatasetMetadata> {
    type Returns = Vec<Uuid>;

    async fn create(self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
        use schema::dataset_metadata::dsl::{dataset_metadata, id};

        let ids = diesel::insert_into(dataset_metadata)
            .values(&self)
            .returning(id)
            .get_results(conn)
            .await?;

        Ok(ids)
    }
}

impl Create for Vec<&NewDatasetMetadata> {
    type Returns = Vec<Uuid>;

    async fn create(self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
        use schema::dataset_metadata::dsl::{dataset_metadata, id};

        let ids = diesel::insert_into(dataset_metadata)
            .values(self)
            .returning(id)
            .get_results(conn)
            .await?;

        Ok(ids)
    }
}

#[derive(Serialize, Selectable, Queryable, JsonSchema)]
#[diesel(table_name = schema::dataset_metadata, check_for_backend(Pg))]
struct DatasetMetadata {
    name: String,
    link: String,
    #[diesel(embed)]
    lab: LabStub,
    data_path: String,
    delivered_at: Option<NaiveDateTime>,
}

impl DatasetMetadata {
    fn base_query() -> InnerJoin<dataset_metadata::table, lab::table> {
        dataset_metadata::table.inner_join(lab::table)
    }
}

#[derive(Deserialize, Default, Valuable)]
#[serde(rename_all = "snake_case")]
enum OrdinalColumns {
    #[default]
    DeliveredAt,
    Name,
}

#[derive(Deserialize, Valuable)]
struct DatasetMetadataQuery {
    name: Option<String>,
    #[valuable(skip)]
    delivered_before: Option<NaiveDateTime>,
    #[valuable(skip)]
    delivered_after: Option<NaiveDateTime>,
}
impl<T> AsDieselExpression<T> for DatasetMetadataQuery
where
    delivered_at: SelectableExpression<T>,
    name_col: SelectableExpression<T>,
    AssumeNotNull<dataset_metadata::delivered_at>: SelectableExpression<T>,
{
    fn as_diesel_expression<'a>(&'a self) -> Option<BoxedDieselExpression<'a, T>>
    where
        T: 'a,
    {
        let Self {
            name,
            delivered_before,
            delivered_after,
        } = self;

        if matches!((name, delivered_before, delivered_after), (None, None, None)) {
            return None;
        }

        let mut query: BoxedDieselExpression<T> = match name {
            None => Box::new(name_col.is_not_null()),
            Some(n) => Box::new(name_col.ilike(n.as_ilike())),
        };

        // This is necessary so that we can safely call `assume_not_null` in the next set of checks
        if [delivered_before, delivered_after].iter().any(|d| d.is_some()) {
            query = Box::new(query.and(delivered_at.is_not_null()));
        }

        if let Some(delivered_before) = delivered_before {
            query = Box::new(query.and(delivered_at.assume_not_null().lt(delivered_before)));
        }

        if let Some(delivered_after) = delivered_after {
            query = Box::new(query.and(delivered_at.assume_not_null().gt(delivered_after)));
        }

        Some(query)
    }
}

#[derive(thiserror::Error, Debug, Serialize, Valuable, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Error {
    #[error("invalid cmdline for dataset")]
    InvalidCmdline {
        chemistry: Option<String>,
        expected_cmdline: String,
        found_cmdline: String,
    },
    #[error("mismatching ")]
    NMetricsFiles {
        expected_n_metrics_files: i32,
        found_n_metrics_files: i32,
    },
}
