#[cfg(feature = "backend")]
use {
    diesel::{pg::Pg, prelude::*},
    garde::Validate,
    scamplers_schema::person,
    serde::{Deserialize, Serialize},
    valuable::Valuable,
};

use {crate::institution::Institution, uuid::Uuid};

use std::default;

#[cfg_attr(
    feature = "backend",
    derive(Validate, Deserialize, Valuable, strum::EnumString, strum::IntoStaticStr)
)]
#[cfg_attr(feature = "backend", strum(serialize_all = "snake_case"), serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "python", pyclass(get_all, set_all, rename_all = "snake_case"))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone))]
#[derive(Default)]
pub enum UserRole {
    #[cfg_attr(feature = "web", js_name = "app_admin")]
    AppAdmin,
    #[cfg_attr(feature = "web", js_name = "computational_staff")]
    ComputationalStaff,
    #[cfg_attr(feature = "web", js_name = "biology_staff")]
    BiologyStaff,
    #[default]
    Unknown
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable, Validate, Deserialize, Valuable)
)]
#[cfg_attr(feature = "backend", diesel(table_name = person, check_for_backend(Pg)), garde(allow_unvalidated))]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone))]
pub struct NewPerson {
    #[cfg_attr(feature = "backend", garde(length(min = 1)))]
    pub name: String,
    #[cfg_attr(feature = "backend", garde(email))]
    pub email: String,
    pub orcid: Option<String>,
    pub institution_id: Uuid,
    pub ms_user_id: Uuid,
    #[cfg_attr(feature = "backend", diesel(skip_insertion), serde(default))]
    pub roles: Vec<UserRole>
}

#[cfg_attr(feature = "backend", derive(Queryable, Selectable, Serialize))]
#[cfg_attr(feature = "backend", diesel(table_name = person, check_for_backend(Pg)))]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone))]
pub struct Person {
    pub id: Uuid,
    pub name: String,
    pub link: String,
    pub email: String,
    pub orcid: Option<String>,
    #[cfg_attr(feature = "backend", diesel(embed))]
    pub institution: Institution,
}

#[cfg_attr(feature = "backend", derive(Deserialize, Valuable))]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone))]
#[derive(Default)]
pub struct PersonQuery {
    #[cfg_attr(feature = "backend", serde(default))]
    pub ids: Vec<Uuid>,
    pub name: Option<String>,
    pub email: Option<String>,
}
