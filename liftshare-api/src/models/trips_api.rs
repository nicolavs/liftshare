use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::models::trips_db::Trip;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LatLng {
    pub lat: f64,
    pub lng: f64,
}

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

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchTripsResponse {
    pub trips: Vec<TripResponse>,
}

impl From<Trip> for TripResponse {
    fn from(t: Trip) -> Self {
        Self {
            id: t.id,
            user_id: t.user_id,
            start_location: t.start_location,
            end_location: t.end_location,
            trip_start_time: t.trip_start_time,
            trip_end_time: t.trip_end_time,
            car_capacity: t.car_capacity,
            car_capacity_used: t.car_capacity_used,
            car_full: t.car_full,
            route_distance_m: t.route_distance_m,
            route_duration_s: t.route_duration_s,
        }
    }
}

// GET /trips/search
#[derive(Debug, Deserialize, IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
pub struct SearchTripsQuery {
    pub trip_start_time: DateTime<Utc>,
    pub start_location: String,
    pub end_location: String,
    pub limit: Option<i32>,
}

// POST /trip/join
#[derive(Debug, Deserialize, ToSchema)]
pub struct JoinTripRequest {
    pub trip_id: Uuid,
    pub user_id: Uuid,
    pub pickup_location: String,
    pub num_passengers: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct JoinTripResponse {
    pub eta: Option<DateTime<Utc>>,
    pub pickup_location: Option<LatLng>,
    pub success: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTripRequest {
    pub trip_end_time: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UpdateTripResponse {
    pub success: bool,
}
