use axum::{
    extract::{FromRequest, State, rejection::JsonRejection},
    response::{IntoResponse, Response},
};
use garde::Validate;
use scamplers_core::model::person::{CreatedUser, NewPerson};
use serde::Serialize;
use valuable::Valuable;

use crate::{
    db::model::person::WriteLogin,
    server::{AppState, auth::Frontend},
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

impl<T: Serialize> IntoResponse for ValidJson<T> {
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

pub(super) async fn new_user(
    _auth: Frontend,
    State(app_state): State<AppState>,
    ValidJson(person): ValidJson<NewPerson>,
) -> Result<ValidJson<CreatedUser>> {
    tracing::debug!(deserialized_person = person.as_value());

    let mut db_conn = app_state.db_conn().await?;

    let session = person.write_ms_login(&mut db_conn).await?;

    Ok(ValidJson(session))
}
