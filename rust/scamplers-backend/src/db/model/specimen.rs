use crate::db::{self, model::Write};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use scamplers_core::model::specimen::{
    NewSpecimen, NewSpecimenMeasurement, Specimen, SpecimenCore, SpecimenMeasurement,
    block::NewBlock, tissue::NewTissue,
};
use scamplers_schema::{person, specimen, specimen_measurement};
use uuid::Uuid;

macro_rules! write_specimen_variant {
    ($specimen_variant:ident, $db_conn:ident) => {{
        diesel::insert_into(specimen::table)
            .values($specimen_variant)
            .returning(SpecimenCore::as_select())
            .get_result($db_conn)
            .await?
    }};
}

impl Write for Vec<NewSpecimenMeasurement> {
    type Returns = Vec<SpecimenMeasurement>;

    async fn write(self, db_conn: &mut AsyncPgConnection) -> db::error::Result<Self::Returns> {
        let sample_ids: Vec<Uuid> = diesel::insert_into(specimen_measurement::table)
            .values(&self)
            .returning(specimen_measurement::specimen_id)
            .get_results(db_conn)
            .await?;

        Ok(specimen_measurement::table
            .inner_join(person::table)
            .filter(specimen_measurement::specimen_id.eq(&sample_ids[0]))
            .select(SpecimenMeasurement::as_select())
            .load(db_conn)
            .await?)
    }
}

impl Write for NewSpecimen {
    type Returns = Specimen;
    async fn write(
        mut self,
        db_conn: &mut diesel_async::AsyncPgConnection,
    ) -> crate::db::error::Result<Self::Returns> {
        let metadata = self.metadata().write(db_conn).await?;
        self.set_metadata_id(*metadata.metadata_id());

        let core: SpecimenCore = match &self {
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

        let new_measurements = self.measurements(*core.id());
        let measurements = new_measurements.write(db_conn).await?;

        Ok(Specimen::builder()
            .metadata(metadata)
            .core(core)
            .measurements(measurements)
            .build())
    }
}
