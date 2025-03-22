
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
use diesel_async::RunQueryDsl;
use garde::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use valuable::Valuable;

use super::{suspension::NewSuspension, suspension_measurement::MeasurementData};
use crate::{
    db::{
        Create,
        utils::{BelongsToExt, DbEnum, DbJson, JunctionStruct, Parent, ParentSet},
    },
    schema::{self, multiplexed_suspension_preparers, multiplexing_tag},
};

#[derive(
    Deserialize,
    Serialize,
    FromSqlRow,
    Clone,
    Copy,
    SqlType,
    AsExpression,
    Debug,
    Default,
    Valuable,
    strum::IntoStaticStr,
    strum::EnumString,
)]
#[serde(rename_all = "snake_case")]
#[diesel(sql_type = sql_types::Text)]
enum MultiplexingTagType {
    FlexBarcode,
    OnChipMultiplexing,
    #[serde(rename = "TotalSeqA")]
    TotalSeqA, // These 3 are proper nouns, so they're not converted to snake case
    #[serde(rename = "TotalSeqB")]
    TotalSeqB,
    #[serde(rename = "TotalSeqC")]
    TotalSeqC,
    #[default]
    Unknown,
}
impl DbEnum for MultiplexingTagType {}
impl FromSql<sql_types::Text, Pg> for MultiplexingTagType {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Self::from_sql_inner(bytes)
    }
}
impl ToSql<sql_types::Text, diesel::pg::Pg> for MultiplexingTagType {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        self.to_sql_inner(out)
    }
}

// This is not an externally facing data structure
#[derive(Deserialize, Insertable)]
#[diesel(table_name = schema::multiplexing_tag, check_for_backend(Pg))]
struct NewMultiplexingTag {
    tag_id: String,
    type_: MultiplexingTagType,
}
impl Create for Vec<NewMultiplexingTag> {
    type Returns = ();

    async fn create(self, conn: &mut diesel_async::AsyncPgConnection) -> crate::db::Result<Self::Returns> {
        use multiplexing_tag::dsl::multiplexing_tag;

        diesel::insert_into(multiplexing_tag)
            .values(&self)
            .execute(conn)
            .await?;

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Validate, FromSqlRow, Default, Debug, AsExpression, JsonSchema)]
#[diesel(sql_type = sql_types::Jsonb)]
#[garde(allow_unvalidated)]
struct MultiplexedSuspensionMeasurement {
    #[serde(flatten)]
    #[garde(dive)]
    data: MeasurementData,
    is_post_storage: bool,
}
impl DbJson for MultiplexedSuspensionMeasurement {}
impl FromSql<sql_types::Jsonb, Pg> for MultiplexedSuspensionMeasurement {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Self::from_sql_inner(bytes)
    }
}
impl ToSql<sql_types::Jsonb, Pg> for MultiplexedSuspensionMeasurement {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        self.to_sql_inner(out)
    }
}

#[derive(Insertable, Deserialize, Validate)]
#[garde(allow_unvalidated)]
#[diesel(table_name = schema::multiplexed_suspension_measurement)]
struct NewMultiplexedSuspensionMeasurement {
    #[serde(default)]
    suspension_id: Uuid,
    measured_by: Uuid,
    #[serde(flatten)]
    #[garde(dive)]
    data: MultiplexedSuspensionMeasurement,
}

#[derive(Deserialize, Insertable, Validate)]
#[diesel(table_name = schema::multiplexed_suspension, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
struct NewMultiplexedSuspension {
    legacy_id: String,
    pooled_at: NaiveDateTime,
    notes: Option<Vec<String>>,
    #[diesel(skip_insertion)]
    #[garde(dive)]
    suspensions: Vec<NewSuspension>,
    #[diesel(skip_insertion)]
    preparer_ids: Vec<Uuid>,
    #[diesel(skip_insertion)]
    #[garde(dive)]
    measurements: Vec<NewMultiplexedSuspensionMeasurement>,
}

