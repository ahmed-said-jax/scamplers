pub mod chromium;
pub mod dataset_metadata;
pub mod index_sets;
pub mod institution;
pub mod lab;
pub mod person;
pub mod sample_metadata;
pub mod sequencing_run;

#[cfg(feature = "typescript")]
use scamplers_macros::api_request;

pub trait AsEndpoint {
    fn as_endpoint() -> &'static str;
}

#[cfg_attr(feature = "typescript", api_request)]
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
