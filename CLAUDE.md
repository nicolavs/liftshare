# Liftshare API

Axum-based Rust REST API for a ridesharing service. Drivers create trips; passengers search and join them.

## Project Layout

```
liftshare/
├── CLAUDE.md
├── docker-compose.yml        # PostGIS PostgreSQL
├── db-and-be-schema.md       # API design reference
├── liftshare.sql             # DB schema (port to PostgreSQL)
└── liftshare-api/
    ├── Cargo.toml
    └── src/
        ├── main.rs
        ├── db.rs             # sqlx pool setup
        ├── models.rs         # shared types / DB row structs
        ├── error.rs          # AppError → axum IntoResponse
        └── routes/
            ├── mod.rs
            ├── trips.rs      # create, get, search
            └── join.rs       # join a trip
```

## Development Stages

Work must proceed **stage by stage**. Do not start a stage until the previous one compiles and its tests pass.

---

### Stage 1 — Infrastructure

Goal: database is reachable and the Rust project compiles.

**Files to create:**
- `docker-compose.yml` — PostGIS PostgreSQL service
- `liftshare-api/.env` — `DATABASE_URL` for local dev (git-ignored)
- `liftshare-api/Cargo.toml` — add all dependencies: `axum`, `tokio`, `sqlx`, `serde`, `serde_json`, `anyhow`, `dotenvy`, `uuid`
- `liftshare-api/src/main.rs` — connect pool, bind axum router to `0.0.0.0:3000`, log startup

**docker-compose.yml shape:**
```yaml
services:
  db:
    image: postgis/postgis:16-3.4
    environment:
      POSTGRES_USER: liftshare
      POSTGRES_PASSWORD: liftshare
      POSTGRES_DB: liftshare
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data

volumes:
  pgdata:
```

**Done when:** `docker-compose up -d` starts the DB and `cargo run` prints the startup message without error.

---

### Stage 2 — Database Migration

Goal: schema is applied and sqlx offline metadata is generated.

**Files to create:**
- `liftshare-api/migrations/0001_init.sql` — port `liftshare.sql` to PostgreSQL (see schema notes in the Database section below)
- `liftshare-api/src/db.rs` — `pub async fn connect() -> PgPool` using `DATABASE_URL`

**Steps:**
1. Install sqlx-cli if needed: `cargo install sqlx-cli --no-default-features --features postgres`
2. `cargo sqlx migrate run` — apply migration
3. `cargo sqlx prepare` — generate `.sqlx/` offline metadata (commit this folder)

**Done when:** `cargo sqlx migrate run` reports all migrations applied and `cargo build` succeeds with offline metadata.

---

### Stage 3 — Shared Scaffolding

Goal: error type and models compile; router skeleton returns `501` for all routes.

**Files to create:**
- `liftshare-api/src/error.rs` — `AppError` enum with `IntoResponse`
- `liftshare-api/src/models.rs` — request/response structs (all routes)
- `liftshare-api/src/routes/mod.rs` — `pub fn router() -> Router` with all four routes stubbed to `StatusCode::NOT_IMPLEMENTED`
- `liftshare-api/src/routes/trips.rs` — stub handlers
- `liftshare-api/src/routes/join.rs` — stub handler

Wire `routes::router()` into `main.rs`.

**Done when:** `cargo build` is clean and `curl localhost:3000/trip/nonexistent` returns `501`.

---

### Stage 4 — Route: `POST /trip/create`

Implement handler, DB insert, and tests before moving on.

**In `routes/trips.rs`:**
1. Parse `CreateTripRequest` from JSON body.
2. Insert into `trips` with `query_as!`, returning the new row.
3. Return `CreateTripResponse` as JSON.

**Tests required (`#[cfg(test)]` in `routes/trips.rs`):**
- `test_post_trip_create_ok` — valid body → `200`, response contains a UUID `trip_id`
- `test_post_trip_create_missing_field` — body missing `car_capacity` → `400`

**Done when:** both tests pass with `cargo test`.

---

### Stage 5 — Route: `GET /trip/:trip_id`

**In `routes/trips.rs`:**
1. Extract `:trip_id` as `uuid::Uuid` from path.
2. Query `trips` joined with `trip_user_rel` count for remaining seats.
3. Return `TripDetailResponse` or `AppError::NotFound`.

**Tests required:**
- `test_get_trip_ok` — insert a trip, fetch it → `200` with correct fields
- `test_get_trip_not_found` — random UUID → `404`

**Done when:** both tests pass.

---

### Stage 6 — Route: `GET /trips/search`

**In `routes/trips.rs`:**
1. Extract `start` and `end` query params (reject if either is missing → `400`).
2. `ILIKE` search on `start_location` and `end_location`.
3. Return array of trip summaries with remaining seats.

**Tests required:**
- `test_get_trips_search_ok` — insert matching trip, search → `200` with it in results
- `test_get_trips_search_no_results` — search for something absent → `200` empty array
- `test_get_trips_search_missing_param` — omit `end` → `400`

