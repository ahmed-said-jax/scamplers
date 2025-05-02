use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString},
};
use axum::{
    RequestExt, RequestPartsExt,
    extract::{FromRequestParts, OptionalFromRequestParts},
    response::IntoResponse,
};
use axum_extra::{
    TypedHeader,
    headers::{self, authorization::Basic},
};
use diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    prelude::*,
    serialize::{ToSql, WriteTuple},
    sql_types::{self, Bool, Record, Text},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use rand::{
    Rng, SeedableRng, TryRngCore,
    distr::Alphanumeric,
    rngs::{OsRng, StdRng},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use valuable::Valuable;

use crate::db;

use super::AppState2;

const KEY_PREFIX_LENGTH: usize = 8;
const KEY_LENGTH: usize = 32;

#[derive(Deserialize, Serialize)]
#[serde(transparent)]
pub struct ApiKey(String);
impl ApiKey {
    pub fn new() -> Self {
        Self::default()
    }

    fn prefix(&self) -> &str {
        let Self(key) = self;
        &key[..KEY_PREFIX_LENGTH]
    }

    pub fn hash(&self) -> HashedApiKey {
        let Self(key) = self;

        let mut salt = [0u8; 16];
        OsRng.try_fill_bytes(&mut salt).unwrap();

        let salt = SaltString::encode_b64(&salt).unwrap();

        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(key.as_bytes(), &salt)
            .unwrap()
            .to_string();

        HashedApiKey {
            prefix: self.prefix().to_string(),
            hash,
        }
    }

    fn is_same_hash(&self, other: &HashedApiKey) -> bool {
        let argon2 = Argon2::default();

        let Ok(parsed_hash) = PasswordHash::new(&other.hash) else {
            return false;
        };

        if argon2
            .verify_password(self.as_str().as_bytes(), &parsed_hash)
            .is_ok()
        {
            true
        } else {
            false
        }
    }

    pub fn as_str(&self) -> &str {
        let Self(inner) = self;

        &inner
    }
}
impl FromStr for ApiKey {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}
impl Default for ApiKey {
    fn default() -> Self {
        let mut rng = StdRng::from_os_rng();
        let key = (0..KEY_LENGTH)
            .map(|_| rng.sample(Alphanumeric) as char)
            .collect();

        Self(key)
    }
}
impl Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.hash().fmt(f)
    }
}

impl Display for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(inner) = self;
        <String as Display>::fmt(inner, f)
    }
}

#[derive(AsExpression, Debug, FromSqlRow, Deserialize, Valuable)]
#[diesel(sql_type = scamplers_schema::sql_types::HashedKey)]
pub struct HashedApiKey {
    prefix: String,
    hash: String,
}

impl ToSql<scamplers_schema::sql_types::HashedKey, Pg> for HashedApiKey {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        let Self { prefix, hash } = self;

        WriteTuple::<(sql_types::Text, sql_types::Text)>::write_tuple(
            &(prefix, hash),
            &mut out.reborrow(),
        )
    }
}

impl FromSql<scamplers_schema::sql_types::HashedKey, Pg> for HashedApiKey {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        let (prefix, hash) =
            FromSql::<Record<(sql_types::Text, sql_types::Text)>, Pg>::from_sql(bytes)?;

        Ok(Self { prefix, hash })
    }
}

#[derive(Clone, Copy, Valuable)]
pub(super) struct User(pub(super) Uuid);
impl User {
    async fn fetch_by_api_key(
        api_key: &ApiKey,
        conn: &mut AsyncPgConnection,
    ) -> db::error::Result<Self> {
        use scamplers_schema::person::dsl::{hashed_api_key, id, person};

        let filter_query = diesel::dsl::sql::<Bool>("(hashed_api_key).prefix = ")
            .bind::<Text, _>(api_key.prefix());

        let (user_id, found_api_key) = person
            .filter(filter_query)
            .select((id, hashed_api_key.assume_not_null()))
            .first(conn)
            .await?;

        if !api_key.is_same_hash(&found_api_key) {
            return Err(db::error::Error::RecordNotFound);
        }

        Ok(Self(user_id))
    }
}

impl FromRequestParts<AppState2> for User {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        app_state: &AppState2,
    ) -> Result<Self, Self::Rejection> {
        if let AppState2::Dev { user_id, .. } = app_state {
            return Ok(Self(*user_id));
        }

        let Some(Ok(api_key)) = parts
            .headers
            .get("X-API-Key")
            .map(|s| s.to_str().unwrap().parse())
        else {
            return Err(Error::InvalidApiKey);
        };

        let mut db_conn = app_state.db_conn().await?;

        let user = User::fetch_by_api_key(&api_key, &mut db_conn).await?;

        Ok(user)
    }
}

impl OptionalFromRequestParts<AppState2> for User {
    type Rejection = ();

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState2,
    ) -> Result<Option<Self>, Self::Rejection> {
        Ok(
            <User as FromRequestParts<_>>::from_request_parts(parts, state)
                .await
                .ok(),
        )
    }
}

pub struct Frontend;
impl FromRequestParts<AppState2> for Frontend {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState2,
    ) -> Result<Self, Self::Rejection> {
        let AppState2::Prod { config, .. } = state else {
            return Ok(Self);
        };

        let err = Error::InvalidFrontendCredentials;

        let Ok(frontend_service_credentials) = parts
            .extract::<TypedHeader<headers::Authorization<Basic>>>()
            .await
        else {
            return Err(err);
        };

        if (
            frontend_service_credentials.username(),
            frontend_service_credentials.password(),
        ) != ("scamplers-frontend", config.lock().unwrap().auth_secret())
        {
            return Err(err);
        }

        Ok(Self)
    }
}

#[derive(thiserror::Error, Serialize, Debug, Clone, Valuable)]
#[serde(rename_all = "snake_case", tag = "type")]
pub(super) enum Error {
    #[error("invalid API key")]
    InvalidApiKey,
    #[error("invalid auth user password")]
    InvalidFrontendCredentials,
    #[error(transparent)]
    Other(db::error::Error),
}
impl From<db::error::Error> for Error {
    fn from(err: db::error::Error) -> Self {
        use db::error::Error::*;

        match err {
            RecordNotFound => Self::InvalidApiKey,
            _ => Self::Other(err),
        }
    }
}
impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;

        tracing::error!(auth_error = self.as_value());

        #[derive(Serialize)]
        struct ErrorResponse {
            status: u16,
            error: Option<Error>,
        }

        match self {
            Self::InvalidApiKey | Self::InvalidFrontendCredentials => (
                StatusCode::UNAUTHORIZED,
                axum::Json(ErrorResponse {
                    status: StatusCode::UNAUTHORIZED.as_u16(),
                    error: Some(self),
                }),
            )
                .into_response(),
            Self::Other(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(ErrorResponse {
                    status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    error: None,
                }),
            )
                .into_response(),
        }
    }
}