impl BelongsToExt<NewMultiplexedSuspension> for NewSuspension {
    fn set_parent_id(&mut self, parent_id: Uuid) {
        self.pooled_into_id = Some(parent_id)
    }
}
impl Parent<NewSuspension> for NewMultiplexedSuspension {
    fn drain_children(&mut self) -> Vec<NewSuspension> {
        self.suspensions.drain(..).collect()
    }
}

#[derive(Insertable)]
#[diesel(table_name = multiplexed_suspension_preparers, check_for_backend(Pg))]
struct MultiplexedSuspensionPreparer {
    suspension_id: Uuid,
    prepared_by: Uuid,
}
impl Create for Vec<MultiplexedSuspensionPreparer> {
    type Returns = ();

    async fn create(self, conn: &mut diesel_async::AsyncPgConnection) -> crate::db::Result<Self::Returns> {
        diesel::insert_into(multiplexed_suspension_preparers::table)
            .values(&self)
            .execute(conn)
            .await?;

        Ok(())
    }
}
impl JunctionStruct for MultiplexedSuspensionPreparer {
    fn new(suspension_id: Uuid, prepared_by: Uuid) -> Self {
        Self {
            suspension_id,
            prepared_by,
        }
    }
}

impl Create for Vec<NewMultiplexedSuspension> {
    type Returns = ();

    async fn create(mut self, conn: &mut diesel_async::AsyncPgConnection) -> crate::db::Result<Self::Returns> {
        use schema::multiplexed_suspension;
        const N_SUSPENSIONS_PER_POOL: usize = 16; // The maximum number of suspensions in a pool
        const N_MEASUREMENTS_PER_POOL: usize = 8;
        const N_PREPARERS_PER_POOL: usize = 2;

        let n_multiplexed_suspensions = self.len();

        // The two operations here should definitely be factored out into two separate functions
        for multiplexed_suspension in self.iter_mut() {
            let NewMultiplexedSuspension { suspensions, .. } = multiplexed_suspension;

            // First, make sure each child suspension's multiplexing tag is specified
            let multiplexing_tag_set: super::Result<Vec<Uuid>> = suspensions
                .iter()
                .map(
                    |NewSuspension {
                         multiplexing_tag_id, ..
                     }| { multiplexing_tag_id.ok_or(super::Error::MultiplexingTagNotProvided) },
                )
                .collect();

            let multiplexing_tag_set = multiplexing_tag_set?;

            // Load the multiplexing tag types from the database
            let types = multiplexing_tag::table
                .select(multiplexing_tag::type_)
                .filter(multiplexing_tag::id.eq_any(&multiplexing_tag_set))
                .load(conn)
                .await?;

            // If the tag types are not all the same, raise an error. Note that we don't have to check that `types` is
            // the same length as `multiplexing_tag_set` because that would imply an invalid foreign key reference,
            // which the database will catch for us
            let first = types
                .get(0)
                .ok_or(super::Error::InvalidMultiplexingTagSet(multiplexing_tag_set))?;
            if !types.iter().all(|t| t == first) {
                return Err(super::Error::DifferentMultiplexingTagTypes(types).into());
            }
        }

        let new_ids: Vec<Uuid> = diesel::insert_into(multiplexed_suspension::table)
            .values(&self)
            .returning(multiplexed_suspension::id)
            .get_results(conn)
            .await?;

        let flattened_suspensions =
            self.flatten_children_and_set_ids(&new_ids, N_SUSPENSIONS_PER_POOL * n_multiplexed_suspensions);
        flattened_suspensions.create(conn).await?;

        let flattened_measurements =
            self.flatten_children_and_set_ids(&new_ids, N_MEASUREMENTS_PER_POOL * n_multiplexed_suspensions);
        flattened_measurements.create(conn).await?;

        let grouped_preparer_ids = self
            .iter()
            .map(|NewMultiplexedSuspension { preparer_ids, .. }| preparer_ids);
        let multiplexed_suspension_preparers = MultiplexedSuspensionPreparer::from_ids_grouped_by_parent1(
            &new_ids,
            grouped_preparer_ids,
            N_PREPARERS_PER_POOL * n_multiplexed_suspensions,
        );
        multiplexed_suspension_preparers.create(conn).await?;

        Ok(())
    }
}
