#[cfg(feature = "backend")]
use {scamplers_macros::backend_insertion, scamplers_schema::lab};

#[cfg(feature = "typescript")]
use scamplers_macros::{
    frontend_enum, frontend_ordering, frontend_query_request, frontend_response,
    frontend_write_request,
};

use uuid::Uuid;

const ENDPOINT: &str = "/labs";

#[cfg_attr(feature = "backend", backend_insertion(lab), derive(Clone))]
#[cfg_attr(feature = "typescript", frontend_write_request)]
pub struct NewLab {
    #[cfg_attr(feature = "backend", garde(length(min = 1)))]
    pub name: String,
    pub pi_id: Uuid,
    #[cfg_attr(feature = "backend", garde(length(min = 1)))]
    pub delivery_dir: String,
    #[cfg_attr(feature = "backend", diesel(skip_insertion))]
    pub member_ids: Vec<Uuid>,
}
