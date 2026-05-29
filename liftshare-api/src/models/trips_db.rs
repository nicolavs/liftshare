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
    #[allow(dead_code)]
    pub start_geom: wkb::Decode<Geometry<f64>>,
    #[allow(dead_code)]
    pub end_geom: wkb::Decode<Geometry<f64>>,
    pub trip_start_time: DateTime<Utc>,
    pub trip_end_time: Option<DateTime<Utc>>,
    pub car_capacity: i32,
    pub car_capacity_used: i32,
    pub car_full: bool,
    pub route_distance_m: Option<f32>,
    pub route_duration_s: Option<f32>,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
}
