use crate::db::{
    model::{AsDieselFilter, FetchById, Write},
    util::{AsIlike, BoxedDieselExpression, NewBoxedDieselExpression},
};
use diesel::{dsl::AssumeNotNull, prelude::*};
use diesel_async::RunQueryDsl;
use sample_metadata::dsl::id as id_col;
use scamplers_core::model::sample_metadata::{
    NewSampleMetadata, SampleMetadata, SampleMetadataQuery,
};
use scamplers_schema::{
    committee_approval, institution, lab, person,
    sample_metadata::{
        self, name as name_col, notes as notes_col, received_at as received_at_col,
        species as species_col,
    },
};
use uuid::Uuid;

#[diesel::dsl::auto_type]
#[must_use]
pub fn summary_query_base() -> _ {
    sample_metadata::table
}

diesel::alias!(person as returned_by: ReturnedByAlias);

#[diesel::dsl::auto_type]
#[must_use]
pub fn query_base() -> _ {
    let submitter_join_condition = sample_metadata::submitted_by.eq(person::id);
    let returner_join_condition =
        sample_metadata::returned_by.eq(returned_by.field(person::id).nullable());

    summary_query_base()
        .inner_join(person::table.on(submitter_join_condition))
        .left_join(returned_by.on(returner_join_condition))
        .inner_join(lab::table)
}

impl FetchById for SampleMetadata {
    type Id = Uuid;

    async fn fetch_by_id(
        id: &Self::Id,
        db_conn: &mut diesel_async::AsyncPgConnection,
    ) -> crate::db::error::Result<Self> {
        Ok(query_base()
            .filter(id_col.eq(id))
            .select(SampleMetadata::as_select())
            .first(db_conn)
            .await?)
    }
}

#[diesel::dsl::auto_type]
#[must_use]
pub fn committee_approval_query_base() -> _ {
    committee_approval::table.inner_join(institution::table)
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

        SampleMetadata::fetch_by_id(&id, db_conn).await
    }
}

impl<QueryExpression> AsDieselFilter<QueryExpression> for SampleMetadataQuery
where
    name_col: SelectableExpression<QueryExpression>,
    received_at_col: SelectableExpression<QueryExpression>,
    species_col: SelectableExpression<QueryExpression>,
    AssumeNotNull<notes_col>: SelectableExpression<QueryExpression>,
{
    fn as_diesel_filter<'a>(&'a self) -> Option<BoxedDieselExpression<'a, QueryExpression>>
    where
        QueryExpression: 'a,
    {
        let Self {
            name,
            received_before,
            received_after,
            species,
            notes,
        } = self;

        let mut query = BoxedDieselExpression::new_expression();

        if let Some(name) = name {
            query = query.and_condition(name_col.ilike(name.as_ilike()));
        }

        if let Some(received_before) = received_before {
            query = query.and_condition(received_at_col.lt(received_before));
        }

        if let Some(received_after) = received_after {
            query = query.and_condition(received_at_col.gt(received_after));
        }

        if !species.is_empty() {
            query = query.and_condition(species_col.overlaps_with(species));
        }

        if let Some(notes) = notes {
            query = query.and_condition(notes_col.assume_not_null().ilike(notes.as_ilike()));
        }

        query.build()
    }
}
