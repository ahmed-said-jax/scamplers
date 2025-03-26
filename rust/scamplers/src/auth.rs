use std::ops::Deref;

use axum::{Router, extract::State, routing::get};

use crate::AppState2;

pub fn router() -> Router<AppState2> {
    Router::new().route("/ms", get(ms_entra_login))
}

async fn ms_entra_login(
    State(app_state): State<AppState2>,
    axum::extract::RawQuery(s): axum::extract::RawQuery,
    body: String,
) -> String {
    let s = s.unwrap_or_default();
    tracing::debug!("{s}");
    tracing::debug!("{body}");

    body
}
