use std::str::FromStr;

use axum::{extract::{FromRequestParts}, response::IntoResponse, Router};
use diesel::{
    backend::Backend, deserialize::{FromSql, FromSqlRow}, expression::AsExpression, pg::Pg, prelude::*, serialize::ToSql, sql_types::{self, SqlType}
};
use diesel_async::{pooled_connection::deadpool, RunQueryDsl};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use strum::VariantArray;
use uuid::Uuid;

use crate::{db::{self, person::UserRole, Entity}, schema::sql_types as custom_types, AppState};
mod v0;

pub fn router() -> Router<AppState> {
    // In theory, we should be able to inspect the header and route the request
    // based on the API version set in the header, but I don't know how to do that
    // yet
    v0::router()
}

#[derive(Serialize)]
#[serde(untagged)]
enum ApiResponse {
    Institution(db::institution::Institution),
    Institutions(Vec<db::institution::Institution>),
}
impl From<db::institution::Institution> for ApiResponse {
    fn from(inst: db::institution::Institution) -> Self {
        Self::Institution(inst)
    }
}
impl From<Vec<db::institution::Institution>> for ApiResponse {
    fn from(insts: Vec<db::institution::Institution>) -> Self {
        Self::Institutions(insts)
    }
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
    }
}

struct ApiUser(db::person::User);

impl FromRequestParts<AppState> for ApiUser {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> std::result::Result<Self, Self::Rejection> {
        use Error::{ApiKeyNotFound, InvalidApiKey};

        use crate::schema::person::dsl::{api_key, id, person, roles};

        if !state.production {
            return Ok(Self(db::person::User::test_user()));
        }

        let raw_api_key = parts
            .headers
            .get("x-api-key")
            .ok_or(ApiKeyNotFound)?
            .as_bytes();
        let extracted_api_key = Uuid::from_slice(raw_api_key).map_err(|_| InvalidApiKey)?;

        let mut conn = state.db_pool.get().await?;

        let user = person
            .filter(api_key.eq(extracted_api_key))
            .select((id, roles))
            .get_result(&mut conn)
            .await
            .map_err(db::Error::from)?;

        Ok(Self(user))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
enum Version {
    V0
}

// The following code is commented-out because it needs a bit more thought

// #[derive(Serialize, Deserialize, Debug, FromSqlRow, AsExpression, Clone)]
// #[diesel(sql_type = sql_types::Jsonb)]
// pub struct EntityLink {
//     version: Version,
//     link: String
// }

// impl EntityLink {
//     pub fn new(api_version: Version, entity: db::Entity, id: Uuid) -> Self {
//         let pattern = Regex::new(r#"\{.*\}"#).unwrap();
        
//         let link_template = match api_version {
//             Version::V0 => entity.v0_endpoint()
//         };

//         let link = pattern.replace(link_template, id.to_string()).to_string();

//         Self {
//             version: api_version,
//             link
//         }
//     }
// }

// impl FromSql<sql_types::Jsonb, Pg> for EntityLink {
//     fn from_sql(
//         bytes: <Pg as Backend>::RawValue<'_>,
//     ) -> diesel::deserialize::Result<Self> {
//         let raw = <serde_json::Value as FromSql<sql_types::Jsonb, Pg>>::from_sql(bytes)?;
//         Ok(serde_json::from_value(raw)?)
//     }
// }

// impl ToSql<sql_types::Jsonb, Pg> for EntityLink {
//     fn to_sql<'b>(
//         &'b self,
//         out: &mut diesel::serialize::Output<'b, '_, Pg>,
//     ) -> diesel::serialize::Result {
//         let value = serde_json::to_value(self)?;
//         ToSql::<sql_types::Jsonb, Pg>::to_sql(&value, &mut out.reborrow())
//     }
// }

#[derive(thiserror::Error, Serialize, Debug)]
#[serde(rename_all = "snake_case", tag = "type")]
enum Error {
    #[error("API key not found in request headers")]
    ApiKeyNotFound,
    #[error("invalid API key")]
    InvalidApiKey,
    #[error("operation not permitted")]
    Permission { message: String },
    #[error(transparent)]
    Database(#[from] db::Error),
}
impl Error {
    fn staus_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;
        use db::Error::{DuplicateRecord, Other, RecordNotFound, ReferenceNotFound};
        use Error::{ApiKeyNotFound, Database, InvalidApiKey, Permission};

        match self {
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
        let mut err = Self::Permission { message: String::new() };

        // the `Display` implementation of this error is what we want the serialized error to show in most cases, so just use that
        let intended_message = err.to_string();

        // set the error
        match err {
            Self::Permission { ref mut message } => {message.push_str(&intended_message);},
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

type Result<T> = std::result::Result<T, Error>;