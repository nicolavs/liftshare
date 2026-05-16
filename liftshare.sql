-- =========================================
-- liftshare.sqlite init script
-- =========================================

PRAGMA foreign_keys = ON;

-- =========================================
-- TABLE: trips
-- =========================================

CREATE TABLE IF NOT EXISTS trips (
    id TEXT PRIMARY KEY,

    driver_name TEXT NOT NULL,

    start_location TEXT NOT NULL,
    end_location TEXT NOT NULL,

    trip_start_time TEXT NOT NULL,
    estimated_trip_end_time TEXT,

    car_capacity INTEGER NOT NULL
        CHECK (car_capacity > 0),

    -- route summary
    route_distance_m REAL,
    route_duration_s REAL,

    -- full route json / steps
    -- {
    --   "steps": [
    --     {
    --       "instruction": "Turn left onto Harbour Street",
    --       "distance_m": 502.2,
    --       "duration_s": 54.4
    --     }
    --   ]
    -- }
    route_data TEXT,

    created_at TEXT NOT NULL
        DEFAULT CURRENT_TIMESTAMP
);

-- =========================================
-- TABLE: trip_user_rel
-- =========================================

CREATE TABLE IF NOT EXISTS trip_user_rel (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    trip_id TEXT NOT NULL,

    user_name TEXT NOT NULL,

    pickup_location TEXT,

    num_passengers INTEGER NOT NULL
        DEFAULT 1
        CHECK (num_passengers > 0),

    joined_at TEXT NOT NULL
        DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (trip_id)
        REFERENCES trips(id)
        ON DELETE CASCADE
);

-- =========================================
-- INDEXES
-- =========================================

CREATE INDEX IF NOT EXISTS idx_trips_locations
ON trips(start_location, end_location);

CREATE INDEX IF NOT EXISTS idx_trips_start_time
ON trips(trip_start_time);

CREATE INDEX IF NOT EXISTS idx_trip_user_rel_trip
ON trip_user_rel(trip_id);

CREATE INDEX IF NOT EXISTS idx_trip_user_rel_user
ON trip_user_rel(user_name);
