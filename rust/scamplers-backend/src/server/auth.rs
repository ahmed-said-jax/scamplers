use std::{fmt::Debug, str::FromStr};

use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString},
};
use axum::{
    RequestExt, RequestPartsExt,
    extract::{FromRequestParts, OptionalFromRequestParts, Request, State},
    http::HeaderValue,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{
        self,
        authorization::{Basic, Bearer},
    },
};
use diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    prelude::*,
    serialize::{ToSql, WriteTuple},
    sql_types::{self, Bool, Record, SqlType, Text},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use rand::{
    Rng, SeedableRng, TryRngCore,
    distr::Alphanumeric,
    rngs::{OsRng, StdRng},
};
use reqwest::{StatusCode, header::AsHeaderName};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use valuable::Valuable;

use crate::db;

use super::AppState2;

const KEY_PREFIX_LENGTH: usize = 8;
const KEY_LENGTH: usize = 32;
const USER_ID_HEADER: &str = "SCAMPLERS_USER_ID";

#[derive(Deserialize, Serialize)]
#[serde(transparent)]
pub(super) struct Key(String);
impl Key {
    pub fn new() -> Self {
        Self::default()
    }

    fn prefix(&self) -> &str {
        let Self(key) = self;
        &key[..KEY_PREFIX_LENGTH]
    }

    pub fn hash(&self) -> HashedKey<&str> {
        let Self(key) = self;

        let mut salt = [0u8; 16];
        OsRng.try_fill_bytes(&mut salt).unwrap();

        let salt = SaltString::encode_b64(&salt).unwrap();

        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(key.as_bytes(), &salt)
            .unwrap()
            .to_string();

        HashedKey {
            prefix: self.prefix(),
            hash,
        }
    }

    fn is_same_hash(&self, other: &HashedKey<String>) -> bool {
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
impl FromStr for Key {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}
impl Default for Key {
    fn default() -> Self {
        let mut rng = StdRng::from_os_rng();
        let key = (0..KEY_LENGTH)
            .map(|_| rng.sample(Alphanumeric) as char)
            .collect();

        Self(key)
    }
}
impl Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.hash().fmt(f)
    }
}

#[derive(AsExpression, Debug, FromSqlRow, Deserialize, Valuable)]
#[diesel(sql_type = scamplers_schema::sql_types::HashedKey)]
pub struct HashedKey<Str: AsExpression<diesel::sql_types::Text> + Valuable>
where
    for<'a> &'a Str: AsExpression<diesel::sql_types::Text>,
{
    prefix: Str,
    hash: String,
}

impl ToSql<scamplers_schema::sql_types::HashedKey, Pg> for HashedKey<&str> {
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

impl FromSql<scamplers_schema::sql_types::HashedKey, Pg> for HashedKey<String> {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        let (prefix, hash) =
            FromSql::<Record<(sql_types::Text, sql_types::Text)>, Pg>::from_sql(bytes)?;

        Ok(Self { prefix, hash })
    }
}

#[derive(Clone, Copy, Valuable)]
pub(super) struct UserId(pub (super) Uuid);
impl UserId {
    async fn fetch_by_api_key(
        api_key: &Key,
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

    async fn fetch_by_session_id(
        session_id: &Key,
        conn: &mut AsyncPgConnection,
    ) -> db::error::Result<Self> {
        use scamplers_schema::{
            person::dsl::{id as person_id, person},
            session::dsl::{hashed_id as hashed_session_id, session},
        };

        let filter_query =
            diesel::dsl::sql::<Bool>("(hashed_id).prefix = ").bind::<Text, _>(session_id.prefix());

        let (user_id, found_session_id): (_, HashedKey<String>) = session
            .inner_join(person)
            .filter(filter_query)
            .select((person_id, hashed_session_id))
            .first(conn)
            .await?;

        if !session_id.is_same_hash(&found_session_id) {
            return Err(db::error::Error::RecordNotFound);
        }

        Ok(Self(user_id))
    }
}

impl FromRequestParts<AppState2> for UserId {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState2,
    ) -> Result<Self, Self::Rejection> {
        if let AppState2::Dev { user_id, .. } = state {
            return Ok(Self(*user_id));
        }

        let Some(user_id) = parts.headers.get(USER_ID_HEADER) else {
            return Err(Error::NoUserId);
        };
        let user_id =
            Uuid::from_str(user_id.to_str().unwrap()).map_err(|_| Error::InvalidUserId)?;

        Ok(Self(user_id))
    }
}

