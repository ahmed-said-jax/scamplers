use axum::{routing::get, Router, extract::State};

use crate::AppState;

pub fn router(state: State<AppState>) -> Router<AppState> {
    Router::new().nest("/", institution::router(state))
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
    use axum::{
        extract::{Path, Request, State, FromRequestParts}, http::Method, middleware::Next, response::{IntoResponse, Response}, routing::get, Router
    };
    use uuid::Uuid;
    use super::super::ApiUser;
    use super::super::Error as ApiError;

    use crate::{api::ApiResponse, AppState};

    pub fn router(State(state): State<AppState>) -> Router<AppState> {
        use axum::middleware;
        use crate::db::Entity::Institution;

        let mut router = Router::new().route(Institution.route(), get(get_institutions));
        
        if state.production {
            router = router.layer(middleware::from_fn_with_state(state.clone(), permissions))
        }

        router
    }

    async fn permissions(ApiUser { roles, .. }: ApiUser, request: Request, next: Next) -> Response {
        use super::super::UserRole::Admin;

        let is_admin = roles.contains(&Admin);
        let response= match (request.method(), is_admin) {
            (&Method::GET, _) => next.run(request).await, // GET is always allowed
            (&Method::POST | &Method::PATCH, true) => next.run(request).await, // POST and PATCH require admin
            _ => ApiError::permission().into_response() // everything else is forbidden
        };

        response
    }

    async fn get_institutions(State(app_state): State<AppState>, institution_id: Option<Path<Uuid>>) -> super::super::Result<ApiResponse> {
        use crate::db::institution::Institution;

        let mut conn = app_state.db_pool.get().await?;

        let Some(Path(institution_id)) = institution_id else {
            let institutions = Institution::fetch_all(&mut conn, Default::default()).await?;
            return Ok(ApiResponse::from(institutions))
        };

        let institution = Institution::fetch_by_id(&mut conn, institution_id).await?;
        Ok(ApiResponse::from(institution))
    }
}
