use crate::string::NonEmptyString;
#[cfg(feature = "typescript")]
use scamplers_macros::{frontend_enum, frontend_insertion, frontend_with_getters};
use time::OffsetDateTime;
use uuid::Uuid;
#[cfg(feature = "backend")]
use {
    scamplers_macros::{backend_db_enum, backend_insertion, backend_with_getters},
    scamplers_schema::{committee_approval, sample_metadata},
};

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
}

#[cfg_attr(feature = "backend", backend_db_enum)]
#[cfg_attr(feature = "typescript", frontend_enum)]
pub enum ComplianceCommitteeType {
    Ibc,
    Irb,
    Iacuc,
}

#[cfg_attr(feature = "backend", backend_insertion(committee_approval))]
#[cfg_attr(feature = "typescript", frontend_insertion)]
pub struct NewCommitteeApproval {
    #[serde(default)]
    pub sample_id: Option<Uuid>,
    pub institution_id: Uuid,
    pub committee_type: ComplianceCommitteeType,
    #[cfg_attr(feature = "backend", garde(dive))]
    pub compliance_identifier: NonEmptyString,
}

#[cfg_attr(feature = "backend", backend_with_getters)]
#[cfg_attr(feature = "typescript", frontend_with_getters)]
mod read_committee_approval {
    use super::ComplianceCommitteeType;
    use crate::model::institution::InstitutionSummary;
    #[cfg(feature = "typescript")]
    use scamplers_macros::frontend_response;
    #[cfg(feature = "backend")]
    use {scamplers_macros::backend_selection, scamplers_schema::committee_approval};

    #[cfg_attr(feature = "backend", backend_selection(committee_approval))]
    #[cfg_attr(feature = "typescript", frontend_response)]
    pub struct CommitteeApproval {
        #[cfg_attr(feature = "backend", diesel(embed))]
        institution: InstitutionSummary,
        committee_type: ComplianceCommitteeType,
        compliance_identifier: String,
    }
}
pub use read_committee_approval::*;

#[cfg_attr(feature = "backend", backend_insertion(sample_metadata))]
pub struct NewSampleMetadata {
    #[cfg_attr(feature = "backend", garde(dive))]
    pub readable_id: NonEmptyString,
    #[cfg_attr(feature = "backend", garde(dive))]
    pub name: NonEmptyString,
    pub submitted_by: Uuid,
    pub lab_id: Uuid,
    #[cfg_attr(feature = "backend", valuable(skip))]
    pub received_at: OffsetDateTime,
    #[cfg_attr(feature = "backend", garde(length(min = 1)))]
    pub species: Vec<Species>,
    #[cfg_attr(feature = "backend", garde(dive))]
    pub tissue: NonEmptyString,
    #[cfg_attr(feature = "backend", diesel(skip_insertion), serde(flatten))]
    pub committee_approvals: Vec<NewCommitteeApproval>,
    #[cfg_attr(feature = "backend", garde(dive))]
    pub notes: Option<Vec<NonEmptyString>>,
    #[cfg_attr(feature = "backend", valuable(skip))]
    pub returned_at: Option<OffsetDateTime>,
    pub returned_by: Option<Uuid>,
}

#[cfg_attr(feature = "backend", backend_with_getters)]
mod read_sample_metadata {
    use crate::model::{
        lab::LabSummary,
        person::PersonSummary,
        sample_metadata::{CommitteeApproval, Species},
    };
    use time::OffsetDateTime;

    #[cfg(feature = "backend")]
    use {scamplers_macros::backend_selection, scamplers_schema::sample_metadata};

    #[cfg_attr(feature = "backend", backend_selection(sample_metadata))]
    pub struct SampleMetadataSummary {
        name: String,
        #[cfg_attr(feature = "backend", valuable(skip))]
        received_at: OffsetDateTime,
        species: Vec<Option<Species>>,
        tissue: String,
        notes: Option<Vec<Option<String>>>,
        #[cfg_attr(feature = "backend", valuable(skip))]
        returned_at: Option<OffsetDateTime>,
    }

    #[cfg_attr(feature = "backend", backend_selection(sample_metadata))]
    pub struct SampleMetadataData {
        #[cfg_attr(feature = "backend", diesel(embed), serde(flatten))]
        summary: SampleMetadataSummary,
        #[cfg_attr(feature = "backend", diesel(embed))]
        submitted_by: PersonSummary,
        #[cfg_attr(feature = "backend", diesel(embed))]
        lab: LabSummary,
        #[cfg_attr(feature = "backend", diesel(embed))]
        returned_by: Option<PersonSummary>,
    }

    #[cfg_attr(feature = "backend", derive(serde::Serialize))]
    pub struct SampleMetadata {
        #[cfg_attr(feature = "backend", serde(flatten))]
        data: SampleMetadataData,
        committee_approvals: Vec<CommitteeApproval>,
    }
}
pub use read_sample_metadata::*;
