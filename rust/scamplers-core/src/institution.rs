use uuid::Uuid;

#[cfg(feature = "backend")]
use {
    diesel::{pg::Pg, prelude::*},
    garde::Validate,
    scamplers_schema::institution,
    valuable::Valuable,
};

#[cfg(feature = "web")]
use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};

#[cfg_attr(
    feature = "backend",
    derive(Insertable, Valuable, Validate, Debug)
)]
#[cfg_attr(feature = "backend", diesel(table_name = institution, check_for_backend(Pg)), garde(allow_unvalidated))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone))]
#[derive(Deserialize, Serialize)]
pub struct NewInstitution {
    pub id: Uuid,
    #[cfg_attr(feature = "backend", garde(length(min = 1)))]
    pub name: String
}

#[cfg_attr(feature = "web", wasm_bindgen)]
impl NewInstitution {
    #[cfg_attr(feature = "web", wasm_bindgen(constructor))]
    #[cfg(feature = "web")]
    pub fn new(id: Uuid, name: String) -> Self {
        Self { id, name }
    }
}

#[cfg_attr(feature = "backend", derive(Queryable, Selectable, Debug))]
#[cfg_attr(feature = "backend", diesel(table_name = institution, check_for_backend(Pg)))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone))]
#[cfg_attr(feature = "web", derive(Clone))]
#[derive(Deserialize, Serialize)]
pub struct Institution {
    pub id: Uuid,
    pub name: String,
    pub link: String,
}
