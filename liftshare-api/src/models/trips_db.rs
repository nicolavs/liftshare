use chrono::{DateTime, Utc};
use geo_types::Geometry;
use geozero::wkb;
use sqlx::{FromRow, types::Uuid};

#[derive(Debug, FromRow)]
pub struct Trip {
    pub id: Uuid,
    pub user_id: Uuid,
    pub start_location: Option<String>,
    pub end_location: Option<String>,
    pub start_geom: wkb::Decode<Geometry<f64>>,
    pub end_geom: wkb::Decode<Geometry<f64>>,
    pub trip_start_time: DateTime<Utc>,
    pub trip_end_time: Option<DateTime<Utc>>,
    pub car_capacity: i32,
    pub car_capacity_used: i32,
    pub car_full: bool,
    pub route_distance_m: Option<f32>,
    pub route_duration_s: Option<f32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct TripUserRel {
    pub trip_id: Uuid,
    pub user_id: Uuid,
    pub pickup_location: Option<String>,
    pub pickup_geom: Option<wkb::Decode<Geometry<f64>>>,
    pub num_passengers: i32,
}

#[derive(Debug, FromRow)]
pub struct TripRoute {
    pub trip_id: Uuid,
    pub step: i32,
    pub geom: wkb::Decode<Geometry<f64>>,
    pub location: Option<String>,
    pub route_distance_m: Option<f32>,
    pub route_duration_s: Option<f32>,
    pub eta: DateTime<Utc>,
}
