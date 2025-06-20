#[cfg(feature = "backend")]
use crate::model::{sample_metadata::NewSampleMetadata, specimen::common::NewSpecimenCommon};
use crate::{
    model::specimen::{block::NewBlock, tissue::NewTissue},
    string::NonEmptyString,
};
use time::OffsetDateTime;
use uuid::Uuid;
#[cfg(feature = "backend")]
use {
    scamplers_macros::{backend_db_json, backend_insertion, backend_with_getters},
    scamplers_schema::specimen_measurement,
};

pub mod block;
mod common;
pub mod tissue;

#[cfg_attr(feature = "backend", backend_db_json, serde(rename_all = "UPPERCASE"))]
pub enum MeasurementData {
    Rin {
        #[cfg_attr(feature = "backend", valuable(skip))]
        measured_at: OffsetDateTime,
        #[cfg_attr(feature = "backend", garde(dive))]
        instrument_name: NonEmptyString, // This should be an enum
        #[cfg_attr(feature = "backend", garde(range(min = 1.0, max = 10.0)))]
        value: f32,
    },
    Dv200 {
        #[cfg_attr(feature = "backend", valuable(skip))]
        measured_at: OffsetDateTime,
        #[cfg_attr(feature = "backend", garde(dive))]
        instrument_name: NonEmptyString, // This should be a different enum
        #[cfg_attr(feature = "backend", garde(range(min = 0.0, max = 1.0)))]
        value: f32,
    },
}

#[cfg_attr(
    feature = "backend",
    backend_insertion(specimen_measurement),
    derive(bon::Builder)
)]
pub struct NewSpecimenMeasurement {
    #[cfg_attr(feature = "backend", serde(default))]
    specimen_id: Uuid,
    measured_by: Uuid,
    #[cfg_attr(
        feature = "backend",
        garde(dive),
        diesel(skip_insertion),
        serde(flatten)
    )]
    data: MeasurementData,
}

#[cfg_attr(feature = "backend", derive(serde::Deserialize, Debug))]
#[cfg_attr(feature = "backend", serde(rename_all = "lowercase", tag = "type"))]
pub enum NewSpecimen {
    Block(NewBlock),
    Tissue(NewTissue),
}

#[cfg(feature = "backend")]
impl NewSpecimen {
    fn common(&mut self) -> &mut NewSpecimenCommon {
        match self {
            Self::Block(b) => match b {
                NewBlock::Fixed(b) => &mut b.common,
                NewBlock::Frozen(b) => &mut b.common,
            },

            Self::Tissue(t) => match t {
                NewTissue::Cryopreserved(t) => &mut t.common,
                NewTissue::Fixed(t) => &mut t.common,
                NewTissue::Frozen(t) => &mut t.common,
            },
        }
    }

    pub fn metadata(&mut self) -> NewSampleMetadata {
        self.common().metadata.clone()
    }

    pub fn set_metadata_id(&mut self, metadata_id: Uuid) {
        let current = &mut self.common().metadata_id;
        *current = metadata_id;
    }

    #[must_use]
    pub fn measurements(mut self, id: Uuid) -> Vec<NewSpecimenMeasurement> {
        let mut measurements = self.common().measurements.drain(..);

        for mut m in &mut measurements {
            m.specimen_id = id;
        }

        measurements.collect()
    }
}

#[cfg_attr(feature = "backend", backend_with_getters)]
mod with_getters {
    use crate::model::sample_metadata::SampleMetadata;
    use crate::model::{
        person::PersonHandle, sample_metadata::SampleMetadataSummary, specimen::MeasurementData,
    };
    use uuid::Uuid;
    #[cfg(feature = "backend")]
    use {
        scamplers_macros::backend_selection,
        scamplers_schema::{specimen, specimen_measurement},
    };

    #[cfg_attr(feature = "backend", backend_selection(specimen))]
    pub struct SpecimenHandle {
        id: Uuid,
        link: String,
    }

