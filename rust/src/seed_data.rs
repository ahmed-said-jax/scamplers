use std::collections::HashMap;

use anyhow::Context;
use garde::Validate;
use serde::Deserialize;
use url::Url;

use crate::{
    db::{
        index_sets::{IndexSetFileUrl, IndexSetName, NewDualIndexSet, NewSingleIndexSet}, Create
    }, AppState
};

// We use anyhow::Result here because we just want to know what went wrong, we
// don't care about serializing structured data to a client
pub async fn download_and_insert_index_sets(AppState {db_pool, http_client, ..}: AppState, file_urls: &[IndexSetFileUrl]) -> anyhow::Result<()> {
    let downloads = file_urls.into_iter().map(|url| url.clone().download(http_client.clone()));
    let mut index_sets = futures::future::try_join_all(downloads).await.context("failed to download index set files")?;

    let mut conn = db_pool.get().await?;
    for set in &mut index_sets {
        set.create(&mut conn).await.context("failed to insert index sets into database")?;
    }

    Ok(())
}
