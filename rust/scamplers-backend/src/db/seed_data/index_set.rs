use std::{collections::HashMap, fmt::Display, hash::Hash, sync::LazyLock};

use diesel::{expression::AsExpression, prelude::*, sql_types};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use garde::{Validate, error::PathComponentKind};
use regex::Regex;
use serde::{Deserialize, Serialize};
use url::Url;

use super::Write;
use scamplers_schema::{dual_index_set, index_kit, single_index_set};

#[derive(Deserialize, Validate, Clone, Serialize, Debug)]
#[serde(transparent)]
pub(super) struct IndexSetFileUrl(#[garde(custom(is_10x_genomics_url))] Url);

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_10x_genomics_url(url: &Url, _: &()) -> garde::Result {
    let Some(domain) = url.domain() else {
        return Err(garde::Error::new("malformed URL"));
    };

    if domain != "cdn.10xgenomics.com" {
        return Err(garde::Error::new(format!(
            "URL domain must be 'cdn.10xgenomics.com', found {domain}"
        )));
    }

    Ok(())
}

// `anyhow::Result` is fine here because this isn't user-facing code
impl IndexSetFileUrl {
    pub(super) async fn download(self, http_client: reqwest::Client) -> anyhow::Result<IndexSets> {
        let Self(url) = self;

        let index_set: IndexSets = http_client.get(url.clone()).send().await?.json().await?;

        index_set.validate()?;

        Ok(index_set)
    }
}

#[derive(Deserialize, Validate)]
#[serde(untagged)]
pub(super) enum IndexSets {
    Single(#[garde(dive)] Vec<NewSingleIndexSet>),
    Dual(#[garde(dive)] HashMap<IndexSetName, NewDualIndexSet>),
}

impl Write for IndexSets {
    type Returns = ();

    async fn write(
        self,
        db_conn: &mut AsyncPgConnection,
    ) -> crate::db::error::Result<Self::Returns> {
        match self {
            Self::Single(sets) => sets.write(db_conn).await?,
            Self::Dual(sets) => sets.write(db_conn).await?,
        }

        Ok(())
    }
}

static INDEX_SET_NAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^SI-([NA]{2}|[TN]{2}|[GA]{2}|[TS]{2}|[TT]{2})-[A-H]\d{1,2}$").unwrap()
});
static DNA_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[ACGT]{8}|[ACGT]{10}$").unwrap());

#[derive(Deserialize, Validate, Hash, PartialEq, Eq, Clone)]
#[garde(transparent)]
pub struct IndexSetName(#[garde(pattern(INDEX_SET_NAME_REGEX))] String);
impl Display for IndexSetName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
const INDEX_SET_NAME_ERROR_MESSAGE: &str = "malformed index set name";

impl IndexSetName {
    fn kit_name(&self) -> crate::db::error::Result<&str> {
        self.0.get(3..5).ok_or(crate::db::error::Error::Other {
            message: INDEX_SET_NAME_ERROR_MESSAGE.to_string(),
        })
    }

    fn well_name(&self) -> super::super::error::Result<&str> {
        self.0.get(6..8).ok_or(super::super::error::Error::Other {
            message: INDEX_SET_NAME_ERROR_MESSAGE.to_string(),
        })
    }
}

impl PathComponentKind for IndexSetName {
    fn component_kind() -> garde::error::Kind {
        // I guess?
        garde::error::Kind::Key
    }
}

async fn insert_kit_name(
    kit_name: &str,
    conn: &mut AsyncPgConnection,
) -> super::super::error::Result<()> {
    diesel::insert_into(index_kit::table)
        .values(index_kit::name.eq(kit_name))
        .on_conflict_do_nothing()
        .execute(conn)
        .await?;

    Ok(())
}

#[derive(Deserialize, Validate, Clone)]
#[garde(transparent)]
struct DnaSequence(#[garde(pattern(DNA_REGEX))] String);

#[derive(Validate, Deserialize, Clone)]
pub struct NewSingleIndexSet(#[garde(dive)] IndexSetName, #[garde(dive)] [DnaSequence; 4]);

// It's expected that one sample index set is a Vec<NewSampleIndex>, so we can
// bake in some validation and do a bunch of things at once
impl Write for Vec<NewSingleIndexSet> {
    type Returns = ();

    // We should technically validate the fact that this whole set has the same kit,
    // but it doesn't really matter because this won't be exposed as an API route -
    // we are downloading these files from 10X ourselves.
    async fn write(
        self,
        conn: &mut diesel_async::AsyncPgConnection,
    ) -> super::super::error::Result<Self::Returns> {
        // This one clone is necessary :/
        #[allow(clippy::get_first)]
        let Some(NewSingleIndexSet(index_set_name, ..)) = self.get(0).cloned() else {
            return Ok(());
        };

        let kit_name = index_set_name.kit_name()?;
        insert_kit_name(kit_name, conn).await?;

        // Doing this all in a functional manner becomes cumbersome
        let mut insertables = Vec::with_capacity(self.len());
        for index_set in &self {
            let NewSingleIndexSet(
                index_set_name,
                [
                    DnaSequence(s1),
                    DnaSequence(s2),
                    DnaSequence(s3),
                    DnaSequence(s4),
                ],
            ) = index_set;

            let well_name = index_set_name.well_name()?;

            // It looks like we're cloning but we're not because we're just cloning
            // references
            insertables.push(SingleIndexSet {
                name: index_set_name.0.as_str(),
                kit: kit_name,
                well: well_name,
                sequences: vec![s1, s2, s3, s4],
            });
        }

        diesel::insert_into(single_index_set::table)
            .values(insertables)
            .on_conflict_do_nothing()
            .execute(conn)
            .await?;

        Ok(())
    }
}

// This and DualIndexSet are written generically so they can either borrow their
// data (useful for cases like insertion from a different type of struct) or own
// their data (useful for reading from the database)
#[derive(Queryable, Selectable, Insertable, Serialize)]
#[diesel(table_name = single_index_set, primary_key(name))]
pub struct SingleIndexSet<Str: AsRef<str> + AsExpression<sql_types::Text>>
where
    for<'a> &'a Str: AsExpression<sql_types::Text>,
{
    name: Str,
    kit: Str,
    well: Str,
    sequences: Vec<Str>,
}

#[derive(Deserialize, Validate)]
pub struct NewDualIndexSet {
    #[garde(dive)]
    #[serde(alias = "index(i7)")]
    index_i7: DnaSequence,
    #[garde(dive)]
    #[serde(alias = "index2_workflow_a(i5)")]
    index2_workflow_a_i5: DnaSequence,
    #[garde(dive)]
    #[serde(alias = "index2_workflow_b(i5)")]
    index2_workflow_b_i5: DnaSequence,
}

#[derive(Queryable, Selectable, Insertable, Serialize)]
#[diesel(table_name = dual_index_set, primary_key(name))]
pub struct DualIndexSet<Str: AsRef<str> + AsExpression<sql_types::Text>>
where
    for<'a> &'a Str: AsExpression<sql_types::Text>,
{
    name: Str,
    kit: Str,
    well: Str,
    index_i7: Str,
    index2_workflow_a_i5: Str,
    index2_workflow_b_i5: Str,
}

#[allow(clippy::implicit_hasher)]
impl Write for HashMap<IndexSetName, NewDualIndexSet> {
    type Returns = ();

    async fn write(
        self,
        conn: &mut diesel_async::AsyncPgConnection,
    ) -> crate::db::error::Result<Self::Returns> {
        let Some(index_set_name) = self.keys().next().cloned() else {
            return Ok(());
        };

        let kit_name = index_set_name.kit_name()?;
        insert_kit_name(kit_name, conn).await?;

        let mut insertables = Vec::with_capacity(self.len());
        for (
            index_set_name,
            NewDualIndexSet {
                index_i7,
                index2_workflow_a_i5,
                index2_workflow_b_i5,
            },
        ) in &self
        {
            let well_name = index_set_name.well_name()?;

            insertables.push(DualIndexSet {
                name: index_set_name.0.as_str(),
                kit: kit_name,
                well: well_name,
                index_i7: &index_i7.0,
                index2_workflow_a_i5: &index2_workflow_a_i5.0,
                index2_workflow_b_i5: &index2_workflow_b_i5.0,
            });
        }

        diesel::insert_into(dual_index_set::table)
            .values(insertables)
            .on_conflict_do_nothing()
            .execute(conn)
            .await?;

        Ok(())
    }
}
