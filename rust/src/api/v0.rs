use axum::{
    debug_handler, extract::{Path, State}, response::IntoResponse, routing::get, Router
};
use axum_extra::extract::Query;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use strum::VariantArray;
use uuid::Uuid;

use super::ApiUser;
use crate::{
    db::{self, institution::Institution, person::{Person, User, UserRole}, Pagination, Read},
    AppState,
};

pub(super) fn router() -> Router<AppState> {
    use handlers::*;

    let mut router = Router::new();

    let endpoints: Vec<&str> = db::Entity::VARIANTS
        .iter()
        .map(|entity| entity.v0_endpoint())
        .collect();
    let endpoints = json!({"available_endpoints": endpoints});

    router = router
        .route("/", get(|| async { axum::Json(endpoints) }))
        .route("/institutions", get(by_filter::<Institution>))
        .route("/institutions/{institution_id}", get(by_id::<Institution>))
        .route("/people", get(by_filter::<Person>))
        .route("/people/{person_id}", get(by_id::<Person>));

    router
}

#[derive(Deserialize)]
struct FilterWithPagination<F> {
    #[serde(flatten)]
    #[serde(default)]
    filter: Option<F>,
    #[serde(flatten)]
    #[serde(default)]
    pagination: Pagination
}

mod handlers {
    use crate::{db, AppState, api};
    use axum::extract::{State, Path};
    use axum_extra::extract::Query;
    use super::FilterWithPagination;

    pub async fn by_id<T: db::Read>(State(state): State<AppState>, Path(id): Path<T::Id>) -> api::Result<axum::Json<T>> {
        let mut conn = state.db_pool.get().await?;
    
        let item = T::fetch_by_id(id, &mut conn).await?;
    
        Ok(axum::Json(item))
    }

    pub async fn by_filter<T: db::Read>(State(state): State<AppState>, Query(query): Query<FilterWithPagination<T::Filter>>) -> api::Result<axum::Json<Vec<T>>> {
        let mut conn = state.db_pool.get().await?;

        let items = T::fetch_many(query.filter.as_ref(), &query.pagination, &mut conn).await?;

        Ok(axum::Json(items))
    }

    pub async fn by_relationship<T, U>(State(state): State<AppState>, Path(id): Path<T>, Query(query): Query<FilterWithPagination<U::Filter>>) -> api::Result<axum::Json<Vec<U>>> where T: db::ReadRelatives<U>, U: db::Read {
        let mut conn = state.db_pool.get().await?;

        let relatives = id.fetch_relatives(query.filter.as_ref(), &query.pagination, &mut conn).await?;

        Ok(axum::Json(relatives))
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
