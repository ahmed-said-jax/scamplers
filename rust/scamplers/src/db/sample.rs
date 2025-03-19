use chrono::NaiveDateTime;
use diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    helper_types::InnerJoin,
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types::{self},
};
use diesel_async::RunQueryDsl;
use garde::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use valuable::Valuable;

use super::{AsDieselExpression, BoxedDieselExpression, Create, DbEnum, lab::LabStub};
use crate::{
    db::ILike,
    schema::{
        self, committee_approval, lab,
        sample_metadata::{self, name as name_col, received_at, species as species_col, tissue as tissue_col},
    },
};
mod multiplexed_suspension;
pub mod specimen;
mod suspension;
mod suspension_measurement;

// This is the first real complexity. We want to abstract away different sample types into one `Sample` enum for ease of
// API usage
#[derive(
    Deserialize,
    Serialize,
    Default,
    FromSqlRow,
    AsExpression,
    Debug,
    strum::VariantArray,
    Clone,
    Copy,
    Valuable,
    JsonSchema,
    strum::IntoStaticStr,
    strum::EnumString,
)]
#[diesel(sql_type = sql_types::Text)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Species {
    AmbystomaMexicanum,
    CanisFamiliaris,
    DrosophilaMelanogaster,
    GasterosteusAculeatus,
    HomoSapiens,
    MusMusculus,
    RattusNorvegicus,
    SminthopsisCrassicaudata,
    #[default]
    Unknown,
}
impl DbEnum for Species {}

impl FromSql<sql_types::Text, Pg> for Species {
    fn from_sql(bytes: <Pg as diesel::backend::Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Self::from_sql_inner(bytes)
    }
}
impl ToSql<sql_types::Text, Pg> for Species {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        self.to_sql_inner(out)
    }
}

#[derive(
    Deserialize,
    Serialize,
    Default,
    FromSqlRow,
    AsExpression,
    Debug,
    Clone,
    Copy,
    strum::IntoStaticStr,
    strum::EnumString,
)]
#[diesel(sql_type = sql_types::Text)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ComplianceCommitteeType {
    Ibc,
    Irb,
    Iacuc,
    #[default]
    Unknown,
}
impl DbEnum for ComplianceCommitteeType {}

impl FromSql<sql_types::Text, Pg> for ComplianceCommitteeType {
    fn from_sql(bytes: <Pg as diesel::backend::Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Self::from_sql_inner(bytes)
    }
}
impl ToSql<sql_types::Text, Pg> for ComplianceCommitteeType {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        self.to_sql_inner(out)
    }
}

#[derive(Deserialize, Insertable, Clone)]
#[diesel(table_name = schema::committee_approval, check_for_backend(Pg))]
pub struct NewCommitteeApproval {
    #[serde(default)]
    sample_id: Uuid,
    institution_id: Uuid,
    committee_type: ComplianceCommitteeType,
    compliance_identifier: String,
}

#[derive(Deserialize, Validate, Insertable)]
#[garde(allow_unvalidated)]
#[diesel(table_name = schema::sample_metadata, check_for_backend(Pg))]
pub struct NewSampleMetadata {
    #[garde(length(min = 1))]
    pub name: String,
    pub submitted_by: Uuid,
    pub lab_id: Uuid,
    pub received_at: NaiveDateTime,
    #[garde(length(min = 1))]
    pub species: Vec<Species>,
    #[garde(length(min = 1))]
    pub tissue: String,
    #[diesel(skip_insertion)]
    #[serde(default)]
    pub committee_approvals: Vec<NewCommitteeApproval>,
    pub notes: Option<Vec<String>>,
    pub returned_at: Option<NaiveDateTime>,
    pub returned_by: Option<Uuid>,
}

impl Create for Vec<NewSampleMetadata> {
    type Returns = Vec<Uuid>;

    async fn create(mut self, conn: &mut diesel_async::AsyncPgConnection) -> crate::db::Result<Self::Returns> {
        let ids = diesel::insert_into(sample_metadata::table)
            .values(&self)
            .returning(sample_metadata::id)
            .get_results(conn)
            .await?;

        let mut committee_approval_insertions = Vec::with_capacity(self.len());
        for (
            NewSampleMetadata {
                committee_approvals, ..
            },
            sample_id,
        ) in self.iter_mut().zip(&ids)
        {
            for approval in committee_approvals {
                approval.sample_id = *sample_id;
                committee_approval_insertions.push(&*approval);
            }
        }

        diesel::insert_into(committee_approval::table)
            .values(committee_approval_insertions)
            .execute(conn)
            .await?;

        Ok(ids)
    }
}

// We don't need to `impl Create for Vec<NewSampleMetadata>` - we actually only use this as part of other structs, so
// it's always used as a reference
// impl Create for Vec<NewSampleMetadata> {
//     // This is a bit of an exception to the pattern established thus far, as we generally don't need the metadata
//     // objects to be returned. The IDs however are useful
//     type Returns = Vec<Uuid>;

