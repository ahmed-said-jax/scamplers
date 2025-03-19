use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use uuid::Uuid;
use valuable::Valuable;

use super::{utils::MappingStruct, Create};
use crate::schema::{self, chromium_sequencing_submissions, sequencing_run};

#[derive(Insertable, Deserialize, Valuable)]
#[diesel(table_name = schema::sequencing_run, check_for_backend(Pg))]
struct NewSequencingRun {
    legacy_id: String,
    #[valuable(skip)]
    begun_at: NaiveDateTime,
    #[valuable(skip)]
    finished_at: Option<NaiveDateTime>,
    notes: Option<Vec<String>>,
    #[diesel(skip_insertion)]
    #[valuable(skip)]
    library_ids: Vec<Uuid>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = schema::chromium_sequencing_submissions, check_for_backend(Pg))]
struct SequencingSubmission {
    sequencing_run_id: Uuid,
    library_id: Uuid,
}

impl MappingStruct for SequencingSubmission {
    fn new(id1: Uuid, id2: Uuid) -> Self {
        Self {sequencing_run_id: id1, library_id: id2}
    }
}

impl Create for Vec<SequencingSubmission> {
    type Returns = ();

    async fn create(self, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self::Returns> {
        diesel::insert_into(chromium_sequencing_submissions::table)
            .values(self)
            .on_conflict_do_nothing()
            .execute(conn)
            .await?;

        Ok(())
    }
}

// This probably won't be a user-facing action, so we don't need to worry about returing a nice data structure
impl Create for Vec<NewSequencingRun> {
    type Returns = ();

    async fn create(self, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self::Returns> {
        use sequencing_run::id;

        const N_LIBS_PER_SEQUENCING_RUNS: usize = 20; // A generous heuristic

        let n_sequencing_runs = self.len();

        let new_run_ids: Vec<Uuid> = diesel::insert_into(sequencing_run::table)
            .values(&self)
            .returning(id)
            .get_results(conn)
            .await?;

        let library_ids = self.iter().map(|NewSequencingRun{library_ids, ..}| library_ids);
        let sequencing_submissions = SequencingSubmission::from_grouped_ids(&new_run_ids, library_ids, N_LIBS_PER_SEQUENCING_RUNS * n_sequencing_runs);

        sequencing_submissions.create(conn).await?;

        Ok(())
    }
}
