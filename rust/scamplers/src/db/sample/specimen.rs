use chrono::NaiveDateTime;
use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    helper_types::InnerJoin,
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types::{self, SqlType},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use futures::FutureExt;
use garde::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use valuable::Valuable;

use super::{NewSampleMetadata, OrdinalColumns as MetadataOrdinalColumns, SampleMetadata, SampleMetadataQuery};
use crate::{
    db::{
        self, AsDieselExpression, BoxedDieselExpression, Create, Read,
        person::PersonStub,
        utils::{Child, Children, ChildrenSets, DbEnum, DbJson},
    },
    schema::{
        self, lab, person,
        sample_metadata::{self, name as name_col, received_at, species as species_col, tissue as tissue_col},
        specimen::{
            self, embedded_in as embedding_col, id as id_col, preserved_with as preservation_col, type_ as type_col,
        },
        specimen_measurement,
    },
};

#[derive(
    Deserialize,
    Serialize,
    FromSqlRow,
    strum::VariantArray,
    Clone,
    Copy,
    SqlType,
    AsExpression,
    Debug,
    Default,
    Valuable,
    strum::IntoStaticStr,
    strum::EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[diesel(sql_type = sql_types::Text)]
pub enum EmbeddingMatrix {
    CarboxymethylCellulose,
    OptimalCuttingTemperatureCompound,
    Paraffin,
    #[default]
    Unknown,
}
impl DbEnum for EmbeddingMatrix {}

impl FromSql<sql_types::Text, Pg> for EmbeddingMatrix {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Self::from_sql_inner(bytes)
    }
}

impl ToSql<sql_types::Text, diesel::pg::Pg> for EmbeddingMatrix {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        self.to_sql_inner(out)
    }
}

#[derive(
    Deserialize,
    Serialize,
    FromSqlRow,
    strum::VariantArray,
    Clone,
    Copy,
    SqlType,
    AsExpression,
    Debug,
    Default,
    Valuable,
    strum::IntoStaticStr,
    strum::EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[diesel(sql_type = sql_types::Text)]
pub enum PreservationMethod {
    Cryopreservation,
    DspFixation,
    FormaldehydeDerivativeFixation,
    Freezing,
    #[default]
    Unknown,
}
impl DbEnum for PreservationMethod {}

impl FromSql<sql_types::Text, Pg> for PreservationMethod {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Self::from_sql_inner(bytes)
    }
}

impl ToSql<sql_types::Text, diesel::pg::Pg> for PreservationMethod {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        self.to_sql_inner(out)
    }
}

fn is_block_preservation_method(preservation_method: &PreservationMethod, _: &()) -> garde::Result {
    if matches!(preservation_method, PreservationMethod::FormaldehydeDerivativeFixation) {
        Ok(())
    } else {
        Err(garde::Error::new("invalid preservation method for block"))
    }
}

#[derive(Deserialize, Serialize, Validate, FromSqlRow, Default, Debug, AsExpression, JsonSchema)]
#[diesel(sql_type = sql_types::Jsonb)]
#[serde(rename_all = "UPPERCASE", tag = "quantity")]
#[garde(allow_unvalidated)]
pub enum MeasurementData {
    Rin {
        measured_at: NaiveDateTime,
        instrument_name: String, // This should be an enum
        #[garde(range(min = 1.0, max = 10.0))]
        value: f32,
    },
    Dv200 {
        measured_at: NaiveDateTime,
        instrument_name: String, // This should be a different enum
        #[garde(range(min = 0.0, max = 1.0))]
        value: f32,
    },
    #[default]
    Unknown,
}

impl DbJson for MeasurementData {}

impl FromSql<sql_types::Jsonb, Pg> for MeasurementData {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Self::from_sql_inner(bytes)
    }
}
impl ToSql<sql_types::Jsonb, Pg> for MeasurementData {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        self.to_sql_inner(out)
    }
}

#[derive(Insertable, Deserialize, Validate)]
#[garde(allow_unvalidated)]
#[diesel(table_name = schema::specimen_measurement)]
pub struct NewSpecimenMeasurement {
    #[serde(default)]
    pub specimen_id: Uuid,
    pub measured_by: Uuid,
    #[serde(flatten)]
    #[garde(dive)]
    pub data: MeasurementData,
}

