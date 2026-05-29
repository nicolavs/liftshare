use crate::errors::Error;
use crate::{models::trips_api, repositories::trip_repo, state::SharedState};
use axum::routing::{get, patch, post};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

pub fn create_route() -> Router<SharedState> {
    Router::new()
        .route("/trip", post(create_trip))
        .route("/trip", get(search_trips))
        .route("/trip/{id}", patch(update_trip))
        .route("/trip/join", post(join_trip))
}

/// Create a new Trip
///
/// Tries to create a new Trip
#[utoipa::path(
    post,
    path = "/trip",
    request_body = trips_api::CreateTripRequest,
    responses(
        (status = 201, description = "Create a new trip", body = trips_api::CreateTripResponse)
    ),
    tag = "Liftshare",
)]
pub async fn create_trip(
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

/// Search Trips
///
/// Search for available trips by start location, end location, and departure time
#[utoipa::path(
    get,
    path = "/trip",
    params(
        trips_api::SearchTripsQuery
    ),
    responses(
        (status = 200, description = "Trip search results", body = trips_api::SearchTripsResponse)
    ),
    tag = "Liftshare",
)]
pub async fn search_trips(
    State(state): State<SharedState>,
    search_query: Query<trips_api::SearchTripsQuery>,
) -> Result<impl IntoResponse, Error> {
    let search_query: trips_api::SearchTripsQuery = search_query.0;

    let trips = trip_repo::search(&state.db_pool, search_query).await?;

    Ok((
        StatusCode::OK,
        Json(trips_api::SearchTripsResponse {
            trips: trips
                .into_iter()
                .map(trips_api::TripResponse::from)
                .collect(),
        }),
    ))
}

/// End a Trip
///
/// Sets the trip end time, marking the trip as completed
#[utoipa::path(
    patch,
    path = "/trip/{id}",
    params(
        ("id" = Uuid, Path, description = "Trip id")
    ),
    request_body = trips_api::UpdateTripRequest,
    responses(
        (status = 200, description = "Trip ended successfully", body = trips_api::UpdateTripResponse),
        (status = 404, description = "Trip not found", body = trips_api::UpdateTripResponse)
    ),
    tag = "Liftshare",
)]
pub async fn update_trip(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(req): Json<trips_api::UpdateTripRequest>,
) -> Result<impl IntoResponse, Error> {
    trip_repo::update(&state.db_pool, id, req).await?;

    Ok((
        StatusCode::OK,
        Json(trips_api::UpdateTripResponse { success: true }),
    ))
}

/// Join a Trip
///
/// Passenger joins an available trip, reserving seats from the pickup location
#[utoipa::path(
    post,
    path = "/trip/join",
    request_body = trips_api::JoinTripRequest,
    responses(
        (status = 201, description = "Successfully joined trip", body = trips_api::JoinTripResponse),
        (status = 400, description = "Trip is full or seats insufficient", body = trips_api::JoinTripResponse),
        (status = 404, description = "Trip not found", body = trips_api::JoinTripResponse)
    ),
    tag = "Liftshare",
)]
pub async fn join_trip(
    State(state): State<SharedState>,
    Json(req): Json<trips_api::JoinTripRequest>,
) -> Result<impl IntoResponse, Error> {
    let (eta, pickup_latlng) = trip_repo::join(&state.db_pool, req).await?;

    Ok((
        StatusCode::CREATED,
        Json(trips_api::JoinTripResponse {
            eta: Some(eta),
            pickup_location: Some(pickup_latlng),
            success: true,
        }),
    ))
}
