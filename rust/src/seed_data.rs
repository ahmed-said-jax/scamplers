use camino::Utf8Path;
use diesel_async::AsyncPgConnection;
use serde::Deserialize;

use crate::db::{Create, institution::NewInstitution};

#[derive(Deserialize)]
#[serde(untagged)]
enum StaticData {
    Institutions(Vec<NewInstitution>),
}

pub(super) async fn insert(path: &Utf8Path, db_conn: &mut AsyncPgConnection) -> anyhow::Result<()> {
    use StaticData::Institutions;

    let contents = std::fs::read_to_string(path)?;
    let data: StaticData = serde_json::from_str(&contents)?;

    match data {
        Institutions(mut new_institutions) => new_institutions.create(db_conn).await?,
    };

    Ok(())
}
