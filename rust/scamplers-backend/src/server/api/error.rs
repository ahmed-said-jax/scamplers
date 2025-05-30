use axum::{
    extract::rejection::{JsonRejection, PathRejection},
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::QueryRejection;
use diesel_async::pooled_connection::deadpool;
use serde::Serialize;
use valuable::Valuable;

use crate::db;

#[derive(thiserror::Error, Serialize, Debug, Clone, Valuable)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Error {
    #[error(transparent)]
    Database(#[from] db::error::Error),
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
    _Permission { message: String },
}
impl Error {
    fn staus_code(&self) -> axum::http::StatusCode {
        use Error::*;
        use db::error::Error::{DuplicateRecord, Other, RecordNotFound, ReferenceNotFound};

        match self {
            SimpleData { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            _Permission { .. } => StatusCode::FORBIDDEN,
            Database(inner) => match inner {
                // Data(_) => StatusCode::UNPROCESSABLE_ENTITY,
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
        Self::Database(db::error::Error::from(err))
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