impl Create for Vec<NewSpecimenMeasurement> {
    type Returns = ();

    async fn create(self, conn: &mut AsyncPgConnection) -> db::Result<Self::Returns> {
        diesel::insert_into(specimen_measurement::table)
            .values(&self)
            .execute(conn)
            .await?;

        Ok(())
    }
}
impl Child<specimen::table> for NewSpecimenMeasurement {
    fn set_parent_id(&mut self, parent_id: Uuid) {
        self.specimen_id = parent_id;
    }
}

// Common fields could be factored out of these enum variants, but it becomes a bit confusing
#[derive(Deserialize, Validate)]
#[serde(tag = "type")]
#[garde(allow_unvalidated)]
pub enum NewSpecimen {
    Block {
        #[serde(flatten)]
        #[garde(dive)]
        metadata: NewSampleMetadata,
        #[garde(length(min = 1))]
        legacy_id: String,
        #[garde(dive)]
        measurements: Vec<NewSpecimenMeasurement>,
        #[garde(skip)]
        notes: Option<Vec<String>>,
        embedded_in: EmbeddingMatrix,
        #[garde(custom(is_block_preservation_method))]
        preserved_with: PreservationMethod,
    },
    Tissue {
        #[serde(flatten)]
        #[garde(dive)]
        metadata: NewSampleMetadata,
        #[garde(length(min = 1))]
        legacy_id: String,
        #[garde(dive)]
        measurements: Vec<NewSpecimenMeasurement>,
        #[garde(skip)]
        notes: Option<Vec<String>>,
        preserved_with: Option<PreservationMethod>,
    },
    Fluid {
        #[serde(flatten)]
        #[garde(dive)]
        metadata: NewSampleMetadata,
        #[garde(length(min = 1))]
        legacy_id: String,
        #[garde(dive)]
        measurements: Vec<NewSpecimenMeasurement>,
        #[garde(skip)]
        notes: Option<Vec<String>>,
        preserved_with: Option<PreservationMethod>,
    },
}

impl Create for Vec<NewSpecimen> {
    type Returns = Vec<Specimen>;

    async fn create(self, conn: &mut AsyncPgConnection) -> db::Result<Self::Returns> {
        #[derive(Insertable)]
        #[diesel(table_name = schema::specimen)]
        struct InsertSpecimen {
            legacy_id: String,
            metadata_id: Uuid,
            type_: SpecimenType,
            embedded_in: Option<EmbeddingMatrix>,
            preserved_with: Option<PreservationMethod>,
            notes: Option<Vec<String>>,
        }

        impl Child<sample_metadata::table> for InsertSpecimen {
            fn set_parent_id(&mut self, parent_id: Uuid) {
                self.metadata_id = parent_id;
            }
        }

        const N_MEASUREMENTS_PER_SPECIMEN: usize = 4;

        let n_specimens = self.len();

        let mut metadatas = Vec::with_capacity(n_specimens);
        let mut specimen_insertions = Vec::with_capacity(n_specimens);
        let mut measurement_sets = Vec::with_capacity(n_specimens);

        for specimen in self {
            let (embedded_in, preserved_with, type_) = match specimen {
                NewSpecimen::Block {
                    embedded_in,
                    preserved_with,
                    ..
                } => (Some(embedded_in), Some(preserved_with), SpecimenType::Block),
                NewSpecimen::Tissue { preserved_with, .. } => (None, preserved_with, SpecimenType::Tissue),
                NewSpecimen::Fluid { preserved_with, .. } => (None, preserved_with, SpecimenType::Fluid),
            };

            let (NewSpecimen::Block {
                metadata,
                legacy_id,
                measurements,
                notes,
                ..
            }
            | NewSpecimen::Tissue {
                metadata,
                legacy_id,
                measurements,
                notes,
                ..
            }
            | NewSpecimen::Fluid {
                metadata,
                legacy_id,
                measurements,
                notes,
                ..
            }) = specimen;

            metadatas.push(metadata);
            specimen_insertions.push(InsertSpecimen {
                legacy_id,
                metadata_id: Uuid::nil(),
                type_,
                embedded_in,
                preserved_with,
                notes,
            });
            measurement_sets.push(measurements);
        }

        let metadata_ids = metadatas.create(conn).await?;
        specimen_insertions.set_parent_ids(&metadata_ids);

        let specimen_ids: Vec<Uuid> = diesel::insert_into(specimen::table)
            .values(specimen_insertions)
            .returning(id_col)
            .get_results(conn)
            .await?;

        let flattened_measurements =
            measurement_sets.flatten_and_set_parent_ids(&specimen_ids, N_MEASUREMENTS_PER_SPECIMEN * n_specimens);
        flattened_measurements.create(conn).await?;

        let query = SpecimenQuery {
            ids: specimen_ids,
            limit: n_specimens as i64,
            ..Default::default()
        };

        let specimens = Specimen::fetch_many(&query, conn).await?;

        Ok(specimens)
    }
}

