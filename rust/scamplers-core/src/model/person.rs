use crate::model::Pagination;

use super::{Endpoint, institution::Institution};

#[cfg(feature = "backend")]
use {
    scamplers_macros::{
        backend_db_enum, backend_insertion, backend_ordering, backend_ordinal_columns_enum,
        backend_query_request, backend_selection,
    },
    scamplers_schema::person,
};

#[cfg(feature = "typescript")]
use scamplers_macros::{
    frontend_enum, frontend_ordering, frontend_query_request, frontend_response,
    frontend_write_request,
};

use super::SEARCH_SUFFIX;
use uuid::Uuid;

const ENDPOINT: &str = "/people";

#[cfg_attr(feature = "backend", backend_db_enum)]
#[cfg_attr(feature = "typescript", frontend_enum)]
pub enum UserRole {
    AppAdmin,
    ComputationalStaff,
    BiologyStaff,
    #[default]
    Unknown,
}

#[cfg_attr(feature = "backend", backend_insertion(person), derive(Clone))]
#[cfg_attr(feature = "typescript", frontend_write_request)]
pub struct NewPerson {
    #[cfg_attr(feature = "backend", garde(length(min = 1)))]
    pub name: String,
    #[cfg_attr(feature = "backend", garde(email))]
    pub email: String,
    pub orcid: Option<String>,
    pub institution_id: Uuid,
    pub ms_user_id: Option<Uuid>,
    #[cfg_attr(feature = "backend", diesel(skip_insertion), serde(default))]
    #[cfg_attr(feature = "typescript", builder(default))]
    pub roles: Vec<UserRole>,
}
impl Endpoint for NewPerson {
    fn endpoint() -> String {
        ENDPOINT.to_string()
    }
}
impl NewPerson {
    #[must_use]
    pub fn new_user_endpoint() -> String {
        "/users".to_string()
    }
}

#[cfg_attr(feature = "backend", backend_selection(person))]
#[cfg_attr(feature = "typescript", frontend_response)]
pub struct Person {
    pub id: Uuid,
    pub name: String,
    pub link: String,
    pub email: Option<String>,
    pub orcid: Option<String>,
    #[cfg_attr(feature = "backend", diesel(embed))]
    pub institution: Institution,
}
impl Endpoint for Person {
    fn endpoint() -> String {
        format!("{ENDPOINT}/{{person_id}}")
    }
}

#[cfg_attr(feature = "backend", backend_selection(person))]
#[cfg_attr(feature = "typescript", frontend_response)]
pub struct PersonSummary {
    pub id: Uuid,
    pub name: String,
    pub link: String,
    pub email: Option<String>,
    pub orcid: Option<String>,
}
impl Endpoint for PersonSummary {
    fn endpoint() -> String {
        format!("{ENDPOINT}/{SEARCH_SUFFIX}")
    }
}

#[cfg_attr(feature = "backend", backend_selection(person))]
#[cfg_attr(feature = "typescript", frontend_response)]
pub struct PersonReference {
    pub id: Uuid,
    pub link: String,
}

#[cfg_attr(feature = "backend", derive(serde::Serialize, Debug))]
#[cfg_attr(feature = "typescript", frontend_response)]
pub struct CreatedUser {
    pub person: Person,
    pub api_key: String,
}

#[cfg_attr(feature = "backend", backend_ordinal_columns_enum)]
#[cfg_attr(feature = "typescript", frontend_enum)]
pub enum PersonOrdinalColumn {
    #[default]
    Name,
    Email,
}

#[cfg_attr(feature = "backend", backend_ordering)]
#[cfg_attr(feature = "typescript", frontend_ordering)]
pub struct PersonOrdering {
    pub column: PersonOrdinalColumn,
    pub descending: bool,
}

#[cfg_attr(feature = "backend", backend_query_request)]
#[cfg_attr(feature = "typescript", frontend_query_request)]
pub struct PersonQuery {
    pub ids: Vec<Uuid>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub order_by: Vec<PersonOrdering>,
    pub pagination: Pagination,
}
