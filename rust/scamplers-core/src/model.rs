pub mod chromium;
pub mod dataset_metadata;
pub mod index_sets;
pub mod institution;
pub mod lab;
pub mod person;
pub mod sample_metadata;
pub mod sequencing_run;

pub trait AsEndpoint {
    fn as_endpoint() -> &'static str;
}
