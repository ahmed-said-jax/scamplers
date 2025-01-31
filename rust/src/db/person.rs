use std::str::FromStr;

use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types::{self, SqlType},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use garde::Validate;
use serde::{Deserialize, Serialize};
use strum::VariantArray;
use uuid::Uuid;

use super::{institution::Institution, Create, Pagination, Read};
use crate::schema::{self, cdna::BoxedQuery, sql_types as custom_types};
use crate::api::api_key::{AsApiKey, ApiKeyHash2};
use argon2::{Argon2, PasswordHash, PasswordVerifier};

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
    Serialize,
)]
#[strum(serialize_all = "snake_case")]
#[diesel(sql_type = custom_types::UserRole)]
pub enum UserRole {
    Admin,
    ComputationalStaff,
    LabStaff,
}

impl FromSql<custom_types::UserRole, Pg> for UserRole {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let raw: String = FromSql::<sql_types::Text, diesel::pg::Pg>::from_sql(bytes)?;
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

// This represents a person as a user of the application. It should be a read-only view
#[derive(Selectable, Queryable, Clone)]
#[diesel(table_name = crate::schema::person, check_for_backend(Pg))]
pub struct User {
    pub id: Uuid,
    pub roles: Vec<UserRole>,
    pub api_key_hash: Option<serde_json::Value> // change this to our custom ApiKey struct
}

impl User {
    pub fn test_user() -> Self {
        Self {
            id: Uuid::nil(),
            roles: UserRole::VARIANTS.to_vec(),
            api_key_hash: None
        }
    }
}

impl Read for User {
    type Id = Uuid;
    type Filter = ();

    async fn fetch_all(
        conn: &mut AsyncPgConnection,
        pagination: Pagination,
    ) -> super::Result<Vec<Self>> {
        use crate::schema::person::dsl::person;

        Ok(person
            .limit(pagination.limit)
            .offset(pagination.offset)
            .select(Self::as_select())
            .load(conn)
            .await?)
    }

    async fn fetch_by_id(conn: &mut AsyncPgConnection, id: Self::Id) -> super::Result<Self> {
        use crate::schema::person::dsl::person;

        Ok(person
            .find(id)
            .select(Self::as_select())
            .first(conn)
            .await?)
    }
}

impl User {
    pub async fn fetch_by_api_key(conn: &mut AsyncPgConnection, api_key: Uuid, ) -> super::Result<Self> {
        use crate::schema::person::dsl::*;

        let prefix = api_key.prefix();

        let found = person.filter(api_key_hash.retrieve_as_text("prefix").eq(prefix)).select(Self::as_select()).first(conn).await?;

        let Some(ref found_api_key_hash) = found.api_key_hash else {
            return Err(super::Error::RecordNotFound);
        };

        // These steps shouldn't fail, and the clone is cheap
        let found_api_key_hash: ApiKeyHash2 = serde_json::from_value(found_api_key_hash.clone()).unwrap();
        let found_api_key_hash = PasswordHash::new(&found_api_key_hash.hash).unwrap();

        Argon2::default().verify_password(api_key.as_bytes(), &found_api_key_hash).map_err(|_| super::Error::RecordNotFound)?;

        Ok(found)
    }
}

#[derive(Insertable, Validate)]
#[diesel(table_name = crate::schema::person, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
struct NewPerson {
    first_name: String,
    last_name: String,
    #[garde(email)]
    email: String,
    orcid: Option<String>,
    institution_id: Uuid
}

impl Create for Vec<NewPerson> {
    type Returns = Vec<Person>;

    async fn create(&mut self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
        use crate::schema::person::dsl::*;
        use crate::schema::institution;
        
        let as_immut = &*self;

        let inserted_ids: Vec<Uuid> = diesel::insert_into(person).values(as_immut).returning(id).get_results(conn).await?;

        let people: Vec<(PersonRow, Institution)> = person.inner_join(institution::table).select((PersonRow::as_select(), Institution::as_select())).filter(id.eq_any(inserted_ids)).load(conn).await?;

        // Does this incur copying?
        let people = people.into_iter().map(|(inner, institution)| Person {inner, institution}).collect();

        Ok(people)
    }
}

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::person, check_for_backend(Pg))]
struct PersonRow {
    id: Uuid,
    first_name: String,
    last_name: String,
    email: String,
    orcid: Option<String>,
}

#[derive(Serialize)]
pub struct Person {
    #[serde(flatten)]
    inner: PersonRow,
    institution: Institution
}

impl Read for Person {
    type Id = Uuid;
    type Filter = ();

    // how do we reus this
    async fn fetch_all(conn: &mut AsyncPgConnection, pagination: Pagination) -> super::Result<Vec<Self>> {
        use crate::schema::{person::dsl::*, institution};

        let people = person.inner_join(institution::table).select((PersonRow::as_select(), Institution::as_select())).load(conn).await?;

        Ok(people.into_iter().map(|(inner, institution)| Person {inner, institution}).collect())
    }


}