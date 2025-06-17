use crate::{model::Pagination, string::NonEmptyString};
#[cfg(feature = "typescript")]
use scamplers_macros::{
    frontend_insertion, frontend_ordering, frontend_ordinal_columns_enum, frontend_query_request,
    frontend_response, frontend_with_getters,
};
use uuid::Uuid;
#[cfg(feature = "backend")]
use {
    scamplers_macros::{
        backend_insertion, backend_ordering, backend_ordinal_columns_enum, backend_query_request,
        backend_selection, backend_with_getters,
    },
    scamplers_schema::institution,
};

#[cfg_attr(feature = "backend", backend_insertion(institution), derive(Clone))]
#[cfg_attr(feature = "typescript", frontend_insertion)]
pub struct NewInstitution {
    pub id: Uuid,
    #[cfg_attr(feature = "backend", garde(dive))]
    pub name: NonEmptyString,
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
    pub struct InstitutionReference {
        id: Uuid,
        link: String,
    }

    #[cfg_attr(feature = "backend", backend_selection(institution))]
    #[cfg_attr(feature = "typescript", frontend_response)]
    pub struct InstitutionSummary {
        #[serde(flatten)]
        #[cfg_attr(feature = "backend", diesel(embed))]
        reference: InstitutionReference,
        name: String,
    }
}
pub use read::*;

#[cfg_attr(
    feature = "backend",
    backend_selection(institution),
    serde(transparent)
)]
#[cfg_attr(feature = "typescript", frontend_response)]
pub struct Institution(#[cfg_attr(feature = "backend", diesel(embed))] InstitutionSummary);

#[cfg_attr(feature = "typescript", wasm_bindgen::prelude::wasm_bindgen)]
impl Institution {
    #[cfg(feature = "backend")]
    #[must_use]
    pub fn id(&self) -> &Uuid {
        self.0.id()
    }

    #[cfg(feature = "typescript")]
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.0.id()
    }

    #[cfg(feature = "backend")]
    #[must_use]
    pub fn link(&self) -> &str {
        self.0.link()
    }

    #[cfg(feature = "typescript")]
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn link(&self) -> String {
        self.0.link()
    }

    #[cfg(feature = "backend")]
    #[must_use]
    pub fn name(&self) -> &str {
        self.0.name()
    }

    #[cfg(feature = "typescript")]
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn name(&self) -> String {
        self.0.name()
    }
}

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
