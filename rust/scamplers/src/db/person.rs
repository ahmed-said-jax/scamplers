use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    helper_types::{InnerJoin, Select},
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types::{self, Bool},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use valuable::Valuable;

use super::{AsDieselExpression, Create, DbEnum, FilterExpression, Read, institution::Institution};
use crate::schema::{institution, person};

#[derive(
    Clone,
    FromSqlRow,
    strum::VariantArray,
    AsExpression,
    Debug,
    strum::IntoStaticStr,
    strum::EnumString,
    PartialEq,
    Deserialize,
    Serialize,
    Copy,
    Default,
)]
#[strum(serialize_all = "snake_case")]
#[diesel(sql_type = sql_types::Text)]
#[serde(rename_all = "snake_case")]
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

define_sql_function! {fn grant_roles_to_user(user_id: sql_types::Uuid, roles: sql_types::Array<sql_types::Text>)}
define_sql_function! {fn revoke_roles_from_user(user_id: sql_types::Uuid, roles: sql_types::Array<sql_types::Text>)}
define_sql_function! {fn create_user_if_not_exists(user_id: sql_types::Uuid)}
define_sql_function! {#[aggregate] fn get_user_roles(user_id: sql_types::Uuid) -> sql_types::Array<sql_types::Text>}

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
    orcid: Option<String>,
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

        let filter = PersonQuery {
            ids: inserted_people_ids,
            ..Default::default()
        };
        let inserted_people = Person::fetch_many(filter, conn).await?;

        Ok(inserted_people)
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

impl AsDieselExpression for PersonQuery {
    fn as_diesel_expression<'a, T>(&'a self) -> Option<FilterExpression<'a, T>> {
        use person::dsl::{email as email_col, full_name as name_col, id as id_col};

        let Self { ids, name, email } = self;

        if matches!((ids.is_empty(), name, email), (true, None, None)) {
            return None;
        }

        let query = Vec::with_capacity(3);

        if !ids.is_empty() {
            query.push(Box::new(id_col.eq_any(ids)));
        }

        if let Some(name) = name {
            query.push(Box::new(name_col.ilike(format!("%{name}%"))));
        }

        if let Some(email) = email {
            query.push(Box::new(email_col.ilike(format!("{email}%"))));
        }

        query.into_iter().reduce(|q1, q2| Box::new(q1.and(q2)))
    }
}

impl Person {
    pub(super) fn base_query() -> InnerJoin<person::table, institution::table> {
        person::table.inner_join(institution::table)
    }
}

impl Read for Person {
    type Filter = PersonQuery;
    type Id = Uuid;

    // I'd like to factor out the `person::table.inner_join(institution::table).select(Person::as_select())` but there
    // doesn't seem to be a nice way to do that without boxing

    async fn fetch_by_id(id: Self::Id, conn: &mut AsyncPgConnection) -> super::Result<Self> {
        use person::id as id_col;

        Ok(Self::base_query()
            .filter(id_col.eq(id))
            .select(Person::as_select())
            .first(conn)
            .await?)
    }

    async fn fetch_many(filter: Self::Filter, conn: &mut AsyncPgConnection) -> super::Result<Vec<Self>> {
        use person::full_name;

        let query = Self::base_query().order_by(full_name).select(Person::as_select());
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
pub(super) struct Person {
    #[serde(flatten)]
    #[diesel(embed)]
    stub: PersonStub,
    email: String,
    orcid: Option<String>,
    #[diesel(embed)]
    institution: Institution,
}

#[derive(Queryable, Selectable, Serialize, Identifiable)]
#[diesel(table_name = person, check_for_backend(Pg))]
pub(super) struct PersonStub {
    id: Uuid,
    #[diesel(column_name = full_name)]
    name: String,
    link: String,
}
