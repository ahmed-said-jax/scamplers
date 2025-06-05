use std::{collections::HashMap, hash::RandomState};

use axum::{
    Router,
    routing::{get, post},
};
use scamplers_core::model::{
    Endpoint,
    institution::{Institution, NewInstitution},
    person::{CreatedUser, Person, PersonSummary},
};

use crate::server::api::handler::{by_id, by_query, write};

use super::AppState;

mod error;
mod handler;

pub(super) fn router() -> Router<AppState> {
    use handler::new_user;

    let endpoints: HashMap<&str, [&str; 2], RandomState> = HashMap::from_iter([(
        "available_endpoints",
        [Institution::endpoint(), Person::endpoint()],
    )]);

    Router::new()
        .route("/", get(|| async { axum::Json(endpoints) }))
        .route(Institution::endpoint(), post(write::<NewInstitution>))
        .route(CreatedUser::endpoint(), post(new_user))
        .route(Person::endpoint(), post(by_id::<Person>))
        .route(PersonSummary::endpoint(), post(by_query::<PersonSummary>))
}
