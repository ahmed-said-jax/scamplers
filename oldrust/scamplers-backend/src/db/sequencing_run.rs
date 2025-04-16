use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use uuid::Uuid;
use valuable::Valuable;

use super::{
    Create,
    utils::{DefaultNowNaiveDateTime, JunctionStruct},
};
use crate::schema::{self, chromium_sequencing_submissions, sequencing_run};

#[derive(Insertable, Deserialize, Valuable)]
#[diesel(table_name = schema::sequencing_run, check_for_backend(Pg))]
struct NewSequencingRun {
    legacy_id: String,
    #[valuable(skip)]
    #[serde(default)]
    begun_at: DefaultNowNaiveDateTime,
    #[valuable(skip)]
    finished_at: Option<NaiveDateTime>,
    notes: Option<Vec<String>>,
    #[diesel(skip_insertion)]
    #[valuable(skip)]
    libraries: Vec<NewSequencingSubmission>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = schema::chromium_sequencing_submissions, check_for_backend(Pg))]
struct NewSequencingSubmission {
    #[serde(default)]
    sequencing_run_id: Uuid,
    library_id: Uuid,
    fastq_paths: Option<Vec<String>>, /* TODO: write a validation function to ensure these are absolute paths
                                       * following a specific naming scheme */
    #[serde(default)]
    submitted_at: DefaultNowNaiveDateTime,
}

impl JunctionStruct<Uuid, NewSequencingSubmission> for NewSequencingSubmission {
    fn new(sequencing_run_id: Uuid, mut submission: NewSequencingSubmission) -> Self {
        submission.sequencing_run_id = sequencing_run_id;
        submission
    }
}

impl Create for Vec<NewSequencingSubmission> {
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

        let libraries = self.into_iter().map(|NewSequencingRun { libraries, .. }| libraries);
        let sequencing_submissions = NewSequencingSubmission::from_ids_grouped_by_parent1(
            new_run_ids,
            libraries,
            N_LIBS_PER_SEQUENCING_RUNS * n_sequencing_runs,
        );

        sequencing_submissions.create(conn).await?;

        Ok(())
    }
}
