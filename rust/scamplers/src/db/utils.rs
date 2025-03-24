use std::str::FromStr;

use chrono::{NaiveDateTime, Utc};
use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    serialize::ToSql,
    sql_types::{self, SqlType},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;

const DEFAULT_QUERY_LIMIT: i64 = 500;

pub fn default_query_limit() -> i64 {
    DEFAULT_QUERY_LIMIT
}

pub trait BelongsToExt<Parent> {
    fn set_parent_id(&mut self, parent_id: Uuid);
}

pub trait Parent<Child> {
    fn owned_children(&mut self) -> Vec<Child>;
}

pub trait ParentSet<'a, P, Child>
where
    P: Parent<Child>,
{
    fn flatten_children_and_set_ids(&'a mut self, parent_ids: &[Uuid], n_children: usize) -> Vec<Child>;
}

impl<'a, P, Child> ParentSet<'a, P, Child> for Vec<P>
where
    P: Parent<Child> + Sized,
    Child: BelongsToExt<P>,
{
    fn flatten_children_and_set_ids(&'a mut self, ids: &[Uuid], n_children: usize) -> Vec<Child> {
        let mut flattened_children = Vec::with_capacity(n_children);
        let nested_children = self.iter_mut().map(|p| p.owned_children());

        for (children, parent_id) in nested_children.zip(ids) {
            for mut child in children.into_iter() {
                child.set_parent_id(*parent_id);
                flattened_children.push(child);
            }
        }

        flattened_children
    }
}

pub trait JunctionStruct<T: Clone = Uuid, U = Uuid>: Sized {
    fn new(parent1: T, parent2_id: U) -> Self;
    fn from_ids_grouped_by_parent1<I1, I2, I3>(parent1: I1, parent2_groups: I2, n_relationships: usize) -> Vec<Self>
    where
        I1: IntoIterator<Item = T>,
        I2: IntoIterator<Item = I3>,
        I3: IntoIterator<Item = U>,
    {
        let mut junction_structs = Vec::with_capacity(n_relationships);

        for (parent_id, children) in parent1.into_iter().zip(parent2_groups) {
            for child_id in children {
                junction_structs.push(Self::new(parent_id.clone(), child_id))
            }
        }

        junction_structs
    }
}

pub trait DbEnum: FromSqlRow<sql_types::Text, Pg> + AsExpression<sql_types::Text> + Default + FromStr
where
    &'static str: From<Self>,
{
    fn from_sql_inner(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let raw: String = FromSql::<sql_types::Text, diesel::pg::Pg>::from_sql(bytes)?;

        Ok(Self::from_str(&raw).unwrap_or_default())
    }

    fn to_sql_inner<'b>(self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        let as_str: &str = self.into();

        ToSql::<sql_types::Text, Pg>::to_sql(&as_str, &mut out.reborrow())
    }
}

pub trait DbJson:
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

pub trait AsIlike {
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

#[derive(Deserialize, Serialize, Debug, AsExpression, SqlType, JsonSchema, Clone)]
#[diesel(sql_type = sql_types::Timestamp)]
#[serde(transparent)]
pub struct DefaultNowNaiveDateTime(NaiveDateTime);
impl From<NaiveDateTime> for DefaultNowNaiveDateTime {
    fn from(value: NaiveDateTime) -> Self {
        Self(value)
    }
}
impl Default for DefaultNowNaiveDateTime {
    fn default() -> Self {
        Self(Utc::now().naive_utc())
    }
}
impl ToSql<sql_types::Timestamp, Pg> for DefaultNowNaiveDateTime {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        let Self(inner) = self;

        ToSql::<sql_types::Timestamp, Pg>::to_sql(inner, &mut out.reborrow())
    }
}
impl FromSql<sql_types::Timestamp, Pg> for DefaultNowNaiveDateTime {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Ok(Self(FromSql::<sql_types::Timestamp, Pg>::from_sql(bytes)?))
    }
}
