use chrono::NaiveDateTime;
use diesel::{
    alias,
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types::{self, SqlType},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use futures::FutureExt;
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{NewSampleMetadata, SampleMetadata, SampleMetadataQuery, Species};
use crate::{
    db::{self, person::PersonStub, AsDieselExpression, Create, DbEnum, DbJson, BoxedDieselExpression, Pagination, Read},
    schema::{self, lab, person, sample_metadata::{name as name_col, tissue as tissue_col, received_at, species as species_col}, specimen::{self, id as id_col, embedded_in as embedding_col, preserved_with as preservation_col, type_ as type_col}, specimen_measurement},
};

#[derive(
    Deserialize,
    Serialize,
    FromSqlRow,
    strum::IntoStaticStr,
    strum::EnumString,
    Clone,
    Copy,
    SqlType,
    AsExpression,
    Debug,
    Default,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[diesel(sql_type = sql_types::Text)]
enum EmbeddingMatrix {
    CarboxymethylCellulose,
    OptimalCuttingTemperature,
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
    strum::IntoStaticStr,
    strum::EnumString,
    Clone,
    Copy,
    SqlType,
    AsExpression,
    Debug,
    Default,
)]
#[serde(rename_all = "snake_case")]
#[diesel(sql_type = sql_types::Text)]
#[strum(serialize_all = "snake_case")]
enum PreservationMethod {
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

#[derive(Deserialize, Serialize, Validate, FromSqlRow, Default, Debug, AsExpression)]
#[diesel(sql_type = sql_types::Jsonb)]
#[serde(rename_all = "snake_case", tag = "quantity")]
#[garde(allow_unvalidated)]
enum MeasurementData {
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

#[derive(Deserialize, Validate)]
#[garde(allow_unvalidated)]
struct NewSpecimenMeasurement {
    measured_by: Uuid,
    #[serde(flatten)]
    data: MeasurementData,
}

// Common fields could be factored out of these enum variants, but it becomes a bit confusing
#[derive(Deserialize, Validate)]
#[serde(tag = "type")]
#[garde(allow_unvalidated)]
enum NewSpecimen {
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

#[derive(Insertable)]
#[diesel(table_name = schema::specimen_measurement)]
struct NewMeasurement<M: AsExpression<sql_types::Jsonb>>
where
    for<'a> &'a M: AsExpression<sql_types::Jsonb>,
{
    specimen_id: Option<Uuid>,
    measured_by: Uuid,
    data: M,
}

impl Create for Vec<NewSpecimen> {
    type Returns = ();

    async fn create(&self, conn: &mut AsyncPgConnection) -> db::Result<Self::Returns> {
        #[derive(Insertable)]
        #[diesel(table_name = schema::specimen)]
        struct InsertSpecimen<'a> {
            legacy_id: &'a str,
            metadata_id: Option<&'a Uuid>, /* This is optional because we want to create this struct before we know
                                            * the metadata ID, so we set it later */
            type_: &'a str,
            embedded_in: Option<&'a EmbeddingMatrix>,
            preserved_with: Option<&'a PreservationMethod>,
            notes: Option<&'a [String]>,
        }

        let mut new_metadata = Vec::with_capacity(self.len());
        let mut specimen_insertions = Vec::with_capacity(self.len());
        let mut new_measurements = Vec::with_capacity(self.len() * 2); // We expect that each specimen has just two measurements, but it's not a big deal if there are more or less

