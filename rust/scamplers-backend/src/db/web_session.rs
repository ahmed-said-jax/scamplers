use diesel::{pg::Pg, prelude::*};
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use uuid::Uuid;

use super::{
    Create,
    person::{NewPerson, Person, PersonQuery},
};
use crate::{auth::HashedKey, db::Read, schema, schema::person::ms_user_id};

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
            person::{
                email as email_col, id as id_col, institution_id as institution_col, name as name_col,
                table as person_table,
            },
            session,
        };

        let Self { person, .. } = &self;

        let user_id = person.create_from_ms_login(conn).await?;

        self.user_id = user_id;

        diesel::insert_into(session::table).values(self).execute(conn).await?;

        Ok(())
    }
}
