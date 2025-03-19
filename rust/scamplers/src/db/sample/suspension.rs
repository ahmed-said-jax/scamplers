use chrono::NaiveDateTime;
use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types::{self, SqlType},
};
use diesel_async::{RunQueryDsl, scoped_futures::ScopedFutureExt};
use futures::{FutureExt, TryFutureExt};
use garde::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use valuable::Valuable;

use super::{Create, NewSampleMetadata, suspension_measurement::MeasurementData};
use crate::{
    db::{
        DbEnum, DbJson,
        utils::{Child, Children, ChildrenSets, MappingStruct},
    },
    schema::{self, sample_metadata, suspension, suspension_measurement, suspension_preparers},
};

#[derive(
    Deserialize,
    Debug,
    Serialize,
    FromSqlRow,
    Clone,
    Copy,
    SqlType,
    AsExpression,
    Default,
    Valuable,
    JsonSchema,
    strum::IntoStaticStr,
    strum::EnumString,
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

#[derive(Deserialize, Serialize, Validate, FromSqlRow, Default, Debug, AsExpression, JsonSchema)]
#[diesel(sql_type = sql_types::Jsonb)]
#[garde(allow_unvalidated)]
struct SuspensionMeasurement {
    #[serde(flatten)]
    #[garde(dive)]
    data: MeasurementData,
    is_post_hybridization: bool,
}
impl DbJson for SuspensionMeasurement {}
impl FromSql<sql_types::Jsonb, Pg> for SuspensionMeasurement {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Self::from_sql_inner(bytes)
    }
}
impl ToSql<sql_types::Jsonb, Pg> for SuspensionMeasurement {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        self.to_sql_inner(out)
    }
}

#[derive(Insertable, Deserialize, Validate)]
#[garde(allow_unvalidated)]
#[diesel(table_name = schema::suspension_measurement)]
struct NewSuspensionMeasurement {
    #[serde(default)]
    suspension_id: Uuid,
    measured_by: Uuid,
    #[serde(flatten)]
    #[garde(dive)]
    data: SuspensionMeasurement,
}
impl Child<suspension::table> for NewSuspensionMeasurement {
    fn set_parent_id(&mut self, parent_id: Uuid) {
        self.suspension_id = parent_id;
    }
}

impl Create for Vec<NewSuspensionMeasurement> {
    type Returns = ();

    async fn create(self, conn: &mut diesel_async::AsyncPgConnection) -> crate::db::Result<Self::Returns> {
        diesel::insert_into(suspension_measurement::table)
            .values(&self)
            .execute(conn)
            .await?;

        Ok(())
    }
}

#[derive(Insertable, Deserialize, Validate)]
#[diesel(table_name = suspension_preparers, check_for_backend(Pg), primary_key(suspension_id, preparer_id))]
#[garde(allow_unvalidated)]
struct SuspensionPreparer {
    suspension_id: Uuid,
    prepared_by: Uuid,
}

impl MappingStruct for SuspensionPreparer {
    fn new(id1: Uuid, id2: Uuid) -> Self {
        Self {
            suspension_id: id1,
            prepared_by: id2,
        }
    }
}

impl Create for Vec<SuspensionPreparer> {
    type Returns = ();

    async fn create(self, conn: &mut diesel_async::AsyncPgConnection) -> crate::db::Result<Self::Returns> {
        diesel::insert_into(suspension_preparers::table)
            .values(&self)
            .on_conflict_do_nothing()
            .execute(conn)
            .await?;

        Ok(())
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
    #[serde(skip)]
    metadata_id: Option<Uuid>,
    #[diesel(skip_insertion)]
    #[serde(skip)]
    has_metadata: bool,
    parent_specimen_id: Option<Uuid>,
    biological_material: BiologicalMaterial,
    created_at: NaiveDateTime,
    pub pooled_into_id: Option<Uuid>,
    pub multiplexing_tag_id: Option<Uuid>,
    #[garde(range(min = 0.0))]
    targeted_cell_recovery: f32,
    #[garde(range(min = 0))]
    target_reads_per_cell: i32,
    #[garde(range(min = 0.0))]
    lysis_duration_min: Option<f32>,
    #[diesel(skip_insertion)]
    preparer_ids: Vec<Uuid>,
    #[diesel(skip_insertion)]
    #[garde(dive)]
    measurements: Vec<NewSuspensionMeasurement>,
}

impl NewSuspension {
    fn validate_metadata(&self) -> crate::db::Result<()> {
        let Self {
            metadata,
            parent_specimen_id,
            ..
        } = self;

        if metadata.is_some() == parent_specimen_id.is_some() {
            return Err(super::Error::InvalidMetadata.into());
        }

        Ok(())
    }

    fn validate_lysis(&self) -> crate::db::Result<()> {
        use BiologicalMaterial::Nuclei;

        let Self {
            lysis_duration_min,
            biological_material,
            ..
        } = self;
        if lysis_duration_min.is_some() && !matches!(biological_material, Nuclei) {
            return Err(crate::db::DataError::Other(
                "lysis duration only applicable to cell suspensions containing nuclei".to_string(),
            )
            .into());
        }

        Ok(())
    }
}

impl Child<sample_metadata::table> for NewSuspension {
    fn set_parent_id(&mut self, parent_id: Uuid) {
        let Self {
            metadata_id,
            has_metadata,
            ..
        } = self;
        if *has_metadata {
            *metadata_id = Some(parent_id);
        } else {
            *metadata_id = None;
        }
    }
}

impl Create for Vec<NewSuspension> {
    type Returns = ();

    // Don't need to return anything yet

    async fn create(mut self, conn: &mut diesel_async::AsyncPgConnection) -> crate::db::Result<Self::Returns> {
        const N_MEASUREMENTS_PER_SUSPENSION: usize = 10;
        const N_PREPARERS_PER_SUSPENSION: usize = 10;

        let n_suspensions = self.len();

        let mut metadatas = Vec::with_capacity(n_suspensions);
        let mut measurement_sets = Vec::with_capacity(n_suspensions);

        for suspension in self.iter_mut() {
            suspension.validate_metadata()?;
            suspension.validate_lysis()?;

            let NewSuspension {
                metadata, measurements, ..
            } = suspension;

            // This step is necessary for Rusty reasons
            let measurements: Vec<_> = measurements.drain(..).collect();

            measurement_sets.push(measurements);

            let Some(metadata) = metadata.take() else {
                continue;
            };

            suspension.has_metadata = true;
            metadatas.push(metadata);
        }

        let metadata_ids = metadatas.create(conn).await?;
        self.set_parent_ids(&metadata_ids);

        let suspension_ids = diesel::insert_into(suspension::table)
            .values(&self)
            .returning(suspension::id)
            .get_results(conn)
            .await?;

        let flattened_measurements =
            measurement_sets.flatten_and_set_parent_ids(&suspension_ids, N_MEASUREMENTS_PER_SUSPENSION * n_suspensions);
        flattened_measurements.create(conn).await?;

        let preparer_id_sets = self.iter().map(|NewSuspension { preparer_ids, .. }| preparer_ids);
        let preparer_insertions = SuspensionPreparer::from_grouped_ids(
            &suspension_ids,
            preparer_id_sets,
            N_PREPARERS_PER_SUSPENSION * n_suspensions,
        );

        preparer_insertions.create(conn).await?;

        Ok(())
    }
}
