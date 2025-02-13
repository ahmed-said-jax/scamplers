use std::{fmt::Display, str::FromStr};

use argon2::password_hash::{SaltString, PasswordHasher};
use diesel::result::DatabaseErrorInformation;
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool};
use regex::Regex;
use serde::{Serialize, de::DeserializeOwned};
use uuid::Uuid;
use valuable::Valuable;

pub mod index_sets;
pub mod person;
pub mod institution;


#[derive(
    Debug,
    Serialize,
    strum::EnumString,
    Default,
    strum::Display,
    strum::VariantNames,
    strum::VariantArray,
    Valuable,
    Clone,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Entity {
    Institution,
    Person,
    Lab,
    Sample,
    Library,
    SequencingRun,
    Dataset,
    #[default]
    Unknown,
}

#[derive(thiserror::Error, Debug, Serialize, Valuable, Clone)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Error {
    #[error("duplicate record")]
    DuplicateRecord {
        entity: Entity,
        field: Option<String>,
        value: Option<String>,
    },
    #[error("referenced record not found")]
    ReferenceNotFound {
        entity: Entity,
        referenced_entity: Entity,
        value: Option<String>,
    },
    #[error("record not found")]
    RecordNotFound,
    #[error("other error")]
    Other { message: String },
}
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    fn from_other_error(err: impl std::error::Error) -> Self {
        Self::Other {
            message: format!("{err:?}"),
        }
    }
}

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

        let entity = Entity::from_str(info.table_name().unwrap_or_default()).unwrap_or_default();

        let field = info.column_name().map(str::to_string);

        let detail_regex = Regex::new(r"Key \(.*\)=(\(.*\)).*").unwrap();
        let details = info.details().unwrap_or_default();
        let value = detail_regex
            .captures(details)
            .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()));

        match kind {
            UniqueViolation => Self::DuplicateRecord {
                entity,
                field,
                value,
            },
            ForeignKeyViolation => {
                let referenced_entity = details
                    .split_whitespace()
                    .last()
                    .unwrap_or_default()
                    .replace('"', "");
                let referenced_entity = referenced_entity.strip_suffix(".").unwrap_or_default();
                let referenced_entity = Entity::from_str(referenced_entity).unwrap_or_default();

                Self::ReferenceNotFound {
                    entity,
                    referenced_entity,
                    value,
                }
            }
            _ => Self::from_other_error(diesel::result::Error::DatabaseError(kind, info)),
        }
    }
}
