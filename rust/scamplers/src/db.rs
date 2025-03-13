#![allow(private_interfaces)]
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use diesel::{
    BoxableExpression,
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    result::DatabaseErrorInformation,
    serialize::ToSql,
    sql_types::{self, Bool},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool};
use regex::Regex;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;
use valuable::Valuable;

pub mod index_sets;
pub mod institution;
pub mod lab;
pub mod person;
pub mod sample;
mod dataset;
mod chemistry;
mod library_type_specification;
mod sequencing_run;

// Avoid implementing this trait for a scalar T - just implement it for Vec<T>
// because diesel allows you to insert many things at once
pub trait Create: Send {
    type Returns: Send;

    fn create(&self, conn: &mut AsyncPgConnection) -> impl Future<Output = Result<Self::Returns>> + Send;
}

type BoxedDieselExpression<'a, T> = Box<dyn BoxableExpression<T, Pg, SqlType = Bool> + 'a>;

trait AsDieselExpression<Tab = ()> {
    fn as_diesel_expression<'a>(&'a self) -> Option<BoxedDieselExpression<'a, Tab>>
    where
        Tab: 'a,
    {
        None
    }
}
// For types where we don't need a filter, we can just use `()`
impl AsDieselExpression for () {}

pub trait Read: Serialize + Sized + Send {
    type Id: Send + Display;
    type QueryParams: Sync + Send;

    fn fetch_many(
        query: Self::QueryParams,
        conn: &mut AsyncPgConnection,
    ) -> impl Future<Output = Result<Vec<Self>>> + Send;

    fn fetch_by_id(id: Self::Id, conn: &mut AsyncPgConnection) -> impl Future<Output = Result<Self>> + Send;
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
        query: T::QueryParams,
        conn: &mut AsyncPgConnection,
    ) -> impl Future<Output = Result<Vec<T>>> + Send;
}

// #[serde_as]
#[derive(Deserialize, Clone, Copy, Valuable)]
struct _Pagination {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i32,
}
fn default_limit() -> i64 {
    500
}

#[derive(Deserialize, Valuable)]
struct _Order<T: Valuable> {
    #[serde(default)]
    order_by: T,
    #[serde(default)]
    descending: bool,
}

trait DbEnum:
    FromStr + Into<&'static str> + FromSqlRow<sql_types::Text, Pg> + AsExpression<sql_types::Text> + Copy + Default
{
    fn from_sql_inner(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let raw: String = FromSql::<sql_types::Text, diesel::pg::Pg>::from_sql(bytes)?;

        Ok(Self::from_str(&raw).unwrap_or_default())
    }

    fn to_sql_inner<'b>(self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        let as_str = self.into();

        ToSql::<sql_types::Text, Pg>::to_sql(as_str, out)
    }
}

trait DbJson:
    DeserializeOwned + Serialize + Default + FromSqlRow<sql_types::Jsonb, Pg> + AsExpression<sql_types::Jsonb>
{
    fn from_sql_inner(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let data: serde_json::Value = FromSql::<sql_types::Jsonb, Pg>::from_sql(bytes)?;

        Ok(serde_json::from_value(data).unwrap_or_default())
    }

    fn to_sql_inner<'b>(&self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        let as_json = serde_json::to_value(self).unwrap();

        ToSql::<sql_types::Jsonb, Pg>::to_sql(&as_json, &mut out.reborrow())
    }
}

// I don't like this trait name
trait ILike {
    fn for_ilike(&self) -> String;
}
impl ILike for String {
    fn for_ilike(&self) -> String {
        format!("%{self}%")
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
            .and_then(|cap| cap.iter().take(3).map(|m| m.map(|s| s.as_str().to_string())).collect())
            .unwrap_or_default();

        let field = field_value.get(1).cloned();
        let value = field_value.get(2).cloned();

        match kind {
            UniqueViolation => Self::DuplicateRecord { entity, field, value },
            ForeignKeyViolation => {
                let referenced_entity = details.split_whitespace().last().unwrap_or_default().replace('"', "");
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
