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
use valuable::Valuable;

use super::{Create, utils::DbEnum};
use crate::schema;

#[derive(
    Clone,
    FromSqlRow,
    AsExpression,
    Debug,
    PartialEq,
    Deserialize,
    Serialize,
    Copy,
    Default,
    strum::IntoStaticStr,
    strum::EnumString,
    Valuable,
    Eq,
    PartialOrd,
    Ord,
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

pub trait LibraryTypeGroup {
    fn validate(&self) -> Result<(), Error>;
    fn valid_combinations() -> [Vec<LibraryType>; 23] {
        type T = LibraryType;

        let valid_combos = [
            vec![T::AntibodyCapture],
            vec![T::AntibodyCapture, T::GeneExpression],
            vec![T::AntibodyCapture, T::GeneExpression, T::MultiplexingCapture],
            vec![T::AntibodyCapture, T::GeneExpression, T::MultiplexingCapture, T::Vdj],
            vec![T::AntibodyCapture, T::GeneExpression, T::MultiplexingCapture, T::VdjB],
            vec![T::AntibodyCapture, T::GeneExpression, T::MultiplexingCapture, T::VdjT],
            vec![T::AntibodyCapture, T::GeneExpression, T::MultiplexingCapture, T::VdjTGd],
            vec![T::AntibodyCapture, T::GeneExpression, T::Vdj],
            vec![T::AntibodyCapture, T::GeneExpression, T::VdjB],
            vec![T::AntibodyCapture, T::GeneExpression, T::VdjT],
            vec![T::AntibodyCapture, T::GeneExpression, T::VdjTGd],
            vec![T::ChromatinAccessibility],
            vec![T::ChromatinAccessibility, T::GeneExpression],
            vec![T::GeneExpression],
            (vec![T::GeneExpression, T::MultiplexingCapture]),
            (vec![T::GeneExpression, T::Vdj]),
            (vec![T::GeneExpression, T::VdjB]),
            (vec![T::GeneExpression, T::VdjT]),
            (vec![T::GeneExpression, T::VdjTGd]),
            (vec![T::Vdj]),
            vec![T::VdjB],
            vec![T::VdjT],
            vec![T::VdjTGd],
        ];

        valid_combos
    }
}
impl LibraryTypeGroup for Vec<LibraryType> {
    fn validate(&self) -> Result<(), Error> {
        type T = LibraryType;

        let expected = Self::valid_combinations();

        let mut library_types = self.clone();
        library_types.sort();

        let err = || Error::new(library_types.clone());

        if self.len() > 4 {
            return Err(err());
        }

        if !expected.contains(&library_types) {
            return Err(err());
        }
        Ok(())
    }
}

#[derive(Insertable, Deserialize, Validate)]
#[diesel(table_name = schema::library_type_specification, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
struct LibraryTypeSpecification {
    chemistry: String,
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

    async fn create(self, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self::Returns> {
        use schema::library_type_specification::dsl::library_type_specification;

        diesel::insert_into(library_type_specification)
            .values(self)
            .on_conflict_do_nothing()
            .execute(conn)
            .await?;

        Ok(())
    }
}

#[derive(thiserror::Error, Debug, Serialize, Valuable, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[error("invalid library type group")]
pub struct Error {
    expected: [Vec<LibraryType>; 23],
    found: Vec<LibraryType>,
}
impl Error {
    pub fn new(library_types: Vec<LibraryType>) -> Self {
        Self {
            expected: <Vec<_> as LibraryTypeGroup>::valid_combinations(),
            found: library_types,
        }
    }
}
