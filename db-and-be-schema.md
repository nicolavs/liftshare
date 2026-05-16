# Recommended Backend Schema

---

# Driver Creates Trip

## POST `/trip/create`

### Request

```json
{
    "name": "john",
    "start_location": "Sydney Central",
    "end_location": "North Sydney",
    "trip_start": "2026-05-16T10:00:00Z",
    "car_capacity": 4
}
```

---

# Better Response

```json
{
    "trip_id": "uuid",

    "start_location": "Sydney Central",
    "end_location": "North Sydney",

    "trip_start_time": "2026-05-16T10:00:00Z",
    "estimated_trip_end_time": "2026-05-16T10:10:00Z",

    "car_capacity": 4,

    "route": {
        "distance_m": 7664.9,
        "duration_s": 571.4,

        "geometry": "encoded_polyline",

        "steps": [
            {
                "instruction": "Depart straight",
                "road_name": "Little Pier Street",

                "distance_m": 78.3,
                "duration_s": 23.8,

                "maneuver_type": "depart",
                "modifier": "straight",

                "location": {
                    "lat": -33.877605,
                    "lng": 151.202394
                }
            },

            {
                "instruction": "Turn left onto Harbour Street",

                "road_name": "Harbour Street",

                "distance_m": 502.2,
                "duration_s": 54.4,

                "maneuver_type": "turn",
                "modifier": "left",

                "location": {
                    "lat": -33.877631,
                    "lng": 151.203232
                }
            }
        ]
    }
}
```

---

# Passenger APIs

---

# GET `/trip`

This is really:

```text
Find available trips
```

So maybe better:

```text
GET /trips/search
```

---

## Request

Usually GET requests use query params:

```http
GET /trips/search?start=sydney&end=north+sydney
```

Not JSON body.

---

# GET `/trip/<trip_id>`

Good structure.

Should return:
- trip info
- remaining seats
- route
- ETA

---

# POST `/trip/join`

Good idea.

But improve naming:

---

## Request

```json
{
    "trip_id": "uuid",
    "pickup_location": "Town Hall",
    "num_passengers": 2
}
```

---

## Response

```json
{
    "status": "success",

    "message": "Trip joined successfully",

    "pickup_location": {
        "lat": -33.873,
        "lng": 151.206
    },

    "estimated_pickup_time": "2026-05-16T10:04:00Z"
}
```

