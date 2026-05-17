CREATE EXTENSION IF NOT EXISTS postgis;

CREATE TABLE IF NOT EXISTS trips (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL,

    start_location TEXT,
    end_location TEXT,
    start_geom  GEOGRAPHY(Point, 4326) NOT NULL,
    end_geom    GEOGRAPHY(Point, 4326) NOT NULL,

    trip_start_time TIMESTAMPTZ NOT NULL,
    trip_end_time   TIMESTAMPTZ,

    car_capacity    INTEGER NOT NULL CHECK (car_capacity > 0),
    car_capacity_used INTEGER NOT NULL DEFAULT 0 CHECK (car_capacity_used >= 0 AND car_capacity_used <= car_capacity),
    car_full          BOOLEAN NOT NULL DEFAULT FALSE,

    route_distance_m REAL,
    route_duration_s REAL,

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS trip_user_rel (
    trip_id         UUID NOT NULL REFERENCES trips(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL,

    pickup_geom     GEOGRAPHY(Point, 4326),
    pickup_location TEXT,
    num_passengers  INTEGER NOT NULL DEFAULT 1 CHECK (num_passengers > 0),

    PRIMARY KEY (trip_id, user_id)
);

CREATE TABLE IF NOT EXISTS trip_routes (
    trip_id          UUID NOT NULL REFERENCES trips(id) ON DELETE CASCADE,
    step             INTEGER NOT NULL,

    geom         GEOGRAPHY(Point, 4326) NOT NULL,
    route_distance_m REAL,
    route_duration_s REAL,

    PRIMARY KEY (trip_id, step)
);

CREATE INDEX IF NOT EXISTS idx_trip_route_geom ON trip_routes USING GIST (geom);

CREATE INDEX IF NOT EXISTS idx_trips_start_geom      ON trips USING GIST (start_geom);
CREATE INDEX IF NOT EXISTS idx_trips_end_geom        ON trips USING GIST (end_geom);
CREATE INDEX IF NOT EXISTS idx_trips_start_time      ON trips (trip_start_time);
CREATE INDEX IF NOT EXISTS idx_trips_end_time        ON trips (trip_end_time);
CREATE INDEX IF NOT EXISTS idx_trip_car_full         ON trips (car_full);
CREATE INDEX IF NOT EXISTS idx_trip_user_rel_trip    ON trip_user_rel (trip_id);