**Done when:** all three tests pass.

---

### Stage 7 — Route: `POST /trip/join`

**In `routes/join.rs`:**
1. Parse `JoinTripRequest`.
2. Verify trip exists and has enough remaining seats → `404` / `400`.
3. Insert `trip_user_rel` row.
4. Return `JoinTripResponse` with geocoded `pickup_location` lat/lng and estimated pickup time.

**Tests required:**
- `test_post_trip_join_ok` — valid join → `200` with pickup coords
- `test_post_trip_join_trip_not_found` — unknown `trip_id` → `404`
- `test_post_trip_join_no_capacity` — join when seats are full → `400`

**Done when:** all three tests pass and `cargo test` is fully green.

---

## Tech Stack

| Layer | Choice |
|---|---|
| Web framework | axum |
| DB driver | sqlx (PostgreSQL, compile-time checked queries) |
| DB | PostgreSQL 16 + PostGIS 3 (via docker-compose) |
| Serialization | serde / serde_json |
| Async runtime | tokio |
| Config | `DATABASE_URL` env var (`.env` via dotenvy in dev) |

## Database

Start the database locally:

```bash
docker-compose up -d
```

Connection string (set in `.env` or environment):

```
DATABASE_URL=postgres://liftshare:liftshare@localhost:5432/liftshare
```

Run migrations (sqlx-cli):

```bash
cargo sqlx migrate run
```

Schema lives in `migrations/`. The initial migration ports `liftshare.sql` to PostgreSQL:
- `trips.id` → `UUID` (generated via `gen_random_uuid()`)
- Timestamps → `TIMESTAMPTZ`
- `route_data` → `JSONB`
- `pickup_location` on `trip_user_rel` → `GEOGRAPHY(Point, 4326)` (PostGIS)
- Spatial index on `pickup_location`

## API Routes

Every route **must** have a corresponding integration test in the same file under `#[cfg(test)]`.

### `POST /trip/create`

Driver creates a trip.

Request:
```json
{
    "name": "john",
    "start_location": "Sydney Central",
    "end_location": "North Sydney",
    "trip_start": "2026-05-16T10:00:00Z",
    "car_capacity": 4
}
```

Response `200`:
```json
{
    "trip_id": "<uuid>",
    "start_location": "Sydney Central",
    "end_location": "North Sydney",
    "trip_start_time": "2026-05-16T10:00:00Z",
    "estimated_trip_end_time": "2026-05-16T10:10:00Z",
    "car_capacity": 4,
    "route": {
        "distance_m": 7664.9,
        "duration_s": 571.4,
        "geometry": "<encoded_polyline>",
        "steps": [
            {
                "instruction": "Depart straight",
                "road_name": "Little Pier Street",
                "distance_m": 78.3,
                "duration_s": 23.8,
                "maneuver_type": "depart",
                "modifier": "straight",
                "location": { "lat": -33.877605, "lng": 151.202394 }
            }
        ]
    }
}
```

### `GET /trips/search?start=<location>&end=<location>`

Passenger searches for available trips by start/end location. Returns an array of matching trips with remaining seat count.

### `GET /trip/:trip_id`

Returns full trip detail: trip info, remaining seats, route, and ETA.

### `POST /trip/join`

Passenger joins a trip.

Request:
```json
{
    "trip_id": "<uuid>",
    "pickup_location": "Town Hall",
    "num_passengers": 2
}
```

Response `200`:
```json
{
    "status": "success",
    "message": "Trip joined successfully",
    "pickup_location": { "lat": -33.873, "lng": 151.206 },
    "estimated_pickup_time": "2026-05-16T10:04:00Z"
}
```

## Testing Rules

- Every route handler file must contain `#[cfg(test)] mod tests { ... }` at the bottom.
- Tests spin up a real PostgreSQL connection (use `DATABASE_URL` from environment or a `.env.test`).
- Each test must create its own isolated data and clean up (use a transaction that rolls back, or a unique schema per test).
- Test naming convention: `test_<method>_<route_description>` e.g. `test_post_trip_create_ok`, `test_get_trips_search_no_results`.
- Required test cases per route:
  - Happy path (valid input → expected response shape and status)
  - Validation error (missing/invalid fields → `400`)
  - Not found where applicable (unknown `trip_id` → `404`)

Run all tests:
```bash
cargo test
```

## Error Handling

Use a single `AppError` enum that implements `axum::response::IntoResponse`. Map variants to HTTP status codes:

| Variant | Status |
|---|---|
| `NotFound` | 404 |
| `BadRequest(String)` | 400 |
| `Internal(anyhow::Error)` | 500 |

All error responses use the JSON body `{ "error": "<message>" }`.

## Code Style

- No `unwrap()` or `expect()` in non-test code.
- Handlers return `Result<Json<T>, AppError>`.
- Keep handlers thin: parse input, call a db function, return the response.
- sqlx `query_as!` macros for all DB queries (compile-time checked).
