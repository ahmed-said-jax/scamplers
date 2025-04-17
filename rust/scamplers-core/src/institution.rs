use uuid::Uuid;

#[cfg(feature = "backend")]
use {
    diesel::{pg::Pg, prelude::*},
    garde::Validate,
    scamplers_schema::institution,
    serde::{Deserialize, Serialize},
    valuable::Valuable,
};

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "web")]
use wasm_bindgen::prelude::*;

#[cfg_attr(
    feature = "backend",
    derive(Insertable, Valuable, Validate, Deserialize, Debug)
)]
#[cfg_attr(feature = "backend", diesel(table_name = institution, check_for_backend(Pg)), garde(allow_unvalidated))]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone))]
pub struct NewInstitution {
    #[cfg_attr(feature = "backend", garde(length(min = 1)))]
    pub name: String,
    pub ms_tenant_id: Option<Uuid>,
}

#[cfg_attr(feature = "web", wasm_bindgen)]
impl NewInstitution {
    #[cfg_attr(feature = "web", wasm_bindgen(constructor))]
    #[cfg(any(feature = "python", feature = "web"))]
    pub fn new(name: String, ms_tenant_id: Option<Uuid>) -> Self {
        Self { name, ms_tenant_id }
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl NewInstitution {
    #[new]
    fn py_new(name: String, ms_tenant_id: Option<Uuid>) -> Self {
        Self::new(name, ms_tenant_id)
    }
}

#[cfg_attr(feature = "backend", derive(Queryable, Selectable, Serialize, Debug))]
#[cfg_attr(feature = "backend", diesel(table_name = institution, check_for_backend(Pg)))]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone))]
pub struct Institution {
    pub id: Uuid,
    pub name: String,
    pub link: String,
}
