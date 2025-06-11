use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use rstest::fixture;
use scamplers_core::model::institution::NewInstitution;
use uuid::Uuid;

use crate::db::model::Write;

trait TestDbConnection {
    async fn new() -> Self;
    #[allow(dead_code)]
    async fn set_user(&mut self, user_id: &Uuid);
    async fn populate_db(&mut self);
}

const DB_URL: Option<&'static str> = option_env!("SCAMPLERS_TEST_DB_URL");
pub const N_INSTITUTIONS: usize = 15;

impl TestDbConnection for AsyncPgConnection {
    async fn new() -> Self {
        let db_url = DB_URL.unwrap();

        AsyncPgConnection::establish(db_url)
            .await
            .expect(&format!("failed to connect to test database at {db_url}"))
    }

    async fn populate_db(&mut self) {
        fn catch_duplicate_record_err<T>(
            result: crate::db::error::Result<T>,
        ) -> crate::db::error::Result<Option<T>> {
            use crate::db::error::Error;

            if let Err(Error::DuplicateRecord { .. }) = result {
                return Ok(None);
            }

            return result.map(|val| Some(val));
        }

        for i in 0..N_INSTITUTIONS {
            let new = NewInstitution {
                id: Uuid::now_v7(),
                name: format!("institution{i}"),
            };

            catch_duplicate_record_err(new.write(self).await).unwrap();
        }
    }

    async fn set_user(&mut self, user_id: &Uuid) {
        diesel::sql_query(format!(r#"set role "{user_id}""#))
            .execute(self)
            .await
            .unwrap();
    }
}

#[fixture]
pub async fn db_conn() -> AsyncPgConnection {
    let mut conn = AsyncPgConnection::new().await;

    conn.populate_db().await;

    conn
}
