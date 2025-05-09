use serde::{Deserialize, Serialize};
#[cfg(feature = "backend")]
use {
    diesel::{pg::Pg, prelude::*},
    garde::Validate,
    scamplers_schema::person,
    valuable::Valuable,
};

#[cfg(feature = "web")]
use wasm_bindgen::prelude::*;

use {crate::institution::Institution, uuid::Uuid};

#[cfg_attr(
    feature = "backend",
    derive(
        Validate,
        Valuable,
        strum::EnumString,
        strum::IntoStaticStr,
        Default,
        Debug
    )
)]
#[cfg_attr(feature = "backend", strum(serialize_all = "snake_case"))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone, inspectable))]
#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    AppAdmin,
    ComputationalStaff,
    BiologyStaff,
    #[cfg_attr(feature = "backend", default)]
    Unknown,
}

#[cfg_attr(feature = "backend", derive(Insertable, Validate, Valuable, Debug))]
#[cfg_attr(feature = "backend", diesel(table_name = person, check_for_backend(Pg)), garde(allow_unvalidated))]
#[cfg_attr(
    feature = "web",
    wasm_bindgen(getter_with_clone, inspectable),
    derive(Default)
)]
#[derive(Deserialize, Serialize)]
pub struct NewPerson {
    #[cfg_attr(feature = "backend", garde(length(min = 1)))]
    pub name: String,
    #[cfg_attr(feature = "backend", garde(email))]
    pub email: String,
    pub orcid: Option<String>,
    pub institution_id: Uuid,
    pub ms_user_id: Option<Uuid>,
    #[cfg_attr(feature = "backend", diesel(skip_insertion), serde(default))]
    pub roles: Vec<UserRole>,
}

#[cfg_attr(feature = "web", wasm_bindgen)]
impl NewPerson {
    #[cfg_attr(feature = "web", wasm_bindgen(constructor))]
    #[cfg(feature = "web")]
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg_attr(feature = "backend", derive(Queryable, Selectable, Debug))]
#[cfg_attr(feature = "backend", diesel(table_name = person, check_for_backend(Pg)))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone, inspectable))]
#[derive(Deserialize, Serialize)]
pub struct Person {
    pub id: Uuid,
    pub name: String,
    pub link: String,
    pub email: String,
    pub orcid: Option<String>,
    #[cfg_attr(feature = "backend", diesel(embed))]
    pub institution: Institution,
}

#[cfg_attr(feature = "backend", derive(Debug))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone, inspectable))]
#[derive(Deserialize, Serialize)]
pub struct CreatedUser {
    pub id: Uuid,
    pub api_key: String,
}

#[cfg_attr(feature = "backend", derive(Valuable, Default, Debug))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone, inspectable))]
#[derive(Deserialize, Serialize)]
pub struct PersonQuery {
    #[cfg_attr(feature = "backend", serde(default))]
    pub ids: Vec<Uuid>,
    pub name: Option<String>,
    pub email: Option<String>,
}
