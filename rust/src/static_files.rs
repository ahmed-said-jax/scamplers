use camino::{Utf8Path, Utf8PathBuf};
use diesel_async::AsyncPgConnection;
use serde::Deserialize;
use valuable::Valuable;

use crate::db::{institution::NewInstitution, Create};

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

#[derive(thiserror::Error, Debug, Valuable)]
enum Error {
    #[error(transparent)]
    Database(#[from] super::db::Error),
    #[error("{0}")]
    SerdeJson(String),
    #[error("{0}")]
    Io(String),
}
type Result<T> = std::result::Result<T, Error>;
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeJson(err.to_string())
    }
}
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StaticData {
    Institutions(Vec<NewInstitution>),
}

async fn sync_db_with_file(path: &Utf8Path, db_conn: &mut AsyncPgConnection) -> Result<()> {
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
