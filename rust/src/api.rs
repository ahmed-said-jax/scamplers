use axum::Router;

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
                get::<(), _, _>(todo!())
                    .post::<(), _>(todo!())
                    .patch::<(), _>(todo!()),
            )
            .route(
                "/people/{person_id}",
                get::<(), _, _>(todo!()).patch::<(), _>(todo!()),
            )
            .route("/people/{person_id}/labs", get::<(), _, _>(todo!()))
            .route("/people/{person_id}/samples", get::<(), _, _>(todo!()))
            .route(
                "/labs/{lab_id}",
                get::<(), _, _>(todo!())
                    .post::<(), _>(todo!())
                    .patch::<(), _>(todo!()),
            )
            .route("/labs/{lab_id}/pi", get::<(), _, _>(todo!()))
            .route("/labs/{lab_id}/members", get::<(), _, _>(todo!()))
            .route("/labs/{lab_id}/samples", get::<(), _, _>(todo!()))
            .route("/labs/{lab_id}/datasets", get::<(), _, _>(todo!()))
            .route(
                "/samples/{sample_id}",
                get::<(), _, _>(todo!())
                    .post::<(), _>(todo!())
                    .patch::<(), _>(todo!()),
            )
            .route("/samples/{sample_id}/datasets", get::<(), _, _>(todo!()))
            .route(
                "/datasets/{dataset_id}",
                get::<(), _, _>(todo!())
                    .post::<(), _>(todo!())
                    .patch::<(), _>(todo!()),
            )
            .route(
                "/libraries/{library_id}",
                get::<(), _, _>(todo!())
                    .post::<(), _>(todo!())
                    .patch::<(), _>(todo!()),
            )
            .route(
                "/sequencing_runs/{sequencing_run_id}",
                get::<(), _, _>(todo!())
                    .post::<(), _>(todo!())
                    .patch::<(), _>(todo!()),
            )
            .route(
                "/sequencing_runs/{sequencing_run_id}/libraries",
                get::<(), _, _>(todo!()),
            )
            .route(
                "/chromium_runs/{chromium_run_id}",
                get::<(), _, _>(todo!())
                    .post::<(), _>(todo!())
                    .patch::<(), _>(todo!()),
            ) // not public
            .route(
                "/chromium_runs/{chromium_run_id}/libraries",
                get::<(), _, _>(todo!()),
            ) // not public
    }
}
