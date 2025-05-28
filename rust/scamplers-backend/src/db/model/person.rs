use diesel::{dsl::InnerJoin, prelude::*};
use diesel_async::{AsyncPgConnection, RunQueryDsl, methods::ExecuteDsl};
use scamplers_core::model::person::{CreatedUser, NewPerson, Person, PersonQuery};
use scamplers_schema::{
    institution,
    person::{
        self,
        dsl::{
            email as email_col, hashed_api_key as hashed_api_key_col, id as id_col,
            institution_id as institution_col, ms_user_id as ms_user_id_col, name as name_col,
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
    // TODO: this function is big and ugly. Split it up
    async fn write_ms_login(
        self,
        db_conn: &mut AsyncPgConnection,
    ) -> crate::db::error::Result<CreatedUser> {
        #[derive(Insertable)]
        #[diesel(table_name = person)]
        struct NewUser {
            #[diesel(embed)]
            person: NewPerson,
            hashed_api_key: HashedApiKey,
        }

        let api_key = ApiKey::new();
        let hashed_api_key = api_key.hash();

        // Instantiate a `NewUser` so we can use it as a wholesale insert in case this user doesn't exist
        let new_user = NewUser {
            person: self,
            hashed_api_key,
        };

        // Also destructure the `NewUser` so we have granular control over which columns to set
        let NewUser {
            person:
                NewPerson {
                    name,
                    email,
                    institution_id,
                    ms_user_id,
                    ..
                },
            hashed_api_key,
        } = &new_user;

        let changes = || {
            (
                name_col.eq(name),
                email_col.eq(email),
                ms_user_id_col.eq(ms_user_id),
                institution_col.eq(institution_id),
            )
        };

        // This should probably be a separate function
        let result = diesel::update(person::table)
            .filter(ms_user_id_col.eq(ms_user_id))
            .set(changes())
            .returning(id_col)
            .get_result(db_conn)
            .await;

        let id = match result.optional() {
            Ok(Some(id)) => id,
            Ok(None) => {
                diesel::insert_into(person::table)
                    .values(&new_user)
                    .on_conflict(email_col)
                    .do_update()
                    .set(changes())
                    .returning(id_col)
                    .get_result(db_conn)
                    .await?
            }
            Err(e) => {
                return Err(e.into());
            }
        };

        let person = fetch_by_id(id, db_conn).await?;

        Ok(CreatedUser {
            person,
            api_key: api_key.into(),
        })
    }
}
