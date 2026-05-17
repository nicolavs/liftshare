use axum::{Json, Router, http::StatusCode, routing::get};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub fn create_route() -> Router {
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
async fn get_health() -> (StatusCode, Json<Status>) {
    // insert your application logic here
    let status = Status {
        status: "ok".to_owned(),
    };

    (StatusCode::OK, Json(status))
}
