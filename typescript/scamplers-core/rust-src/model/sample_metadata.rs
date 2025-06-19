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

#[cfg_attr(
    feature = "backend",
    backend_insertion(committee_approval),
    derive(bon::Builder, Clone)
)]
#[cfg_attr(feature = "backend", builder(on(NonEmptyString, into)))]
#[cfg_attr(feature = "typescript", frontend_insertion)]
pub struct NewCommitteeApproval {
    #[serde(default)]
    sample_id: Option<Uuid>,
    institution_id: Uuid,
    committee_type: ComplianceCommitteeType,
    #[cfg_attr(feature = "backend", garde(dive))]
    compliance_identifier: NonEmptyString,
}

#[cfg_attr(feature = "backend", backend_with_getters)]
#[cfg_attr(feature = "typescript", frontend_with_getters)]
mod with_committee_approval_getters {
    use super::ComplianceCommitteeType;
    use crate::model::institution::InstitutionHandle;
    #[cfg(feature = "typescript")]
    use scamplers_macros::frontend_response;
    #[cfg(feature = "backend")]
    use {scamplers_macros::backend_selection, scamplers_schema::committee_approval};

    #[cfg_attr(feature = "backend", backend_selection(committee_approval))]
    #[cfg_attr(feature = "typescript", frontend_response)]
    pub struct CommitteeApproval {
        #[cfg_attr(feature = "backend", diesel(embed))]
        institution: InstitutionHandle,
        committee_type: ComplianceCommitteeType,
        compliance_identifier: String,
    }
}
pub use with_committee_approval_getters::*;

#[cfg_attr(feature = "backend", backend_insertion(sample_metadata), derive(Clone))]
pub struct NewSampleMetadata {
    #[cfg_attr(feature = "backend", garde(dive))]
    pub(super) readable_id: NonEmptyString,
    #[cfg_attr(feature = "backend", garde(dive))]
    pub(super) name: NonEmptyString,
    pub(super) submitted_by: Uuid,
    pub(super) lab_id: Uuid,
    #[cfg_attr(feature = "backend", valuable(skip))]
    pub(super) received_at: OffsetDateTime,
    #[cfg_attr(feature = "backend", garde(length(min = 1)))]
    pub(super) species: Vec<Species>,
    #[cfg_attr(feature = "backend", diesel(skip_insertion), serde(flatten))]
    pub(super) committee_approvals: Vec<NewCommitteeApproval>,
    #[cfg_attr(feature = "backend", garde(dive))]
    pub(super) notes: Option<Vec<NonEmptyString>>,
    #[cfg_attr(feature = "backend", valuable(skip))]
    pub(super) returned_at: Option<OffsetDateTime>,
    pub(super) returned_by: Option<Uuid>,
}
impl NewSampleMetadata {
    pub fn committee_approvals(&mut self, sample_id: Uuid) -> &[NewCommitteeApproval] {
        for approval in &mut self.committee_approvals {
            approval.sample_id = Some(sample_id);
        }

        &self.committee_approvals
    }
}

#[cfg_attr(feature = "backend", backend_with_getters)]
mod with_sample_metadata_getters {
    use crate::model::{
        lab::LabHandle,
        person::PersonHandle,
        sample_metadata::{CommitteeApproval, Species},
    };
    use time::OffsetDateTime;
    use uuid::Uuid;

    #[cfg(feature = "backend")]
    use {scamplers_macros::backend_selection, scamplers_schema::sample_metadata};

    #[cfg_attr(feature = "backend", backend_selection(sample_metadata))]
    pub struct SampleMetadataSummary {
        #[cfg_attr(feature = "backend", diesel(column_name = id), serde(skip))]
        metadata_id: Uuid,
        name: String,
        #[cfg_attr(feature = "backend", valuable(skip))]
        received_at: OffsetDateTime,
        species: Vec<Option<Species>>,
        notes: Option<Vec<Option<String>>>,
        #[cfg_attr(feature = "backend", valuable(skip))]
        returned_at: Option<OffsetDateTime>,
    }

    #[cfg_attr(feature = "backend", backend_selection(sample_metadata))]
    pub struct SampleMetadataCore {
        #[cfg_attr(feature = "backend", diesel(embed), serde(flatten))]
        summary: SampleMetadataSummary,
        #[cfg_attr(feature = "backend", diesel(embed))]
        submitted_by: PersonHandle,
        #[cfg_attr(feature = "backend", diesel(embed))]
        lab: LabHandle,
        #[cfg_attr(feature = "backend", diesel(embed))]
        returned_by: Option<PersonHandle>,
    }

    #[cfg_attr(feature = "backend", derive(serde::Serialize, bon::Builder))]
    pub struct SampleMetadata {
        #[cfg_attr(feature = "backend", serde(flatten))]
        core: SampleMetadataCore,
        #[cfg_attr(feature = "backend", builder(default))]
        committee_approvals: Vec<CommitteeApproval>,
    }
}
pub use with_sample_metadata_getters::*;
