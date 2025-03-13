use diesel::{prelude::*, pg::Pg};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::schema;

// This won't be a publicly writable route - rather, we'll just ingest these from 10XGenomics/cellranger on application startup using links provided in the config
#[derive(Deserialize, Serialize, Insertable, Debug)]
#[diesel(table_name = schema::chemistry, check_for_backend(Pg))]
struct Chemistry {
    name: String,
    description: String,
    #[serde(flatten)]
    definition: JsonValue
}
