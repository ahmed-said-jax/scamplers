pub mod chromium;
pub mod dataset_metadata;
pub mod index_sets;
pub mod institution;
pub mod lab;
pub mod person;
pub mod sample_metadata;
pub mod sequencing_run;
pub mod specimen;

#[cfg(feature = "typescript")]
use wasm_bindgen::prelude::*;

#[cfg_attr(
    feature = "typescript",
    derive(Clone, serde::Serialize),
    wasm_bindgen(setter)
)]
#[cfg_attr(
    feature = "backend",
    derive(serde::Deserialize, valuable::Valuable, Debug),
    serde(default)
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

pub trait IsUpdate {
    fn is_update(&self) -> bool;
}

#[cfg(feature = "typescript")]
#[wasm_bindgen]
impl Pagination {
    #[wasm_bindgen(constructor)]
    #[must_use]
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

#[cfg(test)]
mod tests {
    #[cfg(feature = "typescript")]
    #[test]
    fn write_request_builder() {
        use scamplers_macros::frontend_insertion;

        #[frontend_insertion]
        #[derive(Debug, PartialEq)]
        struct WriteStruct {
            field: String,
            #[builder(default)]
            optional_field: Option<String>,
        }

        let functional = WriteStruct::new()
            .field(String::new())
            .build()
            .expect("failed to build struct without setting optional field");

        assert_eq!(
            functional,
            WriteStruct {
                field: String::new(),
                optional_field: None
            }
        );

        WriteStruct::new().build().unwrap_err();
    }

    #[cfg(feature = "typescript")]
    #[test]
    fn default_query_request() {
        use scamplers_macros::frontend_query_request;

        #[frontend_query_request]
        #[derive(Debug, PartialEq)]
        struct QueryStruct {
            optional_field: Option<String>,
            order_by: Vec<String>,
        }

        let default_query = QueryStruct::default();

        assert_eq!(
            default_query,
            QueryStruct {
                optional_field: None,
                order_by: vec![String::new()]
            }
        );
    }
}
