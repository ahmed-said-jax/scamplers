use diesel::{backend::Backend, deserialize::{FromSql, FromSqlRow}, expression::AsExpression, pg::Pg, serialize::ToSql, sql_types::{self, SqlType}, prelude::*};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use strum::VariantArray;
use uuid::Uuid;
use crate::schema::sql_types as custom_types;
use std::str::FromStr;

use super::{Pagination, Read};

#[derive(
    Clone,
    SqlType,
    FromSqlRow,
    strum::VariantArray,
    AsExpression,
    Debug,
    strum::IntoStaticStr,
    strum::EnumString,
    PartialEq,
    Deserialize,
    Serialize
)]
#[strum(serialize_all = "snake_case")]
#[diesel(sql_type = custom_types::UserRole)]
pub enum UserRole {
    Admin,
    ComputationalStaff,
    LabStaff,
}

impl FromSql<custom_types::UserRole, Pg> for UserRole {
    fn from_sql(
        bytes: <Pg as Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        let raw: String = FromSql::<sql_types::Text, diesel::pg::Pg>::from_sql(bytes)?;
        // this shouldn't ever fail
        Ok(Self::from_str(&raw).unwrap())
    }
}

impl ToSql<custom_types::UserRole, diesel::pg::Pg> for UserRole {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        let as_str: &str = self.into();
        ToSql::<sql_types::Text, Pg>::to_sql(as_str, out)
    }
}

// This represents a person as a user of the application
#[derive(Selectable, Queryable)]
#[diesel(table_name = crate::schema::person, check_for_backend(Pg))]
pub struct User {
    pub id: Uuid,
    pub roles: Vec<UserRole>
}

impl User {
    pub fn test_user() -> Self {
        Self {
            id: Uuid::nil(),
            roles: UserRole::VARIANTS.to_vec()
        }
    }
}

impl Read for User {
    async fn fetch_all(conn: &mut AsyncPgConnection, pagination: Pagination) -> super::Result<Vec<Self>> {
        use crate::schema::person::dsl::person;

        Ok(person.limit(pagination.limit).offset(pagination.offset).select(Self::as_select()).load(conn).await?)
    }
    async fn fetch_by_id(conn: &mut AsyncPgConnection, id: Self::Id) -> super::Result<Self> {
        use crate::schema::person::dsl::person;

        Ok(person.find(id).select(Self::as_select()).first(conn).await?)
    }
}