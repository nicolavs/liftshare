use axum::extract::State;
use axum::{Json, Router, http::StatusCode, routing::get};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::state::SharedState;

pub fn create_route() -> Router<SharedState> {
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
async fn get_health(State(state): State<SharedState>) -> (StatusCode, Json<Status>) {
    let _ = state;
    // insert your application logic here
    let status = Status {
        status: "ok".to_owned(),
    };

    (StatusCode::OK, Json(status))
}
