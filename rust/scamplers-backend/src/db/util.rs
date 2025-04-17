use std::str::FromStr;
use std::fmt::Debug;

use diesel::{deserialize::{FromSql, FromSqlRow}, expression::AsExpression, pg::Pg, query_dsl::methods::FilterDsl, serialize::ToSql, sql_types, BoxableExpression, QueryDsl};
use diesel_async::AsyncPgConnection;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use valuable::Valuable;

#[derive(Clone, FromSqlRow, AsExpression, Deserialize, Serialize, Debug, Copy)]
#[diesel(sql_type = sql_types::Text)]
pub (super) struct DbEnum<T: FromStr + Default + Debug + Clone + Copy>(T) where &'static str: From<T>;

impl<T: FromStr + Default + Debug + Clone + Copy> FromSql<sql_types::Text, Pg> for DbEnum<T> where &'static str: From<T> {
    fn from_sql(bytes: <Pg as diesel::backend::Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let string: String = FromSql::<sql_types::Text, Pg>::from_sql(bytes)?;

        Ok(Self(T::from_str(&string).unwrap_or_default()))
    }
}

impl<T: FromStr + Clone + Debug + Default + Copy> ToSql<sql_types::Text, Pg> for DbEnum<T> where &'static str: From<T> {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        let Self(inner) = *self;
        let as_str: &str = inner.into();

        ToSql::<sql_types::Text, Pg>::to_sql(&as_str, &mut out.reborrow())
    }
}

#[derive(FromSqlRow, AsExpression, Deserialize, Serialize, Debug, Default)]
#[diesel(sql_type = sql_types::Text)]
pub (super) struct DbJson<T: Default + Debug>(T);

impl<T: Default + Debug + DeserializeOwned> FromSql<sql_types::Jsonb, Pg> for DbJson<T> {
    fn from_sql(bytes: <Pg as diesel::backend::Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let json: serde_json::Value = FromSql::<sql_types::Jsonb, Pg>::from_sql(bytes)?;

        Ok(serde_json::from_value(json).unwrap_or_default())
    }
}

impl<T: Debug + Default + Serialize> ToSql<sql_types::Jsonb, Pg> for DbJson<T> {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        let as_json = serde_json::to_value(self).unwrap();

        ToSql::<sql_types::Jsonb, Pg>::to_sql(&as_json, &mut out.reborrow())
    }
}

#[derive(Deserialize, Valuable)]
#[valuable(transparent)]
pub (super) struct QueryLimit(i64);
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

pub (super) trait AsIlike {
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
