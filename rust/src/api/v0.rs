use axum::{
    debug_handler, extract::{Path, State}, response::IntoResponse, routing::get, Router
};
use axum_extra::extract::Query;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use strum::VariantArray;
use uuid::Uuid;

use super::{ApiResponse, ApiUser};
use crate::{
    db::{self, institution::Institution, person::{Person, User, UserRole}, Pagination, Read},
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
        .route("/institutions", get(handle_get_many::<Institution>));
        // .route(db::Entity::Person.v0_endpoint(), get(generic_get_handler::<Person>));

    router
}

#[derive(Deserialize)]
struct FilterWithPagination<F> {
    #[serde(flatten)]
    filter: F,
    #[serde(flatten)]
    pagination: Pagination
}

async fn handle_get_by_id<T: db::Read + Serialize>(State(state): State<AppState>, Path(id): Path<T::Id>) -> super::Result<axum::Json<T>> {
    let mut conn = state.db_pool.get().await?;

    let item = T::fetch_by_id(&mut conn, id).await?;

    Ok(axum::Json(item))
}


async fn handle_get_many<T: db::Read + Serialize>(State(state): State<AppState>, query: Option<Query<FilterWithPagination<T::Filter>>>) -> super::Result<axum::Json<Vec<T>>> {
    let mut conn = state.db_pool.get().await?;

    if let Some(query) = query {
        let items = T::fetch_many(&mut conn, Some(&query.filter), &query.pagination).await?;

        Ok(axum::Json(items))
    } else {
        let items = T::fetch_many(&mut conn, None, &Default::default()).await?;

        Ok(axum::Json(items))
    }
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
