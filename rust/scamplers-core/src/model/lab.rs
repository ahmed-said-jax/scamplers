use crate::{model::Pagination, string::NonEmptyString};
#[cfg(feature = "typescript")]
use scamplers_macros::{
    frontend_insertion, frontend_ordering, frontend_ordinal_columns_enum, frontend_query_request,
    frontend_update, frontend_with_getters,
};
use uuid::Uuid;
#[cfg(feature = "backend")]
use {
    scamplers_macros::{
        backend_insertion, backend_ordering, backend_ordinal_columns_enum, backend_query_request,
        backend_update, backend_with_getters,
    },
    scamplers_schema::lab,
};

#[cfg_attr(feature = "backend", backend_insertion(lab))]
#[cfg_attr(feature = "typescript", frontend_insertion)]
pub struct NewLab {
    #[cfg_attr(feature = "backend", garde(dive))]
    pub name: NonEmptyString,
    pub pi_id: Uuid,
    #[cfg_attr(feature = "backend", garde(dive))]
    pub delivery_dir: NonEmptyString,
    #[cfg_attr(feature = "backend", diesel(skip_insertion))]
    #[cfg_attr(feature = "typescript", builder(default))]
    pub member_ids: Vec<Uuid>,
}

#[cfg_attr(feature = "backend", backend_with_getters)]
#[cfg_attr(feature = "typescript", frontend_with_getters)]
mod read {
    use crate::model::person::PersonSummary;
    #[cfg(feature = "typescript")]
    use scamplers_macros::frontend_response;
    use uuid::Uuid;
    #[cfg(feature = "backend")]
    use {scamplers_macros::backend_selection, scamplers_schema::lab};

    #[cfg_attr(feature = "backend", backend_selection(lab))]
    #[cfg_attr(feature = "typescript", frontend_response)]
    pub struct LabReference {
        id: Uuid,
        link: String,
    }

    #[cfg_attr(feature = "backend", backend_selection(lab))]
    #[cfg_attr(feature = "typescript", frontend_response)]
    pub struct LabSummary {
        #[serde(flatten)]
        #[cfg_attr(feature = "backend", diesel(embed))]
        reference: LabReference,
        name: String,
        delivery_dir: String,
    }

    #[cfg_attr(feature = "backend", backend_selection(lab))]
    #[cfg_attr(feature = "typescript", frontend_response)]
    pub struct LabData {
        #[serde(flatten)]
        #[cfg_attr(feature = "backend", diesel(embed))]
        summary: LabSummary,
        #[cfg_attr(feature = "backend", diesel(embed))]
        pi: PersonSummary,
    }

    #[cfg_attr(feature = "backend", derive(serde::Serialize))]
    #[cfg_attr(feature = "typescript", frontend_response)]
    pub struct Lab {
        #[serde(flatten)]
        data: LabData,
        members: Vec<PersonSummary>,
    }

    #[cfg(feature = "backend")]
    impl Lab {
        #[must_use]
        pub fn new(data: LabData, members: Vec<PersonSummary>) -> Self {
            Self { data, members }
        }
    }
}
pub use read::*;

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

#[cfg_attr(feature = "backend", backend_update(lab))]
#[cfg_attr(feature = "typescript", frontend_update)]
pub struct LabUpdate {
    pub id: Uuid,
    #[cfg_attr(feature = "backend", garde(dive))]
    pub name: Option<NonEmptyString>,
    pub pi_id: Option<Uuid>,
    #[cfg_attr(feature = "backend", garde(dive))]
    pub delivery_dir: Option<NonEmptyString>,
}

#[cfg_attr(
    feature = "backend",
    derive(serde::Deserialize, Default, garde::Validate),
    serde(default)
)]
#[cfg_attr(feature = "backend", garde(allow_unvalidated))]
#[cfg_attr(feature = "typescript", frontend_update)]
pub struct LabUpdateWithMembers {
    #[serde(flatten)]
    #[cfg_attr(feature = "backend", garde(dive))]
    pub update: LabUpdate,
    pub add_members: Vec<Uuid>,
    pub remove_members: Vec<Uuid>,
}
