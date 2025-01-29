use camino::{Utf8Path, Utf8PathBuf};
use diesel_async::AsyncPgConnection;
use serde::Deserialize;

use crate::db::{institution::NewInstitution, Create};

pub async fn synchronize(files: &[Utf8PathBuf], db_conn: &mut AsyncPgConnection) {
    for file in files {
        if let Err(err) = sync_db_with_file(file, db_conn).await {
            tracing::error!(
                error = err.to_string(),
                "failed to synchronize static data file {file}"
            );
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StaticData {
    Institutions(Vec<NewInstitution>),
}

async fn sync_db_with_file(path: &Utf8Path, db_conn: &mut AsyncPgConnection) -> anyhow::Result<()> {
    use StaticData::Institutions;

    let contents = std::fs::read_to_string(path)?;
    let data: StaticData = serde_json::from_str(&contents)?;

    match data {
        Institutions(new_institutions) => {
            new_institutions.create(db_conn).await?;
        }
    }

    Ok(())
}
