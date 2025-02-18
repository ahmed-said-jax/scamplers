use std::{fmt::Display, str::FromStr};

use diesel::result::DatabaseErrorInformation;
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool};
use regex::Regex;
use serde::{Serialize, de::DeserializeOwned};
use uuid::Uuid;
use valuable::Valuable;

pub mod index_sets;
pub mod institution;
pub mod lab;
pub mod person;
pub mod sample;

// Avoid implementing this trait for a scalar T - just implement it for Vec<T>
// because diesel allows you to insert many things at once
pub trait Create: Send {
    type Returns: Send;

    fn create(
        &self,
        conn: &mut AsyncPgConnection,
    ) -> impl Future<Output = Result<Self::Returns>> + Send;
}

pub trait Read: Serialize + Sized + Send {
    type Id: Send + Display;
    type Filter: Sync + Send + Paginate;

    fn fetch_many(
        filter: Self::Filter,
        conn: &mut AsyncPgConnection,
    ) -> impl Future<Output = Result<Vec<Self>>> + Send;

    fn fetch_by_id(
        id: Self::Id,
        conn: &mut AsyncPgConnection,
    ) -> impl Future<Output = Result<Self>> + Send;
}

pub trait Update {
    type Returns;

    async fn update(&self, conn: &mut AsyncPgConnection) -> Result<Self::Returns>;
}

impl<T: Update> Update for Vec<T> {
    type Returns = Vec<T::Returns>;

    async fn update(&self, conn: &mut AsyncPgConnection) -> Result<Self::Returns> {
        let mut results = Vec::with_capacity(self.len());

        for item in self {
            results.push(item.update(conn).await?)
        }

        Ok(results)
    }
}

pub trait ReadRelatives<T: Read>: DeserializeOwned + Send + Display {
    fn fetch_relatives(
        &self,
        filter: T::Filter,
        conn: &mut AsyncPgConnection,
    ) -> impl Future<Output = Result<Vec<T>>> + Send;
}

pub trait Paginate {
    fn paginate(&self) -> Pagination {
        Pagination::default()
    }
}
// If we don't really need pagination (for example, institutions and people),
// you don't have to `impl` the trait for the corresponding filter
impl<T: Paginate> Paginate for Option<T> {
    fn paginate(&self) -> Pagination {
        match self {
            Some(item) => item.paginate(),
            None => Pagination::default(),
        }
    }
}

pub struct Pagination {
    limit: i64,
    offset: i64,
}
impl Default for Pagination {
    fn default() -> Self {
        Pagination {
            limit: 500,
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
    Other,
}

pub async fn set_transaction_user(user_id: &Uuid, conn: &mut AsyncPgConnection) -> Result<()> {
    diesel::sql_query(format!(r#"set local role "{user_id}""#))
        .execute(conn)
        .await?;

    Ok(())
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

        let detail_regex = Regex::new(r"Key \((.+)\)=\((.+)\).+").unwrap(); // This isn't perfect
        let details = info.details().unwrap_or_default();
        let field_value: Vec<String> = detail_regex
            .captures(details)
            .and_then(|cap| cap.iter().take(3).map(|m| m.map(|s| s.as_str().to_string())).collect()).unwrap_or_default();

        let field = field_value.get(1).cloned();
        let value = field_value.get(2).cloned();

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