impl OptionalFromRequestParts<AppState2> for UserId {
    type Rejection = ();

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState2,
    ) -> Result<Option<Self>, Self::Rejection> {
        Ok(
            <UserId as FromRequestParts<_>>::from_request_parts(parts, state)
                .await
                .ok(),
        )
    }
}

enum RequestType {
    Api,
    Browser,
}

async fn authenticate(
    app_state: &AppState2,
    mut request: Request,
    next: Next,
    key: &str,
    request_type: RequestType,
    err: Response,
) -> Response {
    if request.headers().contains_key(USER_ID_HEADER) {
        return err;
    }

    if let AppState2::Dev { .. } = &app_state {
        return next.run(request).await;
    }

    let Ok(mut db_conn) = app_state.db_conn().await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let key = Key::from_str(key).unwrap();

    let result = match request_type {
        RequestType::Api => UserId::fetch_by_api_key(&key, &mut db_conn).await,
        RequestType::Browser => UserId::fetch_by_session_id(&key, &mut db_conn).await,
    };

    let user_id = match result {
        Ok(UserId(user_id)) => user_id,
        Err(db::error::Error::RecordNotFound) => {
            return err;
        }
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let user_id = user_id.to_string();

    request
        .headers_mut()
        .insert(USER_ID_HEADER, user_id.parse().unwrap());

    next.run(request).await
}

pub(super) async fn authenticate_api_request(
    State(app_state): State<AppState2>,
    request: Request,
    next: Next,
) -> Response {
    let uri = request.uri().to_string();

    // The 'session' route of the API has its own authentication
    if uri.contains("/session") {
        return next.run(request).await;
    }

    let err = Error::InvalidApiKey.into_response();

    let Some(raw_api_key) = request.headers().get("X-API-Key").cloned() else {
        return err;
    };

    let Ok(api_key) = raw_api_key.to_str() else {
        return err;
    };

    authenticate(&app_state, request, next, api_key, RequestType::Api, err).await
}

pub(super) async fn authenticate_browser_request(
    State(app_state): State<AppState2>,
    mut request: Request,
    next: Next,
) -> Response {
    let redirected_from = request.uri().to_string();

    if redirected_from.contains("/login") {
        return next.run(request).await;
    }

    let err = Error::InvalidSessionId { redirected_from }.into_response();

    let Ok(cookies) = request
        .extract_parts::<TypedHeader<headers::Cookie>>()
        .await
    else {
        return err;
    };

    let Some(session_id) = cookies.get("SESSION") else {
        return err;
    };

    authenticate(
        &app_state,
        request,
        next,
        session_id,
        RequestType::Browser,
        err,
    )
    .await
}

pub struct AuthService;
impl FromRequestParts<AppState2> for AuthService {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState2,
    ) -> Result<Self, Self::Rejection> {
        let AppState2::Prod { config, .. } = state else {
            return Ok(Self);
        };

        let err = Error::InvalidAuthUserPassword;

        let Ok(auth_service_credentials) = parts
            .extract::<TypedHeader<headers::Authorization<Basic>>>()
            .await
        else {
            return Err(err);
        };

        if (
            auth_service_credentials.username(),
            auth_service_credentials.password(),
        ) != ("auth_user", config.lock().unwrap().db_auth_user_password())
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
    #[error("invalid session ID")]
    InvalidSessionId { redirected_from: String },
    #[error("invalid user ID")]
    InvalidUserId,
    #[error("invalid auth_user password")]
    InvalidAuthUserPassword,
    #[error("no user ID found in header")]
    NoUserId,
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

        match self {
            Self::InvalidSessionId { redirected_from } => {
                Redirect::temporary(&format!("/login?redirected_from={redirected_from}"))
                    .into_response()
            }
            Self::InvalidApiKey => (
                StatusCode::UNAUTHORIZED,
                axum::Json(ErrorResponse {
                    status: StatusCode::UNAUTHORIZED.as_u16(),
                    error: Some(self),
                }),
            )
                .into_response(),
            Self::InvalidUserId | Self::NoUserId | Self::InvalidAuthUserPassword => {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
