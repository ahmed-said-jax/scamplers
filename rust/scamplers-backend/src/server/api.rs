use axum::{
    Router,
    routing::{get, post},
};
use scamplers_core::{
    endpoint::Endpoint,
    model::{
        institution::{Institution, InstitutionQuery, NewInstitution},
        lab::{Lab, LabQuery, LabSummary, NewLab},
        person::{NewPerson, Person, PersonQuery, PersonSummary},
    },
};
use scamplers_schema::lab::dsl::lab;
use uuid::Uuid;

use crate::server::api::handler::{by_id, by_query, new_user, relatives, write};

use super::AppState;

mod error;
mod handler;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(|| async {}))
        .route(
            &Endpoint::<NewInstitution, Institution>::route(),
            post(write::<NewInstitution>),
        )
        .route(
            &Endpoint::<Uuid, Institution>::route(),
            get(by_id::<Institution>),
        )
        .route(
            &Endpoint::<InstitutionQuery, Institution>::route(),
            post(by_query::<Institution>),
        )
        .route(
            &Endpoint::<NewPerson, Person>::route(),
            post(write::<NewPerson>),
        )
        .route(&NewPerson::new_user_route(), post(new_user))
        .route(&Endpoint::<Uuid, Person>::route(), get(by_id::<Person>))
        .route(
            &Endpoint::<PersonQuery, PersonSummary>::route(),
            post(by_query::<PersonSummary>),
        )
        .route(&Endpoint::<NewLab, Lab>::route(), post(write::<NewLab>))
        .route(&Endpoint::<Uuid, Lab>::route(), get(by_id::<Lab>))
        .route(
            &Endpoint::<LabQuery, LabSummary>::route(),
            post(by_query::<LabSummary>),
        )
        .route(
            &format!("{}/members", Endpoint::<Uuid, Lab>::route()),
            get(relatives::<lab, PersonSummary>),
        )
}
