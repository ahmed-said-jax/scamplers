use std::{collections::HashMap, hash::RandomState};

use axum::{Json, Router, routing::get};

use crate::{
    AppState2,
    db::{
        institution::{Institution, NewInstitution},
        lab::{Lab, LabId, NewLab},
        person::{Person, PersonFilter},
    },
};

pub(super) fn router() -> Router<AppState2> {
    use handlers::*;

    // TODO: get a list of routes from the database and then just put them here
    let endpoints: HashMap<&str, [&str; 1], RandomState> = HashMap::from_iter([("available_endpoints", [""])]);

    let router = Router::new()
        .route("/", get(|| async { Json(endpoints) }))
        .route(
            "/institutions",
            get(by_filter::<Institution>).post(new::<Vec<NewInstitution>>),
        )
        .route("/institutions/{institution_id}", get(by_id::<Institution>))
        .route("/people", get(by_filter::<Person>))
        .route("/people/{person_id}", get(by_id::<Person>))
        .route("/labs", get(by_filter::<Lab>).post(new::<Vec<NewLab>>))
        .route("/labs/{lab_id}", get(by_id::<Lab>))
        .route("/labs/{lab_id}/members", get(by_relationship::<LabId, Person>));
    router
}

mod handlers {
    use axum::extract::{Path, State};
    use axum_extra::extract::Query;
    use diesel_async::{AsyncConnection, scoped_futures::ScopedFutureExt};
    use valuable::Valuable;

    use crate::{
        AppState2,
        api::{self, User, ValidJson},
        db::{self, set_transaction_user},
    };

    pub async fn by_id<T: db::Read + Send>(
        user: User,
        State(app_state): State<AppState2>,
        Path(id): Path<T::Id>,
    ) -> api::Result<ValidJson<T>> {
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

        Ok(ValidJson(item))
    }

    pub async fn by_filter<T: db::Read>(
        user: User,
        State(app_state): State<AppState2>,
        Query(query): Query<T::Filter>,
    ) -> api::Result<ValidJson<Vec<T>>>
    where
        T::Filter: Valuable,
    {
        tracing::debug!(deserialized_query = query.as_value());

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

        Ok(ValidJson(items))
    }

    pub async fn by_relationship<T, U>(
        user: User,
        State(app_state): State<AppState2>,
        Path(id): Path<T>,
        Query(query): Query<U::Filter>,
    ) -> api::Result<ValidJson<Vec<U>>>
    where
        T: db::ReadRelatives<U>,
        U: db::Read,
        U::Filter: Valuable,
    {
        tracing::debug!(parent_id = id.to_string(), deserialized_query = query.as_value());

        let mut conn = app_state.db_conn().await?;

        let relatives = conn
            .transaction(|conn| {
                async move {
                    set_transaction_user(user.id(), conn).await?;

                    id.fetch_relatives(query, conn).await
                }
                .scope_boxed()
            })
            .await?;

        Ok(ValidJson(relatives))
    }

    pub async fn new<T>(
        user: User,
        State(app_state): State<AppState2>,
        ValidJson(data): ValidJson<T>,
    ) -> api::Result<ValidJson<T::Returns>>
    where
        T: Valuable + db::Create + garde::Validate,
    {
        tracing::debug!(deserialized_data = data.as_value());

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

        Ok(ValidJson(created))
    }
}
