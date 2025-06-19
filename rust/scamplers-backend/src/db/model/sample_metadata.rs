use crate::db::model::{FetchById, Write};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use sample_metadata::dsl::id as id_col;
use scamplers_core::model::sample_metadata::{
    CommitteeApproval, NewSampleMetadata, SampleMetadata, SampleMetadataCore,
};
use scamplers_schema::{committee_approval, institution, lab, person, sample_metadata};
use uuid::Uuid;

#[diesel::dsl::auto_type]
#[must_use]
pub fn summary_query_base() -> _ {
    sample_metadata::table
}

diesel::alias!(person as returned_by: ReturnedByAlias);

#[diesel::dsl::auto_type]
#[must_use]
pub fn core_query_base() -> _ {
    let join_condition = sample_metadata::returned_by.eq(returned_by.field(person::id).nullable());

    summary_query_base()
        .inner_join(person::table.on(sample_metadata::submitted_by.eq(person::id)))
        .left_join(returned_by.on(join_condition))
        .inner_join(lab::table)
}

impl FetchById for SampleMetadata {
    type Id = Uuid;

    async fn fetch_by_id(
        id: &Self::Id,
        db_conn: &mut diesel_async::AsyncPgConnection,
    ) -> crate::db::error::Result<Self> {
        let core = core_query_base()
            .filter(id_col.eq(id))
            .select(SampleMetadataCore::as_select())
            .first(db_conn)
            .await?;

        let committee_approvals = committee_approval::table
            .inner_join(institution::table)
            .filter(committee_approval::sample_id.eq(id))
            .select(CommitteeApproval::as_select())
            .load(db_conn)
            .await?;

        Ok(SampleMetadata::builder()
            .core(core)
            .committee_approvals(committee_approvals)
            .build())
    }
}

impl Write for NewSampleMetadata {
    type Returns = SampleMetadata;

    async fn write(
        mut self,
        db_conn: &mut diesel_async::AsyncPgConnection,
    ) -> crate::db::error::Result<Self::Returns> {
        let id = diesel::insert_into(sample_metadata::table)
            .values(&self)
            .returning(id_col)
            .get_result(db_conn)
            .await?;

        let committee_approvals = self.committee_approvals(id);
        diesel::insert_into(committee_approval::table)
            .values(committee_approvals)
            .execute(db_conn)
            .await?;

        let core = core_query_base()
            .filter(id_col.eq(id))
            .select(SampleMetadataCore::as_select())
            .first(db_conn)
            .await?;

        let committee_approvals = committee_approval::table
            .inner_join(institution::table)
            .filter(committee_approval::sample_id.eq(id))
            .select(CommitteeApproval::as_select())
            .load(db_conn)
            .await?;

        Ok(SampleMetadata::builder()
            .core(core)
            .committee_approvals(committee_approvals)
            .build())
    }
}
