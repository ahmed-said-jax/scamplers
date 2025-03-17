use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types,
};
use diesel_async::RunQueryDsl;
use garde::Validate;
use serde::{Deserialize, Serialize};

use super::{Create, DbEnum};
use crate::schema;

#[derive(Clone, FromSqlRow, AsExpression, Debug, PartialEq, Deserialize, Serialize, Copy, Default, strum::IntoStaticStr, strum::EnumString)]
#[diesel(sql_type = sql_types::Text)]
pub enum LibraryType {
    #[serde(rename = "Antibody Capture")]
    #[strum(serialize = "Antibody Capture")]
    AntibodyCapture,

    #[serde(rename = "Antigen Capture")]
    #[strum(serialize = "Antibody Capture")]
    AntigenCapture,

    #[serde(rename = "Chromatin Accessibility")]
    #[strum(serialize = "Antibody Capture")]
    ChromatinAccessibility,

    #[serde(rename = "CRISPR Guide Capture")]
    #[strum(serialize = "Antibody Capture")]
    CrisprGuideCapture,

    Custom,

    #[serde(rename = "Gene Expression")]
    #[strum(serialize = "Antibody Capture")]
    GeneExpression,

    #[serde(rename = "Multiplexing Capture")]
    #[strum(serialize = "Antibody Capture")]
    MultiplexingCapture,

    #[serde(rename = "VDJ")]
    #[strum(serialize = "Antibody Capture")]
    Vdj,

    #[serde(rename = "VDJ-B")]
    #[strum(serialize = "Antibody Capture")]
    VdjB,

    #[serde(rename = "VDJ-T")]
    #[strum(serialize = "Antibody Capture")]
    VdjT,

    #[serde(rename = "VDJ-T-GD")]
    #[strum(serialize = "Antibody Capture")]
    VdjTGd,

    #[default]
    Unknown,
}

impl DbEnum for LibraryType {}
impl FromSql<sql_types::Text, Pg> for LibraryType {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Self::from_sql_inner(bytes)
    }
}

impl ToSql<sql_types::Text, diesel::pg::Pg> for LibraryType {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        self.to_sql_inner(out)
    }
}

#[derive(Insertable, Deserialize, Validate)]
#[diesel(table_name = schema::library_type_specification, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
struct LibraryTypeSpecification {
    chemistry_name: String,
    library_type: LibraryType,
    index_kit: String,
    #[garde(range(min = 0.0))]
    #[diesel(column_name = cdna_volume_l)]
    cdna_volume_µl: f32,
    #[garde(range(min = 0.0))]
    #[diesel(column_name = library_volume_l)]
    library_volume_µl: f32,
}

// We don't need to return anything, as users don't insert into this table
impl Create for Vec<LibraryTypeSpecification> {
    type Returns = ();

    async fn create(&self, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self::Returns> {
        use schema::library_type_specification::dsl::library_type_specification;

        diesel::insert_into(library_type_specification)
            .values(self)
            .on_conflict_do_nothing()
            .execute(conn)
            .await?;

        Ok(())
    }
}
