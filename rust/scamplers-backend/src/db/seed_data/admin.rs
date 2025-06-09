use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use garde::Validate;
use scamplers_core::model::person::{CreatedUser, NewPerson, Person, UserRole};
use scamplers_schema::person;
use serde::Deserialize;

use crate::db::model::person::{WriteLogin, create_user_if_not_exists};

use super::Write;

#[derive(Deserialize, Validate, Insertable, Clone)]
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
        let Self { person } = self;

        let CreatedUser {
            person: Person { id, .. },
            ..
        } = person.write_ms_login(db_conn).await?;

        // For convenience, we create the admin and grant them roles here, though this should be factored out eventually into a `PersonUpdate` struct that we can just populate and call from in here, rather than copying code
        diesel::select(create_user_if_not_exists(
            id.to_string(),
            vec![UserRole::AppAdmin],
        ))
        .execute(db_conn)
        .await?;

        Ok(())
    }
}
