use crate::{models::trips_api, repositories::trip_repo, state::SharedState};
use axum::routing::post;
use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse};

use crate::errors::Error;

pub fn create_route() -> Router<SharedState> {
    Router::new().route("/trip/create", post(create))
}

#[utoipa::path(
    post,
    path = "/trip/create",
    request_body = trips_api::CreateTripRequest,
    responses(
        (status = 201, description = "Create a new trip", body = trips_api::CreateTripResponse)
    ),
    tag = "Liftshare",
)]
pub async fn create(
    State(state): State<SharedState>,
    Json(req): Json<trips_api::CreateTripRequest>,
) -> Result<impl IntoResponse, Error> {
    let trip_start_time = req.trip_start_time;
    let start_location = req.start_location.clone();
    let end_location = req.end_location.clone();
    let car_capacity = req.car_capacity;

    let id = trip_repo::create(&state.db_pool, req).await?;

    Ok((
        StatusCode::CREATED,
        Json(trips_api::CreateTripResponse {
            id,
            start_location: Some(start_location),
            end_location: Some(end_location),
            trip_start_time,
            trip_end_time: None,
            car_capacity,
            car_capacity_used: 0,
            car_full: false,
        }),
    ))
}
