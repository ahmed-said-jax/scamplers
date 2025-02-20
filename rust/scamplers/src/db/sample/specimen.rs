use chrono::NaiveDateTime;
use diesel::backend::Backend;
use diesel::deserialize::{FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::Pg;
use diesel::serialize::ToSql;
use diesel::sql_types::SqlType;
use diesel_async::AsyncPgConnection;
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::db::{self, DbEnum};
use diesel::{prelude::*, sql_types};
use crate::schema;

use crate::db::{measurement::MeasurementMetadata, Create};

use super::NewSampleMetadata;

#[derive(Deserialize, Serialize, FromSqlRow, strum::IntoStaticStr, strum::EnumString, Clone, Copy, SqlType, AsExpression, Debug)]
#[serde(rename_all = "snake_case")]
#[diesel(sql_type = sql_types::Text)]
enum EmbeddingMatrix {
    CarboxymethylCellulose,
    OptimalCuttingTemperature,
    Paraffin
}
impl DbEnum for EmbeddingMatrix {}

impl FromSql<sql_types::Text, Pg> for EmbeddingMatrix {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Self::from_sql_inner(bytes)
    }
}

impl ToSql<sql_types::Text, diesel::pg::Pg> for EmbeddingMatrix {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        Self::to_sql_inner(self, out)
    }
}


#[derive(Deserialize, Serialize, FromSqlRow, strum::IntoStaticStr, strum::EnumString, Clone, Copy, SqlType, AsExpression, Debug)]
#[serde(rename_all = "snake_case")]
#[diesel(sql_type = sql_types::Text)]
enum PreservationMethod {
    Cryopreservation,
    DspFixation,
    FormaldehydeDerivativeFixation,
    Freezing,
}
impl DbEnum for PreservationMethod {}

impl FromSql<sql_types::Text, Pg> for PreservationMethod {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Self::from_sql_inner(bytes)
    }
}

impl ToSql<sql_types::Text, diesel::pg::Pg> for PreservationMethod {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        Self::to_sql_inner(self, out)
    }
}

fn is_block_preservation_method(preservation_method: &PreservationMethod, _: &()) -> garde::Result {
    if matches!(preservation_method, PreservationMethod::FormaldehydeDerivativeFixation) {
        Ok(())
    } else {
        Err(garde::Error::new("invalid preservation method for block"))
    }
}

#[derive(Deserialize, Serialize, Validate)]
#[serde(rename_all = "snake_case", tag = "quantity")]
#[garde(allow_unvalidated)]
enum SpecimenMeasurement {
    Rin {
        #[serde(flatten)]
        #[garde(dive)]
        metadata: MeasurementMetadata,
        instrument_name: String, // This should be an enum
        #[garde(range(min = 1.0, max = 10.0))]
        value: f32,
    },
    Dv200 {
        #[serde(flatten)]
        #[garde(dive)]
        metadata: MeasurementMetadata,
        instrument_name: String, // This should be a different enum
        #[garde(range(min = 0.0, max = 1.0))]
        value: f32,
    }
}

#[derive(Deserialize, Validate)]
struct SpecimenCore {
    #[garde(length(min = 1))]
    legacy_id: String,
    #[garde(dive)]
    measurements: Option<Vec<SpecimenMeasurement>>,
    #[garde(skip)]
    notes: Option<Vec<String>>
}

#[derive(Deserialize, Validate)]
#[serde(tag = "type")]
#[garde(allow_unvalidated)]
enum NewSpecimen {
    Block {
        #[serde(flatten)]
        #[garde(dive)]
        metadata: NewSampleMetadata,
        #[serde(flatten)]
        #[garde(dive)]
        core: SpecimenCore,
        embedded_in: EmbeddingMatrix,
        #[garde(custom(is_block_preservation_method))]
        preserved_with: PreservationMethod,
    },
    Tissue {
        #[serde(flatten)]
        #[garde(dive)]
        metadata: NewSampleMetadata,
        #[serde(flatten)]
        #[garde(dive)]
        core: SpecimenCore,
        preserved_with: Option<PreservationMethod>,
    },
}

impl Create for Vec<NewSpecimen> {
    type Returns = ();

    async fn create(
            &self,
            conn: &mut AsyncPgConnection,
        ) -> db::Result<Self::Returns> {
        #[derive(Insertable)]
        #[diesel(table_name = schema::specimen)]
        struct NewSpecimenQuery<'a> {
            legacy_id: &'a str,
            metadata_id: &'a Uuid,
            type_: &'a str,
            embedded_in: Option<&'a EmbeddingMatrix>,
            preserved_with: Option<&'a PreservationMethod>,
            notes: Option<&'a [String]>,
        }

        Ok(())
    }
}