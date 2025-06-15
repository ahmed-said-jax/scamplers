use super::{Endpoint, Pagination, institution::Institution};

#[cfg(feature = "backend")]
use {
    scamplers_macros::{
        backend_db_enum, backend_insertion, backend_ordering, backend_ordinal_columns_enum,
        backend_query_request, backend_selection, backend_update,
    },
    scamplers_schema::person,
};

#[cfg(feature = "typescript")]
use scamplers_macros::{
    frontend_enum, frontend_insertion, frontend_ordering, frontend_query_request,
    frontend_response, frontend_update,
};

use super::SEARCH_SUFFIX;
use uuid::Uuid;

const ENDPOINT: &str = "/people";

#[derive(PartialEq)]
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
#[cfg_attr(feature = "typescript", frontend_insertion)]
pub struct NewPerson {
    #[cfg_attr(feature = "backend", garde(length(min = 1)))]
    pub name: String,
    #[cfg_attr(feature = "backend", garde(email))]
    pub email: String,
    #[cfg_attr(feature = "typescript", builder(default))]
    pub orcid: Option<String>,
    pub institution_id: Uuid,
    #[cfg_attr(feature = "typescript", builder(default))]
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

#[cfg_attr(feature = "backend", backend_update(person))]
#[cfg_attr(feature = "typescript", frontend_update)]
pub struct PersonUpdate {
    pub id: Uuid,
    pub name: Option<String>,
    pub email: Option<String>,
    pub ms_user_id: Option<Uuid>,
    pub orcid: Option<String>,
    pub institution_id: Option<Uuid>,
}

#[cfg_attr(
    feature = "backend",
    derive(serde::Deserialize, Default),
    serde(default)
)]
#[cfg_attr(feature = "typescript", frontend_update)]
pub struct PersonUpdateWithRoles {
    #[serde(flatten)]
    pub update: PersonUpdate,
    pub add_roles: Vec<UserRole>,
    pub remove_roles: Vec<UserRole>,
}

#[cfg_attr(feature = "backend", backend_selection(person))]
#[cfg_attr(feature = "typescript", frontend_response)]
pub struct PersonReference {
    pub id: Uuid,
    pub link: String,
}

#[cfg_attr(feature = "backend", backend_selection(person))]
#[cfg_attr(feature = "typescript", frontend_response)]
pub struct PersonSummary {
    #[serde(flatten)]
    #[cfg_attr(feature = "backend", diesel(embed))]
    pub reference: PersonReference,
    pub name: String,
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
pub struct Person {
    #[serde(flatten)]
    #[cfg_attr(feature = "backend", diesel(embed))]
    pub summary: PersonSummary,
    #[cfg_attr(feature = "backend", diesel(embed))]
    pub institution: Institution,
}

#[cfg_attr(feature = "backend", derive(serde::Serialize, Debug))]
#[cfg_attr(feature = "typescript", frontend_response)]
pub struct PersonWithRoles {
    #[serde(flatten)]
    pub person: Person,
    pub roles: Vec<UserRole>,
}

impl Endpoint for PersonWithRoles {
    fn endpoint() -> String {
        format!("{ENDPOINT}/{{person_id}}")
    }
}

#[cfg_attr(feature = "backend", derive(serde::Serialize, Debug))]
#[cfg_attr(feature = "typescript", frontend_response)]
pub struct CreatedUser {
    pub person: PersonWithRoles,
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
