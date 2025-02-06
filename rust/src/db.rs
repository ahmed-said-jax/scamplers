use std::str::FromStr;

use diesel::result::DatabaseErrorInformation;
use diesel_async::{pooled_connection::deadpool, AsyncPgConnection, RunQueryDsl};
use regex::Regex;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;
use valuable::Valuable;

pub mod index_sets;
pub mod institution;
pub mod person;

// struct AsyncDbConnection(AsyncPgConnection);
// impl AsyncDbConnection {
//     async fn execute_as_user<F: AsyncFnMut(&mut AsyncPgConnection) -> Result<R>, R: Send>(mut self, user_id: &Uuid, f: F) -> Result<R> {
//         let result = self.0.transaction(|conn| async move {
//             diesel::sql_query(format!(r#"set local role "{user_id}""#)).execute(conn).await?;
//             f(conn).await
//         }.scope_boxed()).await?;

//         Ok(result)
//     }
// }

// Do not implement this trait for a scalar T - just implement it for Vec<T> because diesel allows you to insert many things at once
pub trait Create: Send {
    type Returns: Send;

    fn create(
        &self,
        conn: &mut AsyncPgConnection,
    ) -> impl Future<Output = Result<Self::Returns>> + Send;
}

pub trait Read: Serialize + Sized + Send {
    type Id: Send;
    type Filter: Sync + Send;

    fn fetch_many(
        filter: Option<&Self::Filter>,
        pagination: &Pagination,
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

pub trait ReadRelatives<T: Read>: DeserializeOwned {
    async fn fetch_relatives(
        &self,
        filter: Option<&T::Filter>,
        pagination: &Pagination,
        conn: &mut AsyncPgConnection,
    ) -> Result<Vec<T>>;
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
    Valuable,
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

pub async fn set_transaction_user(user_id: &Uuid, conn: &mut AsyncPgConnection) -> Result<()> {
    diesel::sql_query(format!(r#"set local role "{user_id}""#))
        .execute(conn)
        .await?;

    Ok(())
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
