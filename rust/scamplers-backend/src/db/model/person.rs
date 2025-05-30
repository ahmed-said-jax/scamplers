use diesel::{dsl::InnerJoin, prelude::*};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use scamplers_core::model::person::{CreatedUser, NewPerson, Person, PersonQuery};
use scamplers_schema::{
    institution,
    person::{
        self,
        dsl::{
            email as email_col, hashed_api_key as hashed_api_key_col, id as id_col,
            ms_user_id as ms_user_id_col, name as name_col, verified as verified_col,
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
    db::{AsDieselFilter, AsDieselQueryBase, BoxedDieselExpression, util::AsIlike},
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

        if matches!((ids.is_empty(), name, email), (true, None, None)) {
            return None;
        }

        // In theory, we could initialize this with `let mut query = None;`, but that results in a lot of boilerplate
        let mut query: BoxedDieselExpression<Table> = if ids.is_empty() {
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

impl AsDieselQueryBase for Person {
    type QueryBase = InnerJoin<person::table, institution::table>;

    fn as_diesel_query_base() -> Self::QueryBase {
        person::table.inner_join(institution::table)
    }
}

pub async fn fetch_by_filter(
    filter: Option<PersonQuery>,
    db_conn: &mut AsyncPgConnection,
) -> crate::db::error::Result<Vec<Person>> {
    let query = Person::as_diesel_query_base()
        .order_by(name_col)
        .select(Person::as_select());
    let filter = filter.as_diesel_filter();

    let people = match filter {
        Some(f) => query.filter(f).load(db_conn).await?,
        None => query.load(db_conn).await?,
    };

    Ok(people)
}

pub async fn fetch_by_id(
    id: Uuid,
    db_conn: &mut AsyncPgConnection,
) -> crate::db::error::Result<Person> {
    Ok(Person::as_diesel_query_base()
        .filter(id_col.eq(id))
        .select(Person::as_select())
        .first(db_conn)
        .await?)
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
            hashed_api_key: Option<&'a HashedApiKey>,
            institution_id: &'a Uuid,
            verified: bool,
        }

        let Self {
            name,
            email,
            institution_id,
            ms_user_id,
            ..
        } = &self;

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
            hashed_api_key: None,
            institution_id,
            verified: true,
        };

        let (id, api_key) = match maybe_has_api_key {
            Some(true) => {
                let id = diesel::update(person::table)
                    .filter(ms_user_id_col.eq(ms_user_id))
                    .set(upsert)
                    .returning(id_col)
                    .get_result(db_conn)
                    .await?;

                (id, None)
            }
            Some(false) | None => {
                diesel::update(person::table)
                    .filter(email_col.eq(email))
                    .set(verified_col.eq(false))
                    .execute(db_conn)
                    .await?;

                let api_key = ApiKey::new();
                let hash = api_key.hash();
                upsert.hashed_api_key = Some(&hash);

                let id = diesel::insert_into(person::table)
                    .values(upsert)
                    .returning(id_col)
                    .get_result(db_conn)
                    .await?;

                (id, Some(api_key))
            }
        };

        let person = fetch_by_id(id, db_conn).await?;

        Ok(CreatedUser {
            person,
            api_key: api_key.map(|k| k.into()),
        })
    }
}
