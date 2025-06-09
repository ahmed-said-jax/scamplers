use axum::{
    Json,
    extract::{FromRequest, OptionalFromRequest, Path, State},
    response::{IntoResponse, Response},
};
use diesel_async::{AsyncConnection, scoped_futures::ScopedFutureExt};
use garde::Validate;
use scamplers_core::model::person::{CreatedUser, NewPerson};
use serde::{Serialize, de::DeserializeOwned};
use valuable::Valuable;

use crate::{
    db::{model::person::WriteLogin, set_transaction_user},
    server::{
        AppState,
        auth::{Frontend, User},
    },
};

use super::error::{Error, Result};

#[derive(Default)]
pub(super) struct ValidJson<T>(T);

impl<S, T> FromRequest<S> for ValidJson<T>
where
    S: Send + Sync,
    T: Validate + DeserializeOwned,
    T::Context: std::default::Default,
{
    type Rejection = Error;

    async fn from_request(
        req: axum::extract::Request,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let Json(data) = <Json<T> as FromRequest<S>>::from_request(req, state).await?;
        data.validate()?;

        Ok(Self(data))
    }
}

impl<S, T> OptionalFromRequest<S> for ValidJson<T>
where
    S: Send + Sync,
    T: Validate + DeserializeOwned,
    T::Context: std::default::Default,
{
    type Rejection = Error;

    async fn from_request(
        req: axum::extract::Request,
        state: &S,
    ) -> std::result::Result<Option<Self>, Self::Rejection> {
        let Some(Json(data)) =
            <Json<T> as OptionalFromRequest<S>>::from_request(req, state).await?
        else {
            return Ok(None);
        };

        data.validate()?;

        Ok(Some(Self(data)))
    }
}

impl<T: Serialize> IntoResponse for ValidJson<T> {
    fn into_response(self) -> Response {
        let Self(inner) = self;

        axum::Json(inner).into_response()
    }
}

pub(super) async fn new_user(
    _auth: Frontend,
    State(app_state): State<AppState>,
    ValidJson(person): ValidJson<NewPerson>,
) -> Result<Json<CreatedUser>> {
    tracing::info!(deserialized_new_user = person.as_value());

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
    Data: crate::db::Write + Send + valuable::Valuable,
    Data::Returns: Send,
{
    tracing::info!(deserialized_data = data.as_value());

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
    Resource::Id: Send + Sync + valuable::Valuable,
{
    tracing::info!(deserialized_id = resource_id.as_value());

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
    query: Option<ValidJson<Resource::QueryParams>>,
) -> super::error::Result<Json<Vec<Resource>>>
where
    Resource: crate::db::FetchByQuery + Send,
    Resource::QueryParams: Send + valuable::Valuable + Default,
{
    let ValidJson(query) = query.unwrap_or_default();
    tracing::info!(deserialized_query = query.as_value());

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
