use crate::db::model::Write;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use scamplers_core::model::sample_metadata::{NewSampleMetadata, SampleMetadataSummary};
use scamplers_schema::{committee_approval, sample_metadata};

impl Write for NewSampleMetadata {
    type Returns = SampleMetadataSummary;

    async fn write(
        mut self,
        db_conn: &mut diesel_async::AsyncPgConnection,
    ) -> crate::db::error::Result<Self::Returns> {
        let metadata = diesel::insert_into(sample_metadata::table)
            .values(&self)
            .returning(SampleMetadataSummary::as_select())
            .get_result(db_conn)
            .await?;

        let id = metadata.metadata_id();

        let committee_approvals = self.committee_approvals(*id);
        diesel::insert_into(committee_approval::table)
            .values(committee_approvals)
            .execute(db_conn)
            .await?;

        Ok(metadata)
    }
}
