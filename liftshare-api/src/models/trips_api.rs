use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// POST /trip/create
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTripRequest {
    pub user_id: Uuid,
    pub start_location: String,
    pub end_location: String,
    pub trip_start_time: DateTime<Utc>,
    pub car_capacity: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateTripResponse {
    pub id: Uuid,
    pub start_location: Option<String>,
    pub end_location: Option<String>,
    pub trip_start_time: DateTime<Utc>,
    pub trip_end_time: Option<DateTime<Utc>>,
    pub car_capacity: i32,
    pub car_capacity_used: i32,
    pub car_full: bool,
}

// GET /trip/:trip_id
#[derive(Debug, Serialize, ToSchema)]
pub struct TripResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub start_location: Option<String>,
    pub end_location: Option<String>,
    pub trip_start_time: DateTime<Utc>,
    pub trip_end_time: Option<DateTime<Utc>>,
    pub car_capacity: i32,
    pub car_capacity_used: i32,
    pub car_full: bool,
    pub route_distance_m: Option<f32>,
    pub route_duration_s: Option<f32>,
}

// GET /trips/search
#[derive(Debug, Deserialize)]
pub struct SearchTripsQuery {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Serialize)]
pub struct TripSummary {
    pub id: Uuid,
    pub start_location: Option<String>,
    pub end_location: Option<String>,
    pub trip_start_time: DateTime<Utc>,
    pub car_capacity: i32,
    pub car_capacity_used: i32,
}

// POST /trip/join
#[derive(Debug, Deserialize)]
pub struct JoinTripRequest {
    pub trip_id: Uuid,
    pub user_id: Uuid,
    pub pickup_location: String,
    pub num_passengers: i32,
}

#[derive(Debug, Serialize)]
pub struct JoinTripResponse {
    pub message: String,
}
