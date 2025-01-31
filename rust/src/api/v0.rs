use axum::{
    debug_handler,
    extract::{Path, State},
    routing::get,
    Router,
};
use serde_json::json;
use strum::VariantArray;
use uuid::Uuid;

use super::{ApiResponse, ApiUser};
use crate::{
    db::{self, institution::Institution, Read},
    AppState,
};

pub(super) fn router() -> Router<AppState> {
    let mut router = Router::new();

    let endpoints: Vec<&str> = db::Entity::VARIANTS
        .iter()
        .map(|entity| entity.v0_endpoint())
        .collect();
    let endpoints = json!({"available_endpoints": endpoints});

    router = router
        .route("/", get(|| async { axum::Json(endpoints) }))
        .route(db::Entity::Institution.v0_endpoint(), get(get_institution));

    router
}

#[debug_handler]
async fn get_institution(
    State(state): State<AppState>,
    institution_id: Option<Path<Uuid>>,
) -> super::Result<ApiResponse> {
    let mut conn = state.db_pool.get().await?;

    if institution_id.is_none() {}

    let Some(Path(institution_id)) = institution_id else {
        let mut institutions = Institution::fetch_all(&mut conn, Default::default()).await?;

        // No need to expose MS tenant IDs
        for inst in &mut institutions {
            inst.ms_tenant_id = None;
        }

        return Ok(ApiResponse::from(institutions));
    };

    let mut institution = Institution::fetch_by_id(&mut conn, institution_id).await?;

    institution.ms_tenant_id = None;

    Ok(ApiResponse::from(institution))
}

impl db::Entity {
    pub fn v0_endpoint(&self) -> &'static str {
        use db::Entity::*;

        match self {
            Institution => "/institutions/{institution_id}",
            Person => "/people/{person_id}",
            Lab => "/labs/{lab_id}",
            Sample => "/samples/{sample_id}",
            Library => "/libraries/{library_id}",
            SequencingRun => "/sequencing_runs/{sequencing_run_id}",
            Dataset => "/datasets/{dataset_id}",
            Unknown => "/",
        }
    }
}
