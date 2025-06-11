use crate::model::{Pagination, person::PersonSummary};

use super::{Endpoint, SEARCH_SUFFIX};

#[cfg(feature = "backend")]
use {
    scamplers_macros::{
        backend_insertion, backend_ordering, backend_ordinal_columns_enum, backend_query_request,
        backend_selection, backend_update,
    },
    scamplers_schema::lab,
};

#[cfg(feature = "typescript")]
use scamplers_macros::{
    frontend_enum, frontend_ordering, frontend_query_request, frontend_response,
    frontend_write_request,
};

use uuid::Uuid;

const ENDPOINT: &str = "/labs";

#[cfg_attr(feature = "backend", backend_insertion(lab))]
#[cfg_attr(feature = "typescript", frontend_write_request)]
pub struct NewLab {
    #[cfg_attr(feature = "backend", garde(length(min = 1)))]
    pub name: String,
    pub pi_id: Uuid,
    #[cfg_attr(feature = "backend", garde(length(min = 1)))]
    pub delivery_dir: String,
    #[cfg_attr(feature = "backend", diesel(skip_insertion))]
    #[cfg_attr(feature = "typescript", builder(default))]
    pub member_ids: Vec<Uuid>,
}
impl Endpoint for NewLab {
    fn endpoint() -> String {
        ENDPOINT.to_string()
    }
}

#[cfg_attr(feature = "backend", backend_selection(lab))]
#[cfg_attr(feature = "typescript", frontend_response)]
pub struct LabReference {
    pub id: Uuid,
    pub link: String,
}

#[cfg_attr(feature = "backend", backend_selection(lab))]
#[cfg_attr(feature = "typescript", frontend_response)]
pub struct LabSummary {
    #[serde(flatten)]
    #[cfg_attr(feature = "backend", diesel(embed))]
    pub reference: LabReference,
    pub name: String,
    pub delivery_dir: String,
}
impl Endpoint for LabSummary {
    fn endpoint() -> String {
        format!("{ENDPOINT}/{SEARCH_SUFFIX}")
    }
}

#[cfg_attr(feature = "backend", backend_selection(lab))]
#[cfg_attr(feature = "typescript", frontend_response)]
pub struct Lab {
    #[serde(flatten)]
    #[cfg_attr(feature = "backend", diesel(embed))]
    pub summary: LabSummary,
    #[cfg_attr(feature = "backend", diesel(embed))]
    pub pi: PersonSummary,
}

#[cfg_attr(feature = "backend", derive(serde::Serialize))]
#[cfg_attr(feature = "typescript", derive(serde::Deserialize))]
pub struct LabWithMembers {
    #[serde(flatten)]
    pub lab: Lab,
    pub members: Vec<PersonSummary>,
}

impl Endpoint for LabWithMembers {
    fn endpoint() -> String {
        format!("{ENDPOINT}/{{lab_id}}")
    }
}

#[cfg_attr(feature = "backend", backend_ordinal_columns_enum)]
#[cfg_attr(feature = "typescript", frontend_enum)]
pub enum LabOrdinalColumn {
    #[default]
    Name,
}

#[cfg_attr(feature = "backend", backend_ordering)]
#[cfg_attr(feature = "typescript", frontend_ordering)]
pub struct LabOrdering {
    pub column: LabOrdinalColumn,
    pub descending: bool,
}

#[cfg_attr(feature = "backend", backend_query_request)]
#[cfg_attr(feature = "typescript", frontend_query_request)]
pub struct LabQuery {
    pub ids: Vec<Uuid>,
    pub name: Option<String>,
    pub order_by: Vec<LabOrdering>,
    pub pagination: Pagination,
}

#[cfg_attr(feature = "backend", backend_update(lab))]
#[cfg_attr(feature = "typescript", frontend_write_request)]
pub struct LabUpdate {
    pub id: Uuid,
    pub name: Option<String>,
    pub pi_id: Option<Uuid>,
    pub delivery_dir: Option<String>,
}

#[cfg_attr(
    feature = "backend",
    derive(serde::Deserialize, Default),
    serde(default)
)]
#[cfg_attr(feature = "typescript", frontend_write_request)]
pub struct LabUpdateWithMembers {
    #[serde(flatten)]
    pub update: LabUpdate,
    pub add_members: Vec<Uuid>,
    pub remove_members: Vec<Uuid>,
}
