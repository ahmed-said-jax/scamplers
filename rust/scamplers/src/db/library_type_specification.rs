use diesel::{backend::Backend, deserialize::{FromSql, FromSqlRow}, expression::AsExpression, pg::Pg, prelude::*, serialize::ToSql, sql_types};
use diesel_async::RunQueryDsl;
use garde::Validate;
use serde::{Deserialize, Serialize};
use crate::schema;

use super::{Create, DbEnum};

#[derive(
    Clone,
    FromSqlRow,
    strum::VariantArray,
    AsExpression,
    Debug,
    strum::IntoStaticStr,
    strum::EnumString,
    PartialEq,
    Deserialize,
    Serialize,
    Copy,
    Default,
)]
#[diesel(sql_type = sql_types::Text)]
pub enum LibraryType {
    #[serde(rename = "Antibody Capture")]
    #[strum(serialize = "Antibody Capture")]
    AntibodyCapture,

    #[serde(rename = "Antigen Capture")]
    #[strum(serialize = "Antigen Capture")]
    AntigenCapture,

    #[serde(rename = "Chromatin Accessibility")]
    #[strum(serialize = "Chromatin Accessibility")]
    ChromatinAccessibility,

    #[serde(rename = "CRISPR Guide Capture")]
    #[strum(serialize = "CRISPR Guide Capture")]
    CrisprGuideCapture,

    Custom,

    #[serde(rename = "Gene Expression")]
    #[strum(serialize = "Gene Expression")]
    GeneExpression,

    #[serde(rename = "Multiplexing Capture")]
    #[strum(serialize = "Multiplexing Capture")]
    MultiplexingCapture,

    #[serde(rename = "VDJ")]
    #[strum(serialize = "VDJ")]
    Vdj,

    #[serde(rename = "VDJ-B")]
    #[strum(serialize = "VDJ-B")]
    VdjB,

    #[serde(rename = "VDJ-T")]
    #[strum(serialize = "VDJ-T")]
    VdjT,

    #[serde(rename = "VDJ-T-GD")]
    #[strum(serialize = "VDJ-T-GD")]
    VdjTGd,

    #[default]
    Unknown
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
    library_volume_µl: f32
}

// We don't need to return anything, as users don't insert into this table
impl Create for Vec<LibraryTypeSpecification> {
    type Returns = ();

    async fn create(&self, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self::Returns> {
        use schema::library_type_specification::dsl::library_type_specification;

        diesel::insert_into(library_type_specification).values(self).on_conflict_do_nothing().execute(conn).await?;

        Ok(())
    }
}