use axum::{http::HeaderMap, Router};

use crate::AppState;

pub fn router() -> Router<AppState> {
    // In theory, we should be able to inspect the header and route the request
    // based on the API version set in the header, but I don't know how to do that
    // yet
    v0::router()
}

mod v0 {
    use axum::{routing::get, Router};

    use crate::AppState;

    pub fn router() -> Router<AppState> {
        Router::new()
            .route(
                "/institutions/{institution_id}",
                get(todo!()).post(todo!()).patch(todo!()),
            )
            .route("/people/{person_id}", get(todo!()).patch(todo!()))
            .route("/people/{person_id}/labs", get(todo!()))
            .route("/people/{person_id}/samples", get(todo!()))
            .route("/labs/{lab_id}", get(todo!()).post(todo!()).patch(todo!()))
            .route("/labs/{lab_id}/pi", get(todo!()))
            .route("/labs/{lab_id}/members", get(todo!()))
            .route("/labs/{lab_id}/samples", get(todo!()))
            .route("/labs/{lab_id}/datasets", get(todo!()))
            .route(
                "/samples/{sample_id}",
                get(todo!()).post(todo!()).patch(todo!()),
            )
            .route("/samples/{sample_id}/datasets", get(todo!()))
            .route(
                "/datasets/{dataset_id}",
                get(todo!()).post(todo!()).patch(todo!()),
            )
            .route(
                "/libraries/{library_id}",
                get(todo!()).post(todo!()).patch(todo!()),
            )
            .route(
                "/sequencing_runs/{sequencing_run_id}",
                get(todo!()).post(todo!()).patch(todo!()),
            )
            .route(
                "/sequencing_runs/{sequencing_run_id}/libraries",
                get(todo!()),
            )
            .route(
                "/chromium_runs/{chromium_run_id}",
                get(todo!()).post(todo!()).patch(todo!()),
            ) // not public
            .route("/chromium_runs/{chromium_run_id}/libraries", get(todo!())) // not public
    }
}
