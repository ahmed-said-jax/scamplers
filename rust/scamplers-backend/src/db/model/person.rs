use diesel::{dsl::InnerJoin, prelude::*};
use diesel_async::{AsyncPgConnection, RunQueryDsl, SaveChangesDsl};
use scamplers_core::person::{NewPerson, PersonQuery, Person};
use uuid::Uuid;
use scamplers_schema::{institution, person::dsl::{email as email_col, id as id_col, name as name_col, person}};

use diesel::{define_sql_function, sql_types::{Text, Array}};

define_sql_function! {fn grant_roles_to_user(user_id: Text, roles: Array<Text>)}
define_sql_function! {fn revoke_roles_from_user(user_id: Text, roles: Array<Text>)}
define_sql_function! {fn create_user_if_not_exists(user_id: Text, roles: Array<Text>)}
define_sql_function! {fn get_user_roles(user_id: Text) -> Array<Text>}

use crate::db::{util::AsIlike, AsDieselSelect, AsDieselExpression, BoxedDieselExpression};

impl<Table> AsDieselExpression<Table> for PersonQuery
where
    id_col: SelectableExpression<Table>,
    name_col: SelectableExpression<Table>,
    email_col: SelectableExpression<Table>,
{
    fn as_diesel_expression<'a>(&'a self) -> Option<BoxedDieselExpression<'a, Table>>
    where
        Table: 'a,
    {
        let Self { ids, name, email, .. } = self;

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

impl AsDieselSelect<InnerJoin<person, institution::table>> for Person {
    fn as_diesel_select() -> InnerJoin<person, institution::table> {
        person.inner_join(institution::table)
    }
}

pub async fn fetch_people(filter: Option<PersonQuery>, db_conn: &mut AsyncPgConnection) -> crate::db::error::Result<Vec<Person>> {
    let query = Person::as_diesel_select().order_by(name_col).select(Person::as_select());
    let filter = filter.as_diesel_expression();

    let people = match filter {
        Some(f) => query.filter(f).load(db_conn).await?,
        None => query.load(db_conn).await?,
    };

    Ok(people)
}

pub trait WriteMsLogin {
    async fn write_ms_login(self, db_conn: &mut AsyncPgConnection) -> crate::db::error::Result<()>;
}

impl WriteMsLogin for NewPerson {
    async fn write_ms_login(self, db_conn: &mut AsyncPgConnection) -> crate::db::error::Result<()> {
        use scamplers_schema::person::dsl::*;
        use crate::db::error::Error;

        let result = diesel::insert_into(person)
            .values(self)
            .on_conflict(ms_user_id)
            .do_update()
            .set((
                name.eq(&self.name),
                email.eq(&self.email),
                institution_id.eq(&self.institution_id),
            ))
            .returning(id)
            .get_result(db_conn)
            .await;

        let Err(err) = result else {
            return Ok(());
        };

        let err = Error::from(err);

        let new_id: Uuid = match err {
            Error::DuplicateRecord { ref field, .. } => {
                let Some(field) = field else {
                    return Err(err);
                };
                if field != "email" {
                    return Err(err);
                }

                let query = PersonQuery {email: Some(self.email.clone()), ..Default::default()};

                let p = fetch_people(Some(query),db_conn).await?;
                let p = &p[0];

                p.id
            }
            _ => {
                return Err(err);
            }
        };

        diesel::select(create_user_if_not_exists(new_id.to_string(), &self.roles))
            .execute(db_conn)
            .await?;

        Ok(())
    }
}
