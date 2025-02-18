use std::str::FromStr;

use diesel::{
    backend::Backend, deserialize::{FromSql, FromSqlRow}, expression::AsExpression, helper_types::{AsSelect, InnerJoin, Select, IntoBoxed}, pg::Pg, prelude::*, query_builder::BoxedSelectStatement, serialize::ToSql, sql_types::{self, Bool, SqlType}
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use valuable::Valuable;

use super::{Create, Paginate, Read, institution::Institution};
use crate::{
    db::Pagination,
    schema::{institution, person},
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
#[diesel(sql_type = sql_types::Text)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    AppAdmin,
    ComputationalStaff,
    LabStaff,
}

impl FromSql<sql_types::Text, Pg> for UserRole {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let raw: String = FromSql::<sql_types::Text, diesel::pg::Pg>::from_sql(bytes)?;
        Ok(Self::from_str(&raw).unwrap())
    }
}

impl ToSql<sql_types::Text, diesel::pg::Pg> for UserRole {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        let as_str: &str = self.into();
        ToSql::<sql_types::Text, Pg>::to_sql(as_str, out)
    }
}

define_sql_function! {fn grant_roles_to_user(user_id: sql_types::Uuid, roles: sql_types::Array<sql_types::Text>)}
define_sql_function! {fn revoke_roles_from_user(user_id: sql_types::Uuid, roles: sql_types::Array<sql_types::Text>)}
define_sql_function! {fn create_user_if_not_exists(user_id: sql_types::Uuid)}
define_sql_function! {#[aggregate] fn get_user_roles(user_id: sql_types::Uuid) -> sql_types::Array<sql_types::Text>}

// We don't export this to TypeScript because people will be created using
// Microsoft authentication rather than in the frontend
#[derive(Insertable, Validate, Deserialize, Valuable)]
#[diesel(table_name = person, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
pub struct NewPerson {
    #[garde(length(min = 1))]
    first_name: String,
    #[garde(length(min = 1))]
    last_name: String,
    #[garde(email)]
    email: String,
    orcid: Option<String>, // No need to validate this because the only way to insert a person is if you are an admin or inserting yourself, in which case this field won't be available until you link your orcid
    #[valuable(skip)]
    institution_id: Uuid,
}

impl Create for Vec<NewPerson> {
    type Returns = Vec<Person>;

    async fn create(&self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
        use person::dsl::id;

        // This can be improved by doing the join on the insertion rather than two
        // queries
        let inserted_people_ids: Vec<Uuid> = diesel::insert_into(person::table)
            .values(self)
            .returning(id)
            .get_results(conn)
            .await?;

        let filter = PersonFilter {
            ids: inserted_people_ids,
            ..Default::default()
        };
        let inserted_people = Person::fetch_many(filter, conn).await?;

        Ok(inserted_people)
    }
}

#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = person, check_for_backend(Pg))]
pub struct Person {
    id: Uuid,
    #[diesel(column_name = full_name)]
    name: String,
    email: String,
    orcid: Option<String>,
    link: String,
    #[diesel(embed)]
    institution: Institution,
}

#[derive(Deserialize, Default, Valuable)]
pub struct PersonFilter {
    #[valuable(skip)]
    #[serde(default)]
    pub ids: Vec<Uuid>,
    pub name: Option<String>,
    pub email: Option<String>,
}
impl Paginate for PersonFilter {}
impl PersonFilter {
    pub fn as_query(&self) -> IntoBoxed<Select<InnerJoin<person::table, institution::table>, AsSelect<Person, Pg>>, Pg> {
        use person::dsl::{email as email_col, full_name as name_col, id as id_col};

        let mut query = person::table.into_boxed();

        let Self { ids, name, email } = self;

        if !ids.is_empty() {
            query = query.filter(id_col.eq_any(ids));
        }

        // The next two conditions are pretty much the same thing, there's probably some
        // way to improve this
        if let Some(name) = name {
            query = query.filter(name_col.ilike(format!("%{name}%"))); // This allows searching by first name or last name (or any substring within each)
        }

        if let Some(email) = email {
            query = query.filter(email_col.ilike(format!("{email}%")));
        }

        query.inner_join(institution::table).select(Person::as_select())
    }
}

impl Read for Person {
    type Filter = PersonFilter;
    type Id = Uuid;

    async fn fetch_by_id(id: Self::Id, conn: &mut AsyncPgConnection) -> super::Result<Self> {
        let filter = PersonFilter{ids: vec![id], ..Default::default()};

        let query = filter.as_query();

        Ok(query.first(conn).await?)
    }

    async fn fetch_many(
        filter: Self::Filter,
        conn: &mut AsyncPgConnection,
    ) -> super::Result<Vec<Self>> {
        Ok(filter.as_query().load(conn).await?)
    }
}

// Lol ChatGPT suggested `PersonSummary`, `PersonBasic`, `PersonInfo`, and
// `PersonLite`, and I liked the last one the most
#[derive(Queryable, Selectable, Serialize, Identifiable)]
#[diesel(table_name = person, check_for_backend(Pg))]
pub struct PersonLite {
    id: Uuid,
    #[diesel(column_name = full_name)]
    name: String,
    link: String,
}
