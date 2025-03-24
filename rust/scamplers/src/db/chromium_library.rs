use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use garde::Validate;
use serde::Deserialize;
use uuid::Uuid;

use super::{
    Create, nucleic_acid_measurement,
    utils::{BelongsToExt, DefaultNowNaiveDateTime, JunctionStruct, Parent},
};
use crate::{
    db::utils::ParentSet,
    schema,
};

#[derive(Deserialize, Insertable, Validate)]
#[diesel(table_name = schema::chromium_library_measurement, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
struct LibraryMeasurement {
    #[serde(default)]
    library_id: Uuid,
    measured_by: Uuid,
    data: nucleic_acid_measurement::MeasurementData,
}
impl BelongsToExt<NewChromiumLibrary> for LibraryMeasurement {
    fn set_parent_id(&mut self, parent_id: Uuid) {
        self.library_id = parent_id;
    }
}
impl Create for Vec<LibraryMeasurement> {
    type Returns = ();

    async fn create(self, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self::Returns> {
        use schema::chromium_library_measurement::dsl::*;

        diesel::insert_into(chromium_library_measurement)
            .values(&self)
            .execute(conn)
            .await?;

        Ok(())
    }
}

#[derive(Deserialize, Insertable, Validate)]
#[diesel(table_name = schema::chromium_library_preparers, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
struct LibraryPreparer {
    library_id: Uuid,
    prepared_by: Uuid,
}
impl JunctionStruct for LibraryPreparer {
    fn new(library_id: Uuid, prepared_by: Uuid) -> Self {
        Self {
            library_id,
            prepared_by,
        }
    }
}
impl Create for Vec<LibraryPreparer> {
    type Returns = ();

    async fn create(self, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self::Returns> {
        use schema::chromium_library_preparers::dsl::chromium_library_preparers;

        diesel::insert_into(chromium_library_preparers)
            .values(&self)
            .execute(conn)
            .await?;

        Ok(())
    }
}

#[derive(Deserialize, Insertable, Validate)]
#[diesel(table_name = schema::chromium_library, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
pub struct NewChromiumLibrary {
    legacy_id: String,
    cdna_id: Uuid,
    single_index_set_name: Option<String>,
    dual_index_set_name: Option<String>,
    #[garde(range(min = 1))]
    number_of_sample_index_pcr_cycles: i32,
    #[garde(range(min = 1000))]
    target_reads_per_cell: i32,
    #[serde(default)]
    prepared_at: DefaultNowNaiveDateTime,
    notes: Option<Vec<String>>,
    #[diesel(skip_insertion)]
    measurements: Vec<LibraryMeasurement>,
    #[diesel(skip_insertion)]
    #[garde(length(min = 1))]
    preparer_ids: Vec<Uuid>,
}
impl Parent<LibraryMeasurement> for NewChromiumLibrary {
    fn owned_children(&mut self) -> Vec<LibraryMeasurement> {
        self.measurements.drain(..).collect()
    }
}
impl Create for Vec<NewChromiumLibrary> {
    type Returns = ();

    async fn create(mut self, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self::Returns> {
        use schema::chromium_library::dsl::*;

        const N_MEASUREMENTS_PER_LIBRARY: usize = 4;
        const N_PREPARERS_PER_LIBRARY: usize = 4;
        let n_libs = self.len();

        let library_ids = diesel::insert_into(chromium_library)
            .values(&self)
            .returning(id)
            .get_results(conn)
            .await?;

        let flattened_measurements =
            self.flatten_children_and_set_ids(&library_ids, N_MEASUREMENTS_PER_LIBRARY * n_libs);
        flattened_measurements.create(conn).await?;

        let preparers = self
            .into_iter()
            .map(|NewChromiumLibrary { preparer_ids, .. }| preparer_ids);
        let preparers =
            LibraryPreparer::from_ids_grouped_by_parent1(library_ids, preparers, N_PREPARERS_PER_LIBRARY * n_libs);

        preparers.create(conn).await?;

        Ok(())
    }
}
