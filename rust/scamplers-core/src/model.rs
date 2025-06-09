pub mod chromium;
pub mod dataset_metadata;
pub mod index_sets;
pub mod institution;
pub mod lab;
pub mod person;
pub mod sample_metadata;
pub mod sequencing_run;

#[cfg(feature = "typescript")]
use wasm_bindgen::prelude::*;

pub trait Endpoint {
    fn endpoint() -> String;
}
const SEARCH_SUFFIX: &str = "search";

#[cfg_attr(
    feature = "typescript",
    derive(Clone, serde::Serialize),
    wasm_bindgen(setter, inspectable)
)]
#[cfg_attr(
    feature = "backend",
    derive(serde::Deserialize, valuable::Valuable, Debug)
)]
pub struct Pagination {
    pub limit: i64,
    pub offset: i64,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: 500,
            offset: 0,
        }
    }
}

#[cfg(feature = "typescript")]
#[wasm_bindgen]
impl Pagination {
    #[wasm_bindgen(constructor)]
    pub fn new(limit: i64, offset: i64) -> Self {
        Self { limit, offset }
    }
}

trait DefaultOrdering {
    fn default() -> Self;
}
impl<T> DefaultOrdering for Vec<T>
where
    T: Default,
{
    fn default() -> Self {
        vec![T::default()]
    }
}
