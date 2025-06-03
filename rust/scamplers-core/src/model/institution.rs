use uuid::Uuid;

use super::AsEndpoint;

#[cfg(feature = "backend")]
use {
    scamplers_macros::{insert_struct, select_struct},
    scamplers_schema::institution,
};

#[cfg(feature = "typescript")]
use scamplers_macros::{api_request, api_response};

#[cfg_attr(feature = "backend", insert_struct(institution), derive(Clone))]
#[cfg_attr(feature = "typescript", api_request)]
pub struct NewInstitution {
    pub id: Uuid,
    #[cfg_attr(feature = "backend", garde(length(min = 1)))]
    pub name: String,
}

#[cfg_attr(feature = "backend", select_struct(institution))]
#[cfg_attr(feature = "typescript", api_response)]
pub struct Institution {
    pub id: Uuid,
    pub name: String,
    pub link: String,
}
impl AsEndpoint for Institution {
    fn as_endpoint() -> &'static str {
        "/institutions"
    }
}
