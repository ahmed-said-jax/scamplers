use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    helper_types::InnerJoin,
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types::{self},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use garde::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use valuable::Valuable;

use super::{AsDieselExpression, BoxedDieselExpression, Create, Read, Update, institution::Institution, utils::DbEnum};
use crate::{
    auth::{HashedKey, Key},
    db::utils::AsIlike,
    schema::{
        institution,
        person::{
            self,
            dsl::{email as email_col, id as id_col, name as name_col},
        },
    },
};

#[derive(
    Clone,
    FromSqlRow,
    strum::VariantArray,
    AsExpression,
    Debug,
    PartialEq,
    Deserialize,
    Serialize,
    Copy,
    Default,
    Valuable,
    strum::IntoStaticStr,
    strum::EnumString,
)]
#[diesel(sql_type = sql_types::Text)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum UserRole {
    AppAdmin,
    ComputationalStaff,
    LabStaff,
    #[default]
    Unknown,
}
impl DbEnum for UserRole {}

impl FromSql<sql_types::Text, Pg> for UserRole {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Self::from_sql_inner(bytes)
    }
}

impl ToSql<sql_types::Text, diesel::pg::Pg> for UserRole {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        self.to_sql_inner(out)
    }
}

define_sql_function! {fn grant_roles_to_user(user_id: sql_types::Text, roles: sql_types::Array<sql_types::Text>)}
define_sql_function! {fn revoke_roles_from_user(user_id: sql_types::Text, roles: sql_types::Array<sql_types::Text>)}
define_sql_function! {fn create_user_if_not_exists(user_id: sql_types::Text, roles: sql_types::Array<sql_types::Text>)}
define_sql_function! {fn get_user_roles(user_id: sql_types::Text) -> sql_types::Array<sql_types::Text>}

#[derive(Insertable, Validate, Deserialize, Valuable, Clone)]
#[diesel(table_name = person, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
pub struct NewPerson {
    #[garde(length(min = 1))]
    pub name: String,
    #[garde(email)]
    pub email: String,
    pub orcid: Option<String>,
    #[valuable(skip)]
    #[serde(default)]
    pub institution_id: Uuid,
    #[diesel(skip_insertion)]
    #[serde(default)]
    pub roles: Vec<UserRole>,
}

impl Create for &NewPerson {
    type Returns = Uuid;

    async fn create(self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
        use person::dsl::{id, institution_id as institution_id_col, ms_user_id};

        let NewPerson {
            name,
            email,
            institution_id,
            ..
        } = self;

        let inserted_id = diesel::insert_into(person::table)
            .values(self)
            .on_conflict(ms_user_id)
            .do_update()
            .set((
                name_col.eq(name),
                email_col.eq(email),
                institution_id_col.eq(institution_id),
            ))
            .returning(id)
            .get_result(conn)
            .await?;

        Ok(inserted_id)
    }
}

impl Create for Vec<NewPerson> {
    type Returns = Vec<Person>;

    async fn create(self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
        use person::dsl::id;

        // This can be improved by doing the join on the insertion rather than two
        // queries
        let inserted_people_ids: Vec<Uuid> = diesel::insert_into(person::table)
            .values(&self)
            .returning(id)
            .get_results(conn)
            .await?;

        for (role_set, new_id) in self
            .iter()
            .map(|NewPerson { roles, .. }| roles)
            .zip(&inserted_people_ids)
        {
            diesel::select(create_user_if_not_exists(new_id.to_string(), role_set))
                .execute(conn)
                .await?;
        }

        let filter = PersonQuery {
            ids: inserted_people_ids,
            ..Default::default()
        };
        let inserted_people = Person::fetch_many(&filter, conn).await?;

        Ok(inserted_people)
    }
}

#[derive(Identifiable, AsChangeset)]
#[diesel(table_name = person, check_for_backend(Pg))]
pub struct GrantApiAccess<'a> {
    pub id: Uuid,
    pub hashed_api_key: HashedKey<&'a str>,
}
impl Update for GrantApiAccess<'_> {
    type Returns = Key;

    async fn update(mut self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
        let key = Key::new();
        let hashed_api_key = key.hash();

        self.hashed_api_key = hashed_api_key;

        diesel::update(person::table).set(&self).execute(conn).await?;

        Ok(key)
    }
}

#[derive(Deserialize, Default, Valuable)]
pub struct PersonQuery {
    #[valuable(skip)]
    #[serde(default)]
    pub ids: Vec<Uuid>,
    pub name: Option<String>,
    pub email: Option<String>,
}

impl<T> AsDieselExpression<T> for PersonQuery
where
    id_col: SelectableExpression<T>,
    name_col: SelectableExpression<T>,
    email_col: SelectableExpression<T>,
{
    fn as_diesel_expression<'a>(&'a self) -> Option<BoxedDieselExpression<'a, T>>
    where
        T: 'a,
    {
        let Self { ids, name, email, .. } = self;

        if matches!((ids.is_empty(), name, email), (true, None, None)) {
            return None;
        }

        // In theory, we could initialize this with `let mut query = None;`, but that results in a lot of boilerplate
        let mut query: BoxedDieselExpression<T> = if ids.is_empty() {
            Box::new(id_col.is_not_null())
        } else {
            Box::new(id_col.eq_any(ids))
        };

        if let Some(name) = name {
            query = Box::new(query.and(name_col.ilike(name.as_ilike())));
        }

        if let Some(email) = email {
            query = Box::new(query.and(email_col.ilike(email.as_ilike())));
        }

        Some(query)
    }
}

impl Person {
    pub(super) fn base_query() -> InnerJoin<person::table, institution::table> {
        person::table.inner_join(institution::table)
    }
}

impl Read for Person {
    type Id = Uuid;
    type QueryParams = PersonQuery;

    async fn fetch_by_id(id: &Self::Id, conn: &mut AsyncPgConnection) -> super::Result<Self> {
        Ok(Self::base_query()
            .filter(id_col.eq(id))
            .select(Person::as_select())
            .first(conn)
            .await?)
    }

    async fn fetch_many(filter: &Self::QueryParams, conn: &mut AsyncPgConnection) -> super::Result<Vec<Self>> {
        let query = Self::base_query().order_by(name_col).select(Person::as_select());
        let filter = filter.as_diesel_expression();

        let people = match filter {
            Some(f) => query.filter(f).load(conn).await?,
            None => query.load(conn).await?,
        };

        Ok(people)
    }
}

#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = person, check_for_backend(Pg))]
pub struct Person {
    #[serde(flatten)]
    #[diesel(embed)]
    pub stub: PersonStub,
    email: String,
    orcid: Option<String>,
    #[diesel(embed)]
    institution: Institution,
}

#[derive(Queryable, Selectable, Serialize, Identifiable, JsonSchema)]
#[diesel(table_name = person, check_for_backend(Pg))]
pub struct PersonStub {
    pub id: Uuid,
    name: String,
    link: String,
}
