use crate::schema;
use chrono::NaiveDateTime;
use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types::{self, SqlType},
};
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::utils::{BelongsToExt, DbEnum};

#[derive(
    Debug,
    Deserialize,
    Serialize,
    strum::IntoStaticStr,
    strum::EnumString,
    strum::VariantArray,
    FromSqlRow,
    AsExpression,
    SqlType,
    Default,
    Clone,
    Copy,
)]
#[diesel(sql_type = sql_types::Text)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
enum ChromiumChip {
    J,
    H,
    Q,
    GemxFx,
    #[serde(rename = "gemx_3p")]
    #[strum(serialize = "gemx_3p")]
    Gemx3p,
    #[serde(rename = "gemx_ocm_3p")]
    #[strum(serialize = "gemx_ocm_3p")]
    GemxOcm3p,
    #[serde(rename = "gemx_ocm_5p")]
    #[strum(serialize = "gemx_ocm_5p")]
    Gemx5p,
    #[default]
    Unknown,
}
impl DbEnum for ChromiumChip {}
impl FromSql<sql_types::Text, Pg> for ChromiumChip {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Self::from_sql_inner(bytes)
    }
}

impl ToSql<sql_types::Text, diesel::pg::Pg> for ChromiumChip {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        self.to_sql_inner(out)
    }
}

#[derive(Deserialize, Insertable, Validate)]
#[diesel(table_name = schema::chromium_run, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
struct NewChromiumRun {
    legacy_id: String,
    chip: ChromiumChip,
    run_at: NaiveDateTime,
    succeeded: bool,
    notes: Option<Vec<String>>,
    #[diesel(skip_insertion)]
    runners: Vec<Uuid>,
    #[diesel(skip_insertion)]
    #[garde(length(min = 1))]
    gems: Vec<NewGems>,
}

impl BelongsToExt<NewChromiumRun> for NewGems {
    fn set_parent_id(&mut self, parent_id: Uuid) {
        self.chromium_run_id = parent_id;
    }
}

#[derive(Deserialize, Insertable)]
#[diesel(table_name = schema::gems, check_for_backend(Pg))]
struct NewGems {
    legacy_id: String,
    #[serde(skip)]
    chromium_run_id: Uuid,
    #[diesel(skip_insertion)]
    suspension_ids: Vec<Uuid>,
    #[diesel(skip_insertion)]
    multiplexed_suspension_ids: Vec<Uuid>,
}
