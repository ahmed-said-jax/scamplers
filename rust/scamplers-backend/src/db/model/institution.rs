use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use scamplers_core::model::institution::{Institution, NewInstitution};
use scamplers_schema::institution::dsl::institution;

use crate::db::Write;

impl Write for NewInstitution {
    type Returns = Institution;
    async fn write(
        self,
        db_conn: &mut diesel_async::AsyncPgConnection,
    ) -> crate::db::error::Result<Self::Returns> {
        let inserted = diesel::insert_into(institution)
            .values(self)
            .returning(Institution::as_returning())
            .get_result(db_conn)
            .await?;

        Ok(inserted)
    }
}
