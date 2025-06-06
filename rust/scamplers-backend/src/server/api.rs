use axum::{
    Router,
    routing::{get, post},
};
use scamplers_core::model::{
    Endpoint,
    institution::{Institution, InstitutionSummary, NewInstitution},
    person::{NewPerson, Person, PersonSummary},
};

use crate::server::api::handler::{by_id, by_query, new_user, write};

use super::AppState;

mod error;
mod handler;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(|| async {}))
        .route(&NewInstitution::endpoint(), post(write::<NewInstitution>))
        .route(&Institution::endpoint(), get(by_id::<Institution>))
        .route(
            &InstitutionSummary::endpoint(),
            post(by_query::<InstitutionSummary>),
        )
        .route(&NewPerson::endpoint(), post(write::<NewPerson>))
        .route(&NewPerson::new_user_endpoint(), post(new_user))
        .route(&Person::endpoint(), get(by_id::<Person>))
        .route(&PersonSummary::endpoint(), post(by_query::<PersonSummary>))
}
