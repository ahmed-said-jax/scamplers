use scamplers::models::institution::{Institution, NewInstitution};

use super::Create;

// We don't need to `impl Create` for an individual `Institution` because it's
// more efficient to just do batches
impl Create for Vec<NewInstitution> {
    type Returns = Vec<Institution>;

    async fn create(&self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
        use schema::institution::dsl::*;

        let inserted = diesel::insert_into(institution)
            .values(self)
            .returning(Institution::as_returning())
            .get_results(conn)
            .await?;

        Ok(inserted)
    }
}

// impl Update for UpdatedInstitution {
//     type Returns = Institution;

//     async fn update(&self, conn: &mut AsyncPgConnection) -> super::Result<Self::Returns> {
//         let as_immut = &*self;

//         Ok(diesel::update(as_immut)
//             .set(as_immut)
//             .returning(Self::Returns::as_returning())
//             .get_result(conn)
//             .await?)
//     }
// }

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