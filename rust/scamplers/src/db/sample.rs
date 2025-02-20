use chrono::{NaiveDate, NaiveDateTime};
use diesel::{expression::AsExpression, sql_types, prelude::*, pg::Pg};
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::schema;
mod specimen;
// This is the first real complexity. We want to abstract away different sample types into one `Sample` enum for ease of API usage
#[derive(Deserialize, Serialize)]
pub enum Species {
    AmbystomaMexicanum,
    CanisFamiliaris,
    DrosophilaMelanogaster,
    GasterosteusAculeatus,
    HomoSapiens,
    MusMusculus,
    RattusNorvegicus,
    SminthopsisCrassicaudata,
}

#[derive(Deserialize, Serialize)]
enum ComplianceCommitteeType {
    Ibc,
    Irb,
    Iacuc
}

#[derive(Deserialize)]
pub struct NewCommitteeApproval {
    institution_id: Uuid,
    committee_type: ComplianceCommitteeType,
    compliance_identifier: String
}

pub struct ExistingSampleNewCommitteeApproval<A: AsRef<NewCommitteeApproval>> {
    sample_id: Uuid,
    approval: A
}

#[derive(Deserialize, Validate, Insertable)]
#[garde(allow_unvalidated)]
#[diesel(table_name = schema::sample_metadata, check_for_backend(Pg))]
pub struct NewSampleMetadata {
    #[garde(length(min = 1))]
    name: String,
    submitted_by: Uuid,
    lab_id: Uuid,
    received_at: NaiveDateTime,
    #[garde(length(min = 1))]
    species: Vec<Species>,
    #[garde(length(min = 1))]
    tissue: String,
    #[diesel(skip_insertion)]
    committee_approvals: Option<Vec<NewCommitteeApproval>>,
    notes: Option<Vec<String>>,
    returned_at: Option<NaiveDateTime>,
    returned_by: Option<Uuid>,
}
