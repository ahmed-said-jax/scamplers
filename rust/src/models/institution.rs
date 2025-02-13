use diesel::{pg::Pg, prelude::*};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use uuid::Uuid;
use valuable::Valuable;

#[derive(Insertable, Deserialize, Serialize, Clone, Valuable, Tsify)]
#[diesel(table_name = schema::institution, check_for_backend(Pg))]
#[tsify(into_wasm_abi)]
pub struct NewInstitution {
    name: String,
    #[valuable(skip)]
    ms_tenant_id: Option<Uuid>,
}



#[derive(Queryable, Selectable, Serialize, Tsify)]
#[diesel(table_name = schema::institution, check_for_backend(Pg))]
#[tsify(into_wasm_abi)]
pub struct Institution {
    id: Uuid,
    name: String,
    link: String,
}