#[derive(Serialize, JsonSchema)]
pub struct Specimen {
    #[serde(flatten)]
    core: SpecimenCore,
    measurements: Option<Vec<SpecimenMeasurement>>,
}

#[derive(
    Deserialize,
    Valuable,
    Default,
    Clone,
    Copy,
    Serialize,
    FromSqlRow,
    AsExpression,
    Debug,
    strum::IntoStaticStr,
    strum::EnumString,
)]
#[diesel(sql_type = sql_types::Text)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
enum SpecimenType {
    Block,
    Tissue,
    Fluid,
    #[default]
    Unknown,
}
impl DbEnum for SpecimenType {}

impl FromSql<sql_types::Text, Pg> for SpecimenType {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Self::from_sql_inner(bytes)
    }
}

impl ToSql<sql_types::Text, diesel::pg::Pg> for SpecimenType {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        self.to_sql_inner(out)
    }
}

#[derive(Deserialize, Valuable, Default)]
pub struct SpecimenQuery {
    #[serde(default)]
    #[valuable(skip)]
    ids: Vec<Uuid>,
    #[serde(flatten)]
    metadata_query: Option<SampleMetadataQuery>,
    embedded_in: Option<EmbeddingMatrix>,
    preserved_with: Option<PreservationMethod>,
    #[serde(alias = "type")]
    type_: Option<SpecimenType>,
    #[serde(default = "crate::db::utils::default_query_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
    #[serde(default)]
    with_measurements: bool,
    #[serde(default)]
    order: MetadataOrdinalColumns,
    #[serde(default)]
    descending: bool,
}
impl<T> AsDieselExpression<T> for SpecimenQuery
where
    name_col: SelectableExpression<T>,
    tissue_col: SelectableExpression<T>,
    received_at: SelectableExpression<T>,
    species_col: SelectableExpression<T>,
    id_col: SelectableExpression<T>,
    embedding_col: SelectableExpression<T>,
    preservation_col: SelectableExpression<T>,
    type_col: SelectableExpression<T>,
{
    fn as_diesel_expression<'a>(&'a self) -> Option<db::BoxedDieselExpression<'a, T>>
    where
        T: 'a,
    {
        let Self {
            ids,
            metadata_query,
            embedded_in,
            preserved_with,
            type_,
            ..
        } = self;

        if matches!(
            (ids.is_empty(), metadata_query, embedded_in, preserved_with, type_),
            (true, None, None, None, None)
        ) {
            return None;
        }

        let mut query: BoxedDieselExpression<T> = if ids.is_empty() {
            Box::new(id_col.is_not_null())
        } else {
            Box::new(id_col.eq_any(ids))
        };

        if let Some(metadata_query) = metadata_query.as_ref().map(|q| q.as_diesel_expression()).flatten() {
            query = Box::new(query.and(metadata_query));
        }

        if let Some(embedding_matrix) = embedded_in {
            query = Box::new(query.and(embedding_col.is_not_distinct_from(embedding_matrix)));
        }

        if let Some(preservation_method) = preserved_with {
            query = Box::new(query.and(preservation_col.is_not_distinct_from(preservation_method)));
        }

        if let Some(type_) = type_ {
            query = Box::new(query.and(type_col.eq(type_)));
        }

        Some(query)
    }
}

