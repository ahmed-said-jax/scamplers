use axum::{Router, extract::State, routing::get};

use crate::AppState2;

pub fn router() -> Router<AppState2> {
    Router::new().route("/", get(async || "hello auth"))
}
