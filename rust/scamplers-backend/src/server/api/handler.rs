use axum::{
    Json,
    extract::{FromRequest, Path, State, rejection::JsonRejection},
};
use diesel_async::{AsyncConnection, scoped_futures::ScopedFutureExt};
use garde::Validate;
use scamplers_core::model::person::{CreatedUser, NewPerson};
use valuable::Valuable;

use crate::{
    db::{model::person::WriteLogin, set_transaction_user},
    server::{
        AppState,
        auth::{Frontend, User},
    },
};

use super::error::{Error, Result};

pub(super) struct ValidJson<T>(T);

impl<S, T> FromRequest<S> for ValidJson<T>
where
    axum::Json<T>: FromRequest<S, Rejection = JsonRejection>,
    S: Send + Sync,
    T: Validate,
    <T as Validate>::Context: std::default::Default,
{
    type Rejection = Error;

    async fn from_request(
        req: axum::extract::Request,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let axum::Json(data) = axum::Json::<T>::from_request(req, state).await?;
        data.validate()?;

        Ok(Self(data))
    }
}

pub(super) async fn new_user(
    _auth: Frontend,
    State(app_state): State<AppState>,
    ValidJson(person): ValidJson<NewPerson>,
) -> Result<Json<CreatedUser>> {
    tracing::debug!(deserialized_person = person.as_value());

    let mut db_conn = app_state.db_conn().await?;

    let created_user = person.write_ms_login(&mut db_conn).await?;

    Ok(Json(created_user))
}

pub async fn write<Data>(
    User(user_id): User,
    State(app_state): State<AppState>,
    ValidJson(data): ValidJson<Data>,
) -> super::error::Result<Json<Data::Returns>>
where
    Data: crate::db::Write + Send,
    Data::Returns: Send,
{
    let mut db_conn = app_state.db_conn().await?;

    let item = db_conn
        .transaction(|conn| {
            async move {
                set_transaction_user(&user_id, conn).await?;

                data.write(conn).await
            }
            .scope_boxed()
        })
        .await?;

    Ok(Json(item))
}

pub async fn by_id<Resource>(
    User(user_id): User,
    State(app_state): State<AppState>,
    Path(resource_id): Path<Resource::Id>,
) -> super::error::Result<Json<Resource>>
where
    Resource: crate::db::FetchById + Send,
    Resource::Id: Send + Sync,
{
    let mut db_conn = app_state.db_conn().await?;

    let item = db_conn
        .transaction(|conn| {
            async move {
                set_transaction_user(&user_id, conn).await?;

                Resource::fetch_by_id(&resource_id, conn).await
            }
            .scope_boxed()
        })
        .await?;

    Ok(Json(item))
}

pub async fn by_query<Resource>(
    User(user_id): User,
    State(app_state): State<AppState>,
    Json(query): Json<Resource::QueryParams>,
) -> super::error::Result<Json<Vec<Resource>>>
where
    Resource: crate::db::FetchByQuery + Send,
    Resource::QueryParams: Send,
{
    let mut db_conn = app_state.db_conn().await?;

    let item = db_conn
        .transaction(|conn| {
            async move {
                set_transaction_user(&user_id, conn).await?;

                Resource::fetch_by_query(&query, conn).await
            }
            .scope_boxed()
        })
        .await?;

    Ok(Json(item))
}
