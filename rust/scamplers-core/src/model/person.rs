use crate::model::Pagination;

use super::{Endpoint, institution::Institution};

#[cfg(feature = "backend")]
use {
    scamplers_macros::{db_enum, insert_struct, ordinal_columns, query_struct, select_struct},
    scamplers_schema::person,
};

#[cfg(feature = "typescript")]
use scamplers_macros::{api_enum, api_request, api_response};

use uuid::Uuid;

#[cfg_attr(feature = "backend", db_enum)]
#[cfg_attr(feature = "typescript", api_enum)]
pub enum UserRole {
    AppAdmin,
    ComputationalStaff,
    BiologyStaff,
    #[default]
    Unknown,
}

#[cfg_attr(feature = "backend", insert_struct(person), derive(Clone))]
#[cfg_attr(feature = "typescript", api_request)]
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

#[cfg_attr(feature = "backend", select_struct(person))]
#[cfg_attr(feature = "typescript", api_response)]
pub struct Person {
    pub id: Uuid,
    pub name: String,
    pub link: String,
    pub email: String,
    pub orcid: Option<String>,
    #[cfg_attr(feature = "backend", diesel(embed))]
    pub institution: Institution,
}
impl Endpoint for Person {
    fn endpoint() -> &'static str {
        "/people/{id}"
    }
}

#[cfg_attr(feature = "backend", select_struct(person))]
#[cfg_attr(feature = "typescript", api_response)]
pub struct PersonSummary {
    pub id: Uuid,
    pub name: String,
    pub link: String,
    pub email: String,
    pub orcid: Option<String>,
}
impl Endpoint for PersonSummary {
    fn endpoint() -> &'static str {
        "/people"
    }
}

#[cfg_attr(feature = "backend", select_struct(person))]
#[cfg_attr(feature = "typescript", api_response)]
pub struct PersonReference {
    pub id: Uuid,
    pub link: String,
}

#[cfg_attr(feature = "backend", derive(serde::Serialize, Debug))]
#[cfg_attr(feature = "typescript", api_response)]
pub struct CreatedUser {
    pub person: Person,
    pub api_key: Option<String>,
}
impl Endpoint for CreatedUser {
    fn endpoint() -> &'static str {
        "/users"
    }
}

#[cfg_attr(feature = "backend", ordinal_columns)]
#[cfg_attr(feature = "typescript", api_enum)]
pub enum PersonOrdinalColumn {
    Id,
    #[default]
    Name,
    Email,
}

#[cfg_attr(feature = "backend", query_struct)]
#[cfg_attr(feature = "typescript", api_request)]
pub struct PersonOrdering {
    pub column: PersonOrdinalColumn,
    pub descending: bool,
}

#[cfg_attr(feature = "backend", query_struct)]
#[cfg_attr(feature = "typescript", api_request)]
pub struct PersonQuery {
    #[cfg_attr(feature = "backend", serde(default))]
    #[cfg_attr(feature = "typescript", builder(default))]
    pub ids: Vec<Uuid>,
    #[cfg_attr(feature = "typescript", builder(default))]
    pub name: Option<String>,
    #[cfg_attr(feature = "typescript", builder(default))]
    pub email: Option<String>,
    #[cfg_attr(feature = "backend", serde(default))]
    pub order_by: Vec<PersonOrdering>,
    #[cfg_attr(feature = "backend", serde(default))]
    pub pagination: Pagination,
}
