use crate::db::{NewBoxedDieselExpression, util::BoxedDieselExpression};
use diesel::{dsl::InnerJoin, prelude::*};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use scamplers_core::model::{
    Pagination,
    person::{CreatedUser, NewPerson, Person, PersonOrdering, PersonQuery, PersonSummary},
};
use scamplers_schema::{
    institution,
    person::{
        self,
        dsl::{
            email as email_col, hashed_api_key as hashed_api_key_col, id as id_col,
            ms_user_id as ms_user_id_col, name as name_col, verified_email as verified_email_col,
        },
    },
};
use uuid::Uuid;

use diesel::{
    define_sql_function,
    sql_types::{Array, Text},
};

define_sql_function! {fn grant_roles_to_user(user_id: Text, roles: Array<Text>)}
define_sql_function! {fn revoke_roles_from_user(user_id: Text, roles: Array<Text>)}
define_sql_function! {fn create_user_if_not_exists(user_id: Text, roles: Array<Text>)}
define_sql_function! {fn get_user_roles(user_id: Text) -> Array<Text>}

use crate::{
    db::{AsDieselFilter, AsDieselQueryBase, FetchByFilter, FetchById, util::AsIlike},
    server::auth::{ApiKey, HashedApiKey},
};

impl<Table> AsDieselFilter<Table> for PersonQuery
where
    id_col: SelectableExpression<Table>,
    name_col: SelectableExpression<Table>,
    email_col: SelectableExpression<Table>,
{
    fn as_diesel_filter<'a>(&'a self) -> Option<BoxedDieselExpression<'a, Table>>
    where
        Table: 'a,
    {
        let Self {
            ids, name, email, ..
        } = self;

        let mut query = BoxedDieselExpression::new_expression();

        if !ids.is_empty() {
            query = query.with_condition(id_col.eq_any(ids));
        }

        if let Some(name) = name {
            query = query.with_condition(name_col.ilike(name.as_ilike()));
        }

        if let Some(email) = email {
            query = query.with_condition(email_col.ilike(email.as_ilike()));
        }

        query.build()
    }
}

impl AsDieselQueryBase for PersonSummary {
    type QueryBase = person::table;

    fn as_diesel_query_base() -> Self::QueryBase {
        person::table
    }
}

impl FetchByFilter for PersonSummary {
    type QueryParams = PersonQuery;

    async fn fetch_by_query(
        query: Self::QueryParams,
        db_conn: &mut AsyncPgConnection,
    ) -> crate::db::error::Result<Vec<Self>> {
        use scamplers_core::model::person::PersonOrdinalColumn::{Email, Id, Name};

        let PersonQuery {
            order_by,
            pagination: Pagination { limit, offset },
            ..
        } = &query;

        let mut statement = Self::as_diesel_query_base()
            .select(Self::as_select())
            .limit(*limit)
            .offset(*offset)
            .into_boxed();

        let query = query.as_diesel_filter();

        if let Some(query) = query {
            statement = statement.filter(query);
        }

        // This is horrendous and not scalable
        for PersonOrdering { column, descending } in order_by {
            statement = match (column, descending) {
                (Id, false) => statement.then_order_by(id_col.asc()),
                (Id, true) => statement.then_order_by(id_col.desc()),
                (Name, false) => statement.then_order_by(name_col.asc()),
                (Name, true) => statement.then_order_by(name_col.desc()),
                (Email, false) => statement.then_order_by(email_col.asc()),
                (Email, true) => statement.then_order_by(email_col.desc()),
            };
        }

        let people = statement.load(db_conn).await?;

        Ok(people)
    }
}

impl AsDieselQueryBase for Person {
    type QueryBase = InnerJoin<person::table, institution::table>;

    fn as_diesel_query_base() -> Self::QueryBase {
        PersonSummary::as_diesel_query_base().inner_join(institution::table)
    }
}

impl FetchById for Person {
    type Id = Uuid;

    async fn fetch_by_id(
        id: Self::Id,
        db_conn: &mut AsyncPgConnection,
    ) -> crate::db::error::Result<Self> {
        Ok(Self::as_diesel_query_base()
            .select(Self::as_select())
            .filter(id_col.eq(id))
            .get_result(db_conn)
            .await?)
    }
}

pub trait WriteLogin {
    async fn write_ms_login(
        self,
        db_conn: &mut AsyncPgConnection,
    ) -> crate::db::error::Result<CreatedUser>;
}

impl WriteLogin for NewPerson {
    async fn write_ms_login(
        self,
        db_conn: &mut AsyncPgConnection,
    ) -> crate::db::error::Result<CreatedUser> {
        #[derive(Insertable, AsChangeset)]
        #[diesel(table_name = person, primary_key(ms_user_id))]
        struct Upsert<'a> {
            ms_user_id: Option<&'a Uuid>,
            name: &'a str,
            email: &'a str,
            verified_email: bool,
            hashed_api_key: Option<&'a HashedApiKey>,
            institution_id: &'a Uuid,
        }

        let Self {
            name,
            email,
            institution_id,
            ms_user_id,
            ..
        } = &self;

        // If the user exists, do they have an API key (this indicates they've logged in before)?
        let maybe_has_api_key = person::table
            .filter(ms_user_id_col.eq(ms_user_id))
            .select(hashed_api_key_col.is_not_null())
            .get_result(db_conn)
            .await
            .optional()?;

        let mut upsert = Upsert {
            ms_user_id: ms_user_id.as_ref(),
            name,
            email,
            verified_email: true,
            hashed_api_key: None,
            institution_id,
        };

        // If a user exists with that `ms_user_id` and has an API key, we just need to update their name and email with the information we get from Microsoft Entra ID. This is a non-operation almost every single time because people rarely change their names, emails, and institutions
        let (id, api_key) = if let Some(true) = maybe_has_api_key {
            let id = diesel::update(person::table)
                .filter(ms_user_id_col.eq(ms_user_id))
                .set(upsert)
                .returning(id_col)
                .get_result(db_conn)
                .await?;

            // We know this is someone who has logged in before and generated an API key. We shouldn't change their API key, so setting it to `None` informs the frontend not to change the API key value in the session
            (id, None)

        // If not, then either:
        // 1. A user with that `ms_user_id` exists but has no API key (so they haven't logged in before)
        // 2. No user with that `ms_user_id` exists
        } else {
            // We know that whoever just logged in is the actual owner of this email address. Anyone else that has this email should be unverified. This is a rare case, but we emit this command nonetheless just to be sure
            diesel::update(person::table)
                .filter(email_col.eq(email))
                .set(verified_email_col.eq(false))
                .execute(db_conn)
                .await?;

            // Since this is a new login, the new user needs an API key
            let api_key = ApiKey::new();
            let hash = api_key.hash();

            // Add it to the `upsert` to be emitted to the db
            upsert.hashed_api_key = Some(&hash);

            // Insert the new user, specifying their email is verified (see the definition of `upsert` variable above)
            let id = diesel::insert_into(person::table)
                .values(upsert)
                .returning(id_col)
                .get_result(db_conn)
                .await?;

            (id, Some(api_key))
        };

        let person = Person::fetch_by_id(id, db_conn).await?;

        Ok(CreatedUser {
            person,
            api_key: api_key.map(std::convert::Into::into),
        })
    }
}
