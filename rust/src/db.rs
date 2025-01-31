use std::str::FromStr;

use diesel::{pg::Pg, result::DatabaseErrorInformation, BoxableExpression, Table};
use diesel_async::{pooled_connection::deadpool, AsyncPgConnection};
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use valuable::Valuable;

pub mod institution;
pub mod person;

// the following traits are not used to enforce anything, they just help to
// provide a uniform interface for database CRUD operations

// we don't need to `impl<T> Create for Vec<T>` for generic `T` because diesel
// allows us to insert multiple records at once, so we can just `impl Create`
// for a concrete Vec<T>
pub trait Create {
    type Returns;

    // Mutability here so that the method can change what it needs
    async fn create(&mut self, conn: &mut AsyncPgConnection) -> Result<Self::Returns>;
}

pub trait Read: Sized {
    type Id = Uuid;
    type Filter = ();

    async fn fetch_all(conn: &mut AsyncPgConnection, pagination: Pagination) -> Result<Vec<Self>>;

    async fn fetch_by_id(conn: &mut AsyncPgConnection, id: Self::Id) -> Result<Self>;

    async fn fetch_by_filter(
        conn: &mut AsyncPgConnection,
        filter: Self::Filter,
        pagination: Pagination,
    ) -> Result<Vec<Self>> {
        Self::fetch_all(conn, pagination).await
    }
}

pub trait Update {
    type Returns;

    async fn update(&mut self, conn: &mut AsyncPgConnection) -> Result<Self::Returns>;
}

pub trait Upsert {
    type Returns;
    async fn upsert(&mut self, conn: &mut AsyncPgConnection) -> Result<Self::Returns>;
}

impl<T: Upsert> Upsert for Vec<T> {
    type Returns = Vec<T::Returns>;

    async fn upsert(&mut self, conn: &mut AsyncPgConnection) -> Result<Self::Returns> {
        let mut updated = Vec::with_capacity(self.len());

        for record in self {
            updated.push(record.upsert(conn).await?);
        }

        Ok(updated)
    }
}

#[derive(Deserialize)]
pub struct Pagination {
    limit: i64,
    offset: i64,
}
impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: 100,
            offset: 0,
        }
    }
}

#[derive(
    Debug,
    Serialize,
    strum::EnumString,
    Default,
    strum::Display,
    strum::VariantNames,
    strum::VariantArray,
    Valuable
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

#[derive(thiserror::Error, Debug, Serialize, Valuable)]
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