    #[cfg_attr(feature = "backend", backend_selection(specimen))]
    pub struct SpecimenData {
        #[cfg_attr(feature = "backend", diesel(embed), serde(flatten))]
        handle: SpecimenHandle,
        type_: String,
        embedded_in: Option<String>,
        fixative: Option<String>,
        frozen: bool,
        cryopreserved: bool,
        storage_buffer: Option<String>,
    }

    #[cfg_attr(feature = "backend", backend_selection(specimen))]
    pub struct SpecimenSummary {
        #[cfg_attr(feature = "backend", diesel(embed), serde(flatten))]
        metadata: SampleMetadataSummary,
        #[cfg_attr(feature = "backend", diesel(embed), serde(flatten))]
        data: SpecimenData,
    }

    #[cfg_attr(feature = "backend", backend_selection(specimen_measurement))]
    pub struct SpecimenMeasurement {
        #[cfg_attr(feature = "backend", diesel(embed))]
        measured_by: PersonHandle,
        data: MeasurementData,
    }

    #[cfg_attr(feature = "backend", backend_selection(specimen), derive(bon::Builder))]
    pub struct SpecimenCore {
        #[cfg_attr(feature = "backend", diesel(embed), serde(flatten))]
        metadata: SampleMetadata,
        #[cfg_attr(feature = "backend", diesel(embed), serde(flatten))]
        data: SpecimenData,
    }

    #[cfg_attr(feature = "backend", derive(serde::Serialize, bon::Builder))]
    pub struct Specimen {
        core: SpecimenCore,
        measurements: Vec<SpecimenMeasurement>,
    }
}
pub use with_getters::*;

#[cfg(all(feature = "backend", test))]
mod tests {

    use pretty_assertions::assert_eq;
    use serde_json::{Value, json};
    use time::OffsetDateTime;
    use uuid::Uuid;

    use crate::model::specimen::{NewSpecimen, block::NewBlock};

    #[test]
    fn deserialize_specimen() {
        let uuid = Uuid::now_v7();
        let received_at = OffsetDateTime::now_utc();
        let frozen_embedding_matrix = "carboxymethyl_cellulose";

        let mut fixed_block = json!({
          "readable_id": "id",
          "lab_id": uuid,
          "name": "krabby_patty",
          "submitted_by": uuid,
          "received_at": received_at,
          "species": ["homo_sapiens"],
          "type": "block",
          "preservation": "fixed",
          "embedded_in": frozen_embedding_matrix,
          "fixative": "formaldehyde_derivative"
        });

        let deserialize = |json_val| serde_json::from_value::<NewSpecimen>(json_val);

        let err = deserialize(fixed_block.clone()).unwrap_err();
        assert_eq!(err.classify(), serde_json::error::Category::Data);

        fixed_block["embedded_in"] = Value::String("paraffin".to_string());
        let specimen = deserialize(fixed_block.clone()).unwrap();
        let NewSpecimen::Block(NewBlock::Fixed(_)) = specimen else {
            panic!("expected frozen block, got {specimen:?}");
        };

        let mut frozen_block = fixed_block;
        frozen_block["preservation"] = Value::String("frozen".to_string());
        frozen_block["embedded_in"] = Value::String(frozen_embedding_matrix.to_string());
        frozen_block["fixative"] = Value::Null;
        let specimen = deserialize(frozen_block.clone()).unwrap();
        let NewSpecimen::Block(NewBlock::Frozen(_)) = specimen else {
            panic!("expected frozen block, got {specimen:?}");
        };

        let mut tissue = frozen_block;
        tissue["preservation"] = Value::String("fixed".to_string());
        tissue["type"] = Value::String("tissue".to_string());
        let err = deserialize(tissue.clone()).unwrap_err();
        assert_eq!(err.classify(), serde_json::error::Category::Data);
    }
}
