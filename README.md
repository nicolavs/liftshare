# Liftshare API

Axum-based Rust REST API for a ridesharing service. Drivers create trips; passengers search and join them.

## Quick Start

```bash
# Start PostGIS database
docker-compose up -d

# Copy and configure environment
cp liftshare-api/.env.example liftshare-api/.env

# Run migrations
cd liftshare-api && cargo sqlx migrate run

# Start the server (listens on :3000)
cargo run
```

## Tech Stack

| Layer | Choice                                          |
|---|-------------------------------------------------|
| Web framework | axum 0.8                                        |
| DB driver | sqlx (PostgreSQL, compile-time checked queries) |
| DB | PostgreSQL 18 + PostGIS 3 (docker-compose)      |
| Routing | OSRM (public demo endpoint)                     |
| Geocoding | Nominatim / OpenStreetMap (public endpoint)     |
| Serialization | serde / serde_json                              |
| Async runtime | tokio                                           |
| Config | env vars + dotenvy (`.env` in dev)              |
| Docs | utoipa (OpenAPI)                                |

## API Routes

### `POST /trip` — Create a trip (driver)

```json
{
  "user_id": "<uuid>",
  "start_location": "Sydney Central",
  "end_location": "North Sydney",
  "trip_start_time": "2026-05-16T10:00:00Z",
  "car_capacity": 4
}
```

Geocodes start/end via Nominatim, fetches turn-by-turn route from OSRM, stores route steps with per-step ETA.

### `GET /trip?start_location=&end_location=&trip_start_time=` — Search trips (passenger)

Returns trips whose route passes within **2 km** of the passenger's pickup and whose destination is within **2 km** of the passenger's destination, ordered by distance to destination. Only returns trips that are not full and have not ended.

### `PATCH /trip/{id}` — End a trip (driver)

```json
{ "trip_id": "<uuid>", "trip_end_time": "2026-05-16T11:00:00Z" }
```

### `POST /trip/join` — Join a trip (passenger)

```json
{
  "trip_id": "<uuid>",
  "user_id": "<uuid>",
  "pickup_location": "Town Hall",
  "num_passengers": 2
}
```

Atomically reserves seats and returns the ETA for the nearest route waypoint to the pickup.

## Database Schema

Three tables:

- **`trips`** — trip header; `car_full` is a generated column (`car_capacity_used >= car_capacity`)
- **`trip_routes`** — one row per OSRM route step, with `eta TIMESTAMPTZ` computed at creation time
- **`trip_user_rel`** — passenger booking; stores geocoded `pickup_geom`

## Limitations

- **No authentication** — `user_id` is a bare UUID passed in the request body; anyone can act as any user
- **No user table** — users are not validated or stored; `user_id` is trust-based
- **Passenger destination must match driver destination** — search matches trips whose `end_geom` is within 2 km of the passenger's requested end location; there is no intermediate drop-off support
- **External service dependency** — geocoding (Nominatim) and routing (OSRM) use public demo endpoints with no rate-limit handling or caching; production use requires self-hosted or paid services
- **No trip state machine** — a trip is "ongoing" when `trip_end_time IS NULL`; there is no cancelled/pending state
