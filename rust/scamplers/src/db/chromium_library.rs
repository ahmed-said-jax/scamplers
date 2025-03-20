use crate::schema;
use diesel::{pg::Pg, prelude::*};
use uuid::Uuid;

#[derive(Identifiable)]
#[diesel(table_name = schema::chromium_library, check_for_backend(Pg))]
pub struct ChromiumLibrary {
    id: Uuid,
}
