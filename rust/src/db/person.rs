use std::{borrow::Borrow, str::FromStr};

use diesel::{
    backend::Backend, deserialize::{FromSql, FromSqlRow}, dsl::InnerJoinQuerySource, expression::AsExpression, pg::Pg, prelude::*, serialize::ToSql, sql_types::{self, SqlType}
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use garde::Validate;
use serde::{Deserialize, Serialize};
use strum::VariantArray;
use uuid::Uuid;

use super::{institution::Institution, Create, Pagination, Read};
use crate::{api::api_key, schema::{institution, person, sql_types as custom_types}};
use crate::api::api_key::{AsApiKey, ApiKeyHash2};
use argon2::{password_hash::rand_core::le, Argon2, PasswordHash, PasswordVerifier};

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

    async fn fetch_many(conn: &mut AsyncPgConnection, _filter: Option<&Self::Filter>, Pagination{limit, offset}: &Pagination) -> super::Result<Vec<Self>> {
        use crate::schema::person::dsl::*;

        Ok(person.limit(*limit).offset(*offset).select(Self::as_select()).load(conn).await?)
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
        
        let as_immut = &*self;

        let inserted_people_ids: Vec<Uuid> = diesel::insert_into(person).values(as_immut).returning(id).get_results(conn).await?;
        let n = inserted_people_ids.len() as i64;

        let filter = PersonFilter {ids: inserted_people_ids, ..Default::default()};
        let inserted_people = Person::fetch_many(conn, Some(&filter), &Pagination {limit: n, ..Default::default()}).await?;

        Ok(inserted_people)
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

#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = schema::person, check_for_backend(Pg))]
pub struct Person {
    #[serde(flatten)]
    #[diesel(embed)]
    person: PersonRow,
    #[diesel(embed)]
    institution: Institution
}

#[derive(Deserialize, Default)]
pub struct PersonFilter {
    ids: Vec<Uuid>,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    lab_id: Option<Uuid>
}

impl Person {
    #[diesel::dsl::auto_type(no_type_alias)]
    fn base_query() -> _ {
        person::table.inner_join(institution::table)
    }
}

impl Read for Person {
    type Id = Uuid;
    type Filter = PersonFilter;

    async fn fetch_by_id(conn: &mut AsyncPgConnection, person_id: Self::Id) -> super::Result<Self> {
        let base_query = Self::base_query().select(Self::as_select()); // I want to factor out this whole expression into `Self::base_query`, but it doesn't work

        Ok(base_query.filter(person::id.eq(person_id)).first(conn).await?)
    }

    async fn fetch_many(conn: &mut AsyncPgConnection, filter: Option<&Self::Filter>, Pagination{limit, offset}: &Pagination) -> super::Result<Vec<Self>> {
        use person::dsl::{id, first_name as fname_col, last_name as lname_col, email as email_col};

        let mut base_query = Self::base_query().into_boxed().select(Self::as_select()).limit(*limit).offset(*offset);

        let Some(PersonFilter {ids, first_name, last_name, email, lab_id}) = filter else {
            return Ok(base_query.load(conn).await?);
        };

        if !ids.is_empty() {
            base_query = base_query.filter(id.eq_any(ids));
        }

        // These next three conditions are the same - how do we make it less repetetive
        if let Some(first_name) = first_name {
            base_query = base_query.filter(fname_col.ilike(format!("{first_name}%")));
        }

        if let Some(last_name) = last_name {
            base_query = base_query.filter(lname_col.ilike(format!("{last_name}%")));
        }

        if let Some(email) = email {
            base_query = base_query.filter(email_col.ilike(format!("{email}%")));
        }

        // ignore lab_id for now
        Ok(base_query.load(conn).await?)

    }
}
