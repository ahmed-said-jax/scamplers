use diesel::result::DatabaseErrorInformation;
use diesel_async::pooled_connection::deadpool;
use regex::Regex;
use serde::Serialize;
use valuable::Valuable;

use scamplers_schema::*;

#[derive(thiserror::Error, Debug, Serialize, Valuable, Clone)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Error {
    #[error("{entity} with {} = {} already exists", field.clone().unwrap_or_default(), value.clone().unwrap_or_default())]
    DuplicateRecord {
        entity: String,
        field: Option<String>,
        value: Option<String>,
    },
    #[error("unable to create reference between {entity} and {referenced_entity} with value {} not found", value.clone().unwrap_or_default())]
    ReferenceNotFound {
        entity: String,
        referenced_entity: String,
        value: Option<String>,
    },
    #[error("record not found")]
    RecordNotFound,
    #[error("{message}")]
    Other { message: String },
}

impl Error {
    fn from_other_error(err: impl std::error::Error) -> Self {
        Self::Other {
            message: format!("{err:?}"),
        }
    }
}

// impl From<dataset::Error> for Error {
//     fn from(err: dataset::Error) -> Self {
//         Self::Data(DataError::from(err))
//     }
// }

// impl From<sample::Error> for Error {
//     fn from(err: sample::Error) -> Self {
//         Self::Data(DataError::from(err))
//     }
// }
// impl From<library_type_specification::Error> for Error {
//     fn from(err: library_type_specification::Error) -> Self {
//         Self::Data(DataError::from(err))
//     }
// }

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        use diesel::result::Error::{DatabaseError, NotFound};
        match err {
            DatabaseError(kind, info) => Self::from((kind, info)),
            NotFound => Self::RecordNotFound,
            _ => Self::from_other_error(err),
        }
    }
}

impl From<deadpool::PoolError> for Error {
    fn from(err: deadpool::PoolError) -> Self {
        Self::from_other_error(err)
    }
}

impl From<diesel::ConnectionError> for Error {
    fn from(err: diesel::ConnectionError) -> Self {
        Self::from_other_error(err)
    }
}

impl
    From<(
        diesel::result::DatabaseErrorKind,
        Box<dyn DatabaseErrorInformation + Send + Sync>,
    )> for Error
{
    fn from(
        (kind, info): (
            diesel::result::DatabaseErrorKind,
            Box<dyn DatabaseErrorInformation + Send + Sync>,
        ),
    ) -> Self {
        use diesel::result::DatabaseErrorKind::{ForeignKeyViolation, UniqueViolation};
        let entity = info.table_name().unwrap_or_default();

        let detail_regex = Regex::new(r"Key \((.+)\)=\((.+)\).+").unwrap(); // This isn't perfect
        let details = info.details().unwrap_or_default();
        let field_value: Vec<String> = detail_regex
            .captures(details)
            .and_then(|cap| cap.iter().take(3).map(|m| m.map(|s| s.as_str().to_string())).collect())
            .unwrap_or_default();

        let field = field_value.get(1).cloned();
        let value = field_value.get(2).cloned();

        match kind {
            UniqueViolation => Self::DuplicateRecord { entity: entity.to_string(), field, value },
            ForeignKeyViolation => {
                let referenced_entity = details.split_whitespace().last().unwrap_or_default().replace('"', "");
                let referenced_entity = referenced_entity.strip_suffix(".").unwrap_or_default();

                Self::ReferenceNotFound {
                    entity: entity.to_string(),
                    referenced_entity: referenced_entity.to_string(),
                    value,
                }
            }
            _ => Self::from_other_error(diesel::result::Error::DatabaseError(kind, info)),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
