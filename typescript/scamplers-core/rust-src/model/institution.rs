use uuid::Uuid;

use crate::model::Pagination;

use super::Endpoint;

#[cfg(feature = "backend")]
use {
    scamplers_macros::{
        backend_insertion, backend_ordering, backend_ordinal_columns_enum, backend_query_request,
        backend_selection,
    },
    scamplers_schema::institution,
};

use super::SEARCH_SUFFIX;
#[cfg(feature = "typescript")]
use scamplers_macros::{
    frontend_enum, frontend_ordering, frontend_query_request, frontend_response,
    frontend_write_request,
};

const ENDPOINT: &str = "/institutions";

#[cfg_attr(feature = "backend", backend_insertion(institution), derive(Clone))]
#[cfg_attr(feature = "typescript", frontend_write_request)]
pub struct NewInstitution {
    pub id: Uuid,
    #[cfg_attr(feature = "backend", garde(length(min = 1)))]
    pub name: String,
}
impl Endpoint for NewInstitution {
    fn endpoint() -> String {
        ENDPOINT.to_string()
    }
}

#[cfg_attr(feature = "backend", backend_selection(institution))]
#[cfg_attr(feature = "typescript", frontend_response)]
pub struct InstitutionReference {
    pub id: Uuid,
    pub link: String,
}

#[cfg_attr(feature = "backend", backend_selection(institution))]
#[cfg_attr(feature = "typescript", frontend_response)]
pub struct InstitutionSummary {
    #[serde(flatten)]
    #[cfg_attr(feature = "backend", diesel(embed))]
    pub reference: InstitutionReference,
    pub name: String,
}
impl Endpoint for InstitutionSummary {
    fn endpoint() -> String {
        format!("{ENDPOINT}/{SEARCH_SUFFIX}")
    }
}

#[cfg_attr(
    feature = "backend",
    backend_selection(institution),
    serde(transparent)
)]
#[cfg_attr(feature = "typescript", frontend_response)]
pub struct Institution(#[cfg_attr(feature = "backend", diesel(embed))] pub InstitutionSummary);
impl Endpoint for Institution {
    fn endpoint() -> String {
        format!("{ENDPOINT}/{{institution_id}}")
    }
}

#[cfg_attr(feature = "backend", backend_ordinal_columns_enum)]
#[cfg_attr(feature = "typescript", frontend_enum)]
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
