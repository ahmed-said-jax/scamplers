use diesel::{pg::Pg, prelude::*};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use futures::FutureExt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Create, Upsert};
use crate::schema;

#[derive(Insertable, Deserialize, AsChangeset, Clone)]
#[diesel(table_name = schema::institution, check_for_backend(Pg))]
pub struct NewInstitution {
    name: String,
    ms_tenant_id: Option<Uuid>,
}

impl Upsert for NewInstitution {
    type Returns = Institution;

    async fn upsert(&self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
        use schema::institution::dsl::{institution, name};

        Ok(diesel::insert_into(institution).values(self).on_conflict(name).do_update().set(self).returning(Institution::as_returning()).get_result(conn).await?)
    }
}

impl Create for Vec<NewInstitution> {
    type Returns = Vec<Institution>;

    async fn create(&self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
        use schema::institution::dsl::institution;

        Ok(diesel::insert_into(institution)
            .values(self)
            .returning(Institution::as_returning())
            .get_results(conn)
            .await?)
    }
}

#[derive(Serialize, Selectable, Queryable)]
#[diesel(table_name = schema::institution, check_for_backend(Pg))]
pub struct Institution {
    id: Uuid,
    name: String,
}