//     async fn create(self, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self::Returns> {
//         let ids = diesel::insert_into(sample_metadata::table)
//             .values(&self)
//             .returning(sample_metadata::id)
//             .get_results(conn)
//             .await?;

//         let mut commitee_approval_insertions =
//             Vec::with_capacity(self.iter().map(|m| m.committee_approvals.len()).sum());

//         for (metadata, sample_id) in self.iter().zip(&ids) {
//             commitee_approval_insertions.extend(metadata.committee_approvals.iter().map(
//                 |NewCommitteeApproval {
//                      institution_id,
//                      committee_type,
//                      compliance_identifier,
//                  }| ExistingSampleNewCommitteeApproval {
//                     sample_id: *sample_id,
//                     institution_id: *institution_id,
//                     committee_type: *committee_type,
//                     compliance_identifier,
//                 },
//             ));
//         }

//         Ok(ids)
//     }
// }

// This is useful because frequently, samples have optional metadata (like `Suspension`)
// impl Create for Vec<Option<NewSampleMetadata>> {
//     type Returns = Vec<Option<Uuid>>;

//     async fn create(self, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self::Returns> {
//         let mut indices_with_value = Vec::with_capacity(self.len());
//         let mut elements_with_value = Vec::with_capacity(self.len());

//         for (i, metadata) in self.iter().enumerate() {
//             let Some(metadata) = metadata else {
//                 continue;
//             };
//             indices_with_value.push(i);
//             elements_with_value.push(*metadata);
//         }
//         let with_value: Vec<_> = self.into_iter().filter_map(|s| s).collect();
//         with_value.create(conn).await?;

//         Ok(vec![])

//     }
// }

#[derive(Selectable, Queryable, Serialize, JsonSchema)]
#[diesel(table_name = schema::sample_metadata, check_for_backend(Pg))]
struct SampleMetadata {
    name: String,
    #[diesel(embed)]
    lab: LabStub,
    received_at: NaiveDateTime,
    species: Vec<Species>,
    tissue: String,
    returned_at: Option<NaiveDateTime>,
}

#[derive(Deserialize, Default, Valuable)]
#[serde(rename_all = "snake_case")]
enum OrdinalColumns {
    #[default]
    ReceivedAt,
    Name,
}

#[derive(Deserialize, Valuable)]
struct SampleMetadataQuery {
    name: Option<String>,
    tissue: Option<String>,
    #[valuable(skip)]
    received_before: Option<NaiveDateTime>,
    #[valuable(skip)]
    received_after: Option<NaiveDateTime>,
    #[serde(default)]
    species: Vec<Species>,
}

impl SampleMetadata {
    fn base_query() -> InnerJoin<sample_metadata::table, lab::table> {
        sample_metadata::table.inner_join(lab::table)
    }
}

impl<T> AsDieselExpression<T> for SampleMetadataQuery
where
    name_col: SelectableExpression<T>,
    tissue_col: SelectableExpression<T>,
    received_at: SelectableExpression<T>,
    species_col: SelectableExpression<T>,
{
    fn as_diesel_expression<'a>(&'a self) -> Option<BoxedDieselExpression<'a, T>>
    where
        T: 'a,
    {
        let Self {
            name,
            tissue,
            received_before,
            received_after,
            species,
        } = self;

        if matches!(
            (name, tissue, received_before, received_after, species.is_empty()),
            (None, None, None, None, true)
        ) {
            return None;
        }

        // This is a hack but not sure what else I can do
        let mut query: BoxedDieselExpression<T> = match name {
            None => Box::new(name_col.is_not_null()),
            Some(n) => Box::new(name_col.ilike(n.for_ilike())),
        };

        if let Some(tissue) = tissue {
            query = Box::new(query.and(tissue_col.ilike(tissue.for_ilike())));
        }

        if let Some(received_before) = received_before {
            query = Box::new(query.and(received_at.lt(received_before)));
        }

        if let Some(received_after) = received_after {
            query = Box::new(query.and(received_at.gt(received_after)));
        }

        if !species.is_empty() {
            query = Box::new(query.and(species_col.overlaps_with(species)));
        }

        Some(query)
    }
}

#[derive(thiserror::Error, Debug, Serialize, Valuable, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Error {
    #[error("all suspensions pooled into a multiplexed suspension must have the same tag type")]
    DifferentMultiplexingTagTypes(Vec<String>),
    #[error("multiplexing tag must be provided for suspensions pooled into a multiplexed suspension")]
    MultiplexingTagNotProvided,
    #[error("a sample must specify exactly one of its own metadata or a parental specimen")]
    InvalidMetadata,
    #[error("invalid multiplexing tag IDs")]
    InvalidMultiplexingTagSet(#[valuable(skip)] Vec<Uuid>),
}

type Result<T> = std::result::Result<T, Error>;
