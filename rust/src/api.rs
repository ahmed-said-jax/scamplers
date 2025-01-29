use axum::{extract::FromRequestParts, response::IntoResponse, Router};
use diesel::{deserialize::FromSqlRow, sql_types::SqlType, Identifiable, Queryable, Selectable};
use serde::Serialize;
use strum::VariantArray;
use uuid::Uuid;

use crate::{db, AppState};
mod v0;

// This could easily go in `db::person`, but it's used for API permissions so it makes sense here too
#[derive(Clone, strum::Display, SqlType, FromSqlRow, strum::VariantArray)]
#[strum(serialize_all = "snake_case")]
#[diesel(sql_type = crate::schema::sql_types::UserRole)]
enum Role {
    Admin,
    ComputationalStaff,
    LabStaff
}

#[derive(Selectable, Queryable, Identifiable)]
#[diesel(table_name = crate::schema::person, check_for_backend(diesel::pg::Pg))]
struct ApiUser{
    id: Uuid,
    roles: Vec<Role>
}

impl FromRequestParts<AppState> for ApiUser {
    type Rejection = Error;

    async  fn from_request_parts(parts: &mut axum::http::request::Parts,state: &AppState,) -> Result<Self,Self::Rejection> {
        use Error::*;
        use crate::schema::person::dsl::*;

        if !state.production {
            return Ok(Self {id: Uuid::nil(), roles: Role::VARIANTS.to_vec()})
        }

        // I hate this chain of `ok_or` and `map_err`s
        let maybe_api_key: Uuid = parts.headers.get("x-api-key").ok_or(ApiKeyNotFound)?.to_str().map_err(|_| InvalidApiKey)?.parse().map_err(|_| InvalidApiKey)?;

        let query = diesel::select(person).filter(api_key.eq(maybe_api_key));


    }

}

pub fn router() -> Router<AppState> {
    // In theory, we should be able to inspect the header and route the request
    // based on the API version set in the header, but I don't know how to do that
    // yet
    v0::router()
}

pub fn route(entity: db::Entity) -> &'static str {
    use db::Entity::*;

    match entity {
        Institution => "/institutions/{institution_id}",
        Person => "/people/{person_id}",
        Lab => "/labs/{lab_id}",
        Sample => "/samples/{sample_id}",
        Library => "/libraries/{library_id}",
        SequencingRun => "/sequencing_runs/{sequencing_run_id}",
        Dataset => "/datasets/{dataset_id}",
        Unknown => "/",
    }
}

#[derive(thiserror::Error, Serialize, Debug)]
#[serde(rename_all = "snake_case", tag = "type")]
enum Error {
    #[error("API key not found in request headers")]
    ApiKeyNotFound,
    #[error("invalid API key")]
    InvalidApiKey,
    #[error("operation not permitted")]
    Permission{message: String},
    #[error(transparent)]
    Database(#[from] db::Error)
}
impl Error {
    fn staus_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;
        use Error::*;
        use db::Error::*;

        match self {
            ApiKeyNotFound | InvalidApiKey => StatusCode::UNAUTHORIZED,
            Permission { .. } => StatusCode::FORBIDDEN,
            Database(inner) => match inner {
                Other { .. } => StatusCode::INTERNAL_SERVER_ERROR,
                DuplicateRecord { .. } => StatusCode::CONFLICT,
                RecordNotFound => StatusCode::NOT_FOUND,
                ReferenceNotFound { ..} => StatusCode::UNPROCESSABLE_ENTITY,
            }
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;

        #[derive(Serialize)]
        struct ErrorResponse {
            status: u16,
            error: Option<Error>
        }

        let status = self.staus_code();

        if status == StatusCode::INTERNAL_SERVER_ERROR {
            (status, axum::Json(ErrorResponse {status: status.as_u16(), error: None})).into_response()
        }
        else {
            (status, axum::Json(ErrorResponse {status: status.as_u16(), error: Some(self)})).into_response()
        }
    }
}