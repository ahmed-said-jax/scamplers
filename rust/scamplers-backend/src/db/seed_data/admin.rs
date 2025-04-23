use diesel::{pg::Pg, prelude::*};
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;
use garde::Validate;
use scamplers_core::{
    institution::NewInstitution,
    person::{NewPerson, UserRole},
};
use scamplers_schema::person;
use serde::Deserialize;
use uuid::Uuid;

use crate::db::model::person::WriteLogin;

use super::Write;

#[derive(Deserialize, Validate, Insertable)]
#[diesel(table_name = person, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
#[serde(transparent)]
pub(super) struct NewAdmin {
    #[garde(dive)]
    #[diesel(embed)]
    person: NewPerson,
}

impl Write for NewAdmin {
    type Returns = ();

    async fn write(
        self,
        db_conn: &mut AsyncPgConnection,
    ) -> super::super::error::Result<Self::Returns> {
        let Self {
            mut person
        } = self;

        person.roles.push(UserRole::AppAdmin);

        person.write_ms_login(db_conn).await?;

        Ok(())
    }
}
