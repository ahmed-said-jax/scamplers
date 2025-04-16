use axum::extract::State;
use serde_json::json;
use uuid::Uuid;
use valuable::Valuable;

use crate::{
    AppState2,
    api::{self, ValidJson},
    auth::{AuthService, Key},
    db::{Create, person::NewPerson, web_session::NewSession},
};

pub async fn new_session(
    _auth_service: AuthService,
    State(app_state): State<AppState2>,
    ValidJson(person): ValidJson<NewPerson>,
) -> api::Result<ValidJson<serde_json::Value>> {
    tracing::debug!(deserialized_person = person.as_value());

    let session_id = Key::new();
    let hashed_id = session_id.hash();

    let session = NewSession {
        hashed_id,
        person,
        user_id: Uuid::default(),
    };

    let mut conn = app_state.db_conn().await?;

    session.create(&mut conn).await?;

    let response = json!({"session_id": session_id});

    Ok(ValidJson(response))
}
