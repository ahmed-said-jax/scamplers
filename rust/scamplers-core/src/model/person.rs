use super::Pagination;

#[cfg(feature = "backend")]
use {
    scamplers_macros::{
        backend_db_enum, backend_insertion, backend_ordering, backend_ordinal_columns_enum,
        backend_query_request, backend_update, backend_with_getters,
    },
    scamplers_schema::person,
};

#[cfg(feature = "typescript")]
use scamplers_macros::{
    frontend_enum, frontend_insertion, frontend_ordering, frontend_query_request, frontend_update,
    frontend_with_getters,
};

use uuid::Uuid;

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
impl NewPerson {
    #[must_use]
    pub fn new_user_route() -> String {
        "/users".to_string()
    }
}

#[cfg_attr(feature = "backend", backend_with_getters)]
#[cfg_attr(feature = "typescript", frontend_with_getters)]
mod read {
    use crate::model::{institution::Institution, person::UserRole};
    #[cfg(feature = "typescript")]
    use scamplers_macros::frontend_response;
    use uuid::Uuid;
    #[cfg(feature = "backend")]
    use {scamplers_macros::backend_selection, scamplers_schema::person};

    #[cfg_attr(feature = "backend", backend_selection(person))]
    #[cfg_attr(feature = "typescript", frontend_response)]
    pub struct PersonReference {
        id: Uuid,
        link: String,
    }

    #[cfg_attr(feature = "backend", backend_selection(person))]
    #[cfg_attr(feature = "typescript", frontend_response)]
    pub struct PersonSummary {
        #[serde(flatten)]
        #[cfg_attr(feature = "backend", diesel(embed))]
        reference: PersonReference,
        name: String,
        email: Option<String>,
        orcid: Option<String>,
    }

    #[cfg_attr(feature = "backend", backend_selection(person))]
    #[cfg_attr(feature = "typescript", frontend_response)]
    pub struct PersonData {
        #[serde(flatten)]
        #[cfg_attr(feature = "backend", diesel(embed))]
        summary: PersonSummary,
        #[cfg_attr(feature = "backend", diesel(embed))]
        institution: Institution,
    }

    #[cfg_attr(feature = "backend", derive(serde::Serialize, Debug))]
    #[cfg_attr(feature = "typescript", frontend_response)]
    pub struct Person {
        #[serde(flatten)]
        data: PersonData,
        roles: Vec<UserRole>,
    }

    #[cfg(feature = "backend")]
    impl Person {
        #[must_use]
        pub fn new(data: PersonData, roles: Vec<UserRole>) -> Self {
            Self { data, roles }
        }
    }

    #[cfg_attr(feature = "backend", derive(serde::Serialize, Debug))]
    #[cfg_attr(feature = "typescript", frontend_response)]
    pub struct CreatedUser {
        data: Person,
        api_key: String,
    }

    #[cfg(feature = "backend")]
    impl CreatedUser {
        #[must_use]
        pub fn new(data: Person, api_key: String) -> Self {
            Self { data, api_key }
        }
    }
}
pub use read::*;

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

#[cfg_attr(feature = "backend", backend_update(person))]
#[cfg_attr(feature = "typescript", frontend_update)]
pub struct PersonDataUpdate {
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
pub struct PersonUpdate {
    #[serde(flatten)]
    pub data_update: PersonDataUpdate,
    pub add_roles: Vec<UserRole>,
    pub remove_roles: Vec<UserRole>,
}
