use std::{collections::HashMap, fmt::Display, hash::Hash, sync::LazyLock};

use diesel::{expression::AsExpression, prelude::*, sql_types};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use garde::{error::PathComponentKind, Validate};
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::Create;
use crate::schema::{dual_index_set, index_kit, single_index_set};

static INDEX_SET_NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^SI-[A-Z]{2}-[A-Z]\d{1,2}$").unwrap()); // this should be improved
static DNA_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[ACGT]+$").unwrap()); // so should this

#[derive(Deserialize, Validate, Hash, PartialEq, Eq, Clone)]
#[garde(transparent)]
struct IndexSetName(#[garde(pattern(INDEX_SET_NAME_REGEX))] String);
impl Display for IndexSetName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
const INDEX_SET_NAME_ERROR_MESSAGE: &str = "malformed index set name";

impl IndexSetName {
    fn kit_name(&self) -> super::Result<&str> {
        Ok(self.0.get(3..5).ok_or(super::Error::Other{message: INDEX_SET_NAME_ERROR_MESSAGE.to_string()})?)
    }

    fn well_name(&self) -> super::Result<&str> {
        Ok(self.0.get(6..8).ok_or(super::Error::Other{message: INDEX_SET_NAME_ERROR_MESSAGE.to_string()})?)
    }
}

impl PathComponentKind for IndexSetName {
    fn component_kind() -> garde::error::Kind {
        // I guess?
        garde::error::Kind::Key
    }
}

async fn insert_kit_name(kit_name: &str, conn: &mut AsyncPgConnection) -> super::Result<()> {
    diesel::insert_into(index_kit::table)
    .values(index_kit::name.eq(kit_name))
    .on_conflict(index_kit::name)
    .do_nothing()
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
impl Create for Vec<NewSingleIndexSet> {
    type Returns = Vec<SingleIndexSet<String>>;

    // return an owned type

    // We should technically validate the fact that this whole set has the same kit,
    // but it doesn't really matter because this won't be exposed as an API route -
    // we are downloading these files from 10X ourselves.
    async fn create(
        &mut self,
        conn: &mut diesel_async::AsyncPgConnection,
    ) -> super::Result<Self::Returns> {
        // This one clone is necessary
        let Some(NewSingleIndexSet(index_set_name, ..)) = self.get(0).cloned() else {
            return Ok(Vec::with_capacity(0));
        };

        // We expect a very specific structure for these
        let kit_name = index_set_name.kit_name()?;
        insert_kit_name(kit_name, conn).await?;

        // Doing this all in a functional manner becomes cumbersome
        let mut insertables = Vec::with_capacity(self.len());
        for index_set in self {
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

        // This is kinda dumb because we're literally getting out the same thing we put
        // in
        let inserted = diesel::insert_into(single_index_set::table)
            .values(insertables)
            .returning(SingleIndexSet::as_returning())
            .get_results(conn)
            .await?;

        Ok(inserted)
    }
}

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
    for<'a> &'a Str: AsExpression<sql_types::Text> {
    name: Str,
    kit: Str,
    well: Str,
    index_i7: Str,
    index2_workflow_a_i5: Str,
    index2_workflow_b_i5: Str
}

impl Create for HashMap<IndexSetName, NewDualIndexSet> {
    type Returns = Vec<DualIndexSet<String>>;

    async fn create(&mut self, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self::Returns> {
        let Some(index_set_name) = self.keys().next().cloned() else {
            return Ok(Vec::with_capacity(0));
        };

        let kit_name = index_set_name.kit_name()?;

        let mut insertables = Vec::with_capacity(self.len());
        for (index_set_name, NewDualIndexSet{index_i7, index2_workflow_a_i5, index2_workflow_b_i5}) in self {
            let well_name = index_set_name.well_name()?;

            insertables.push(DualIndexSet {name: index_set_name.0.as_str(), kit: kit_name, well: well_name, index_i7: &index_i7.0, index2_workflow_a_i5: &index2_workflow_a_i5.0, index2_workflow_b_i5: &index2_workflow_b_i5.0});
        }

        let inserted = diesel::insert_into(dual_index_set::table).values(insertables).returning(DualIndexSet::as_returning()).get_results(conn).await?;

        Ok(inserted)
    }
}

#[derive(Deserialize, Validate)]
#[serde(untagged)]
enum IndexSetFile {
    Single(#[garde(dive)] Vec<NewSingleIndexSet>),
    Dual(#[garde(dive)] HashMap<IndexSetName, NewDualIndexSet>)
}