use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    serialize::ToSql,
    sql_types,
};
use serde::{Serialize, de::DeserializeOwned};
use std::str::FromStr;
use uuid::Uuid;

const DEFAULT_QUERY_LIMIT: i64 = 500;

pub fn default_query_limit() -> i64 {
    DEFAULT_QUERY_LIMIT
}

// Type parametrization here so we can use the same trait multiple times for the same struct, since tables are
// frequently children of more than one table
pub trait Child<T: diesel::Table> {
    fn set_parent_id(&mut self, parent_id: Uuid);
}

pub trait Children<T, U>
where
    T: Child<U>,
    U: diesel::Table,
{
    fn set_parent_ids(&mut self, parent_ids: &[Uuid]);
}

impl<T, U> Children<T, U> for Vec<T>
where
    T: Child<U>,
    U: diesel::Table,
{
    fn set_parent_ids(&mut self, parent_ids: &[Uuid]) {
        for (item, parent_id) in self.iter_mut().zip(parent_ids) {
            item.set_parent_id(*parent_id);
        }
    }
}

pub trait ChildrenSets<T, U, V>: IntoIterator<Item = T> + Sized
where
    T: IntoIterator<Item = U>,
    U: Child<V>,
    V: diesel::Table,
{
    fn flatten_and_set_parent_ids(self, parent_ids: &[Uuid], n_children: usize) -> Vec<U> {
        let mut flattened_children = Vec::with_capacity(n_children);

        for (children, parent_id) in self.into_iter().zip(parent_ids) {
            for mut child in children {
                child.set_parent_id(*parent_id);
                flattened_children.push(child);
            }
        }

        flattened_children
    }
}
impl<T, U, V> ChildrenSets<T, U, V> for Vec<T>
where
    T: IntoIterator<Item = U>,
    U: Child<V>,
    V: diesel::Table,
{
}

pub trait MappingStruct: Sized {
    fn new(id1: Uuid, id2: Uuid) -> Self;
    fn from_grouped_ids<I1, I2, I3, U>(parents: I1, children_sets: I2, n_relationships: usize) -> Vec<Self>
    where
        I1: IntoIterator<Item = U>,
        I2: IntoIterator<Item = I3>,
        I3: IntoIterator<Item = U>,
        U: AsRef<Uuid>,
    {
        let mut mapping_structs = Vec::with_capacity(n_relationships);

        for (parent_id, children) in parents.into_iter().zip(children_sets) {
            for child_id in children {
                mapping_structs.push(Self::new(*parent_id.as_ref(), *child_id.as_ref()))
            }
        }

        mapping_structs
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
