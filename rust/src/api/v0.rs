use std::{collections::HashMap, hash::RandomState};

use axum::{Json, Router, routing::get};

use crate::{
    AppState2,
    db::{
        institution::{Institution, NewInstitution},
        person::Person,
    },
};

pub(super) fn router() -> Router<AppState2> {
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

mod handlers {
    use axum::extract::{Path, State};
    use axum_extra::extract::Query;
    use diesel_async::{AsyncConnection, scoped_futures::ScopedFutureExt};
    use valuable::Valuable;

    use crate::{
        api::{self, ApiJson, User}, db::{self, set_transaction_user}, AppState2
    };

    pub async fn by_id<T: db::Read + Send>(
        user: User,
        State(app_state): State<AppState2>,
        Path(id): Path<T::Id>,
    ) -> api::Result<ApiJson<T>> {
        let mut conn = app_state.db_conn().await?;

        let item = conn
            .transaction(|conn| {
                async move {
                    set_transaction_user(user.id(), conn).await?;

                    T::fetch_by_id(id, conn).await
                }
                .scope_boxed()
            })
            .await?;

        Ok(ApiJson(item))
    }

    pub async fn by_filter<T: db::Read>(
        user: User,
        State(app_state): State<AppState2>,
        Query(query): Query<T::Filter>,
    ) -> api::Result<ApiJson<Vec<T>>> {
        tracing::info!(deserialized_query = query.as_value());

        let mut conn = app_state.db_conn().await?;

        let items = conn
            .transaction(|conn| {
                async move {
                    set_transaction_user(user.id(), conn).await?;

                    T::fetch_many(query, conn).await
                }
                .scope_boxed()
            })
            .await?;

        Ok(ApiJson(items))
    }

    pub async fn by_relationship<T, U>(
        user: User,
        State(app_state): State<AppState2>,
        Path(id): Path<T>,
        Query(query): Query<U::Filter>,
    ) -> api::Result<ApiJson<Vec<U>>>
    where
        T: db::ReadRelatives<U>,
        U: db::Read,
    {
        let mut conn = app_state.db_conn().await?;

        let relatives = conn
            .transaction(|conn| {
                async move {
                    set_transaction_user(user.id(), conn).await?;

                    id.fetch_relatives(query, conn)
                        .await
                }
                .scope_boxed()
            })
            .await?;

        Ok(ApiJson(relatives))
    }

    pub async fn new<T: db::Create>(
        user: User,
        State(app_state): State<AppState2>,
        ApiJson(data): ApiJson<T>,
    ) -> api::Result<ApiJson<T::Returns>> {
        let mut conn = app_state.db_conn().await?;

        let created = conn
            .transaction(|conn| {
                async move {
                    set_transaction_user(user.id(), conn).await?;

                    data.create(conn).await
                }
                .scope_boxed()
            })
            .await?;

        Ok(ApiJson(created))
    }
}
