use axum::{routing::get, Router};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(todo!()))
        .route("institutions/{institution_id}", get(todo!()).post(todo!()).patch(todo!()))
        .route("people/{person_id}", get(todo!()).patch(todo!()))
        .route("people/{person_id}/labs", get(todo!()))
        .route("people/{person_id}/samples", get(todo!()))
        .route("labs/{lab_id}", get(todo!()).post(todo!()).patch(todo!()))
        .route("labs/{lab_id}/pi", get(todo!()))
        .route("labs/{lab_id}/members", get(todo!()))
        .route("labs/{lab_id}/samples", get(todo!()))
        .route("labs/{lab_id}/datasets", get(todo!()))
        .route("samples/{sample_id}", get(todo!()).post(todo!()).patch(todo!()))
        .route("samples/{sample_id}/datasets", get(todo!()))
        .route("datasets/{dataset_id}", get(todo!()).post(todo!()).patch(todo!()))
        .route("libraries/{library_id}", get(todo!()).post(todo!()).patch(todo!()))
        .route("sequencing_runs/{sequencing_run_id}", get(todo!()).post(todo!()).patch(todo!()))
        .route("sequencing_runs/{sequencing_run_id}/libraries", get(todo!()))
        .route("chromium_runs/{chromium_run_id}", get(todo!()).post(todo!()).patch(todo!())) // not public
}
