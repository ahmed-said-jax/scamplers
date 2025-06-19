use crate::{model::Pagination, string::NonEmptyString};
#[cfg(feature = "typescript")]
use scamplers_macros::{
    frontend_insertion, frontend_ordering, frontend_ordinal_columns_enum, frontend_query_request,
    frontend_with_getters,
};
use uuid::Uuid;
#[cfg(feature = "backend")]
use {
    scamplers_macros::{
        backend_insertion, backend_ordering, backend_ordinal_columns_enum, backend_query_request,
        backend_with_getters,
    },
    scamplers_schema::lab,
};

#[cfg_attr(feature = "backend", backend_insertion(lab), derive(bon::Builder))]
#[cfg_attr(feature = "backend", builder(on(NonEmptyString, into)))]
#[cfg_attr(feature = "typescript", frontend_insertion)]
pub struct NewLab {
    #[cfg_attr(feature = "backend", garde(dive))]
    name: NonEmptyString,
    pi_id: Uuid,
    #[cfg_attr(feature = "backend", garde(dive))]
    delivery_dir: NonEmptyString,
    #[cfg_attr(feature = "backend", diesel(skip_insertion), builder(default))]
    #[cfg_attr(feature = "typescript", builder(default))]
    member_ids: Vec<Uuid>,
}
impl NewLab {
    #[must_use]
    pub fn member_ids(&self) -> &[Uuid] {
        &self.member_ids
    }
}

#[cfg_attr(feature = "backend", backend_with_getters)]
#[cfg_attr(feature = "typescript", frontend_with_getters)]
mod with_getters {
    use crate::{
        model::{IsUpdate, person::PersonSummary},
        string::NonEmptyString,
    };
    #[cfg(feature = "typescript")]
    use scamplers_macros::{frontend_response, frontend_update};
    use uuid::Uuid;
    #[cfg(feature = "backend")]
    use {
        scamplers_macros::{backend_selection, backend_update},
        scamplers_schema::lab,
    };

    #[cfg_attr(feature = "backend", backend_selection(lab))]
    #[cfg_attr(feature = "typescript", frontend_response)]
    pub struct LabHandle {
        id: Uuid,
        link: String,
    }

    #[cfg_attr(feature = "backend", backend_selection(lab))]
    #[cfg_attr(feature = "typescript", frontend_response)]
    pub struct LabSummary {
        #[serde(flatten)]
        #[cfg_attr(feature = "backend", diesel(embed))]
        handle: LabHandle,
        name: String,
        delivery_dir: String,
    }

    #[cfg_attr(feature = "backend", backend_selection(lab))]
    #[cfg_attr(feature = "typescript", frontend_response)]
    pub struct LabCore {
        #[serde(flatten)]
        #[cfg_attr(feature = "backend", diesel(embed))]
        summary: LabSummary,
        #[cfg_attr(feature = "backend", diesel(embed))]
        pi: PersonSummary,
    }

    #[cfg_attr(feature = "backend", derive(serde::Serialize, bon::Builder))]
    #[cfg_attr(feature = "typescript", frontend_response)]
    pub struct Lab {
        #[serde(flatten)]
        core: LabCore,
        members: Vec<PersonSummary>,
    }

    #[cfg_attr(feature = "backend", backend_update(lab))]
    #[cfg_attr(feature = "typescript", frontend_update)]
    pub struct LabUpdateCore {
        id: Uuid,
        #[cfg_attr(feature = "backend", garde(dive))]
        name: Option<NonEmptyString>,
        pi_id: Option<Uuid>,
        #[cfg_attr(feature = "backend", garde(dive))]
        delivery_dir: Option<NonEmptyString>,
    }
    impl IsUpdate for LabUpdateCore {
        fn is_update(&self) -> bool {
            !matches!(
                self,
                Self {
                    name: None,
                    pi_id: None,
                    delivery_dir: None,
                    ..
                },
            )
        }
    }

    #[cfg_attr(
        feature = "backend",
        derive(serde::Deserialize, Default, garde::Validate),
        serde(default)
    )]
    #[cfg_attr(feature = "backend", garde(allow_unvalidated))]
    #[cfg_attr(feature = "typescript", frontend_update)]
    pub struct LabUpdate {
        #[serde(flatten)]
        #[cfg_attr(feature = "backend", garde(dive))]
        core: LabUpdateCore,
        add_members: Vec<Uuid>,
        remove_members: Vec<Uuid>,
    }
    impl LabUpdate {
        #[must_use]
        pub fn core(&self) -> &LabUpdateCore {
            &self.core
        }
    }

    #[cfg(feature = "backend")]
    #[bon::bon]
    impl LabUpdate {
        #[builder(on(NonEmptyString, into))]
        pub fn new(
            #[builder(field)] add_members: Vec<Uuid>,
            #[builder(field)] remove_members: Vec<Uuid>,
            id: Uuid,
            name: Option<NonEmptyString>,
            pi_id: Option<Uuid>,
            delivery_dir: Option<NonEmptyString>,
        ) -> Self {
            Self {
                core: LabUpdateCore {
                    id,
                    name,
                    pi_id,
                    delivery_dir,
                },
                add_members,
                remove_members,
            }
        }
    }

    #[cfg(feature = "backend")]
    impl<S: lab_update_builder::State> LabUpdateBuilder<S> {
        pub fn add_member(mut self, member_id: Uuid) -> Self {
            self.add_members.push(member_id);
            self
        }
        pub fn remove_member(mut self, member_id: Uuid) -> Self {
            self.remove_members.push(member_id);
            self
        }
    }
}
pub use with_getters::*;

#[cfg_attr(feature = "backend", backend_ordinal_columns_enum)]
#[cfg_attr(feature = "typescript", frontend_ordinal_columns_enum)]
pub enum LabOrdinalColumn {
    #[default]
    Name,
}

#[cfg_attr(feature = "backend", backend_ordering)]
#[cfg_attr(feature = "typescript", frontend_ordering)]
pub struct LabOrdering {
    pub column: LabOrdinalColumn,
    pub descending: bool,
}

#[cfg_attr(feature = "backend", backend_query_request)]
#[cfg_attr(feature = "typescript", frontend_query_request)]
pub struct LabQuery {
    pub ids: Vec<Uuid>,
    pub name: Option<String>,
    pub order_by: Vec<LabOrdering>,
    pub pagination: Pagination,
}
