use crate::db::{NewBoxedDieselExpression, Write, util::BoxedDieselExpression};
use diesel::{
    dsl::{AssumeNotNull, InnerJoin},
    prelude::*,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use scamplers_core::model::{
    Pagination,
    person::{CreatedUser, NewPerson, Person, PersonOrdering, PersonQuery, PersonSummary},
};
use scamplers_schema::{
    institution,
    person::{
        self,
        dsl::{email as email_col, id as id_col, ms_user_id as ms_user_id_col, name as name_col},
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
    db::{AsDieselFilter, AsDieselQueryBase, FetchById, FetchByQuery, util::AsIlike},
    server::auth::{ApiKey, HashedApiKey},
};

impl<QuerySource> AsDieselFilter<QuerySource> for PersonQuery
where
    id_col: SelectableExpression<QuerySource>,
    name_col: SelectableExpression<QuerySource>,
    AssumeNotNull<email_col>: SelectableExpression<QuerySource>,
{
    fn as_diesel_filter<'a>(&'a self) -> Option<BoxedDieselExpression<'a, QuerySource>>
    where
        QuerySource: 'a,
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
            query = query.with_condition(email_col.assume_not_null().ilike(email.as_ilike()));
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

impl FetchByQuery for PersonSummary {
    type QueryParams = PersonQuery;

    async fn fetch_by_query(
        query: &Self::QueryParams,
        db_conn: &mut AsyncPgConnection,
    ) -> crate::db::error::Result<Vec<Self>> {
        use scamplers_core::model::person::PersonOrdinalColumn::{Email, Name};

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
        id: &Self::Id,
        db_conn: &mut AsyncPgConnection,
    ) -> crate::db::error::Result<Self> {
        Ok(Self::as_diesel_query_base()
            .select(Self::as_select())
            .filter(id_col.eq(id))
            .get_result(db_conn)
            .await?)
    }
}

impl Write for NewPerson {
    type Returns = Person;
    async fn write(
        self,
        db_conn: &mut AsyncPgConnection,
    ) -> crate::db::error::Result<Self::Returns> {
        let id = diesel::insert_into(person::table)
            .values(self)
            .returning(id_col)
            .get_result(db_conn)
            .await?;

        Person::fetch_by_id(&id, db_conn).await
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
        #[derive(Insertable, AsChangeset, Clone, Copy)]
        #[diesel(table_name = person, primary_key(ms_user_id))]
        struct Upsert<'a> {
            ms_user_id: Option<&'a Uuid>,
            name: &'a str,
            email: &'a str,
            hashed_api_key: &'a HashedApiKey,
            institution_id: &'a Uuid,
        }

        let Self {
            name,
            email,
            institution_id,
            ms_user_id,
            ..
        } = &self;

        // TODO: We shouldn't overwrite the user's API key on every single login
        let api_key = ApiKey::new();
        let hashed_api_key = api_key.hash();

        let upsert = Upsert {
            ms_user_id: ms_user_id.as_ref(),
            name,
            email,
            hashed_api_key: &hashed_api_key,
            institution_id,
        };

        // We know that whoever just logged in is the actual owner of this email address. Anyone else that has this email should have it removed from them
        diesel::update(person::table)
            .filter(email_col.eq(email))
            .set(email_col.eq(None::<String>))
            .execute(db_conn)
            .await?;

        let id = diesel::insert_into(person::table)
            .values(upsert)
            .on_conflict(ms_user_id_col)
            .do_update()
            .set(upsert)
            .returning(id_col)
            .get_result(db_conn)
            .await?;

        let person = Person::fetch_by_id(&id, db_conn).await?;

        Ok(CreatedUser {
            person,
            api_key: api_key.into(),
        })
    }
}
