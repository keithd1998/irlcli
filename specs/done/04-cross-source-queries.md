# Spec: Cross-Source Queries

## Problem Statement

Users ask natural questions that span multiple data sources: "which bus route gets the most rain?", "what property prices are near my bus stop?", "are there weather warnings affecting my commute?". Currently the LLM must manually orchestrate multiple CLI calls and join the data. The tool could support common cross-source patterns natively.

## Objectives

1. **Enable geographic joining** — connect data sources that share location data
2. **Provide composite commands** for common cross-source questions
3. **Keep it simple** — don't build a query engine, just support the most useful combinations

## Detailed Changes

### Phase 1: Location-aware data enrichment

- Add a shared `Location` type to `irl-core` with latitude, longitude, and optional name
- Sources that have location data expose it:
  - `irl-transport`: stops have lat/lon
  - `irl-met`: stations have lat/lon
  - `irl-water`: stations have lat/lon
  - `irl-property`: addresses have county (could geocode)
  - `irl-epa`: air quality stations have lat/lon

### Phase 2: `irl nearby` command

- New top-level command: `irl nearby --lat <LAT> --lon <LON> --radius <KM>`
- Queries all location-aware sources and returns what's nearby:
  - Nearest weather station + current conditions
  - Nearest transport stops + next departures
  - Nearest water monitoring station + current level
  - Nearest air quality station + current reading
- Also support: `irl nearby --location "dublin"` using Met station coordinates as seed

### Phase 3: Route weather overlay

- `irl transport route-weather --route <ROUTE_ID>`
- Fetches vehicle positions on a route
- Maps each position to the nearest Met station
- Returns weather conditions along the route
- This directly answers "which bus route gets the most rain?"

## Success Criteria

1. `irl nearby --location dublin` returns a composite view of nearby data
2. `irl transport route-weather --route 39A` shows weather along the route
3. An LLM can answer cross-source questions with a single CLI call

## Complexity

Medium — mostly composing existing API calls with geographic proximity logic.

## Dependencies

- Spec 01 (data fidelity) — need full data from each source
- Spec 02 (smart resolution) — need GTFS static data for route geometry
