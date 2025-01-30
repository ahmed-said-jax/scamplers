use diesel::{pg::Pg, prelude::*};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Create, Pagination, Upsert};
use crate::schema::{self, cdna::SqlType, institution};

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

        Ok(diesel::insert_into(institution)
            .values(self)
            .on_conflict(name)
            .do_update()
            .set(self)
            .returning(Institution::as_returning())
            .get_result(conn)
            .await?)
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

impl Institution {
    pub async fn fetch_all(conn: &mut AsyncPgConnection, Pagination{limit, offset}: Pagination) -> super::Result<Vec<Self>> {
        use schema::institution::dsl::institution;

        Ok(institution.limit(limit).offset(offset).select(Self::as_select()).load(conn).await?)
    }

    pub async fn fetch_by_id(conn: &mut AsyncPgConnection, id: Uuid) -> super::Result<Self> {
        use schema::institution::dsl::{institution};

        Ok(institution.find(id).select(Self::as_select()).first(conn).await?)
    }
}