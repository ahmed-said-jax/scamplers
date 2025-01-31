use diesel::{pg::Pg, prelude::*};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Create, Read, Pagination, Upsert};
use crate::{api::EntityLink, schema};


// We can just use one struct for inserting, upserting, updating, and fetching institutions because they're simple. We also don't need to implement `Update` because `Upsert` works for this case
#[derive(Insertable, Deserialize, AsChangeset, Clone, Identifiable, Serialize, Queryable, Selectable,)]
#[diesel(table_name = schema::institution, check_for_backend(Pg))]
pub struct Institution {
    #[diesel(skip_insertion)] // The db will generate an ID, we don't want the user to be able to set it. It's just handy to have this in the struct because it allows us to update the institution using its ID if the user does set it
    #[serde(default)]
    id: Uuid,
    name: String,
    #[serde(skip_deserializing)]
    links: Vec<EntityLink>, // We will generate this, we don't want the user to set it
    #[serde(skip_serializing_if = "Option::is_none")]
    ms_tenant_id: Option<Uuid>,
}

// We don't need to `impl Create` for an individual `Institution` because it's more efficient to just do batches
impl Create for Vec<Institution> {
    type Returns = Vec<Institution>;

    async fn create(&mut self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
        use schema::institution::dsl::institution;

        Ok(diesel::insert_into(institution)
            .values(self)
            .returning(Institution::as_returning())
            .get_results(conn)
            .await?)
    }
}

impl Upsert for Institution {
    type Returns = Institution;

    async fn upsert(&self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
        use schema::institution::dsl::*;

        // if an institution with this ID or name exists, update it
        Ok(diesel::insert_into(institution)
            .values(self)
            .on_conflict((id, name))
            .do_update()
            .set(self)
            .returning(Institution::as_returning())
            .get_result(conn)
            .await?)
    }
}

// For simplicity, we are using the same 
impl Read for Institution {
    async fn fetch_all(conn: &mut AsyncPgConnection, Pagination{limit, offset}: Pagination) -> super::Result<Vec<Self>> {
        use schema::institution::dsl::institution;

        let mut institutions = institution.limit(limit).offset(offset).select(Self::as_select()).load(conn).await?;

        // we don't need to expose the `ms_tenant_id`
        for inst in &mut institutions {
            inst.ms_tenant_id = None;
        }

        Ok(institutions)
    }

    async fn fetch_by_id(conn: &mut AsyncPgConnection, id: Self::Id) -> super::Result<Self> {
        use schema::institution::dsl::institution;

        let mut found = institution.find(id).select(Self::as_select()).first(conn).await?;
        found.ms_tenant_id = None;

        Ok(found)
    }
}