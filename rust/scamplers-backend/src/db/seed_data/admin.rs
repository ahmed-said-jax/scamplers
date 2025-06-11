use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use garde::Validate;
use scamplers_core::model::person::{NewPerson, UserRole};
use scamplers_schema::person;
use serde::Deserialize;

use crate::db::model::person::{WriteLogin, grant_roles_to_user};

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

        let created_user = person.write_ms_login(db_conn).await?;

        // For convenience, grant the admin roles here, though this should be factored out eventually into a `PersonUpdate` struct that we can just populate and call from in here, rather than copying code
        diesel::select(grant_roles_to_user(
            created_user.person.summary.reference.id.to_string(),
            vec![UserRole::AppAdmin],
        ))
        .execute(db_conn)
        .await?;

        Ok(())
    }
}
