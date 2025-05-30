use std::{collections::HashMap, hash::RandomState};

use axum::{
    Router,
    routing::{get, post},
};
use scamplers_core::{model::AsEndpoint, model::person::CreatedUser};

use super::AppState2;

mod error;
mod handler;

pub(super) fn router() -> Router<AppState2> {
    use handler::new_user;

    let endpoints: HashMap<&str, [&str; 1], RandomState> =
        HashMap::from_iter([("available_endpoints", [""])]);

    let router = Router::new()
        .route("/", get(|| async { axum::Json(endpoints) }))
        .route(CreatedUser::as_endpoint(), post(new_user));
    router
}
