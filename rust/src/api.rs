use std::str::FromStr;

use axum::{extract::FromRequestParts, response::IntoResponse, Router};
use argon2::{password_hash::{rand_core::OsRng, SaltString}, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types::{self, SqlType},
};
use diesel_async::{pooled_connection::deadpool, AsyncPgConnection, RunQueryDsl};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use strum::VariantArray;
use uuid::Uuid;

use crate::{
    db::{self, person::{User, UserRole}, Entity},
    schema::sql_types as custom_types,
    AppState,
};
mod v0;
pub mod api_key;
use api_key::{AsApiKey, ApiKeyHash};

pub fn router() -> Router<AppState> {
    // In theory, we should be able to inspect the header and route the request
    // based on the API version set in the header, but I don't know how to do that
    // yet
    v0::router()
}


struct ApiUser(db::person::User);

impl ApiUser {
    async fn fetch_by_api_key(conn: &mut AsyncPgConnection, api_key: Uuid) -> Result<Self> {
        let user = User::fetch_by_api_key(conn, api_key).await.map_err(|e| Error::InvalidApiKey)?;

        Ok(Self(user))
    }
}

impl FromRequestParts<AppState> for ApiUser {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> std::result::Result<Self, Self::Rejection> {
        use Error::{ApiKeyNotFound, InvalidApiKey};

        use crate::schema::person::dsl::{api_key_hash, id, person, roles};

        if !state.production {
            return Ok(Self(db::person::User::test_user()));
        }

        let raw_api_key = parts
            .headers
            .get("X-API-key")
            .ok_or(ApiKeyNotFound)?
            .as_bytes();

        let api_key = Uuid::from_slice(raw_api_key).map_err(|_| Error::InvalidApiKey)?;
        let mut conn = state.db_pool.get().await?;

        ApiUser::fetch_by_api_key(&mut conn, api_key).await
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
        use axum::http::StatusCode;
        use db::Error::{DuplicateRecord, Other, RecordNotFound, ReferenceNotFound};
        use Error::{ApiKeyNotFound, Database, InvalidApiKey, Permission, ApiKeyGeneration};

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
