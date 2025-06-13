use crate::{
    db::{
        model::{self, AsDieselQueryBase, FetchById},
        util::{AsIlike, BoxedDieselExpression, NewBoxedDieselExpression},
    },
    fetch_by_query,
    server::auth::{ApiKey, HashedApiKey},
};
use diesel::{
    dsl::{AssumeNotNull, InnerJoin},
    prelude::*,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use scamplers_core::model::{
    Pagination,
    person::{CreatedUser, NewPerson, Person, PersonQuery, PersonSummary, UserRole},
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

impl<QuerySource> model::AsDieselFilter<QuerySource> for PersonQuery
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
            query = query.and_condition(id_col.eq_any(ids));
        }

        if let Some(name) = name {
            query = query.and_condition(name_col.ilike(name.as_ilike()));
        }

        if let Some(email) = email {
            query = query.and_condition(email_col.assume_not_null().ilike(email.as_ilike()));
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

impl model::FetchById for PersonSummary {
    type Id = Uuid;

    async fn fetch_by_id(
        id: &Self::Id,
        db_conn: &mut AsyncPgConnection,
    ) -> super::error::Result<Self> {
        Ok(Self::as_diesel_query_base()
            .find(id)
            .select(Self::as_select())
            .first(db_conn)
            .await?)
    }
}

impl model::FetchByQuery for PersonSummary {
    type QueryParams = PersonQuery;

    async fn fetch_by_query(
        query: &Self::QueryParams,
        db_conn: &mut AsyncPgConnection,
    ) -> super::error::Result<Vec<Self>> {
        use scamplers_core::model::person::PersonOrdinalColumn::{Email, Name};

        fetch_by_query!(query, [(Name, name_col), (Email, email_col)], db_conn)
    }
}

impl AsDieselQueryBase for Person {
    type QueryBase = InnerJoin<person::table, institution::table>;

    fn as_diesel_query_base() -> Self::QueryBase {
        PersonSummary::as_diesel_query_base().inner_join(institution::table)
    }
}

impl model::FetchById for Person {
    type Id = Uuid;

    async fn fetch_by_id(
        id: &Self::Id,
        db_conn: &mut AsyncPgConnection,
    ) -> super::error::Result<Self> {
        Ok(Self::as_diesel_query_base()
            .select(Self::as_select())
            .filter(id_col.eq(id))
            .get_result(db_conn)
            .await?)
    }
}

impl model::Write for NewPerson {
    type Returns = Person;
    async fn write(self, db_conn: &mut AsyncPgConnection) -> super::error::Result<Self::Returns> {
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
    ) -> super::error::Result<CreatedUser> {
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

        let id: Uuid = diesel::insert_into(person::table)
            .values(upsert)
            .on_conflict(ms_user_id_col)
            .do_update()
            .set(upsert)
            .returning(id_col)
            .get_result(db_conn)
            .await?;

        // Create the user, but give them no roles
        let empty_roles: Vec<UserRole> = Vec::with_capacity(0);
        diesel::select(create_user_if_not_exists(id.to_string(), empty_roles))
            .execute(db_conn)
            .await?;

        let person = Person::fetch_by_id(&id, db_conn).await?;

        Ok(CreatedUser {
            person,
            api_key: api_key.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use diesel_async::{AsyncConnection, scoped_futures::ScopedFutureExt};
    use rstest::rstest;
    use scamplers_core::model::{
        institution::{InstitutionQuery, InstitutionSummary},
        person::{NewPerson, PersonOrdering, PersonOrdinalColumn, PersonQuery, PersonSummary},
    };
    use uuid::Uuid;

    use crate::{
        config::LOGIN_USER,
        db::{
            DbTransaction,
            model::{FetchByQuery, Write},
            test_util::{DbConnection, N_PEOPLE, db_conn, test_query},
        },
    };

    fn comparison_fn(PersonSummary { name, .. }: &PersonSummary) -> String {
        name.to_string()
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn default_person_query(#[future] db_conn: DbConnection) {
        let expected = [(0, "person0"), (N_PEOPLE - 1, "person99")];
        test_query(
            PersonQuery::default(),
            db_conn,
            N_PEOPLE,
            comparison_fn,
            &expected,
        )
        .await;
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn specific_person_query(#[future] db_conn: DbConnection) {
        let query = PersonQuery {
            name: Some("person1".to_string()),
            order_by: vec![PersonOrdering {
                column: PersonOrdinalColumn::Name,
                descending: true,
            }],
            ..Default::default()
        };

        let expected = [(0, "person19"), (10, "person1")];
        test_query(query, db_conn, 11, comparison_fn, &expected).await;
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn write_ms_login_as_login_user(#[future] mut db_conn: DbConnection) {
        db_conn
            .test_transaction::<_, crate::db::error::Error, _>(|tx| {
                async move {
                    tx.set_transaction_user(LOGIN_USER).await.unwrap();

                    let institution_id =
                        InstitutionSummary::fetch_by_query(&InstitutionQuery::default(), tx)
                            .await
                            .unwrap()
                            .get(0)
                            .unwrap()
                            .reference
                            .id;

                    let pete = NewPerson {
                        name: "Peter Parker".to_string(),
                        email: "peter.parker@example.com".to_string(),
                        ms_user_id: Some(Uuid::now_v7()),
                        orcid: None,
                        institution_id,
                        roles: vec![],
                    };

                    pete.write(tx).await.unwrap();

                    Ok(())
                }
                .scope_boxed()
            })
            .await;
    }
}
