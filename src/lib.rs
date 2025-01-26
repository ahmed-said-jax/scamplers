use diesel_async::{pooled_connection::deadpool::Pool, AsyncPgConnection};

pub mod api_v1;
pub mod db;
pub mod schema;

#[derive(Clone)]
struct AppState {
    db_pool: Pool<AsyncPgConnection>,
    http_client: reqwest::Client,
    production: bool,
}
