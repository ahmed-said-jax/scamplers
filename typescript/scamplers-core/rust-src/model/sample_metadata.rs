// use time::OffsetDateTime;
use uuid::Uuid;

#[cfg(feature = "backend")]
use {
    scamplers_macros::{backend_db_enum, backend_insertion},
    scamplers_schema::{committee_approval, sample_metadata},
};

#[cfg(feature = "typescript")]
use scamplers_macros::{frontend_enum, frontend_insertion};

#[cfg_attr(feature = "backend", backend_db_enum)]
#[cfg_attr(feature = "typescript", frontend_enum)]
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

#[cfg_attr(feature = "backend", backend_db_enum)]
#[cfg_attr(feature = "typescript", frontend_enum)]
pub enum ComplianceCommitteeType {
    Ibc,
    Irb,
    Iacuc,
    #[default]
    Unknown,
}

#[cfg_attr(feature = "backend", backend_insertion(committee_approval))]
#[cfg_attr(feature = "typescript", frontend_insertion)]
pub struct NewCommitteeApproval {
    #[serde(default)]
    pub sample_id: Option<Uuid>,
    pub institution_id: Uuid,
    pub committee_type: ComplianceCommitteeType,
    #[cfg_attr(feature = "backend", garde(length(min = 1)))]
    pub compliance_identifier: String,
}

#[cfg_attr(feature = "backend", backend_insertion(sample_metadata))]
#[cfg_attr(feature = "typescript", frontend_insertion)]
pub struct NewSampleMetadata {
    #[cfg_attr(feature = "backend", garde(length(min = 1)))]
    pub name: String,
    pub submitted_by: Uuid,
    pub lab_id: Uuid,
    // #[cfg_attr(feature = "backend", valuable(skip))]
    // pub received_at: OffsetDateTime,
    #[cfg_attr(feature = "backend", garde(length(min = 1)))]
    pub species: Vec<Species>,
    #[cfg_attr(feature = "backend", garde(length(min = 1)))]
    pub tissue: String,
    #[cfg_attr(feature = "backend", diesel(skip_insertion))]
    #[serde(default)]
    pub committee_approvals: Vec<NewCommitteeApproval>,
    pub notes: Option<Vec<String>>,
    // #[cfg_attr(feature = "backend", valuable(skip))]
    // pub returned_at: Option<OffsetDateTime>,
    pub returned_by: Option<Uuid>,
}
