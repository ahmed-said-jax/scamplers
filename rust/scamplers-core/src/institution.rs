#[cfg(feature = "backend")]
use {diesel::{pg::Pg, prelude::*}, garde::Validate, valuable::Valuable, serde::{Deserialize, Serialize}};

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "web")]
use wasm_bindgen::prelude::*;
use uuid::Uuid;

use scamplers_schema::institution;

#[cfg_attr(feature = "backend", derive(Insertable, Valuable, Validate, Deserialize))]
#[cfg_attr(feature = "backend", diesel(table_name = institution, check_for_backend(Pg)))]
#[cfg_attr(feature = "backend", garde(allow_unvalidated))]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone))]
pub struct NewInstitution {
    #[cfg_attr(feature = "backend", garde(length(min = 1)))]
    pub name: String,
    #[cfg_attr(feature = "backend", valuable(skip))]
    pub ms_tenant_id: Option<Uuid>,
}

#[cfg(feature = "python")]
#[pymethods]
impl NewInstitution {
    #[new]
    fn new(name: String, ms_tenant_id: Option<Uuid>) -> Self {
        Self {
            name,
            ms_tenant_id,
        }
    }
}

// #[cfg_attr(feature = "python", pymethods)]
// // #[cfg_attr(feature = "web", wasm_bindgen)]
// impl NewInstitution {
//     #[cfg_attr(feature = "python", new)]
//     // #[cfg_attr(feature = "web", wasm_bindgen(constructor))]
//     pub fn new(name: String, ms_tenant_id: Option<Uuid>) -> Self {
//         Self {
//             name,
//             ms_tenant_id,
//         }
//     }
// }

#[cfg_attr(feature = "backend", derive(Queryable, Selectable, Serialize))]
#[cfg_attr(feature = "backend", diesel(table_name = institution, check_for_backend(Pg)))]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone))]
pub struct Institution {
    id: Uuid,
    name: String,
    link: String,
}
