use axum::extract::State;
use axum::{Json, Router, http::StatusCode, routing::get};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;

pub fn create_route() -> Router<PgPool> {
    Router::new().route("/health", get(get_health))
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct Status {
    status: String,
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Get health check", body = Status)
    ),
    tag = "Liftshare",
)]
async fn get_health(State(pool): State<PgPool>) -> (StatusCode, Json<Status>) {
    let _ = pool;
    // insert your application logic here
    let status = Status {
        status: "ok".to_owned(),
    };

    (StatusCode::OK, Json(status))
}
