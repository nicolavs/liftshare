# Liftshare API

Axum-based Rust REST API for a ridesharing service. Drivers create trips; passengers search and join them.

## Project Layout

```
liftshare/
├── CLAUDE.md
├── README.md
├── docker-compose.yml                    # PostGIS PostgreSQL 16
└── liftshare-api/
    ├── Cargo.toml
    ├── .env                              # DATABASE_URL (git-ignored)
    ├── migrations/
    │   └── 20260517092657_init-tables.sql
    └── src/
        ├── main.rs                       # entry point, dotenvy, axum serve
        ├── app.rs                        # router assembly + utoipa OpenAPI
        ├── db.rs                         # sqlx PgPool setup
        ├── settings.rs                   # config via env vars (once_cell Lazy)
        ├── state.rs                      # SharedState (PgPool)
        ├── errors.rs                     # Error enum → axum IntoResponse
        ├── models/
        │   ├── mod.rs
        │   ├── trips_api.rs              # request/response structs
        │   └── trips_db.rs              # DB row structs (FromRow)
        ├── repositories/
        │   ├── mod.rs
        │   └── trip_repo.rs             # geocode, OSRM, all DB queries
        └── routes/
            ├── mod.rs
            ├── health.rs
            └── trip_handlers.rs         # axum handlers + utoipa docs
```

## Tech Stack

| Layer | Choice |
|---|---|
| Web framework | axum 0.8 |
| DB driver | sqlx (PostgreSQL, compile-time checked queries) |
| DB | PostgreSQL 16 + PostGIS 3 (docker-compose) |
| Routing | OSRM public demo endpoint |
| Geocoding | Nominatim / OpenStreetMap public endpoint |
| Serialization | serde / serde_json |
| Async runtime | tokio |
| Config | env vars + dotenvy (`.env` in dev) |
| Docs | utoipa (OpenAPI / Swagger UI) |

## Running Locally

```bash
docker-compose up -d
cd liftshare-api
cargo sqlx migrate run
cargo run
```

`.env` must contain:
```
DATABASE_URL=postgres://liftshare:liftshare@localhost:5432/liftshare
```

## API Routes

### `POST /trip` — Create trip (driver)

Geocodes `start_location` and `end_location` via Nominatim, fetches turn-by-turn from OSRM, inserts into `trips` and `trip_routes` (one row per step with computed `eta`).

### `GET /trip` — Search trips (passenger)

Query params: `start_location`, `end_location`, `trip_start_time`, `limit` (optional, default 10).

CTE query:
1. `ongoing` — trips not ended and not full
2. `near_pickup_ids` — trip IDs where a route step is within `PICKUP_RADIUS_M` (2 km) of geocoded start and `eta >= trip_start_time`
3. `candidates` — full trip rows for those IDs
4. Final filter: `end_geom` within `DESTINATION_RADIUS_M` (2 km) of geocoded end, ordered by distance to end

### `PATCH /trip/{id}` — End trip (driver)

Sets `trip_end_time`. Returns `404` if ID unknown.

### `POST /trip/join` — Join trip (passenger)

Atomically:
1. `UPDATE trips SET car_capacity_used = car_capacity_used + $2 WHERE car_full = FALSE AND remaining >= $2` — returns `400` if full, `404` if not found
2. Inserts `trip_user_rel` with geocoded `pickup_geom`
3. Queries nearest `trip_routes` waypoint within `PICKUP_RADIUS_M` for ETA

## Database Schema

### `trips`
- `car_full BOOLEAN GENERATED ALWAYS AS (car_capacity_used >= car_capacity) STORED` — never written directly
- Spatial indexes on `start_geom`, `end_geom`

### `trip_routes`
- One row per OSRM step; `eta TIMESTAMPTZ` = `trip_start_time + cumulative_duration_ms`
- GIST index on `geom`, B-tree index on `eta`

### `trip_user_rel`
- PK: `(trip_id, user_id)` — one booking per passenger per trip
- `pickup_geom GEOGRAPHY(Point, 4326)` from geocoded pickup address

## Proximity Constants (`trip_repo.rs`)

```rust
const PICKUP_RADIUS_M: f64 = 2_000.0;      // route waypoint → passenger pickup
const DESTINATION_RADIUS_M: f64 = 2_000.0; // trip end_geom → passenger destination
```

## Error Handling

`Error` enum in `errors.rs` maps to HTTP via `IntoResponse`:

| Variant | Status |
|---|---|
| `NotFound` | 404 |
| `BadRequest` | 400 |
| `GeocodeMiss(String)` | 400 |
| `External(reqwest::Error)` | 500 |
| `Db(sqlx::Error)` | 500 |

All error bodies: `{ "code": <u16>, "message": "<string>" }`.

## Key Patterns

- `query_as_unchecked!` for queries returning `GEOGRAPHY` columns (sqlx has no built-in geography type mapping)
- `query_as!` for all other queries
- Handlers are thin: extract → call repo fn → return response
- No `unwrap()` / `expect()` outside tests

## Limitations

- **No authentication** — `user_id` is a bare UUID in the request body
- **No user table** — users are not validated or stored
- **Passenger destination must match driver destination** — only trips whose `end_geom` is within 2 km of the passenger's requested end are returned; no intermediate drop-off
- **External service dependency** — Nominatim and OSRM are public demo endpoints with no rate limiting or caching
- **No trip state machine** — ongoing = `trip_end_time IS NULL`; no cancelled/pending states
