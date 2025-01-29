use axum::{routing::get, Router};

use crate::AppState;

use super::route;

pub fn router() -> Router<AppState> {
    use crate::db::Entity::{Institution as I, Person as P, Lab, Sample as S, Dataset as D, SequencingRun as SR, Library as Lib};
    Router::new()
        .route(route(I), get(todo!()).post(todo!()).patch(todo!()))
        .route(
            route(P),
            get::<(), _, _>(todo!()),
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

mod institution {
    use axum::{extract::{Path, State}, routing::get, Router};
    use uuid::Uuid;

    use crate::AppState;
    use super::super::route;
    use crate::db::Entity::Institution;

    pub fn router() -> Router<AppState> {
        Router::new().route(route(Institution), get(todo!()))
    }

    fn get_institutions(State(app_state): State<AppState>, institution_id: Option<Path<Uuid>>) {

    }
}