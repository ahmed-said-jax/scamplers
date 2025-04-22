use std::{collections::HashMap, hash::RandomState};

use axum::{routing::{get, post}, Router};

use super::AppState2;
use crate::db::model::person::fetch_by_filter;

mod error;

pub(super) fn router() -> Router<AppState2> {
    use handlers::*;
    // TODO: get a list of routes from the database and then just put them here
    let endpoints: HashMap<&str, [&str; 1], RandomState> =
        HashMap::from_iter([("available_endpoints", [""])]);

    let router = Router::new().route("/", get(|| async { axum::Json(endpoints) }))
        .route("/me", get(me));
    // .route(
    //     "/institutions",
    //     get(by_filter::<Institution>).post(new::<Vec<NewInstitution>>),
    // )
    // .route("/institutions/{institution_id}", get(by_id::<Institution>))
    // .route("/people", get())
    // .route("/people/{person_id}", get(by_id::<Person>))
    // .route("/api-key", post(new_api_key))
    // .route("/labs", get(by_filter::<Lab>).post(new::<Vec<NewLab>>))
    // .route("/labs/{lab_id}", get(by_id::<Lab>))
    // .route("/labs/{lab_id}/members", get(by_relationship::<LabId, Person>))
    // .route("/samples", get(by_filter::<Specimen>));
    router
}

mod handlers {

    use axum::{
        debug_handler,
        extract::{rejection::JsonRejection, FromRequest, Path, State}, response::{IntoResponse, Response},
    };
    use axum_extra::extract::Query;
    use diesel::prelude::*;
    use diesel_async::{AsyncConnection, scoped_futures::ScopedFutureExt, RunQueryDsl};
    use garde::Validate;
    use scamplers_core::person::{CreatedUser, NewPerson, Person};
    use serde::Serialize;
    use serde_json::json;
    use uuid::Uuid;
    use valuable::Valuable;

    use crate::{db::model::person::{WriteLogin}, server::{auth::{Frontend, HashedKey, Key, User}, AppState2}};
    use super::error::{Result, Error};
    use crate::db::Write;

    pub (super) struct ValidJson<T>(T);

    impl<S, T> FromRequest<S> for ValidJson<T>
    where
        axum::Json<T>: FromRequest<S, Rejection = JsonRejection>,
        S: Send + Sync,
        T: Validate,
        <T as Validate>::Context: std::default::Default,
    {
        type Rejection = Error;

        async fn from_request(req: axum::extract::Request, state: &S) -> std::result::Result<Self, Self::Rejection> {
            let axum::Json(data) = axum::Json::<T>::from_request(req, state).await?;
            data.validate()?;

            Ok(Self(data))
        }
    }

    impl<T: Serialize> IntoResponse for ValidJson<T> {
        fn into_response(self) -> Response {
            axum::Json(self.0).into_response()
        }
    }

    // pub async fn by_id<T: db::Read + Send>(
    //     UserId(user_id): UserId,
    //     State(app_state): State<AppState2>,
    //     Path(id): Path<T::Id>,
    // ) -> api::Result<ValidJson<T>> {
    //     let mut conn = app_state.db_conn().await?;

    //     let item = conn
    //         .transaction(|conn| {
    //             async move {
    //                 set_transaction_user(&user_id, conn).await?;

    //                 T::fetch_by_id(&id, conn).await
    //             }
    //             .scope_boxed()
    //         })
    //         .await?;

    //     Ok(ValidJson(item))
    // }

    // pub async fn by_filter<T: db::Read>(
    //     UserId(user_id): UserId,
    //     State(app_state): State<AppState2>,
    //     Query(query): Query<T::QueryParams>,
    // ) -> api::Result<ValidJson<Vec<T>>>
    // where
    //     T::QueryParams: Valuable,
    // {
    //     tracing::debug!(deserialized_query = query.as_value());

    //     let mut conn = app_state.db_conn().await?;

    //     let items = conn
    //         .transaction(|conn| {
    //             async move {
    //                 set_transaction_user(&user_id, conn).await?;

    //                 T::fetch_many(&query, conn).await
    //             }
    //             .scope_boxed()
    //         })
    //         .await?;

    //     Ok(ValidJson(items))
    // }

    // pub async fn by_relationship<T, U>(
    //     UserId(user_id): UserId,
    //     State(app_state): State<AppState2>,
    //     Path(id): Path<T>,
    //     Query(query): Query<U::QueryParams>,
    // ) -> api::Result<ValidJson<Vec<U>>>
    // where
    //     T: db::ReadRelatives<U>,
    //     U: db::Read,
    //     U::QueryParams: Valuable,
    // {
    //     tracing::debug!(parent_id = id.to_string(), deserialized_query = query.as_value());

    //     let mut conn = app_state.db_conn().await?;

    //     let relatives = conn
    //         .transaction(|conn| {
    //             async move {
    //                 set_transaction_user(&user_id, conn).await?;

    //                 id.fetch_relatives(&query, conn).await
    //             }
    //             .scope_boxed()
    //         })
    //         .await?;

    //     Ok(ValidJson(relatives))
    // }

    // pub async fn new<T>(
    //     UserId(user_id): UserId,
    //     State(app_state): State<AppState2>,
    //     ValidJson(data): ValidJson<T>,
    // ) -> api::Result<ValidJson<T::Returns>>
    // where
    //     T: Valuable + db::Write + garde::Validate,
    // {
    //     tracing::debug!(deserialized_data = data.as_value());

    //     let mut conn = app_state.db_conn().await?;

    //     let created = conn
    //         .transaction(|conn| {
    //             async move {
    //                 set_transaction_user(&user_id, conn).await?;

    //                 data.create(conn).await
    //             }
    //             .scope_boxed()
    //         })
    //         .await?;

    //     Ok(ValidJson(created))
    // }

    // pub async fn update<T>(
    //     UserId(user_id): UserId,
    //     State(app_state): State<AppState2>,
    //     ValidJson(data): ValidJson<T>,
    // ) -> api::Result<ValidJson<T::Returns>>
    // where
    //     T: Valuable + db::Update + garde::Validate,
    // {
    //     tracing::debug!(deserialized_data = data.as_value());

    //     let mut conn = app_state.db_conn().await?;

    //     let updated = conn
    //         .transaction(|conn| {
    //             async move {
    //                 set_transaction_user(&user_id, conn).await?;

    //                 data.update(conn).await
    //             }
    //             .scope_boxed()
    //         })
    //         .await?;

    //     Ok(ValidJson(updated))
    // }

    pub async fn new_user(_frontend_service: Frontend, State(app_state): State<AppState2>, ValidJson(person): ValidJson<NewPerson>) -> Result<ValidJson<CreatedUser>> {
        tracing::debug!(deserialized_person = person.as_value());

        let mut db_conn = app_state.db_conn().await?;

        let created_user = person.write_ms_login(&mut db_conn).await?;

        Ok(ValidJson(created_user))
    }

    // This is kind of repetetive but it's fine for now
    #[debug_handler]
    pub async fn me(
        user_id: Option<User>,
        State(app_state): State<AppState2>,
    ) -> Result<ValidJson<Option<Person>>> {
        use crate::db::model::person::fetch_by_id;

        tracing::debug!(user_id = user_id.as_value());

        let Some(User(user_id)) = user_id else {
            return Ok(ValidJson(None));
        };

        let mut conn = app_state.db_conn().await?;
        Ok(ValidJson(Some(fetch_by_id(user_id, &mut conn).await?)))
    }
}
