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
    AppState2,
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

struct ValidJson<T>(T);
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

trait SessionIdOrApiKey {
    fn hash(&self, salt_string: &str) -> String;
}

impl SessionIdOrApiKey for Uuid {
    fn hash(&self, salt_string: &str) -> String {
        let hasher = argon2::Argon2::default();
        let bytes = self.as_bytes();

        let salt = SaltString::from_b64(salt_string).unwrap();

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

    async fn fetch_by_api_key(api_key: &Uuid, salt_string: &str, conn: &mut AsyncPgConnection) -> db::Result<Self> {
        use crate::schema::person::dsl::*;

        let hash = api_key.hash(salt_string);

        let user_id = person.filter(api_key_hash.eq(hash)).select(id).first(conn).await?;

        Ok(Self::Api { user_id })
    }

    async fn fetch_by_session_id(
        session_id: &Uuid,
        salt_string: &str,
        conn: &mut AsyncPgConnection,
    ) -> db::Result<Self> {
        use crate::schema::{
            person::dsl::{id as person_id, name as person_name, person},
            session::dsl::{id_hash, session},
        };

        let hash = session_id.hash(salt_string);

        let (user_id, first_name): (Uuid, String) = session
            .inner_join(person)
            .filter(id_hash.eq(hash))
            .select((person_id, person_name))
            .first(conn)
            .await?;
        // let roles = get_user_roles(user_id.to_string()).execute(conn).await?;

        Ok(Self::Web {
            user_id,
            first_name,
            roles: Default::default(),
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
                user_id: *user_id,
                first_name: "you".to_string(),
                roles: UserRole::VARIANTS.to_vec(),
            });
        }
        let requested_resource = parts.uri.path().to_string();
        let Query(Web { web }) = parts.extract().await.unwrap_or_default();

        let mut conn = app_state.db_conn().await?;

        let session_id_salt_string = match app_state {
            AppState2::Dev { .. } => "00000000",
            AppState2::Prod { config, .. } => config.session_id_salt_string(),
        };

        if !web {
            let raw_api_key = parts.headers.get("X-API-Key").ok_or(ApiKeyNotFound)?.as_bytes();

            let api_key = Uuid::from_slice(raw_api_key).map_err(|_| InvalidApiKey)?;
            let result = User::fetch_by_api_key(&api_key, session_id_salt_string, &mut conn).await;

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

        let err = InvalidSessionId {
            redirected_from: requested_resource.to_string(),
        };

        let Ok(cookies) = parts.extract::<TypedHeader<headers::Cookie>>().await else {
            return Err(err.clone());
        };

        let session_id = cookies
            .get("SESSION")
            .ok_or(err.clone())?
            .parse()
            .map_err(|_| err.clone())?;

        let result = User::fetch_by_session_id(&session_id, session_id_salt_string, &mut conn).await;
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

#[derive(thiserror::Error, Serialize, Debug, Clone, Valuable)]
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
    #[error("simple invalid data")]
    SimpleData { reason: String },
    #[error("invalid session ID")]
    InvalidSessionId { redirected_from: String },
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
        use Error::{
            ApiKeyGeneration, ApiKeyNotFound, Database, InvalidApiKey, InvalidSessionId, MalformedRequest, Permission,
            SimpleData,
        };
        use db::Error::{Data, DuplicateRecord, Other, RecordNotFound, ReferenceNotFound};

        match self {
            ApiKeyGeneration(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiKeyNotFound | InvalidApiKey => StatusCode::UNAUTHORIZED,
            SimpleData { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            InvalidSessionId { .. } => StatusCode::TEMPORARY_REDIRECT,
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

    fn new_permission() -> Self {
        // initialize with empty message
        let mut err = Self::Permission { message: String::new() };

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

        match self {
            Self::InvalidSessionId { redirected_from } => {
                Redirect::temporary(&format!("/auth?redirected_from={redirected_from}")).into_response()
            }
            _ => (
                status,
                axum::Json(ErrorResponse {
                    status: status.as_u16(), // why did I choose a u16 for this
                    error: Some(self),
                }),
            )
                .into_response(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
