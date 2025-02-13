use anyhow::Context;
use diesel_async::SimpleAsyncConnection;

use crate::{
    AppState2,
    db::{Create, index_sets::IndexSetFileUrl},
};

// We use anyhow::Result here because we just want to know what went wrong, we
// don't care about serializing structured data to a client
pub async fn download_and_insert_index_sets(
    app_state: AppState2,
    file_urls: &[IndexSetFileUrl],
) -> anyhow::Result<()> {
    // Clone is fine here because everything in AppState is meant to be cloned
    // (cheaply clonable)
    let AppState2::Prod { http_client, .. } = app_state.clone() else {
        return Err(anyhow::Error::msg(
            "index sets should only be downloaded in production builds",
        ));
    };

    let downloads = file_urls
        .into_iter()
        .map(|url| url.clone().download(http_client.clone()));
    let mut index_sets = futures::future::try_join_all(downloads)
        .await
        .context("failed to download index set files")?;

    // A for-loop is fine because this is like 10 URLs max, and each of these is a
    // bulk insert
    let mut conn = app_state.db_conn().await?;
    for set in &mut index_sets {
        set.create(&mut conn)
            .await
            .context("failed to insert index sets into database")?;
    }

    Ok(())
}

pub async fn insert_test_data(app_state: AppState2) -> anyhow::Result<()> {
    let db_setup = include_str!("../../dev-test_db.sql");

    let mut conn = app_state.db_conn().await?;
    conn.batch_execute(db_setup).await.context("failed to populate database with test data")?;

    Ok(())
}