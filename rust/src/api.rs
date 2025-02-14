use argon2::{PasswordHasher, password_hash::SaltString};
use axum::{
    RequestPartsExt, Router,
    extract::{
        FromRequest, FromRequestParts, Query,
        rejection::{JsonRejection, PathRejection},
    },
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::{TypedHeader, extract::QueryRejection, headers};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool};
use serde::{Deserialize, Serialize};
use strum::VariantArray;
use uuid::Uuid;

use crate::{
    AppState2,
    db::{self, person::UserRole},
};
mod v0;

pub fn router() -> Router<AppState2> {
    // In theory, we should be able to inspect the header and route the request
    // based on the API version set in the header, but I don't know how to do that
    // yet
    v0::router()
}

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(Error))]
struct ApiJson<T>(T);

impl<T: Serialize> IntoResponse for ApiJson<T> {
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

trait SessionIdOrApiKey {
    fn hash(&self) -> String;
}

impl SessionIdOrApiKey for Uuid {
    fn hash(&self) -> String {
        let hasher = argon2::Argon2::default();
        let bytes = self.as_bytes();

        // We don't care about salting because it's already highly random, being a UUID
        let salt = SaltString::from_b64("0000").unwrap();

        let hash = hasher.hash_password(bytes, &salt).unwrap().to_string();

        hash
    }
}

enum User {
    Web {
        user_id: Uuid,
        first_name: String,
        roles: Vec<UserRole>,
    },
    Api {
        user_id: Uuid,
    },
}

impl User {
    fn id(&self) -> &Uuid {
        match self {
            User::Web { user_id, .. } | User::Api { user_id, .. } => user_id,
        }
    }

    async fn fetch_by_api_key(api_key: &Uuid, conn: &mut AsyncPgConnection) -> db::Result<Self> {
        use crate::schema::person::dsl::*;

        let hash = api_key.hash();

        let user_id = person
            .filter(api_key_hash.eq(hash))
            .select(id)
            .first(conn)
            .await?;

        Ok(Self::Api { user_id })
    }

    async fn fetch_by_session_id(
        session_id: &Uuid,
        conn: &mut AsyncPgConnection,
    ) -> db::Result<Self> {
        use crate::schema::{
            cache::dsl::{cache, session_id_hash},
            person::dsl::{first_name as person_first_name, id as person_id, person},
        };

        let hash = session_id.hash();

        let (user_id, user_first_name) = cache
            .inner_join(person)
            .filter(session_id_hash.eq(hash))
            .select((person_id, person_first_name))
            .first(conn)
            .await?;
        let roles = Vec::with_capacity(0); // TODO: actually get user_roles

        Ok(Self::Web {
            user_id,
            first_name: user_first_name,
            roles,
        })
    }
}

impl FromRequestParts<AppState2> for User {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        app_state: &AppState2,
    ) -> std::result::Result<Self, Self::Rejection> {
        use Error::{ApiKeyNotFound, InvalidApiKey, InvalidSessionId};

        #[derive(Deserialize, Default)]
        struct Web {
            web: bool,
        }

        if let AppState2::Dev { user_id, .. } = app_state {
            return Ok(User::Web {
                user_id: user_id.clone(),
                first_name: "you".to_string(),
                roles: UserRole::VARIANTS.to_vec(),
            });
        }

        let Query(Web { web }): Query<Web> = parts.extract().await.unwrap_or_default();

        let mut conn = app_state.db_conn().await?;

        if !web {
            let raw_api_key = parts
                .headers
                .get("X-API-Key")
                .ok_or(ApiKeyNotFound)?
                .as_bytes();

            let api_key = Uuid::from_slice(raw_api_key).map_err(|_| InvalidApiKey)?;
            let result = User::fetch_by_api_key(&api_key, &mut conn).await;

            let Ok(user) = result else {
                let err = match result {
                    Err(db::Error::RecordNotFound) => Err(Error::InvalidApiKey),
                    Err(e) => Err(Error::from(e)),
                    Ok(_) => unreachable!(),
                };

                return err;
            };

            return Ok(user);
        }

        let (AppState2::Test { auth_url, .. } | AppState2::Prod { auth_url, .. }) = app_state
        else {
            unreachable!("we already tested for the only other variant")
        };

        let err = InvalidSessionId {
            auth_url: auth_url.to_string(),
        };

        let Ok(cookies) = parts.extract::<TypedHeader<headers::Cookie>>().await else {
            return Err(err);
        };

        let session_id = cookies
            .get("SESSION")
            .ok_or(err.clone())?
            .parse()
            .map_err(|_| err.clone())?;

        let result = User::fetch_by_session_id(&session_id, &mut conn).await;
        let Ok(user) = result else {
            let err = match result {
                Err(db::Error::RecordNotFound) => Err(err),
                Err(db_err) => Err(Error::from(db_err)),
                Ok(_) => unreachable!("we already extracted the 'Ok' variant"),
            };

            return err;
        };

        Ok(user)
    }
}

#[derive(thiserror::Error, Serialize, Debug, Clone)]
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
    #[error("invalid session ID")]
    InvalidSessionId { auth_url: String },
    #[error("malformed request")]
    MalformedRequest {
        #[serde(skip)]
        status: StatusCode,
        message: String,
    },
    #[error("operation not permitted")]
    Permission { message: String },
}
impl Error {
    fn staus_code(&self) -> axum::http::StatusCode {
        use Error::{
            ApiKeyGeneration, ApiKeyNotFound, Database, InvalidApiKey, InvalidSessionId,
            MalformedRequest, Permission,
        };
        use db::Error::{DuplicateRecord, Other, RecordNotFound, ReferenceNotFound};

        match self {
            ApiKeyGeneration(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiKeyNotFound | InvalidApiKey => StatusCode::UNAUTHORIZED,
            InvalidSessionId { .. } => StatusCode::TEMPORARY_REDIRECT,
            Permission { .. } => StatusCode::FORBIDDEN,
            Database(inner) => match inner {
                Other { .. } => StatusCode::INTERNAL_SERVER_ERROR,
                DuplicateRecord { .. } => StatusCode::CONFLICT,
                RecordNotFound => StatusCode::NOT_FOUND,
                ReferenceNotFound { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            },
            MalformedRequest { status, .. } => *status,
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
            return (
                status,
                axum::Json(ErrorResponse {
                    status: status.as_u16(),
                    error: None,
                }),
            )
                .into_response();
        }

        match self {
            Self::InvalidSessionId { auth_url } => Redirect::temporary(&auth_url).into_response(),
            _ => (
                status,
                axum::Json(ErrorResponse {
                    status: status.as_u16(),
                    error: Some(self),
                }),
            )
                .into_response(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
