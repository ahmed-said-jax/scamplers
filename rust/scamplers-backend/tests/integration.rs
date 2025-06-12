use pretty_assertions::assert_eq;
use scamplers_backend::{
    config::Config,
    server::{self, util::DevContainer},
};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn prod_api() {
    let container = DevContainer::new("scamplers-backend_integration_test", true)
        .await
        .unwrap();

    let institution_id = Uuid::now_v7();

    let seed_data = json!({
        "institution": {
            "id": institution_id,
            "name": "Hogwarts School for Witchcraft and Wizardry"
        },
        "app_admin": {
            "name": "Ahmed",
            "email": "ahmed.said@jax.org",
            "institution_id": institution_id
        },
        "index_set_urls": []
    });

    let db_root_password = container.password().unwrap();
    let db_login_user_password = Uuid::now_v7().to_string();
    let db_host = container.db_host().await.unwrap();
    let db_port = container.db_port().await.unwrap();

    let config = json!({
      "dev": false,
      "db_root_user": "postgres",
      "db_root_password": db_root_password,
      "db_login_user_password": db_login_user_password,
      "db_host": db_host,
      "db_port": db_port,
      "db_name": "postgres",
      "frontend_token": "",
      "host": "localhost",
      "port": 8000,
      "seed_data": seed_data
    });

    let config: Config = serde_json::from_value(config).unwrap();
    let app_address = format!("http://{}", config.app_address());
    let server_handle = tokio::spawn(server::serve(config, None));

    let client = reqwest::Client::new();

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let response = client
        .get(&app_address)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert_eq!(response, "");

    let test_endpoint = format!("{app_address}/institutions/search");

    let response: serde_json::Value = client
        .post(&test_endpoint)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let invalid_api_key_response = json!({
        "error": {
            "type":"invalid_api_key"
        },
        "status":401});

    assert_eq!(invalid_api_key_response, response);

    let response: serde_json::Value = client
        .post(&test_endpoint)
        .header("X-API-Key", "krabby patty secret formula")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(invalid_api_key_response, response);

    server_handle.abort();
}
