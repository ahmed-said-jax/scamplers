use std::str::FromStr;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
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

use super::{Create, Pagination, Read, institution::Institution};
use crate::{
    api::api_key::{ApiKeyHash2, AsApiKey},
    schema::{institution, person, sql_types as custom_types},
};

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

// This represents a person as a user of the application. It should be a
// read-only view
#[derive(Selectable, Queryable, Clone)]
#[diesel(table_name = person, check_for_backend(Pg))]
pub struct User {
    pub id: Uuid,
    pub roles: Vec<UserRole>,
    pub api_key_hash: Option<serde_json::Value>, // change this to our custom ApiKey struct
}

impl User {
    pub fn test_user() -> Self {
        Self {
            id: Uuid::nil(),
            roles: UserRole::VARIANTS.to_vec(),
            api_key_hash: None,
        }
    }
}

impl User {
    pub async fn fetch_by_api_key(
        conn: &mut AsyncPgConnection,
        api_key: Uuid,
    ) -> super::Result<Self> {
        use person::dsl::api_key_hash;

        let prefix = api_key.prefix();

        let found = person::table
            .filter(api_key_hash.retrieve_as_text("prefix").eq(prefix))
            .select(Self::as_select())
            .first(conn)
            .await?;

        let Some(ref found_api_key_hash) = found.api_key_hash else {
            return Err(super::Error::RecordNotFound);
        };

        // These steps shouldn't fail, and the clone is cheap
        let found_api_key_hash: ApiKeyHash2 =
            serde_json::from_value(found_api_key_hash.clone()).unwrap();
        let found_api_key_hash = PasswordHash::new(&found_api_key_hash.hash).unwrap();

        Argon2::default()
            .verify_password(api_key.as_bytes(), &found_api_key_hash)
            .map_err(|_| super::Error::RecordNotFound)?;

        Ok(found)
    }
}

#[derive(Insertable, Validate)]
#[diesel(table_name = person, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
struct NewPerson {
    first_name: String,
    last_name: String,
    #[garde(email)]
    email: String,
    orcid: Option<String>,
    institution_id: Uuid,
}

impl Create for Vec<NewPerson> {
    type Returns = Vec<Person>;

    async fn create(&mut self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
        use person::dsl::id;

        let as_immut = &*self;

        let inserted_people_ids: Vec<Uuid> = diesel::insert_into(person::table)
            .values(as_immut)
            .returning(id)
            .get_results(conn)
            .await?;
        let n = inserted_people_ids.len() as i64;

        let filter = PersonFilter {
            ids: inserted_people_ids,
            ..Default::default()
        };
        let inserted_people = Person::fetch_many(
            Some(&filter),
            &Pagination {
                limit: n,
                ..Default::default()
            },
            conn,
        )
        .await?;

        Ok(inserted_people)
    }
}

// Do we like this struct name? Or is something like `PersonData` better
#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = person, check_for_backend(Pg))]
pub struct PersonRow {
    id: Uuid,
    #[diesel(column_name = full_name)]
    name: String,
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
    institution: Institution,
}

#[derive(Deserialize, Default)]
pub struct PersonFilter {
    ids: Vec<Uuid>,
    name: Option<String>,
    email: Option<String>,
    lab_id: Option<Uuid>,
}

impl Person {
    #[diesel::dsl::auto_type(no_type_alias)]
    fn base_query() -> _ {
        person::table.inner_join(institution::table)
    }
}

impl Read for Person {
    type Filter = PersonFilter;
    type Id = Uuid;

    async fn fetch_by_id(person_id: Self::Id, conn: &mut AsyncPgConnection) -> super::Result<Self> {
        let base_query = Self::base_query().select(Self::as_select()); // I want to factor out this whole expression into `Self::base_query`, but it doesn't work

        Ok(base_query
            .filter(person::id.eq(person_id))
            .first(conn)
            .await?)
    }

    async fn fetch_many(
        filter: Option<&Self::Filter>,
        Pagination { limit, offset }: &Pagination,
        conn: &mut AsyncPgConnection,
    ) -> super::Result<Vec<Self>> {
        use person::dsl::{email as email_col, full_name as name_col, id};

        let mut base_query = Self::base_query()
            .into_boxed()
            .select(Self::as_select())
            .limit(*limit)
            .offset(*offset);

        let Some(PersonFilter {
            ids,
            name,
            email,
            lab_id,
        }) = filter
        else {
            return Ok(base_query.load(conn).await?);
        };

        if !ids.is_empty() {
            base_query = base_query.filter(id.eq_any(ids));
        }

        // The next two conditions are pretty much the same thing, there's probably some
        // way to improve this
        if let Some(name) = name {
            base_query = base_query.filter(name_col.ilike(format!("%{name}%"))); // Notice we allow for any characters on either side of the string, so that a person can search by last name if that's all they remember
        }

        if let Some(email) = email {
            base_query = base_query.filter(email_col.ilike(format!("{email}%")));
        }

        // ignore lab_id for now
        Ok(base_query.load(conn).await?)
    }
}
