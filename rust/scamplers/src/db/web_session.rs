use diesel::{pg::Pg, prelude::*};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::HashedKey;
use crate::schema;

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
        use schema::session;

        let Self { person, .. } = &self;

        let user_id = person.create(conn).await?;

        self.user_id = user_id;

        diesel::insert_into(session::table).values(self).execute(conn).await?;

        Ok(())
    }
}