        for specimen in self {
            let (embedded_in, preserved_with, type_) = match specimen {
                NewSpecimen::Block {
                    embedded_in,
                    preserved_with,
                    ..
                } => (Some(embedded_in), Some(preserved_with), "block"),
                NewSpecimen::Tissue { preserved_with, .. } => (None, preserved_with.as_ref(), "tissue"),
                NewSpecimen::Fluid { preserved_with, .. } => (None, preserved_with.as_ref(), "fluid"),
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

            new_metadata.push(metadata);
            specimen_insertions.push(InsertSpecimen {
                legacy_id,
                metadata_id: None,
                type_,
                embedded_in,
                preserved_with,
                notes: notes.as_ref().map(|n| n.as_slice()),
            });
            new_measurements.extend(measurements.iter().map(|m| NewMeasurement {
                specimen_id: None,
                measured_by: m.measured_by,
                data: &m.data,
            }));
        }

        let metadata_ids = new_metadata.create(conn).await?;

        for (specimen, metadata_id) in specimen_insertions.iter_mut().zip(&metadata_ids) {
            specimen.metadata_id = Some(metadata_id);
        }

        let specimen_ids = diesel::insert_into(specimen::table)
            .values(specimen_insertions)
            .returning(id_col)
            .get_results(conn)
            .await?;

        for (measurement, specimen_id) in new_measurements.iter_mut().zip(&specimen_ids) {
            measurement.specimen_id = Some(*specimen_id)
        }

        diesel::insert_into(specimen_measurement::table)
            .values(new_measurements)
            .execute(conn)
            .await?;

        Ok(())
    }
}

#[derive(Serialize)]
pub enum Specimen {
    Lite(SpecimenLite),
    Full(SpecimenFull),
}

#[derive(Deserialize, strum::IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
enum SpecimenType {
    Block,
    Tissue,
    Fluid,
}

#[derive(Deserialize)]
pub struct SpecimenQuery {
    #[serde(default)]
    ids: Vec<Uuid>,
    #[serde(flatten)]
    metadata_query: Option<SampleMetadataQuery>,
    embedded_in: Option<EmbeddingMatrix>,
    preserved_with: Option<PreservationMethod>,
    type_: Option<SpecimenType>,
    #[serde(default, flatten)]
    pagination: Pagination,
    #[serde(default)]
    with_measurements: bool
}
impl<T> AsDieselExpression<T> for SpecimenQuery  where name_col: SelectableExpression<T>, tissue_col: SelectableExpression<T>, received_at: SelectableExpression<T>, species_col: SelectableExpression<T>, id_col: SelectableExpression<T>, embedding_col: SelectableExpression<T>, preservation_col: SelectableExpression<T>, type_col: SelectableExpression<T> {
    fn as_diesel_expression<'a>(&'a self) -> Option<db::BoxedDieselExpression<'a, T>> where T: 'a {
        let Self {
            ids,
            metadata_query,
            embedded_in,
            preserved_with,
            type_,
            ..
        } = self;

        if matches!((ids.is_empty(), metadata_query, embedded_in, preserved_with, type_), (true, None, None, None, None)) {
            return None;
        }

        let mut query: BoxedDieselExpression<T> = if ids.is_empty() {Box::new(id_col.is_not_null())} else {Box::new(id_col.eq_any(ids))};

        if let Some(metadata_query) = metadata_query.as_ref().map(|q| q.as_diesel_expression()).flatten() {
            query = Box::new(query.and(metadata_query));
        }

        if let Some(embedding_matrix) = embedded_in {
            query = Box::new(query.and(embedding_col.is_distinct_from(embedding_matrix)));
        }

        if let Some(preservation_method) = preserved_with {
            query = Box::new(query.and(preservation_col.is_distinct_from(preservation_method)));
        }

        if let Some(type_) = type_ {
            let type_: &str = type_.into();
            query = Box::new(query.and(type_col.eq(type_)));
        }

        Some(query)
    }
}

impl Read for Specimen {
    type Id = Uuid;
    type QueryParams = SpecimenQuery;

    async fn fetch_by_id(id: Self::Id, conn: &mut AsyncPgConnection) -> db::Result<Self> {
        let inner = specimen::table
            .find(id)
            .inner_join(sample_metadata::table.inner_join(lab::table))
            .select(SpecimenLite::as_select())
            .first(conn)
            .boxed();

        let measurements = specimen_measurement::table
            .filter(specimen_measurement::specimen_id.eq(id))
            .inner_join(person::table)
            .select(SpecimenMeasurement::as_select())
            .load(conn)
            .boxed();

        let (inner, measurements) = tokio::try_join!(inner, measurements)?;

        Ok(Self::Full(SpecimenFull { inner, measurements }))
    }

    async fn fetch_many(filter: Self::QueryParams, conn: &mut AsyncPgConnection) -> db::Result<Vec<Self>> {
        let (specimen_query, metadata_query) = filter.as_sql();

        let statement = specimen_query
            .inner_join(sample_metadata::table.inner_join(lab::table))
            .filter();

        let specimens = statement.select(SpecimenLite::as_select()).load(conn).await?;

        Ok(specimens.into_iter().map(|s| Self::Lite(s)).collect())
    }
}

#[derive(Serialize, Selectable, Queryable)]
#[diesel(table_name = schema::specimen_measurement, check_for_backend(Pg))]
struct SpecimenMeasurement {
    #[diesel(embed)]
    measured_by: PersonStub,
    #[serde(flatten)]
    data: MeasurementData,
}

#[derive(Serialize)]
struct SpecimenFull {
    #[serde(flatten)]
    inner: SpecimenLite,
    measurements: Vec<SpecimenMeasurement>,
}

#[derive(Serialize, Selectable, Queryable)]
#[diesel(table_name = schema::specimen, check_for_backend(Pg))]
struct SpecimenLite {
    id: Uuid,
    #[serde(flatten)]
    #[diesel(embed)]
    metadata: SampleMetadata,
    embedded_in: Option<EmbeddingMatrix>,
    preserved_with: Option<PreservationMethod>,
    #[serde(rename = "type")]
    type_: String,
    notes: Option<Vec<String>>,
}
