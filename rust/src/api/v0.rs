use axum::{handler::Handler, middleware, routing::get, Router};
use serde::Deserialize;
use serde_json::json;
use strum::VariantArray;

use crate::{
    api::ApiUser, db::{self, institution::{Institution, NewInstitution}, person::Person, Pagination}, AppState
};

pub(super) fn router() -> Router<AppState> {
    use handlers::*;
    use auth::*;

    let mut router = Router::new();

    let endpoints: Vec<&str> = db::Entity::VARIANTS
        .iter()
        .map(|entity| entity.v0_endpoint())
        .collect();
    let endpoints = json!({"available_endpoints": endpoints});

    router = router
        .route("/", get(|| async { axum::Json(endpoints) }))
        .route("/institutions", get(by_filter::<Institution>).post(new::<Vec<NewInstitution>>.layer(middleware::from_fn(user_has_admin))))
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
    pagination: Pagination,
}

mod handlers {
    use axum::{extract::{Path, State}, Json};
    use axum_extra::extract::Query;

    use super::FilterWithPagination;
    use crate::{api::{self, ApiUser}, db, AppState};

    pub async fn by_id<T: db::Read>(
        State(state): State<AppState>,
        Path(id): Path<T::Id>,
    ) -> api::Result<Json<T>> {
        let mut conn = state.db_pool.get().await?;

        let item = T::fetch_by_id(id, &mut conn).await?;

        Ok(axum::Json(item))
    }

    pub async fn by_filter<T: db::Read>(
        State(state): State<AppState>,
        Query(query): Query<FilterWithPagination<T::Filter>>,
    ) -> api::Result<Json<Vec<T>>> {
        let mut conn = state.db_pool.get().await?;

        let items = T::fetch_many(query.filter.as_ref(), &query.pagination, &mut conn).await?;

        Ok(axum::Json(items))
    }

    pub async fn by_relationship<T, U>(
        State(state): State<AppState>,
        Path(id): Path<T>,
        Query(query): Query<FilterWithPagination<U::Filter>>,
    ) -> api::Result<Json<Vec<U>>>
    where
        T: db::ReadRelatives<U>,
        U: db::Read,
    {
        let mut conn = state.db_pool.get().await?;

        let relatives = id
            .fetch_relatives(query.filter.as_ref(), &query.pagination, &mut conn)
            .await?;

        Ok(axum::Json(relatives))
    }

    pub async fn new<T: db::Create>(State(state): State<AppState>, Json(data): Json<T>) -> api::Result<Json<T::Returns>> {
        let mut conn = state.db_pool.get().await?;

        let created = data.create(&mut conn).await?;

        Ok(Json(created))
    }
}

mod auth {
    use axum::{debug_handler, debug_middleware, extract::Request, middleware::Next, response::{IntoResponse, Response}};

    use crate::{api::ApiUser, db::person::UserRole};

    #[debug_middleware]
    pub async fn user_has_admin(ApiUser(user): ApiUser, req: Request, next: Next) -> Response {
        if !user.roles.contains(&UserRole::Admin) {
            super::super::Error::permission().into_response()
        } else {
            next.run(req).await
        }
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
