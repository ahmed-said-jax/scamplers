use chrono::NaiveDateTime;
use diesel::{
    deserialize::FromSqlRow,
    expression::AsExpression,
    sql_types::{self, SqlType},
    prelude::*,
    pg::Pg
};
use diesel_async::RunQueryDsl;
use futures::TryFutureExt;
use garde::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use valuable::Valuable;

use crate::schema;

use super::{Create, NewSampleMetadata};

#[derive(
    Deserialize, Debug, Serialize, FromSqlRow, Clone, Copy, SqlType, AsExpression, Default, Valuable, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
#[diesel(sql_type = sql_types::Text)]
pub enum BiologicalMaterial {
    Cells,
    Nuclei,
    #[default]
    Unknown,
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

        let mut suspensions_with_metadata = (Vec::with_capacity(self.len()), Vec::with_capacity(self.len()));
        let mut suspensions_with_parent_specimen = Vec::with_capacity(self.len());

        for s in self {
            s.validate_metadata()?;

            if let Some(metadata) = &s.metadata {
                suspensions_with_metadata.0.push(metadata);
                suspensions_with_metadata.1.push(s);
            } else {
                suspensions_with_parent_specimen.push(s);
            }
        }

        let (new_metadatas, new_suspensions) = suspensions_with_metadata;

        let metadata_ids = new_metadatas.create(conn);
        let suspension_insertions = diesel::insert_into(suspension::table).values(suspensions_with_parent_specimen).execute(conn);

        let (metadata_ids, _) = tokio::try_join!(metadata_ids, suspension_insertions.err_into())?;
        

        Ok(())
    }
}
