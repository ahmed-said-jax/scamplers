use crate::string::NonEmptyString;

use super::Pagination;
#[cfg(feature = "typescript")]
use scamplers_macros::{
    frontend_enum, frontend_insertion, frontend_ordering, frontend_ordinal_columns_enum,
    frontend_query_request, frontend_with_getters,
};
use uuid::Uuid;
#[cfg(feature = "backend")]
use {
    scamplers_macros::{
        backend_db_enum, backend_insertion, backend_ordering, backend_ordinal_columns_enum,
        backend_query_request, backend_with_getters,
    },
    scamplers_schema::person,
};

#[derive(PartialEq)]
#[cfg_attr(feature = "backend", backend_db_enum)]
#[cfg_attr(feature = "typescript", frontend_enum)]
pub enum UserRole {
    AppAdmin,
    ComputationalStaff,
    BiologyStaff,
}

#[cfg_attr(
    feature = "backend",
    backend_insertion(person),
    derive(Clone, bon::Builder)
)]
#[cfg_attr(
    feature = "backend",
    builder(on(NonEmptyString, into), on(String, into))
)]
#[cfg_attr(feature = "typescript", frontend_insertion)]
pub struct NewPerson {
    #[cfg_attr(feature = "backend", garde(dive))]
    pub name: NonEmptyString,
    #[cfg_attr(feature = "backend", garde(email))]
    pub email: String,
    #[cfg_attr(feature = "backend", garde(dive))]
    #[cfg_attr(feature = "typescript", builder(default))]
    pub orcid: Option<NonEmptyString>,
    pub institution_id: Uuid,
    #[cfg_attr(feature = "typescript", builder(default))]
    pub ms_user_id: Option<Uuid>,
    #[cfg_attr(
        feature = "backend",
        diesel(skip_insertion),
        serde(default),
        builder(default)
    )]
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
mod with_getters {
    #[cfg(feature = "backend")]
    use crate::model::IsUpdate;
    use crate::{
        model::{institution::Institution, person::UserRole},
        string::NonEmptyString,
    };
    #[cfg(feature = "typescript")]
    use scamplers_macros::{frontend_response, frontend_update};
    use uuid::Uuid;
    #[cfg(feature = "backend")]
    use {
        scamplers_macros::{backend_selection, backend_update},
        scamplers_schema::person,
    };

    #[cfg_attr(feature = "backend", backend_selection(person))]
    #[cfg_attr(feature = "typescript", frontend_response)]
    pub struct PersonHandle {
        id: Uuid,
        link: String,
    }

    #[cfg_attr(feature = "backend", backend_selection(person))]
    #[cfg_attr(feature = "typescript", frontend_response)]
    pub struct PersonSummary {
        #[serde(flatten)]
        #[cfg_attr(feature = "backend", diesel(embed))]
        handle: PersonHandle,
        name: String,
        email: Option<String>,
        orcid: Option<String>,
    }

    #[cfg_attr(feature = "backend", backend_selection(person))]
    #[cfg_attr(feature = "typescript", frontend_response)]
    pub struct PersonCore {
        #[serde(flatten)]
        #[cfg_attr(feature = "backend", diesel(embed))]
        summary: PersonSummary,
        #[cfg_attr(feature = "backend", diesel(embed))]
        institution: Institution,
    }

    #[cfg_attr(feature = "backend", derive(serde::Serialize, Debug, bon::Builder))]
    #[cfg_attr(feature = "typescript", frontend_response)]
    pub struct Person {
        #[serde(flatten)]
        core: PersonCore,
        roles: Vec<UserRole>,
    }

    #[cfg_attr(feature = "backend", derive(serde::Serialize, Debug, bon::Builder))]
    #[cfg_attr(feature = "backend", builder(on(String, into)))]
    #[cfg_attr(feature = "typescript", frontend_response)]
    pub struct CreatedUser {
        person: Person,
        api_key: String,
    }

    #[cfg_attr(feature = "backend", backend_update(person))]
    #[cfg_attr(feature = "typescript", frontend_update)]
    pub struct PersonUpdateCore {
        id: Uuid,
        #[cfg_attr(feature = "backend", garde(dive))]
        name: Option<NonEmptyString>,
        #[cfg_attr(feature = "backend", garde(email))]
        email: Option<String>,
        ms_user_id: Option<Uuid>,
        #[cfg_attr(feature = "backend", garde(dive))]
        orcid: Option<NonEmptyString>,
        institution_id: Option<Uuid>,
    }

    #[cfg(feature = "backend")]
    impl IsUpdate for PersonUpdateCore {
        fn is_update(&self) -> bool {
            !matches!(
                self,
                Self {
                    name: None,
                    email: None,
                    orcid: None,
                    institution_id: None,
                    ..
                }
            )
        }
    }

    #[cfg_attr(
        feature = "backend",
        derive(serde::Deserialize, Default, garde::Validate)
    )]
    #[cfg_attr(feature = "backend", garde(allow_unvalidated), serde(default))]
    #[cfg_attr(feature = "typescript", frontend_update)]
    pub struct PersonUpdate {
        grant_roles: Vec<UserRole>,
        revoke_roles: Vec<UserRole>,
        #[cfg_attr(feature = "backend", garde(dive), serde(flatten))]
        core: PersonUpdateCore,
    }

    #[cfg(feature = "backend")]
    impl PersonUpdate {
        #[must_use]
        pub fn core(&self) -> &PersonUpdateCore {
            &self.core
        }
    }

    #[cfg(feature = "backend")]
    #[bon::bon]
    impl PersonUpdate {
        #[builder(on(String, into), on(NonEmptyString, into))]
        pub fn new(
            #[builder(field)] grant_roles: Vec<UserRole>,
            #[builder(field)] revoke_roles: Vec<UserRole>,
            id: Uuid,
            name: Option<NonEmptyString>,
            email: Option<String>,
            ms_user_id: Option<Uuid>,
            orcid: Option<NonEmptyString>,
            institution_id: Option<Uuid>,
        ) -> Self {
            let core = PersonUpdateCore {
                id,
                name,
                email,
                ms_user_id,
                orcid,
                institution_id,
            };
            Self {
                grant_roles,
                revoke_roles,
                core,
            }
        }
    }

    #[cfg(feature = "backend")]
    impl<S: person_update_builder::State> PersonUpdateBuilder<S> {
        pub fn grant_role(mut self, role: UserRole) -> Self {
            self.grant_roles.push(role);
            self
        }
        pub fn revoke_role(mut self, role: UserRole) -> Self {
            self.revoke_roles.push(role);
            self
        }
    }
}
pub use with_getters::*;

#[cfg_attr(feature = "backend", backend_ordinal_columns_enum)]
#[cfg_attr(feature = "typescript", frontend_ordinal_columns_enum)]
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
