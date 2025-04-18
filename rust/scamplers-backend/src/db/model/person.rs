use diesel::{dsl::InnerJoin, prelude::*};
use diesel_async::{AsyncPgConnection, RunQueryDsl, SaveChangesDsl, methods::ExecuteDsl};
use scamplers_core::person::{NewPerson, Person, PersonQuery};
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
}, server::auth::HashedKey};

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

pub trait WriteMsLogin {
    async fn write_ms_login(
        self,
        db_conn: &mut AsyncPgConnection,
    ) -> crate::db::error::Result<Uuid>;
}

impl WriteMsLogin for NewPerson {
    async fn write_ms_login(
        self,
        db_conn: &mut AsyncPgConnection,
    ) -> crate::db::error::Result<Uuid> {
        use crate::db::error::Error;

        let Self {
            name,
            email,
            institution_id,
            roles,
            ..
        } = &self;

        let result = diesel::insert_into(person::table)
            .values(&self)
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

        let new_id = match &result {
            Ok(id) => *id,
            Err(Error::DuplicateRecord { field, .. }) => {
                let Some(field) = field else {
                    return result;
                };
                if field != "email" {
                    return result;
                }

                let query = PersonQuery {
                    email: Some(email.clone()),
                    ..Default::default()
                };

                let p = fetch_by_filter(Some(query), db_conn).await?;
                let p = &p[0];

                p.id
            }
            _ => {
                return result;
            }
        };

        let roles: Vec<_> = roles.clone().into_iter().map(|r| DbEnum(r)).collect();

        diesel::select(create_user_if_not_exists(new_id.to_string(), &roles))
            .execute(db_conn)
            .await?;

        Ok(new_id)
    }
}

#[derive(Identifiable, AsChangeset, Queryable)]
#[diesel(table_name = person, check_for_backend(Pg))]
pub struct GrantApiAccess<'a> {
    pub id: Uuid,
    pub hashed_api_key: HashedKey<&'a str>,
}

impl Write for GrantApiAccess<'_> {
    type Returns = ();

    async fn write(self, conn: &mut AsyncPgConnection) -> db::error::Result<Self::Returns> {
        diesel::update(&self).set(&self).execute(conn).await?;
        Ok(())
    }
}
