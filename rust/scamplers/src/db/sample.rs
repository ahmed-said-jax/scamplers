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
        self, lab,
        sample_metadata::{self, name as name_col, received_at, species as species_col, tissue as tissue_col},
    },
};
pub mod specimen;
mod multiplexed_suspension;
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
)]
#[diesel(sql_type = sql_types::Text)]
#[serde(rename_all = "snake_case")]
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
)]
#[diesel(sql_type = sql_types::Text)]
#[serde(rename_all = "snake_case")]
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

#[derive(Deserialize, Insertable)]
#[diesel(table_name = schema::committee_approval, check_for_backend(Pg))]
pub struct NewCommitteeApproval {
    institution_id: Uuid,
    committee_type: ComplianceCommitteeType,
    compliance_identifier: String,
}

// We create a generic struct that can hold either a `String` or an `&str`. The reason is because we need both depending
// on whether we're inserting a new sample metadata that has compliance numbers already, or if we're adding compliance
// numbers to an existing sample metadata. However, we don't really care about copying `Uuid` and `CommitteeType`
// because those are very small and cheap to copy
#[derive(Deserialize, Insertable)]
#[diesel(table_name = schema::committee_approval, check_for_backend(Pg))]
pub struct ExistingSampleNewCommitteeApproval<Str: AsExpression<sql_types::Text>>
where
    for<'a> &'a Str: AsExpression<sql_types::Text>,
{
    sample_id: Uuid,
    institution_id: Uuid,
    committee_type: ComplianceCommitteeType,
    compliance_identifier: Str,
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

// We don't need to `impl Create for Vec<NewSampleMetadata>` - we actually only use this as part of other structs, so
// it's always used as a reference
impl Create for Vec<&NewSampleMetadata> {
    // This is a bit of an exception to the pattern established thus far, as we generally don't need the metadata
    // objects to be returned. The IDs however are useful
    type Returns = Vec<Uuid>;

    async fn create(&self, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self::Returns> {
        let owned_refs = self.clone();

        let ids = diesel::insert_into(sample_metadata::table)
            .values(owned_refs)
            .returning(sample_metadata::id)
            .get_results(conn)
            .await?;

        let mut commitee_approval_insertions =
            Vec::with_capacity(self.iter().map(|m| m.committee_approvals.len()).sum());

        for (metadata, sample_id) in self.iter().zip(&ids) {
            commitee_approval_insertions.extend(metadata.committee_approvals.iter().map(
                |NewCommitteeApproval {
                     institution_id,
                     committee_type,
                     compliance_identifier,
                 }| ExistingSampleNewCommitteeApproval {
                    sample_id: *sample_id,
                    institution_id: *institution_id,
                    committee_type: *committee_type,
                    compliance_identifier,
                },
            ));
        }

        Ok(ids)
    }
}

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
