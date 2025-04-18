use diesel::{pg::Pg, prelude::*};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use garde::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use valuable::Valuable;

use super::{Create, Read, Update};
use crate::schema;

#[derive(Insertable, Deserialize, Clone, Valuable, JsonSchema, Validate)]
#[diesel(table_name = schema::institution, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
pub struct NewInstitution {
    #[garde(length(min = 1))]
    pub name: String,
    #[valuable(skip)]
    pub ms_tenant_id: Option<Uuid>,
}

// We don't need to `impl Create` for an individual `Institution` because it's
// more efficient to just do batches
impl Create for Vec<NewInstitution> {
    type Returns = Vec<Institution>;

    async fn create(self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
        use schema::institution::dsl::*;

        let inserted = diesel::insert_into(institution)
            .values(self)
            .returning(Institution::as_returning())
            .get_results(conn)
            .await?;

        Ok(inserted)
    }
}

// It's unlikely we'll need this, but it serves as a simple example for the
// patterns I want to establish in this package
#[derive(Identifiable, AsChangeset, Deserialize, JsonSchema, Validate)]
#[diesel(table_name = schema::institution, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
pub struct UpdatedInstitution {
    id: Uuid,
    #[garde(length(min = 1))]
    name: Option<String>,
    ms_tenant_id: Option<Uuid>,
}

impl Update for UpdatedInstitution {
    type Returns = Institution;

    async fn update(self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
        Ok(diesel::update(&self)
            .set(&self)
            .returning(Self::Returns::as_returning())
            .get_result(conn)
            .await?)
    }
}

#[derive(Queryable, Selectable, Serialize, JsonSchema)]
#[diesel(table_name = schema::institution, check_for_backend(Pg))]
pub struct Institution {
    pub id: Uuid,
    pub name: String,
    link: String,
}

impl Read for Institution {
    type Id = Uuid;
    type QueryParams = ();

    async fn fetch_by_id(id: &Self::Id, conn: &mut AsyncPgConnection) -> super::Result<Self> {
        use schema::institution::dsl::institution;

        let found = institution.find(id).select(Self::as_select()).first(conn).await?;

        Ok(found)
    }

    async fn fetch_many(_filter: &Self::QueryParams, conn: &mut AsyncPgConnection) -> super::Result<Vec<Self>> {
        use schema::institution::dsl::institution;

        let institutions = institution.select(Self::as_select()).load(conn).await?;

        Ok(institutions)
    }
}
