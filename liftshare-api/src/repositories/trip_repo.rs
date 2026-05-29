use crate::errors::Error;
use crate::models::trips_api::{
    CreateTripRequest, JoinTripRequest, LatLng, SearchTripsQuery, UpdateTripRequest,
};
use crate::models::trips_db::Trip;
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
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

// ST_Distance with GEOGRAPHY inputs returns metres
const PICKUP_RADIUS_M: f64 = 2_000.0;
const DESTINATION_RADIUS_M: f64 = 2_000.0;

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

    let mut duration_s: f64 = 0.0;

    for (step, step_data) in route.steps.iter().enumerate() {
        sqlx::query!(
            "INSERT INTO trip_routes (trip_id, step, location, geom, route_distance_m, route_duration_s, eta)
             VALUES ($1, $2, $3, ST_SetSRID(ST_MakePoint($4, $5), 4326), $6, $7, $8)",
            trip.id,
            step as i32,
            step_data.location,
            step_data.lng,
            step_data.lat,
            step_data.distance_m as f64,
            step_data.duration_s as f64,
            req.trip_start_time + Duration::milliseconds((duration_s * 1000.0) as i64)
        )
        .execute(&mut *tx)
        .await?;

        duration_s += step_data.duration_s as f64;
    }

    tx.commit().await?;

    Ok(trip.id)
}

pub async fn join(pool: &PgPool, req: JoinTripRequest) -> Result<(DateTime<Utc>, LatLng), Error> {
    let (start_lat, start_lng) = geocode(&req.pickup_location).await?;

    let mut tx = pool.begin().await?;

    let updated = sqlx::query!(
        "UPDATE trips
         SET car_capacity_used = car_capacity_used + $2
         WHERE id = $1
           AND car_full = FALSE
           AND car_capacity - car_capacity_used >= $2",
        req.trip_id,
        req.num_passengers,
    )
    .execute(&mut *tx)
    .await?;

    if updated.rows_affected() == 0 {
        let exists = sqlx::query_scalar!("SELECT 1 FROM trips WHERE id = $1", req.trip_id)
            .fetch_optional(&mut *tx)
            .await?;
        return Err(if exists.is_some() {
            Error::bad_request()
        } else {
            Error::not_found()
        });
    }

    sqlx::query!(
        "INSERT INTO trip_user_rel (trip_id, user_id, pickup_location, pickup_geom, num_passengers)
         VALUES ($1, $2, $3, ST_SetSRID(ST_MakePoint($4, $5), 4326), $6)",
        req.trip_id,
        req.user_id,
        req.pickup_location,
        start_lng,
        start_lat,
        req.num_passengers,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let row = sqlx::query!(
        r#"
        SELECT eta,
               ST_Y(geom::geometry) AS "lat!: f64",
               ST_X(geom::geometry) AS "lng!: f64"
        FROM trip_routes
        WHERE trip_id = $1
          AND ST_Distance(
                      geom,
                      ST_SetSRID(ST_MakePoint($2, $3), 4326)::geography
              ) <= $4
        ORDER BY ST_Distance(
                         geom,
                         ST_SetSRID(ST_MakePoint($2, $3), 4326)::geography
                 )
        LIMIT 1
        "#,
        req.trip_id,
        start_lng,
        start_lat,
        PICKUP_RADIUS_M
    )
    .fetch_one(pool)
    .await?;

    Ok((
        row.eta,
        LatLng {
            lat: row.lat,
            lng: row.lng,
        },
    ))
}

pub async fn search(pool: &PgPool, req: SearchTripsQuery) -> Result<Vec<Trip>, Error> {
    let limit = req.limit.unwrap_or(10);
    let (start_lat, start_lng) = geocode(&req.start_location).await?;
    let (end_lat, end_lng) = geocode(&req.end_location).await?;

    let query_result = sqlx::query_as_unchecked!(
        Trip,
        r#"
        WITH ongoing AS (
            SELECT * FROM trips
            WHERE trip_end_time IS NULL AND car_full = FALSE
            AND ST_Distance(
                end_geom,
                ST_SetSRID(ST_MakePoint($5, $4), 4326)::geography
              ) <= $7
        ),
        near_pickup_ids AS (
            SELECT DISTINCT t.id
            FROM ongoing t
            INNER JOIN trip_routes tr ON tr.trip_id = t.id
            WHERE tr.eta >= $1
              AND ST_Distance(
                    tr.geom,
                    ST_SetSRID(ST_MakePoint($3, $2), 4326)::geography
                  ) <= $6
        ),
        candidates AS (
            SELECT t.*
            FROM ongoing t
            WHERE t.id IN (SELECT id FROM near_pickup_ids)
        )
        SELECT *
        FROM candidates
        ORDER BY ST_Distance(
                    end_geom,
                    ST_SetSRID(ST_MakePoint($5, $4), 4326)::geography
                 )
        LIMIT $8
        "#,
        req.trip_start_time,  // $1
        start_lat,            // $2
        start_lng,            // $3
        end_lat,              // $4
        end_lng,              // $5
        PICKUP_RADIUS_M,      // $6
        DESTINATION_RADIUS_M, // $7
        limit,                // $8
    )
    .fetch_all(pool)
    .await?;

    Ok(query_result)
}

pub async fn update(pool: &PgPool, trip_id: Uuid, req: UpdateTripRequest) -> Result<(), Error> {
    let result = sqlx::query!(
        "UPDATE trips SET trip_end_time = $2 WHERE id = $1",
        trip_id,
        req.trip_end_time,
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(Error::not_found());
    }

    Ok(())
}
