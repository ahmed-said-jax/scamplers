use diesel::{dsl::InnerJoin, prelude::*};
use diesel_async::{AsyncPgConnection, RunQueryDsl, SaveChangesDsl, methods::ExecuteDsl};
use scamplers_core::person::{CreatedUser, NewPerson, Person, PersonQuery};
use scamplers_schema::{
    institution,
    person::{
        self,
        dsl::{
            email as email_col, id as id_col, institution_id as institution_col,
            ms_user_id as ms_user_id_col, name as name_col,
        },
    }
};
use serde::Serialize;
use uuid::Uuid;

use diesel::{
    define_sql_function,
    sql_types::{Array, Text},
};

define_sql_function! {fn grant_roles_to_user(user_id: Text, roles: Array<Text>)}
define_sql_function! {fn revoke_roles_from_user(user_id: Text, roles: Array<Text>)}
define_sql_function! {fn create_user_if_not_exists(user_id: Text, roles: Array<Text>)}
define_sql_function! {fn get_user_roles(user_id: Text) -> Array<Text>}

use crate::{db::{
    self, util::{AsIlike, DbEnum}, AsDieselFilter, AsDieselQueryBase, BoxedDieselExpression, Write
}, server::auth::{HashedKey, Key}};

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

pub async fn fetch_by_id(id: Uuid, db_conn: &mut AsyncPgConnection) -> crate::db::error::Result<Person> {
    Ok(Person::as_diesel_query_base().filter(id_col.eq(id)).select(Person::as_select()).first(db_conn).await?)
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
        use crate::db::error::Error;

        #[derive(Insertable)]
        #[diesel(table_name = person)]
        struct NewUser {
            #[diesel(embed)]
            person: NewPerson,
            hashed_api_key: HashedKey
        }

        let api_key = Key::new();
        let hashed_api_key = api_key.hash();

        let new_user = NewUser {person: self, hashed_api_key};
        let NewUser { person: NewPerson { name, email, institution_id, roles, .. }, .. } = &new_user;

        // If it's a new user, then we can add the API key. If not,
        let result = diesel::insert_into(person::table)
            .values(&new_user)
            .on_conflict(ms_user_id_col)
            .do_update()
            .set((
                name_col.eq(name),
                email_col.eq(email),
                institution_col.eq(institution_id),
            ))
            .returning(id_col)
            .get_result(db_conn)
            .await;

        let result = result.map_err(Error::from);

        let (user_id, user_is_new) = match &result {
            Ok(id) => (*id, true),
            Err(Error::DuplicateRecord { field, .. }) => {
                let Some(field) = field else {
                    return Err(result.unwrap_err());
                };
                if field != "email" {
                    return Err(result.unwrap_err());
                }

                let query = PersonQuery {
                    email: Some(email.clone()),
                    ..Default::default()
                };

                let p = fetch_by_filter(Some(query), db_conn).await?;
                let p = &p[0];

                (p.id, false)
            }
            _ => {
                return Err(result.unwrap_err());
            }
        };

        let roles: Vec<_> = roles.clone().into_iter().map(|r| DbEnum(r)).collect();

        diesel::select(create_user_if_not_exists(user_id.to_string(), &roles))
            .execute(db_conn)
            .await?;

        let api_key = if user_is_new {
            Some(api_key.to_string())
        } else {
            None
        };

        Ok(CreatedUser{id: user_id, api_key})
    }
}
