use diesel::{pg::Pg, prelude::*};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use garde::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use valuable::Valuable;

use super::{Create, Pagination, Read, Update};
use crate::{db::Paginate, schema};

#[derive(Insertable, Deserialize, Clone, Valuable, JsonSchema, Validate)]
#[diesel(table_name = schema::institution, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
pub struct NewInstitution {
    name: String,
    #[valuable(skip)]
    ms_tenant_id: Option<Uuid>,
}

// We don't need to `impl Create` for an individual `Institution` because it's
// more efficient to just do batches
impl Create for Vec<NewInstitution> {
    type Returns = Vec<Institution>;

    async fn create(&self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
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

// It's unlikely we'll need this, but it serves as a simple example for the
// patterns I want to establish in this package
#[derive(Identifiable, AsChangeset, Deserialize, JsonSchema)]
#[diesel(table_name = schema::institution, check_for_backend(Pg))]
pub struct UpdatedInstitution {
    id: Uuid,
    name: Option<String>,
    ms_tenant_id: Option<Uuid>,
}

impl Update for UpdatedInstitution {
    type Returns = Institution;

    async fn update(&self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
        let as_immut = &*self;

        Ok(diesel::update(as_immut)
            .set(as_immut)
            .returning(Self::Returns::as_returning())
            .get_result(conn)
            .await?)
    }
}

#[derive(Queryable, Selectable, Serialize, JsonSchema)]
#[diesel(table_name = schema::institution, check_for_backend(Pg))]
pub struct Institution {
    id: Uuid,
    name: String,
    link: String,
}
impl Paginate for () {}

impl Read for Institution {
    type Filter = ();
    type Id = Uuid;

    async fn fetch_many(
        filter: Self::Filter,
        conn: &mut AsyncPgConnection,
    ) -> super::Result<Vec<Self>> {
        use schema::institution::dsl::institution;

        // Calling this over and over again for all of our methods sucks, but it's the
        // simplest way to do it
        let Pagination { limit, offset } = filter.paginate();

        let institutions = institution
            .select(Self::as_select())
            .limit(limit)
            .offset(offset)
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
