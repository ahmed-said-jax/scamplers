use crate::model::specimen::core::NewSpecimenCore;
#[cfg(feature = "backend")]
use crate::{
    model::{
        sample_metadata::{NewCommitteeApproval, Species},
        specimen::MeasurementData,
    },
    string::NonEmptyString,
};
#[cfg(feature = "typescript")]
use scamplers_macros::frontend_enum;
#[cfg(feature = "backend")]
use time::OffsetDateTime;
#[cfg(feature = "backend")]
use uuid::Uuid;
#[cfg(feature = "backend")]
use {
    scamplers_macros::{backend_db_enum, backend_insertion},
    scamplers_schema::specimen,
};

#[cfg_attr(feature = "backend", backend_db_enum)]
#[cfg_attr(feature = "typescript", frontend_enum)]
#[derive(Default)]
pub enum BlockType {
    #[default]
    Block,
}

#[cfg_attr(feature = "backend", backend_insertion(specimen))]
pub struct NewBlockCore {
    #[cfg_attr(
        feature = "backend",
        diesel(skip_insertion),
        serde(flatten),
        garde(dive)
    )]
    core: NewSpecimenCore,
    type_: BlockType,
}

#[cfg_attr(feature = "backend", backend_db_enum)]
pub enum FixedBlockEmbeddingMatrix {
    Paraffin,
}

#[cfg_attr(feature = "backend", backend_db_enum)]
pub enum BlockFixative {
    FormaldehydeDerivative,
}

#[cfg_attr(feature = "backend", backend_insertion(specimen))]
pub struct NewFixedBlock {
    #[cfg_attr(feature = "backend", diesel(embed), serde(flatten), garde(dive))]
    core: NewBlockCore,
    embedded_in: FixedBlockEmbeddingMatrix,
    fixative: BlockFixative,
}

#[cfg(feature = "backend")]
#[bon::bon]
impl NewFixedBlock {
    #[builder(on(NonEmptyString, into))]
    pub fn new(
        readable_id: NonEmptyString,
        name: NonEmptyString,
        submitted_by: Uuid,
        lab_id: Uuid,
        received_at: OffsetDateTime,
        species: Vec<Species>,
        #[builder(default)] committee_approvals: Vec<NewCommitteeApproval>,
        notes: Option<Vec<NonEmptyString>>,
        returned_at: Option<OffsetDateTime>,
        returned_by: Option<Uuid>,
        #[builder(default)] measurements: Vec<MeasurementData>,
    ) -> Self {
        use crate::model::sample_metadata::NewSampleMetadata;

        Self {
            core: NewBlockCore {
                core: NewSpecimenCore {
                    metadata: NewSampleMetadata {
                        readable_id,
                        name,
                        submitted_by,
                        lab_id,
                        received_at,
                        species,
                        committee_approvals,
                        notes,
                        returned_at,
                        returned_by,
                    },
                    measurements,
                },
                type_: BlockType::Block,
            },
            embedded_in: FixedBlockEmbeddingMatrix::Paraffin,
            fixative: BlockFixative::FormaldehydeDerivative,
        }
    }
}

#[cfg_attr(feature = "backend", backend_db_enum)]
#[cfg_attr(feature = "typescript", frontend_enum)]
pub enum FrozenBlockEmbeddingMatrix {
    CarboxymethylCellulose,
    OptimalCuttingTemperatureCompound,
}

#[cfg_attr(feature = "backend", backend_insertion(specimen))]
pub struct NewFrozenBlock {
    #[cfg_attr(feature = "backend", diesel(embed), serde(flatten), garde(dive))]
    core: NewBlockCore,
    embedded_in: FrozenBlockEmbeddingMatrix,
    fixative: Option<BlockFixative>,
    #[cfg_attr(
        feature = "backend",
        serde(skip, default = "crate::util::default_true")
    )]
    frozen: bool,
}

#[cfg(feature = "backend")]
#[bon::bon]
impl NewFrozenBlock {
    #[builder(on(NonEmptyString, into))]
    pub fn new(
        readable_id: NonEmptyString,
        name: NonEmptyString,
        submitted_by: Uuid,
        lab_id: Uuid,
        received_at: OffsetDateTime,
        species: Vec<Species>,
        #[builder(default)] committee_approvals: Vec<NewCommitteeApproval>,
        notes: Option<Vec<NonEmptyString>>,
        returned_at: Option<OffsetDateTime>,
        returned_by: Option<Uuid>,
        #[builder(default)] measurements: Vec<MeasurementData>,
        embedded_in: FrozenBlockEmbeddingMatrix,
        fixative: Option<BlockFixative>,
    ) -> Self {
        use crate::model::sample_metadata::NewSampleMetadata;

        Self {
            core: NewBlockCore {
                core: NewSpecimenCore {
                    metadata: NewSampleMetadata {
                        readable_id,
                        name,
                        submitted_by,
                        lab_id,
                        received_at,
                        species,
                        committee_approvals,
                        notes,
                        returned_at,
                        returned_by,
                    },
                    measurements,
                },
                type_: BlockType::Block,
            },
            embedded_in,
            fixative,
            frozen: true,
        }
    }
}

#[cfg_attr(feature = "backend", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "backend",
    serde(rename_all = "snake_case", tag = "preservation")
)]
pub enum NewBlock {
    Fixed(NewFixedBlock),
    Frozen(NewFrozenBlock),
}
