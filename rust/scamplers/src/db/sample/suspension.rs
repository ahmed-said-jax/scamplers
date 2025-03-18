use chrono::NaiveDateTime;
use diesel::{
    backend::Backend, deserialize::{FromSql, FromSqlRow}, expression::AsExpression, pg::Pg, prelude::*, serialize::ToSql, sql_types::{self, SqlType}
};
use diesel_async::RunQueryDsl;
use futures::TryFutureExt;
use garde::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use valuable::Valuable;

use crate::{db::DbEnum, schema};

use super::{Create, NewSampleMetadata};

#[derive(
    Deserialize, Debug, Serialize, FromSqlRow, Clone, Copy, SqlType, AsExpression, Default, Valuable, JsonSchema, strum::IntoStaticStr, strum::EnumString
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[diesel(sql_type = sql_types::Text)]
pub enum BiologicalMaterial {
    Cells,
    Nuclei,
    #[default]
    Unknown,
}
impl DbEnum for BiologicalMaterial {}
impl FromSql<sql_types::Text, Pg> for BiologicalMaterial {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Self::from_sql_inner(bytes)
    }
}

impl ToSql<sql_types::Text, diesel::pg::Pg> for BiologicalMaterial {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        self.to_sql_inner(out)
    }
}

#[derive(Deserialize, Insertable, Validate)]
#[diesel(table_name = schema::suspension, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
pub struct NewSuspension {
    legacy_id: String,
    #[serde(flatten)]
    #[diesel(skip_insertion)]
    #[garde(dive)]
    metadata: Option<NewSampleMetadata>,
    parent_specimen_id: Option<Uuid>,
    biological_material: BiologicalMaterial,
    created_at: NaiveDateTime,
    pooled_into_id: Option<Uuid>,
    multiplexing_tag_id: Option<Uuid>,
    #[garde(range(min = 0.0))]
    targeted_cell_recovery: f32,
    #[garde(range(min = 0))]
    target_reads_per_cell: i32,
}

impl NewSuspension {
    fn validate_metadata(&self) -> crate::db::Result<()> {
        let Self {metadata, parent_specimen_id, ..} = self;

        if metadata.is_some() == parent_specimen_id.is_some() {
            return Err(crate::db::Error::Other { message: "a suspension may not both derive from a parent specimen and have its own metadata".to_string() }); // TODO: this should be a strongly-typed `InvalidData` error
        }

        Ok(())
    }
}

impl Create for Vec<NewSuspension> {
    type Returns = (); // Don't need to return anything yet

    async fn create(&self, conn: &mut diesel_async::AsyncPgConnection) -> crate::db::Result<Self::Returns> {
        use schema::suspension;
        #[derive(Insertable)]
        #[diesel(table_name = schema::suspension, check_for_backend(Pg))]
        struct InsertSuspension<'a> {
            metadata_id: Option<Uuid>,
            #[diesel(embed)]
            data: &'a NewSuspension
        }

        let mut independent_suspensions = (Vec::with_capacity(self.len()), Vec::with_capacity(self.len()));
        let mut derived_suspensions = Vec::with_capacity(self.len());

        for s in self {
            s.validate_metadata()?;

            if let Some(metadata) = &s.metadata {
                independent_suspensions.0.push(s);
                independent_suspensions.1.push(metadata);
            } else {
                derived_suspensions.push(s);
            }
        }

        let (independent_suspensions, new_metadatas) = independent_suspensions;

        let metadata_ids = new_metadatas.create(conn);
        let derived_suspensions = diesel::insert_into(suspension::table).values(derived_suspensions).execute(conn);

        let (metadata_ids, _) = tokio::try_join!(metadata_ids, derived_suspensions.err_into())?;

        let independent_suspensions: Vec<_> = independent_suspensions.into_iter().zip(metadata_ids).map(move |(suspension, metadata_id)| InsertSuspension{metadata_id: Some(metadata_id), data: suspension}).collect();

        diesel::insert_into(suspension::table).values(independent_suspensions).execute(conn).await?;

        Ok(())
    }
}
