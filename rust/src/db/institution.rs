use diesel::{pg::Pg, prelude::*};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use super::{Create, Pagination, Read, Update};
use crate::schema;

// We can just use one struct for inserting, upserting, updating, and fetching
// institutions because they're simple. We also don't need to implement `Update`
// because `Upsert` works for this case
#[derive(Insertable, Deserialize, Clone, TS)]
#[diesel(table_name = schema::institution, check_for_backend(Pg))]
#[ts(export)]
pub struct NewInstitution {
    name: String,
    ms_tenant_id: Option<Uuid>,
}

// We don't need to `impl Create` for an individual `Institution` because it's
// more efficient to just do batches
impl Create for Vec<NewInstitution> {
    type Returns = Vec<Institution>;

    async fn create(&mut self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
        use schema::institution::dsl::*;

        let as_immut = &*self;

        let inserted = diesel::insert_into(institution)
            .values(as_immut)
            .returning(Institution::as_returning())
            .get_results(conn)
            .await?;

        Ok(inserted)
    }
}

// It's unlikely we'll need this, but it serves as a simple example for the patterns I want to establish in this package
#[derive(Identifiable, AsChangeset, Deserialize, TS)]
#[diesel(table_name = schema::institution, check_for_backend(Pg))]
#[ts(export)]
struct UpdatedInstitution {
    id: Uuid,
    name: Option<String>,
    ms_tenant_id: Option<Uuid>,
}

impl Update for UpdatedInstitution {
    type Returns = Institution;

    async fn update(&mut self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
        let as_immut = &*self;

        Ok(diesel::update(as_immut)
            .set(as_immut)
            .returning(Self::Returns::as_returning())
            .get_result(conn)
            .await?)
    }
}

#[derive(Queryable, Selectable, Serialize, TS)]
#[diesel(table_name = schema::institution, check_for_backend(Pg))]
#[ts(export)]
pub struct Institution {
    id: Uuid,
    name: String,
    link: String,
}

impl Read for Institution {
    type Filter = ();
    type Id = Uuid;

    async fn fetch_many(
        _filter: Option<&Self::Filter>,
        Pagination { limit, offset }: &Pagination,
        conn: &mut AsyncPgConnection,
    ) -> super::Result<Vec<Self>> {
        use schema::institution::dsl::institution;

        let institutions = institution
            .limit(*limit)
            .offset(*offset)
            .select(Self::as_select())
            .load(conn)
            .await?;

        Ok(institutions)
    }

    async fn fetch_by_id(id: Self::Id, conn: &mut AsyncPgConnection) -> super::Result<Self> {
        use schema::institution::dsl::institution;

        let found = institution
            .find(id)
            .select(Self::as_select())
            .first(conn)
            .await?;

        Ok(found)
    }
}
