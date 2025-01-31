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
    db::{self, institution::Institution, person::{Person, User, UserRole}, Read},
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
        // .route(db::Entity::Person.v0_endpoint(), get(generic_get_handler::<Person>));

    router
}

async fn generic_get_handler<T: db::Read + Into<ApiResponse>>(State(state): State<AppState>, id: Option<Path<T::Id>>) -> super::Result<ApiResponse> where Vec<T>: Into<ApiResponse> {
    let mut conn = state.db_pool.get().await?;

    let Some(Path(id)) = id else {
        let items = T::fetch_all(&mut conn, Default::default()).await?;

        return Ok(items.into())
    };

    let item = T::fetch_by_id(&mut conn, id).await?;

    Ok(item.into())
}

#[debug_handler]
async fn get_institution(
    State(state): State<AppState>,
    institution_id: Option<Path<Uuid>>,
) -> super::Result<ApiResponse> {
    let mut conn = state.db_pool.get().await?;

    let Some(Path(institution_id)) = institution_id else {
        let institutions = Institution::fetch_all(&mut conn, Default::default()).await?;

        return Ok(ApiResponse::Institutions(institutions));
    };

    let institution = Institution::fetch_by_id(&mut conn, institution_id).await?;

    Ok(ApiResponse::Institution(institution))
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
