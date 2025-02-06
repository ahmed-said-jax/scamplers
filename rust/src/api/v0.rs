use std::{collections::HashMap, hash::RandomState};

use axum::{Json, Router, routing::get};
use serde::Deserialize;

use crate::{
    AppState,
    db::{
        Pagination,
        institution::{Institution, NewInstitution},
        person::Person,
    },
};

pub(super) fn router() -> Router<AppState> {
    use handlers::*;

    // TODO: get a list of routes from the database and then just put them here
    let endpoints: HashMap<&str, [&str; 1], RandomState> =
        HashMap::from_iter([("available_endpoints", [""])]);

    let router = Router::new()
        .route("/", get(|| async { Json(endpoints) }))
        .route(
            "/institutions",
            get(by_filter::<Institution>).post(new::<Vec<NewInstitution>>),
        )
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
    use axum::{
        Json,
        extract::{Path, State},
    };
    use axum_extra::extract::Query;
    use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};

    use super::FilterWithPagination;
    use crate::{
        AppState,
        api::{self, ApiUser},
        db::{self, set_transaction_user},
    };

    pub async fn by_id<T: db::Read + Send>(
        ApiUser(user): ApiUser,
        State(state): State<AppState>,
        Path(id): Path<T::Id>,
    ) -> api::Result<Json<T>> {
        let mut conn = state.db_pool.get().await?;

        let item = conn
            .transaction(|conn| {
                async move {
                    set_transaction_user(user.id(), conn).await?;

                    T::fetch_by_id(id, conn).await
                }
                .scope_boxed()
            })
            .await?;

        Ok(axum::Json(item))
    }

    pub async fn by_filter<T: db::Read>(
        ApiUser(user): ApiUser,
        State(state): State<AppState>,
        Query(query): Query<FilterWithPagination<T::Filter>>,
    ) -> api::Result<Json<Vec<T>>> {
        let mut conn = state.db_pool.get().await?;

        let items = conn
            .transaction(|conn| {
                async move {
                    set_transaction_user(user.id(), conn).await?;

                    T::fetch_many(query.filter.as_ref(), &query.pagination, conn).await
                }
                .scope_boxed()
            })
            .await?;

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

    pub async fn new<T: db::Create>(
        ApiUser(user): ApiUser,
        State(state): State<AppState>,
        Json(data): Json<T>,
    ) -> api::Result<Json<T::Returns>> {
        let mut conn = state.db_pool.get().await?;

        let created = conn
            .transaction(|conn| {
                async move {
                    set_transaction_user(user.id(), conn).await?;

                    data.create(conn).await
                }
                .scope_boxed()
            })
            .await?;

        Ok(Json(created))
    }
}
