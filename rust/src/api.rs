use axum::{
    Router,
    extract::{FromRequestParts, State},
    response::IntoResponse,
};
use diesel_async::pooled_connection::deadpool;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    AppState,
    db::{self, person::User},
};
pub mod api_key;
mod v0;

pub fn router() -> Router<AppState> {
    // In theory, we should be able to inspect the header and route the request
    // based on the API version set in the header, but I don't know how to do that
    // yet
    v0::router()
}

pub struct ApiUser(db::person::User);

impl FromRequestParts<AppState> for ApiUser {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> std::result::Result<Self, Self::Rejection> {
        use Error::ApiKeyNotFound;

        if !state.production {
            return Ok(Self(User::test_user()));
        }

        let raw_api_key = parts
            .headers
            .get("X-API-Key")
            .ok_or(ApiKeyNotFound)?
            .as_bytes();

        let api_key = Uuid::from_slice(raw_api_key).map_err(|_| Error::InvalidApiKey)?;
        let mut conn = state.db_pool.get().await?;

        ApiUser::fetch_by_api_key(api_key, &mut conn).await
    }
}

#[derive(thiserror::Error, Serialize, Debug)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Error {
    #[error("failed to generate API key")]
    ApiKeyGeneration(String),
    #[error("API key not found in request headers")]
    ApiKeyNotFound,
    #[error(transparent)]
    Database(#[from] db::Error),
    #[error("invalid API key")]
    InvalidApiKey,
    #[error("operation not permitted")]
    Permission { message: String },
}
impl Error {
    fn staus_code(&self) -> axum::http::StatusCode {
        use Error::{ApiKeyGeneration, ApiKeyNotFound, Database, InvalidApiKey, Permission};
        use axum::http::StatusCode;
        use db::Error::{DuplicateRecord, Other, RecordNotFound, ReferenceNotFound};

        match self {
            ApiKeyGeneration(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiKeyNotFound | InvalidApiKey => StatusCode::UNAUTHORIZED,
            Permission { .. } => StatusCode::FORBIDDEN,
            Database(inner) => match inner {
                Other { .. } => StatusCode::INTERNAL_SERVER_ERROR,
                DuplicateRecord { .. } => StatusCode::CONFLICT,
                RecordNotFound => StatusCode::NOT_FOUND,
                ReferenceNotFound { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            },
        }
    }

    fn permission() -> Self {
        // initialize with empty message
        let mut err = Self::Permission {
            message: String::new(),
        };

        // the `Display` implementation of this error is what we want the serialized
        // error to show in most cases, so just use that
        let intended_message = err.to_string();

        // set the error
        match err {
            Self::Permission { ref mut message } => {
                message.push_str(&intended_message);
            }
            _ => {}
        }

        err
    }
}

impl From<deadpool::PoolError> for Error {
    fn from(err: deadpool::PoolError) -> Self {
        Self::Database(db::Error::from(err))
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;

        #[derive(Serialize)]
        struct ErrorResponse {
            status: u16,
            error: Option<Error>,
        }

        let status = self.staus_code();

        if status == StatusCode::INTERNAL_SERVER_ERROR {
            (
                status,
                axum::Json(ErrorResponse {
                    status: status.as_u16(),
                    error: None,
                }),
            )
                .into_response()
        } else {
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
}

pub type Result<T> = std::result::Result<T, Error>;