impl Specimen {
    fn base_query() -> InnerJoin<specimen::table, InnerJoin<sample_metadata::table, lab::table>> {
        specimen::table.inner_join(SampleMetadata::base_query())
    }
}

impl Read for Specimen {
    type Id = Uuid;
    type QueryParams = SpecimenQuery;

    async fn fetch_by_id(id: &Self::Id, conn: &mut AsyncPgConnection) -> db::Result<Self> {
        let core = Self::base_query()
            .filter(id_col.eq(id))
            .select(SpecimenCore::as_select())
            .first(conn)
            .boxed();

        // We use this instead of the `belonging_to` function because loading the measurements and the actual spcimen
        // opbject at the same time is faster than loading one then the other
        let measurements = SpecimenMeasurement::base_query()
            .filter(specimen_measurement::specimen_id.eq(id))
            .select(SpecimenMeasurement::as_select())
            .load(conn)
            .boxed();

        let (core, measurements) = tokio::try_join!(core, measurements)?;

        Ok(Self {
            core,
            measurements: Some(measurements),
        })
    }

    async fn fetch_many(query: &Self::QueryParams, conn: &mut AsyncPgConnection) -> db::Result<Vec<Self>> {
        let Self::QueryParams {
            order,
            descending,
            with_measurements,
            limit,
            offset,
            ..
        } = &query;

        let mut specimens_statement = Self::base_query()
            .select(SpecimenCore::as_select())
            .limit(*limit)
            .offset(*offset)
            .into_boxed();

        specimens_statement = match order {
            MetadataOrdinalColumns::ReceivedAt => {
                if *descending {
                    specimens_statement.order(received_at.desc())
                } else {
                    specimens_statement.order(received_at)
                }
            }
            MetadataOrdinalColumns::Name => {
                if *descending {
                    specimens_statement.order(name_col.desc())
                } else {
                    specimens_statement.order(name_col)
                }
            }
        };

        if let Some(query) = query.as_diesel_expression() {
            specimens_statement = specimens_statement.filter(query);
        }

        let specimens = specimens_statement.load(conn).await?;

        if !with_measurements {
            return Ok(specimens
                .into_iter()
                .map(|core| Self {
                    core,
                    measurements: None,
                })
                .collect());
        }

        let measurements: Vec<Vec<SpecimenMeasurement>> = SpecimenMeasurement::belonging_to(&specimens)
            .inner_join(specimen::table)
            .inner_join(person::table)
            .select(SpecimenMeasurement::as_select())
            .load(conn)
            .await?
            .grouped_by(&specimens);

        let specimens = specimens
            .into_iter()
            .zip(measurements)
            .map(|(core, measurements)| Self {
                core,
                measurements: Some(measurements),
            })
            .collect();

        Ok(specimens)
    }
}

#[derive(Serialize, Selectable, Queryable, Associations, Identifiable, JsonSchema)]
#[diesel(table_name = schema::specimen_measurement, check_for_backend(Pg), belongs_to(SpecimenCore, foreign_key = specimen_id))]
struct SpecimenMeasurement {
    #[serde(skip)]
    id: Uuid,
    #[serde(skip)]
    specimen_id: Uuid,
    #[diesel(embed)]
    measured_by: PersonStub,
    #[serde(flatten)]
    data: MeasurementData,
}
impl SpecimenMeasurement {
    fn base_query() -> InnerJoin<InnerJoin<specimen_measurement::table, specimen::table>, person::table> {
        specimen_measurement::table
            .inner_join(specimen::table)
            .inner_join(person::table)
    }
}

#[derive(Serialize, Selectable, Queryable, Identifiable, JsonSchema)]
#[diesel(table_name = schema::specimen, check_for_backend(Pg))]
struct SpecimenCore {
    id: Uuid,
    #[serde(flatten)]
    #[diesel(embed)]
    metadata: SampleMetadata,
    embedded_in: Option<String>,
    preserved_with: Option<String>,
    #[serde(rename = "type")]
    type_: String,
    notes: Option<Vec<String>>,
}
