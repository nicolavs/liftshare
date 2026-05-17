use crate::errors::Error;
use crate::models::trips_api::CreateTripRequest;
use serde::Deserialize;
use sqlx::PgPool;

// ── Geocode ──────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct NominatimResult {
    lat: String,
    lon: String,
}

async fn geocode(location: &str) -> Result<(f64, f64), Error> {
    let results: Vec<NominatimResult> = reqwest::Client::new()
        .get("https://nominatim.openstreetmap.org/search")
        .query(&[("q", location), ("format", "jsonv2")])
        .header("User-Agent", "liftshare-api/0.1")
        .send()
        .await?
        .json()
        .await?;

    let first = results
        .into_iter()
        .next()
        .ok_or_else(|| Error::GeocodeMiss(location.to_string()))?;

    let lat = first
        .lat
        .parse::<f64>()
        .map_err(|_| Error::GeocodeMiss(location.to_string()))?;
    let lon = first
        .lon
        .parse::<f64>()
        .map_err(|_| Error::GeocodeMiss(location.to_string()))?;

    Ok((lat, lon))
}

// ── Route ─────────────────────────────────────────────────────────────────────

pub struct RouteStep {
    pub location: String,
    pub lat: f64,
    pub lng: f64,
    pub distance_m: f32,
    pub duration_s: f32,
}

pub struct Route {
    pub distance_m: f32,
    pub duration_s: f32,
    pub steps: Vec<RouteStep>,
}

// OSRM deserialization types
#[derive(Deserialize)]
struct OsrmResponse {
    routes: Vec<OsrmRoute>,
}

#[derive(Deserialize)]
struct OsrmRoute {
    distance: f64,
    duration: f64,
    legs: Vec<OsrmLeg>,
}

#[derive(Deserialize)]
struct OsrmLeg {
    steps: Vec<OsrmStep>,
}

#[derive(Deserialize)]
struct OsrmStep {
    distance: f64,
    duration: f64,
    name: String,
    maneuver: OsrmManeuver,
}

#[derive(Deserialize)]
struct OsrmManeuver {
    // [longitude, latitude]
    location: [f64; 2],
}

async fn get_route(
    start_lat: f64,
    start_lng: f64,
    end_lat: f64,
    end_lng: f64,
) -> Result<Route, Error> {
    let url = format!(
        "https://router.project-osrm.org/route/v1/driving/{},{};{},{}",
        start_lng, start_lat, end_lng, end_lat
    );

    let resp: OsrmResponse = reqwest::Client::new()
        .get(&url)
        .query(&[("steps", "true"), ("overview", "false")])
        .send()
        .await?
        .json()
        .await?;

    let route = resp
        .routes
        .into_iter()
        .next()
        .ok_or_else(|| Error::GeocodeMiss("no route found".to_string()))?;

    let steps = route
        .legs
        .into_iter()
        .flat_map(|leg| leg.steps)
        .map(|step| RouteStep {
            location: step.name,
            lng: step.maneuver.location[0],
            lat: step.maneuver.location[1],
            distance_m: step.distance as f32,
            duration_s: step.duration as f32,
        })
        .collect();

    Ok(Route {
        distance_m: route.distance as f32,
        duration_s: route.duration as f32,
        steps,
    })
}

// ── Repository ────────────────────────────────────────────────────────────────

pub async fn create(pool: &PgPool, req: CreateTripRequest) -> Result<uuid::Uuid, Error> {
    let (start_lat, start_lng) = geocode(&req.start_location).await?;
    let (end_lat, end_lng) = geocode(&req.end_location).await?;

    let route = get_route(start_lat, start_lng, end_lat, end_lng).await?;

    let mut tx = pool.begin().await?;

    let trip = sqlx::query!(
        "INSERT INTO trips (user_id, start_location, start_geom, end_location, end_geom,
                            route_distance_m, route_duration_s, trip_start_time, car_capacity)
         VALUES ($1, $2, ST_SetSRID(ST_MakePoint($3, $4), 4326),
                     $5, ST_SetSRID(ST_MakePoint($6, $7), 4326),
                     $8, $9, $10, $11)
         RETURNING id",
        req.user_id,
        req.start_location,
        start_lng,
        start_lat,
        req.end_location,
        end_lng,
        end_lat,
        route.distance_m as f64,
        route.duration_s as f64,
        req.trip_start_time,
        req.car_capacity,
    )
    .fetch_one(&mut *tx)
    .await?;

    for (step, step_data) in route.steps.iter().enumerate() {
        sqlx::query!(
            "INSERT INTO trip_routes (trip_id, step, location, geom, route_distance_m, route_duration_s)
             VALUES ($1, $2, $3, ST_SetSRID(ST_MakePoint($4, $5), 4326), $6, $7)",
            trip.id,
            step as i32,
            step_data.location,
            step_data.lng,
            step_data.lat,
            step_data.distance_m as f64,
            step_data.duration_s as f64,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(trip.id)
}
