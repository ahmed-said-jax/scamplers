use std::{collections::HashSet, hash::Hash, sync::LazyLock};

use diesel::{expression::AsExpression, prelude::*, sql_types};
use diesel_async::RunQueryDsl;
use garde::Validate;
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
        let Some(NewSingleIndexSet(IndexSetName(index_set_name), ..)) = self.get(0).cloned() else {
            return Ok(Vec::with_capacity(0));
        };

        let error_message = "malformed sample index set".to_string();

        // We expect a very specific structure for these
        let Some(kit_name) = index_set_name.get(3..5) else {
            return Err(super::Error::Other {
                message: error_message,
            });
        };

        diesel::insert_into(index_kit::table)
            .values(index_kit::name.eq(kit_name))
            .on_conflict(index_kit::name)
            .do_nothing()
            .execute(conn)
            .await?;

        // Doing this all in a functional manner becomes cumbersome
        let mut insert_structs = Vec::with_capacity(self.len());
        for index_set in self {
            let NewSingleIndexSet(
                IndexSetName(n),
                [
                    DnaSequence(s1),
                    DnaSequence(s2),
                    DnaSequence(s3),
                    DnaSequence(s4),
                ],
            ) = index_set;

            let Some(well_name) = n.get(6..8) else {
                return Err(super::Error::Other {
                    message: error_message,
                });
            };

            // It looks like we're cloning but we're not because we're just cloning
            // references
            insert_structs.push(SingleIndexSet {
                name: n.as_str(),
                kit: kit_name,
                well: well_name,
                sequences: vec![s1, s2, s3, s4],
            });
        }

        // This is kinda dumb because we're literally getting out the same thing we put
        // in
        let inserted = diesel::insert_into(single_index_set::table)
            .values(insert_structs)
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
