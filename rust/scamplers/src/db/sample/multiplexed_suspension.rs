use std::collections::HashSet;

use chrono::NaiveDateTime;
use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types,
};
use diesel_async::RunQueryDsl;
use garde::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use valuable::Valuable;

use super::{suspension::NewSuspension, suspension_measurement::MeasurementData};
use crate::{
    db::{Create, DbEnum, DbJson},
    schema::{self, multiplexing_tag},
};

// #[derive(
//     Deserialize,
//     Serialize,
//     FromSqlRow,
//     Clone,
//     Copy,
//     SqlType,
//     AsExpression,
//     Debug,
//     Default,
//     Valuable,
//     strum::IntoStaticStr,
//     strum::EnumString,
// )]
// #[serde(rename_all = "snake_case")]
// enum MultiplexingTagType {
//     FlexBarcode,
//     OnChipMultiplexing,
//     #[serde(rename = "TotalSeqA")]
//     TotalSeqA, // These 3 are proper nouns, so they're not converted to snake case
//     #[serde(rename = "TotalSeqB")]
//     TotalSeqB,
//     #[serde(rename = "TotalSeqC")]
//     TotalSeqC,
// }

#[derive(Deserialize, Serialize, Validate, FromSqlRow, Default, Debug, AsExpression, JsonSchema)]
#[diesel(sql_type = sql_types::Jsonb)]
#[garde(allow_unvalidated)]
struct MultiplexedSuspensionMeasurement {
    #[serde(flatten)]
    data: MeasurementData,
    taken_after_storage: bool,
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

#[derive(Deserialize)]
struct NewMultiplexedSuspensionMeasurement {
    measured_by: Uuid,
    #[serde(flatten)]
    data: MultiplexedSuspensionMeasurement,
}

#[derive(Deserialize, Insertable)]
#[diesel(table_name = schema::multiplexed_suspension_measurement, check_for_backend(Pg))]
struct NewMeasurement<M: AsExpression<sql_types::Jsonb>>
where
    for<'a> &'a M: AsExpression<sql_types::Jsonb>,
{
    suspension_id: Uuid,
    #[serde(flatten)]
    data: M,
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
    measurements: Vec<NewMultiplexedSuspensionMeasurement>,
}

impl Create for Vec<NewMultiplexedSuspension> {
    type Returns = (); // we don't need to return anything yet

    async fn create(mut self, conn: &mut diesel_async::AsyncPgConnection) -> crate::db::Result<Self::Returns> {
        use schema::multiplexed_suspension;
        const N_SUSPENSIONS_PER_POOL: usize = 16; // The maximum

        // The two operations here should definitely be factored out into two separate functions
        for multiplexed_suspension in &self {
            let multiplexing_tag_set: std::result::Result<Vec<Uuid>, Error> = multiplexed_suspension
                .suspensions
                .iter()
                .map(
                    |NewSuspension {
                         multiplexing_tag_id, ..
                     }| { multiplexing_tag_id.ok_or(Error::MultiplexingTagNotProvided) },
                )
                .collect();

            let multiplexing_tag_set = multiplexing_tag_set?;

            let types: Vec<String> = multiplexing_tag::table
                .select(multiplexing_tag::type_)
                .filter(multiplexing_tag::id.eq_any(&multiplexing_tag_set))
                .load(conn)
                .await?;

            if HashSet::from_iter(&types).len() != 1 {
                return Err(crate::db::Error::from(super::Error::from(Error::DifferentMultiplexingTagTypes(types))));
            }
        }

        let new_ids: Vec<Uuid> = diesel::insert_into(multiplexed_suspension::table)
            .values(self)
            .returning(multiplexed_suspension::id)
            .get_results(conn)
            .await?;

        let mut new_suspensions = Vec::with_capacity(N_SUSPENSIONS_PER_POOL * self.len());
        for (NewMultiplexedSuspension{suspensions, ..}, multiplexed_suspension_id) in self.iter().zip(new_ids) {
            for s in suspensions.drain(..) {
                s.pooled_into_id = Some(multiplexed_suspension_id);
                new_suspensions.push(s);
            }
        }

        new_suspensions.create(conn).await?;

        Ok(())
    }
}

#[derive(thiserror::Error, Debug, Serialize, Valuable, Clone)]
pub enum Error {
    #[error("all suspensions pooled into a multiplexed suspension must have the same tag type")]
    DifferentMultiplexingTagTypes(Vec<String>),
    #[error("multiplexing tag must be provided for suspensions pooled into a multiplexed suspension")]
    MultiplexingTagNotProvided,
}
