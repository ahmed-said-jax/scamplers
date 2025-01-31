use diesel::{pg::Pg, prelude::*};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Create, Pagination, Read, Upsert};
use crate::schema;

// We can just use one struct for inserting, upserting, updating, and fetching
// institutions because they're simple. We also don't need to implement `Update`
// because `Upsert` works for this case
#[derive(
    Insertable, Deserialize, AsChangeset, Clone, Identifiable, Serialize, Queryable, Selectable,
)]
#[diesel(table_name = schema::institution, check_for_backend(Pg))]
pub struct Institution {
    #[diesel(skip_insertion)]
    // The db will generate an ID, we don't want the user to be able to set it. It's just handy to
    // have this in the struct because it allows us to update the institution using its ID if the
    // user does set it
    #[serde(default)]
    id: Uuid,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ms_tenant_id: Option<Uuid>,
}

// We don't need to `impl Create` for an individual `Institution` because it's
// more efficient to just do batches
impl Create for Vec<Institution> {
    type Returns = Vec<Institution>;

    async fn create(&mut self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
        use schema::institution::dsl::institution;

        let as_immut = &*self;

        Ok(diesel::insert_into(institution)
            .values(as_immut)
            .returning(Institution::as_returning())
            .get_results(conn)
            .await?)
    }
}

impl Upsert for Institution {
    type Returns = Institution;

    async fn upsert(&mut self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
        use schema::institution::dsl::*;

        let as_immut = &*self;

        let base_stmt = diesel::insert_into(institution).values(as_immut);

        if as_immut.id.is_nil() {
            Ok(base_stmt.on_conflict(name).do_update().set(as_immut).returning(Self::as_returning()).get_result(conn).await?)
        } else {
            Ok(base_stmt.on_conflict(id).do_update().set(as_immut).returning(Self::as_returning()).get_result(conn).await?)
        }
    }
}

// For simplicity, we are using the same
impl Read for Institution {
    async fn fetch_all(
        conn: &mut AsyncPgConnection,
        Pagination { limit, offset }: Pagination,
    ) -> super::Result<Vec<Self>> {
        use schema::institution::dsl::institution;

        let mut institutions = institution
            .limit(limit)
            .offset(offset)
            .select(Self::as_select())
            .load(conn)
            .await?;

        Ok(institutions)
    }

    async fn fetch_by_id(conn: &mut AsyncPgConnection, id: Self::Id) -> super::Result<Self> {
        use schema::institution::dsl::institution;

        let mut found = institution
            .find(id)
            .select(Self::as_select())
            .first(conn)
            .await?;

        Ok(found)
    }
}
