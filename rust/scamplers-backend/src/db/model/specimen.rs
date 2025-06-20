use crate::db::{
    self,
    model::{AsDieselFilter, FetchById, FetchRelatives, Write, sample_metadata},
    util::{AsIlike, BoxedDieselExpression, NewBoxedDieselExpression},
};
use diesel::{dsl::AssumeNotNull, prelude::*};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use scamplers_core::model::specimen::{
    BlockEmbeddingMatrix, Fixative, NewSpecimen, NewSpecimenMeasurement, Specimen, SpecimenCore,
    SpecimenData, SpecimenMeasurement, SpecimenQuery, block::NewBlock, tissue::NewTissue,
};
use scamplers_schema::{
    person,
    sample_metadata::{
        name as name_col, notes as notes_col, received_at as received_at_col,
        species as species_col,
    },
    specimen::{
        self, cryopreserved as cryopreserved_col, embedded_in as embedding_col,
        fixative as fixative_col, frozen as frozen_col, id as id_col, storage_buffer as buffer_col,
        type_ as type_col,
    },
    specimen_measurement,
};
use uuid::Uuid;

macro_rules! write_specimen_variant {
    ($specimen_variant:ident, $db_conn:ident) => {{
        diesel::insert_into(specimen::table)
            .values($specimen_variant)
            .returning(SpecimenData::as_select())
            .get_result($db_conn)
            .await?
    }};
}

#[diesel::dsl::auto_type]
fn specimen_measurement_query_base() -> _ {
    specimen_measurement::table.inner_join(person::table)
}

impl FetchRelatives<SpecimenMeasurement> for specimen::table {
    type Id = Uuid;

    async fn fetch_relatives(
        id: &Self::Id,
        db_conn: &mut AsyncPgConnection,
    ) -> db::error::Result<Vec<SpecimenMeasurement>> {
        Ok(specimen_measurement_query_base()
            .filter(specimen_measurement::specimen_id.eq(id))
            .select(SpecimenMeasurement::as_select())
            .load(db_conn)
            .await?)
    }
}

impl Write for Vec<NewSpecimenMeasurement> {
    type Returns = Vec<SpecimenMeasurement>;

    async fn write(self, db_conn: &mut AsyncPgConnection) -> db::error::Result<Self::Returns> {
        let specimen_ids: Vec<Uuid> = diesel::insert_into(specimen_measurement::table)
            .values(&self)
            .returning(specimen_measurement::specimen_id)
            .get_results(db_conn)
            .await?;

        if specimen_ids.is_empty() {
            return Ok(vec![]);
        }

        specimen::table::fetch_relatives(&specimen_ids[0], db_conn).await
    }
}

impl Write for NewSpecimen {
    type Returns = Specimen;
    async fn write(
        mut self,
        db_conn: &mut diesel_async::AsyncPgConnection,
    ) -> crate::db::error::Result<Self::Returns> {
        let metadata = self.metadata().write(db_conn).await?;
        self.set_metadata_id(*metadata.id());

        let data = match &self {
            Self::Block(block) => match block {
                NewBlock::Fixed(block) => write_specimen_variant!(block, db_conn),
                NewBlock::Frozen(block) => write_specimen_variant!(block, db_conn),
            },
            Self::Tissue(tissue) => match tissue {
                NewTissue::Cryopreserved(tissue) => write_specimen_variant!(tissue, db_conn),
                NewTissue::Fixed(tissue) => write_specimen_variant!(tissue, db_conn),
                NewTissue::Frozen(tissue) => write_specimen_variant!(tissue, db_conn),
            },
        };

        let new_measurements = self.measurements(*data.id());
        let measurements = new_measurements.write(db_conn).await?;

        let specimen_core = SpecimenCore::builder()
            .metadata(metadata)
            .data(data)
            .build();

        Ok(Specimen::builder()
            .core(specimen_core)
            .measurements(measurements)
            .build())
    }
}

#[diesel::dsl::auto_type]
#[must_use]
pub fn core_query_base() -> _ {
    specimen::table.inner_join(sample_metadata::query_base())
}

impl FetchById for Specimen {
    type Id = Uuid;
    async fn fetch_by_id(
        id: &Self::Id,
        db_conn: &mut AsyncPgConnection,
    ) -> db::error::Result<Self> {
        let specimen_core = core_query_base()
            .select(SpecimenCore::as_select())
            .filter(id_col.eq(id))
            .first(db_conn)
            .await?;

        let measurements = specimen::table::fetch_relatives(specimen_core.id(), db_conn).await?;

        Ok(Specimen::builder()
            .core(specimen_core)
            .measurements(measurements)
            .build())
    }
}

impl<QueryExpression> AsDieselFilter<QueryExpression> for SpecimenQuery
where
    id_col: SelectableExpression<QueryExpression>,
    name_col: SelectableExpression<QueryExpression>,
    received_at_col: SelectableExpression<QueryExpression>,
    species_col: SelectableExpression<QueryExpression>,
    AssumeNotNull<notes_col>: SelectableExpression<QueryExpression>,
    type_col: SelectableExpression<QueryExpression>,
    AssumeNotNull<embedding_col>: SelectableExpression<QueryExpression>,
    AssumeNotNull<fixative_col>: SelectableExpression<QueryExpression>,
    AssumeNotNull<buffer_col>: SelectableExpression<QueryExpression>,
    frozen_col: SelectableExpression<QueryExpression>,
    cryopreserved_col: SelectableExpression<QueryExpression>,
{
    fn as_diesel_filter<'a>(
        &'a self,
    ) -> Option<db::util::BoxedDieselExpression<'a, QueryExpression>>
    where
        QueryExpression: 'a,
    {
        let Self {
            ids,
            metadata,
            type_,
            embedded_in,
            fixative,
            storage_buffer,
            frozen,
            cryopreserved,
            ..
        } = self;

        let mut query = BoxedDieselExpression::new_expression();

        if !ids.is_empty() {
            query = query.and_condition(id_col.eq_any(ids));
        }

        if let Some(metadata) = metadata {
            if let Some(metadata_query) = metadata.as_diesel_filter() {
                query = query.and_condition(metadata_query);
            }
        }

        if let Some(type_) = type_ {
            query = query.and_condition(type_col.eq(type_));
        }

        if let Some(embedded_in) = embedded_in {
            match embedded_in {
                BlockEmbeddingMatrix::Fixed(e) => {
                    query = query.and_condition(embedding_col.assume_not_null().eq(e));
                }
                BlockEmbeddingMatrix::Frozen(e) => {
                    query = query.and_condition(embedding_col.assume_not_null().eq(e));
                }
            }
        }

        if let Some(fixative) = fixative {
            match fixative {
                Fixative::Block(f) => {
                    query = query.and_condition(fixative_col.assume_not_null().eq(f));
                }
                Fixative::Tissue(f) => {
                    query = query.and_condition(fixative_col.assume_not_null().eq(f));
                }
            }
        }

        if let Some(storage_buffer) = storage_buffer {
            query = query.and_condition(
                buffer_col
                    .assume_not_null()
                    .ilike(storage_buffer.as_ilike()),
            );
        }

        if let Some(frozen) = frozen {
            query = query.and_condition(frozen_col.eq(frozen));
        }

        if let Some(cryopreserved) = cryopreserved {
            query = query.and_condition(cryopreserved_col.eq(cryopreserved));
        }

        query.build()
    }
}
