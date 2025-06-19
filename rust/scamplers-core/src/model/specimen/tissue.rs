use crate::{model::specimen::core::NewSpecimenCore, string::NonEmptyString};
#[cfg(feature = "typescript")]
use scamplers_macros::frontend_enum;
#[cfg(feature = "backend")]
use {
    scamplers_macros::{backend_db_enum, backend_insertion},
    scamplers_schema::specimen,
};

#[cfg_attr(feature = "backend", backend_db_enum)]
#[cfg_attr(feature = "typescript", frontend_enum)]
#[derive(Default)]
pub enum TissueType {
    #[default]
    Tissue,
}

#[cfg_attr(feature = "backend", backend_insertion(specimen))]
pub struct NewTissueCore {
    #[cfg_attr(
        feature = "backend",
        diesel(skip_insertion),
        serde(flatten),
        garde(dive)
    )]
    core: NewSpecimenCore,
    #[cfg_attr(feature = "backend", serde(skip))]
    type_: TissueType,
    storage_buffer: Option<NonEmptyString>,
}

#[cfg_attr(feature = "backend", backend_db_enum)]
pub enum TissueFixative {
    DithiobisSuccinimidylropionate,
}

#[cfg_attr(feature = "backend", backend_insertion(specimen))]
pub struct NewFixedTissue {
    #[cfg_attr(feature = "backend", diesel(embed), serde(flatten), garde(dive))]
    core: NewTissueCore,
    fixative: TissueFixative,
}

#[cfg_attr(feature = "backend", backend_insertion(specimen))]
pub struct NewFrozenTissue {
    #[cfg_attr(feature = "backend", diesel(embed), serde(flatten), garde(dive))]
    pub core: NewTissueCore,
    #[cfg_attr(
        feature = "backend",
        serde(skip, default = "crate::util::default_true")
    )]
    pub frozen: bool,
}

#[cfg_attr(feature = "backend", backend_insertion(specimen))]
pub struct NewCryoPreservedTissue {
    #[cfg_attr(feature = "backend", diesel(embed), serde(flatten), garde(dive))]
    pub core: NewTissueCore,
    #[cfg_attr(
        feature = "backend",
        serde(skip, default = "crate::util::default_true")
    )]
    pub cryopreserved: bool,
}

#[cfg_attr(feature = "backend", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "backend",
    serde(rename_all = "snake_case", tag = "preservation")
)]
pub enum NewTissue {
    Cryopreserved(NewCryoPreservedTissue),
    Fixed(NewFixedTissue),
    Frozen(NewFrozenTissue),
}
