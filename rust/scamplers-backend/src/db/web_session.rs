use super::person::{Person, PersonQuery};
use diesel::{pg::Pg, prelude::*};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::HashedKey;
use crate::db::Read;
use crate::schema;
use crate::schema::person::ms_user_id;

use super::Create;
use super::person::NewPerson;
use diesel_async::RunQueryDsl;

#[derive(Insertable)]
#[diesel(table_name = schema::session, check_for_backend(Pg))]
pub struct NewSession<'a> {
    pub hashed_id: HashedKey<&'a str>,
    #[diesel(skip_insertion)]
    pub person: NewPerson,
    pub user_id: Uuid,
}

impl Create for NewSession<'_> {
    type Returns = ();

    async fn create(mut self, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self::Returns> {
        use schema::{
            person::email as email_col, person::id as id_col, person::institution_id as institution_col,
            person::name as name_col, person::table as person_table, session,
        };

        let Self { person, .. } = &self;

        let user_id = person.create_from_ms_login(conn).await?;

        self.user_id = user_id;

        diesel::insert_into(session::table).values(self).execute(conn).await?;

        Ok(())
    }
}
