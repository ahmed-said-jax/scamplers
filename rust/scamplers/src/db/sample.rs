use chrono::{NaiveDate, NaiveDateTime};
use diesel::{expression::AsExpression, sql_types, prelude::*, pg::Pg};
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::schema;

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
    pub name: String,
    pub submitted_by: Uuid,
    pub lab_id: Uuid,
    pub received_at: NaiveDateTime,
    #[garde(length(min = 1))]
    pub species: Vec<Species>,
    #[garde(length(min = 1))]
    pub tissue: String,
    #[diesel(skip_insertion)]
    pub committee_approvals: Option<Vec<NewCommitteeApproval>>,
    pub notes: Option<Vec<String>>,
    pub returned_at: Option<NaiveDateTime>,
    pub returned_by: Option<Uuid>,
}


struct Measurement {
    quantity: uom::
}

pub enum NewSample {
    Specimen {
        metadata: NewSampleMetadata,
        
    }
}