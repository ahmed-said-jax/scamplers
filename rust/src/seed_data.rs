use camino::Utf8Path;
use diesel_async::AsyncPgConnection;
use serde::Deserialize;


#[derive(Deserialize)]
#[serde(untagged)]
enum SeedData {}

// TODO: we want to insert index sets and chemistries here
pub(super) async fn insert(path: &Utf8Path, db_conn: &mut AsyncPgConnection) -> anyhow::Result<()> {
    let contents = std::fs::read_to_string(path)?;
    let _data: SeedData = serde_json::from_str(&contents)?;

    Ok(())
}
