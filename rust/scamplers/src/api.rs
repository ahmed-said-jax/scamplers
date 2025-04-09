use argon2::{PasswordHasher, password_hash::SaltString};
use axum::{
    RequestPartsExt, Router,
    extract::{
        FromRequest, FromRequestParts, Query, Request,
        rejection::{JsonRejection, PathRejection},
    },
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::{TypedHeader, extract::QueryRejection, headers};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool};
use garde::Validate;
use serde::{Deserialize, Serialize};
use strum::VariantArray;
use tower_http::trace::TraceLayer;
use uuid::Uuid;
use valuable::Valuable;

use crate::{
    AppState2, auth,
    db::{
        self,
        person::{UserRole, get_user_roles},
    },
};
mod v0;

pub fn router() -> Router<AppState2> {
    // In theory, we should be able to inspect the header and route the request
    // based on the API version set in the header, but I don't know how to do that
    // yet
    v0::router()
}

pub struct ValidJson<T>(pub T);
impl<S, T> FromRequest<S> for ValidJson<T>
where
    axum::Json<T>: FromRequest<S, Rejection = JsonRejection>,
    S: Send + Sync,
    T: Validate,
    <T as Validate>::Context: std::default::Default,
{
    type Rejection = Error;

    async fn from_request(req: Request, state: &S) -> std::result::Result<Self, Self::Rejection> {
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

#[derive(thiserror::Error, Serialize, Debug, Clone, Valuable)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Error {
    #[error(transparent)]
    Database(#[from] db::Error),
    #[error("simple invalid data")]
    SimpleData { reason: String },
    #[error("malformed request")]
    MalformedRequest {
        #[serde(skip)]
        #[valuable(skip)]
        status: StatusCode,
        message: String,
    },
    #[error("operation not permitted")]
    Permission { message: String },
}
impl Error {
    fn staus_code(&self) -> axum::http::StatusCode {
        use Error::*;
        use db::Error::{Data, DuplicateRecord, Other, RecordNotFound, ReferenceNotFound};

        match self {
            SimpleData { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Permission { .. } => StatusCode::FORBIDDEN,
            Database(inner) => match inner {
                Data(_) => StatusCode::UNPROCESSABLE_ENTITY,
                Other { .. } => StatusCode::INTERNAL_SERVER_ERROR,
                DuplicateRecord { .. } => StatusCode::CONFLICT,
                RecordNotFound => StatusCode::NOT_FOUND,
                ReferenceNotFound { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            },
            MalformedRequest { status, .. } => *status,
        }
    }
}

impl From<JsonRejection> for Error {
    fn from(err: JsonRejection) -> Self {
        Self::MalformedRequest {
            status: err.status(),
            message: err.body_text(),
        }
    }
}

impl From<QueryRejection> for Error {
    fn from(err: QueryRejection) -> Self {
        Self::MalformedRequest {
            status: err.status(),
            message: format!("{err:#}"),
        }
    }
}

impl From<PathRejection> for Error {
    fn from(err: PathRejection) -> Self {
        Self::MalformedRequest {
            status: err.status(),
            message: err.body_text(),
        }
    }
}

impl From<deadpool::PoolError> for Error {
    fn from(err: deadpool::PoolError) -> Self {
        Self::Database(db::Error::from(err))
    }
}

impl From<garde::Report> for Error {
    fn from(err: garde::Report) -> Self {
        Self::SimpleData {
            reason: format!("{err:#}"),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;

        tracing::error!(error = self.as_value());

        #[derive(Serialize)]
        struct ErrorResponse {
            status: u16,
            error: Option<Error>,
        }

        let status = self.staus_code();

        if status == StatusCode::INTERNAL_SERVER_ERROR {
            return (
                status,
                axum::Json(ErrorResponse {
                    status: status.as_u16(),
                    error: None,
                }),
            )
                .into_response();
        }

        (
            status,
            axum::Json(ErrorResponse {
                status: status.as_u16(),
                error: Some(self),
            }),
        )
            .into_response()
    }
}

pub type Result<T> = std::result::Result<T, Error>;
