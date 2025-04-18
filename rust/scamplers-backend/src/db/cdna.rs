use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use garde::Validate;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    Create,
    library_type_specification::{self, LibraryType, LibraryTypeGroup},
    nucleic_acid_measurement,
    utils::{BelongsToExt, DefaultNowNaiveDateTime, JunctionStruct, Parent},
};
use crate::{db::utils::ParentSet, schema};
const N_MEASUREMENTS_PER_CDNA: usize = 2;
const N_PREPARERS_PER_CDNA: usize = 2;

#[derive(Insertable, Deserialize, Serialize, Validate)]
#[diesel(table_name = schema::cdna_measurement, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
struct CdnaMeasurement {
    #[serde(default)]
    cdna_id: Uuid,
    measured_by: Uuid,
    #[garde(dive)]
    data: nucleic_acid_measurement::MeasurementData,
}
impl BelongsToExt<NewCdna> for CdnaMeasurement {
    fn set_parent_id(&mut self, parent_id: Uuid) {
        self.cdna_id = parent_id;
    }
}

impl Create for Vec<CdnaMeasurement> {
    type Returns = ();

    async fn create(self, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self::Returns> {
        use schema::cdna_measurement::dsl::cdna_measurement;

        diesel::insert_into(cdna_measurement)
            .values(&self)
            .execute(conn)
            .await?;

        Ok(())
    }
}

#[derive(Deserialize, Insertable, Validate)]
#[diesel(table_name = schema::cdna, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
struct NewCdna {
    library_type: LibraryType,
    legacy_id: String,
    #[serde(default)]
    prepared_at: DefaultNowNaiveDateTime,
    gems_id: Uuid,
    #[garde(range(min = 1))]
    n_amplification_cycles: i32,
    storage_location: Option<String>,
    notes: Option<Vec<String>>,
    #[diesel(skip_insertion)]
    #[garde(dive)]
    measurements: Vec<CdnaMeasurement>,
    #[diesel(skip_insertion)]
    #[garde(length(min = 1))]
    preparer_ids: Vec<Uuid>,
}
impl Parent<CdnaMeasurement> for NewCdna {
    fn owned_children(&mut self) -> Vec<CdnaMeasurement> {
        self.measurements.drain(..).collect()
    }
}
trait CdnaGroup {
    async fn validate_library_types(&self, conn: &mut diesel_async::AsyncPgConnection) -> crate::db::Result<()>;
}
impl CdnaGroup for Vec<&NewCdna> {
    async fn validate_library_types(&self, conn: &mut diesel_async::AsyncPgConnection) -> crate::db::Result<()> {
        use schema::{
            chemistry::{library_types as library_types_col, table as chemistry_table},
            gems::{id, table as gems_table},
        };

        let Some(NewCdna { gems_id, .. }) = self.get(0) else {
            return Ok(());
        };

        let lib_types: Vec<_> = self.iter().map(|NewCdna { library_type, .. }| *library_type).collect();
        lib_types.validate()?;

        let err = || library_type_specification::Error::new(lib_types.clone());

        let expected_lib_types: Option<Vec<LibraryType>> = gems_table
            .filter(id.eq(gems_id))
            .left_join(chemistry_table)
            .select(library_types_col.nullable())
            .first(conn)
            .await?;
        let Some(mut expected_lib_types) = expected_lib_types else {
            if lib_types != [LibraryType::AntibodyCapture] {
                return Err(err().into());
            }
            return Ok(());
        };

        expected_lib_types.sort();
        if lib_types != expected_lib_types {
            return Err(err().into());
        }

        Ok(())
    }
}

#[derive(Insertable)]
#[diesel(table_name = schema::cdna_preparers)]
struct CdnaPreparer {
    cdna_id: Uuid,
    prepared_by: Uuid,
}

impl JunctionStruct for CdnaPreparer {
    fn new(cdna_id: Uuid, prepared_by: Uuid) -> Self {
        Self { cdna_id, prepared_by }
    }
}
impl Create for Vec<CdnaPreparer> {
    type Returns = ();

    async fn create(self, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self::Returns> {
        use schema::cdna_preparers::dsl::cdna_preparers;

        diesel::insert_into(cdna_preparers).values(&self).execute(conn).await?;

        Ok(())
    }
}

impl Create for Vec<NewCdna> {
    type Returns = ();

    async fn create(mut self, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self::Returns> {
        use schema::cdna::dsl::{cdna, id};

        let n_cdnas = self.len();

        let grouped_cdnas = self.iter().into_group_map_by(|c| c.gems_id);
        for group in grouped_cdnas.values() {
            group.validate_library_types(conn).await?;
        }

        let cdna_ids = diesel::insert_into(cdna)
            .values(&self)
            .returning(id)
            .get_results(conn)
            .await?;

        let flattened_measurements = self.flatten_children_and_set_ids(&cdna_ids, N_MEASUREMENTS_PER_CDNA * n_cdnas);
        flattened_measurements.create(conn).await?;

        let preparer_id_sets = self.into_iter().map(|NewCdna { preparer_ids, .. }| preparer_ids);

        let cdna_preparers =
            CdnaPreparer::from_ids_grouped_by_parent1(cdna_ids, preparer_id_sets, N_PREPARERS_PER_CDNA * n_cdnas);

        cdna_preparers.create(conn).await?;

        Ok(())
    }
}
