use std::{collections::HashMap, fs};

use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};
use diesel_async::{AsyncConnection, AsyncPgConnection};
use serde::Deserialize;
use crate::db::{institution::{NewInstitution, NewInstitutions}, Upsert, UpsertMany};

pub async fn synchronize(files: &[Utf8PathBuf], db_conn: &mut AsyncPgConnection) -> anyhow::Result<()> {
    for file in files.iter() {
        if let Err(err) = sync_db_with_file(file, db_conn).await {
            tracing::error!("failed to synchronize static data file {file}: {err}"); // change this to use valuable for better formatting
        }
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StaticData {
    Institutions(Vec<NewInstitution>),
}

async fn sync_db_with_file(path: &Utf8Path, db_conn: &mut AsyncPgConnection) -> anyhow::Result<()> {
    use StaticData::*;

    let contents = std::fs::read_to_string(path).context(format!("failed to read file {path}"))?;
    let data: StaticData = serde_json::from_str(&contents).context(format!("failed to synchronize static data file {path}"))?;

    match data {
        Institutions(new_institutions) => {UpsertMany(new_institutions).upsert(db_conn).await?;}
    }

    Ok(())
}