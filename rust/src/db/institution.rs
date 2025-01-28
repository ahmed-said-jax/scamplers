use diesel::{prelude::*, pg::Pg};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::schema;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use super::{Create, Upsert};

#[derive(Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = schema::institution, check_for_backend(Pg))]
pub struct NewInstitution {
    name: String,
    ms_tenant_id: Option<Uuid>,
}
impl Upsert for NewInstitution {
    type Returns = Institution;

    async fn upsert(&mut self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
        use schema::institution::dsl::*;

        let as_immut = &*self;

        Ok(diesel::insert_into(institution).values(as_immut).on_conflict(name).do_update().set(as_immut).returning(Institution::as_returning()).get_result(conn).await?)
    }
}

#[derive(Deserialize)]
#[serde(transparent)]
pub struct NewInstitutions(pub Vec<NewInstitution>);
impl Create for NewInstitutions {
    type Returns = Vec<Institution>;

    async fn create(&mut self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
        use schema::institution::dsl::*;

        Ok(diesel::insert_into(institution).values(&self.0).returning(Institution::as_returning()).get_results(conn).await?)
    }
}


#[derive(Serialize, Selectable, Queryable)]
#[diesel(table_name = schema::institution, check_for_backend(Pg))]
pub struct Institution {
    id: Uuid,
    name: String
}