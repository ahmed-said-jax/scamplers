use std::{collections::HashMap, hash::RandomState};

use axum::{
    Router,
    routing::{get, post},
};
use scamplers_core::model::{
    AsEndpoint,
    person::{CreatedUser, Person, PersonSummary},
};

use crate::server::api::handler::{by_id, by_query};

use super::AppState;

mod error;
mod handler;

pub(super) fn router() -> Router<AppState> {
    use handler::new_user;

    let endpoints: HashMap<&str, [&str; 1], RandomState> =
        HashMap::from_iter([("available_endpoints", [""])]);

    Router::new()
        .route("/", get(|| async { axum::Json(endpoints) }))
        .route(CreatedUser::as_endpoint(), post(new_user))
        .route(Person::as_endpoint(), post(by_id::<Person>))
        .route(
            PersonSummary::as_endpoint(),
            post(by_query::<PersonSummary>),
        )
}
