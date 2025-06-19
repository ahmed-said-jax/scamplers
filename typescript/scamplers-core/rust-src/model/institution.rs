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
    scamplers_schema::institution,
};

#[cfg_attr(
    feature = "backend",
    backend_insertion(institution),
    derive(Clone, bon::Builder)
)]
#[cfg_attr(feature = "backend", builder(on(NonEmptyString, into)))]
#[cfg_attr(feature = "typescript", frontend_insertion)]
pub struct NewInstitution {
    id: Uuid,
    #[cfg_attr(feature = "backend", garde(dive))]
    name: NonEmptyString,
}

#[cfg_attr(feature = "backend", backend_with_getters)]
#[cfg_attr(feature = "typescript", frontend_with_getters)]
mod read {
    #[cfg(feature = "typescript")]
    use scamplers_macros::frontend_response;
    use uuid::Uuid;
    #[cfg(feature = "backend")]
    use {scamplers_macros::backend_selection, scamplers_schema::institution};

    #[cfg_attr(feature = "backend", backend_selection(institution))]
    #[cfg_attr(feature = "typescript", frontend_response)]
    pub struct InstitutionHandle {
        id: Uuid,
        link: String,
    }

    #[cfg_attr(feature = "backend", backend_selection(institution))]
    #[cfg_attr(feature = "typescript", frontend_response)]
    pub struct Institution {
        #[serde(flatten)]
        #[cfg_attr(feature = "backend", diesel(embed))]
        handle: InstitutionHandle,
        name: String,
    }
}
pub use read::*;

#[cfg_attr(feature = "backend", backend_ordinal_columns_enum)]
#[cfg_attr(feature = "typescript", frontend_ordinal_columns_enum)]
pub enum InstitutionOrdinalColumn {
    #[default]
    Name,
}

#[cfg_attr(feature = "backend", backend_ordering)]
#[cfg_attr(feature = "typescript", frontend_ordering)]
pub struct InstitutionOrdering {
    pub column: InstitutionOrdinalColumn,
    pub descending: bool,
}

#[cfg_attr(feature = "backend", backend_query_request)]
#[cfg_attr(feature = "typescript", frontend_query_request)]
pub struct InstitutionQuery {
    pub ids: Vec<Uuid>,
    pub name: Option<String>,
    pub order_by: Vec<InstitutionOrdering>,
    pub pagination: Pagination,
}
