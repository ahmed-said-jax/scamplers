use uuid::Uuid;

use crate::model::Pagination;

use super::Endpoint;

#[cfg(feature = "backend")]
use {
    scamplers_macros::{
        insert_struct, ordering_struct, ordinal_columns, query_struct, select_struct,
    },
    scamplers_schema::institution,
};

use super::SEARCH_SUFFIX;
#[cfg(feature = "typescript")]
use scamplers_macros::{api_enum, api_request, api_response};

const ENDPOINT: &str = "/institutions";

#[cfg_attr(feature = "backend", insert_struct(institution), derive(Clone))]
#[cfg_attr(feature = "typescript", api_request)]
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

#[cfg_attr(feature = "backend", select_struct(institution))]
#[cfg_attr(feature = "typescript", api_response)]
pub struct Institution {
    pub id: Uuid,
    pub name: String,
    pub link: String,
}
impl Endpoint for Institution {
    fn endpoint() -> String {
        format!("{ENDPOINT}/{{institution_id}}")
    }
}

#[cfg_attr(feature = "backend", select_struct(institution))]
#[cfg_attr(feature = "typescript", api_response)]
pub struct InstitutionSummary {
    pub id: Uuid,
    pub name: String,
    pub link: String,
}
impl Endpoint for InstitutionSummary {
    fn endpoint() -> String {
        format!("{ENDPOINT}/{SEARCH_SUFFIX}")
    }
}

#[cfg_attr(feature = "backend", select_struct(institution))]
#[cfg_attr(feature = "typescript", api_response)]
pub struct InstitutionReference {
    pub id: Uuid,
    pub link: String,
}

#[cfg_attr(feature = "backend", ordinal_columns)]
#[cfg_attr(feature = "typescript", api_enum)]
pub enum InstitutionOrdinalColumn {
    #[default]
    Name,
}

#[cfg_attr(feature = "backend", ordering_struct)]
#[cfg_attr(feature = "typescript", api_request)]
pub struct InstitutionOrdering {
    pub column: InstitutionOrdinalColumn,
    pub descending: bool,
}

#[cfg_attr(feature = "backend", query_struct)]
#[cfg_attr(feature = "typescript", api_request)]
pub struct InstitutionQuery {
    #[cfg_attr(feature = "backend", serde(default))]
    #[cfg_attr(feature = "typescript", builder(default))]
    pub ids: Vec<Uuid>,
    #[cfg_attr(feature = "typescript", builder(default))]
    pub name: Option<String>,
    #[cfg_attr(feature = "backend", serde(default = "super::default_ordering"))]
    pub order_by: Vec<InstitutionOrdering>,
    #[cfg_attr(feature = "backend", serde(default))]
    pub pagination: Pagination,
}
