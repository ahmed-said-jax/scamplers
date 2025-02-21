
use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types::{self},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use valuable::Valuable;

use super::{Create, DbEnum, Paginate, Read, institution::Institution};
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
    orcid: Option<String>, /* No need to validate this because the only way to insert a person is if you are an
                            * admin or inserting yourself, in which case this field won't be available until you
                            * link your orcid */
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
    pub fn as_sql(&self) -> person::BoxedQuery<'_, Pg> {
        use person::dsl::{email as email_col, full_name as name_col, id as id_col};

        let mut query = person::table.into_boxed();

        let Self { ids, name, email } = self;

        if !ids.is_empty() {
            query = query.filter(id_col.eq_any(ids));
        }

        // The next two conditions are pretty much the same thing, there's probably some
        // way to improve this
        if let Some(name) = name {
            query = query.filter(name_col.ilike(format!("%{name}%"))); // This allows searching by first name or last name (or any substring within either)
        }

        if let Some(email) = email {
            query = query.filter(email_col.ilike(format!("{email}%")));
        }

        query
    }
}

impl Read for Person {
    type Filter = PersonFilter;
    type Id = Uuid;

    async fn fetch_by_id(id: Self::Id, conn: &mut AsyncPgConnection) -> super::Result<Self> {
        let info = person::table
            .find(id)
            .inner_join(institution::table)
            .select(PersonFull::as_select())
            .first(conn)
            .await?;

        Ok(Self::Full(info))
    }

    async fn fetch_many(filter: Self::Filter, conn: &mut AsyncPgConnection) -> super::Result<Vec<Self>> {
        let query = filter.as_sql();

        let people = query.select(PersonLite::as_select()).load(conn).await?;

        Ok(people.into_iter().map(|p| Self::Lite(p)).collect())
    }
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum Person {
    Full(PersonFull),
    Lite(PersonLite),
}

#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = person, check_for_backend(Pg))]
pub(super) struct PersonFull {
    #[serde(flatten)]
    #[diesel(embed)]
    lite: PersonLite,
    #[diesel(embed)]
    institution: Institution,
}

#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = person, check_for_backend(Pg))]
pub(super) struct PersonLite {
    #[serde(flatten)]
    #[diesel(embed)]
    stub: PersonStub,
    email: String,
    orcid: Option<String>,
}

#[derive(Queryable, Selectable, Serialize, Identifiable)]
#[diesel(table_name = person, check_for_backend(Pg))]
pub(super) struct PersonStub {
    id: Uuid,
    #[diesel(column_name = full_name)]
    name: String,
    link: String,
}
