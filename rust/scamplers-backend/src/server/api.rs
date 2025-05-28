use std::{collections::HashMap, hash::RandomState};

use axum::{
    Router,
    routing::{get, post},
};
use scamplers_core::{model::AsEndpoint, model::person::CreatedUser};

use super::AppState2;

mod error;
mod handler;

pub(super) fn router() -> Router<AppState2> {
    use handler::*;

    let endpoints: HashMap<&str, [&str; 1], RandomState> =
        HashMap::from_iter([("available_endpoints", [""])]);

    let router = Router::new()
        .route("/", get(|| async { axum::Json(endpoints) }))
        // .route("/me", get(me))
        .route(CreatedUser::as_endpoint(), post(new_user));
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

// mod handlers {

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

// // This is kind of repetetive but it's fine for now
// #[debug_handler]
// pub (super) async fn me(
//     user_id: Option<User>,
//     State(app_state): State<AppState2>,
// ) -> Result<ValidJson<Option<Person>>> {
//     use crate::db::model::person::fetch_by_id;

//     tracing::debug!(user_id = user_id.as_value());

//     let Some(User(user_id)) = user_id else {
//         return Ok(ValidJson(None));
//     };

//     let mut conn = app_state.db_conn().await?;
//     Ok(ValidJson(Some(fetch_by_id(user_id, &mut conn).await?)))
// }
// }
