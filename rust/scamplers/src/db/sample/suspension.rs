use diesel::{
    deserialize::FromSqlRow,
    expression::AsExpression,
    sql_types::{self, SqlType},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use valuable::Valuable;

use super::Create;

#[derive(
    Deserialize, Debug, Serialize, FromSqlRow, Clone, Copy, SqlType, AsExpression, Default, Valuable, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
#[diesel(sql_type = sql_types::Text)]
pub enum BiologicalMaterial {
    Cells,
    Nuclei,
    #[default]
    Unknown,
}

#[derive(Deserialize)]
pub struct NewSuspension {}

impl Create for NewSuspension {
    type Returns = ();

    async fn create(&self, conn: &mut diesel_async::AsyncPgConnection) -> crate::db::Result<Self::Returns> {
        Ok(())
    }
}
