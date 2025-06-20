use crate::db::{
    self,
    model::{FetchById, FetchRelatives, Write, sample_metadata},
};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use scamplers_core::model::specimen::{
    NewSpecimen, NewSpecimenMeasurement, Specimen, SpecimenCore, SpecimenData, SpecimenMeasurement,
    block::NewBlock, tissue::NewTissue,
};
use scamplers_schema::{
    person,
    specimen::{self, id as id_col},
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
