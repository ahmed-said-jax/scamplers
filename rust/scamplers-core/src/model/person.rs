use std::default;

use super::{AsEndpoint, institution::Institution};

#[cfg(feature = "backend")]
use {
    scamplers_macros::{db_enum, filter_struct, insert_struct, select_struct},
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

#[cfg_attr(feature = "backend", insert_struct(person))]
#[cfg_attr(feature = "typescript", api_request)]
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
impl AsEndpoint for Person {
    fn as_endpoint() -> &'static str {
        "people"
    }
}

#[cfg_attr(feature = "backend", derive(serde::Serialize, Debug))]
#[cfg_attr(feature = "typescript", api_response)]
pub struct CreatedUser {
    pub person: Person,
    pub api_key: String,
}
impl AsEndpoint for CreatedUser {
    fn as_endpoint() -> &'static str {
        "/users"
    }
}

#[cfg_attr(feature = "backend", filter_struct(person))]
#[cfg_attr(feature = "typescript", api_request)]
pub struct PersonQuery {
    #[cfg_attr(feature = "backend", serde(default))]
    #[cfg_attr(feature = "typescript", builder(default))]
    pub ids: Vec<Uuid>,
    #[cfg_attr(feature = "typescript", builder(default))]
    pub name: Option<String>,
    #[cfg_attr(feature = "typescript", builder(default))]
    pub email: Option<String>,
}
