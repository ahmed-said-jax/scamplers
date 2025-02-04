use std::collections::HashMap;

use garde::Validate;
use serde::Deserialize;
use url::Url;

use crate::{
    AppState,
    db::{
        Create,
        index_sets::{IndexSetName, NewDualIndexSet, NewSingleIndexSet},
    },
};

#[derive(Deserialize, Validate)]
#[serde(untagged)]
pub enum IndexSetFile {
    SingleIndexSets(#[garde(dive)] Vec<NewSingleIndexSet>),
    DualIndexSets(#[garde(dive)] HashMap<IndexSetName, NewDualIndexSet>),
}

// We use anyhow::Result here because we just want to know what went wrong, we
// don't care about serializing structured data to a client
impl IndexSetFile {
    pub async fn download_and_insert(
        AppState {
            db_pool,
            http_client,
            ..
        }: AppState,
        url: Url,
    ) -> anyhow::Result<()> {
        let mut conn = db_pool.get().await?;
        let data: Self = http_client.get(url).send().await?.json().await?;

        match data {
            Self::DualIndexSets(mut sets) => sets.create(&mut conn).await?,
            Self::SingleIndexSets(mut sets) => sets.create(&mut conn).await?,
        };

        Ok(())
    }
}
