use std::fmt::Debug;
use std::str::FromStr;

use diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    serialize::ToSql,
    sql_types,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use valuable::Valuable;

#[derive(Deserialize, Valuable)]
#[valuable(transparent)]
pub(super) struct QueryLimit(i64);
impl Default for QueryLimit {
    fn default() -> Self {
        const DEFAULT_QUERY_LIMIT: i64 = 500;
        Self(DEFAULT_QUERY_LIMIT)
    }
}
impl From<QueryLimit> for i64 {
    fn from(value: QueryLimit) -> Self {
        value.0
    }
}
impl From<&QueryLimit> for i64 {
    fn from(value: &QueryLimit) -> Self {
        value.0
    }
}
impl From<i64> for QueryLimit {
    fn from(value: i64) -> Self {
        Self(value)
    }
}
impl From<i32> for QueryLimit {
    fn from(value: i32) -> Self {
        Self(value as i64)
    }
}
impl From<usize> for QueryLimit {
    fn from(value: usize) -> Self {
        Self(value as i64)
    }
}

pub(super) trait AsIlike {
    fn as_ilike(&self) -> String;
}

impl AsIlike for &str {
    fn as_ilike(&self) -> String {
        format!("%{self}%")
    }
}

impl AsIlike for String {
    fn as_ilike(&self) -> String {
        self.as_str().as_ilike()
    }
}
