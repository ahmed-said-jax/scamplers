use axum::{
    Router,
    routing::{get, post},
};
use scamplers_core::model::{
    Endpoint,
    institution::{Institution, InstitutionQuery, InstitutionSummary, NewInstitution},
    lab::{LabQuery, LabSummary, LabWithMembers, NewLab},
    person::{NewPerson, Person, PersonQuery, PersonSummary},
};
use scamplers_schema::lab::dsl::lab;

use crate::server::api::handler::{by_id, by_query, new_user, relatives, write};

use super::AppState;

mod error;
mod handler;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(|| async {}))
        .route(&NewInstitution::endpoint(), post(write::<NewInstitution>))
        .route(&Institution::endpoint(), get(by_id::<Institution>))
        .route(
            &InstitutionQuery::endpoint(),
            post(by_query::<InstitutionSummary>),
        )
        .route(&NewPerson::endpoint(), post(write::<NewPerson>))
        .route(&NewPerson::new_user_endpoint(), post(new_user))
        .route(&Person::endpoint(), get(by_id::<Person>))
        .route(&PersonQuery::endpoint(), post(by_query::<PersonSummary>))
        .route(&NewLab::endpoint(), post(write::<NewLab>))
        .route(&LabWithMembers::endpoint(), get(by_id::<LabWithMembers>))
        .route(&LabQuery::endpoint(), post(by_query::<LabSummary>))
        .route(
            &format!("{}/members", LabWithMembers::endpoint()),
            get(relatives::<lab, PersonSummary>),
        )
}
