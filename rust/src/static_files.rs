use camino::{Utf8Path, Utf8PathBuf};
use diesel_async::AsyncPgConnection;
use serde::Deserialize;
use valuable::Valuable;

use crate::db::{self, institution::Institution, Upsert};

pub async fn synchronize(files: &[Utf8PathBuf], db_conn: &mut AsyncPgConnection) {
    for file in files {
        if let Err(err) = sync_db_with_file(file, db_conn).await {
            tracing::error!(
                error = err.as_value(),
                "failed to synchronize static data file {file}"
            );
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StaticData {
    Institutions(Vec<Institution>),
}

async fn sync_db_with_file(path: &Utf8Path, db_conn: &mut AsyncPgConnection) ->  db::Result<()>{
    use StaticData::Institutions;

    // fix unwraps
    let contents = std::fs::read_to_string(path).unwrap();
    let data: StaticData = serde_json::from_str(&contents).unwrap();

    let updated_data = match data {
        Institutions(mut new_institutions) => new_institutions.upsert(db_conn).await?
    };

    // fix unwrap
    let as_json = serde_json::to_vec_pretty(&updated_data).unwrap();

    std::fs::write(path, as_json).unwrap();

    Ok(())
}
