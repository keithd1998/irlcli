# Specialist Sources: EPA, Water, Tailte & Geo Modules

## Ambiguity Assessment

| Dimension | Score (0-1) | Notes |
|-----------|-------------|-------|
| Goal Clarity | 0.75 | Commands defined but most API endpoints need discovery |
| Constraint Clarity | 0.80 | GeoJSON, ArcGIS patterns; EPA has multiple sub-APIs |
| Success Criteria | 0.75 | Several endpoints unverified |
| Context Clarity | 0.95 | Builds on all previous patterns |
| **Weighted Total** | **0.18** | Proceeding (borderline — API discovery is critical) |

## Problem Statement

The final four modules cover specialist Irish data sources — environmental monitoring (EPA), hydrology (OPW Water), commercial property valuation (Tailte Éireann), and spatial/geographic data (GeoHive/OSi). These modules have the least-documented APIs and require the most exploration, but complete the CLI's coverage of major Irish public data sources.

## Objectives

1. Implement `irl epa` — air quality and environmental data from EPA
2. Implement `irl water` — real-time water levels from OPW
3. Implement `irl tailte` — commercial property valuation data from Tailte Éireann
4. Implement `irl geo` — spatial/boundary data from GeoHive/OSi (ArcGIS REST)

## Assumptions Exposed

1. **EPA air quality API exists as JSON** — airquality.ie likely has an API but needs discovery
2. **waterlevel.ie has a machine-readable data API** — the GeoJSON station list is confirmed; data format for readings needs discovery
3. **Tailte Éireann REST API is publicly accessible** — may be at api.valoff.ie or opendata.tailte.ie; needs discovery
4. **GeoHive uses standard ArcGIS REST API** — likely at services-eu1.arcgis.com; needs discovery of layer IDs

## Technical Approach

### EPA Module (irl-epa)

**API Discovery targets:**
- Air quality: `https://airquality.ie` (look for API/JSON endpoints)
- EPA data portal: `https://data.epa.ie/api-list/`
- Start with air quality as most immediately useful

**Commands:**
```
irl epa air-quality
irl epa air-quality --station "Dublin"
irl epa water-quality --catchment "Shannon"
irl epa facilities --county Dublin
irl epa emissions --sector energy
```

**Strategy:** Discover air quality API first. If airquality.ie has no API, check if data.epa.ie wraps it. Build air-quality subcommand first, add others iteratively.

### Water Module (irl-water)

**API Discovery targets:**
- Station list (GeoJSON): `https://waterlevel.ie/geojson/`
- Station data: `https://waterlevel.ie/data/station/{ID}/` (format unknown)
- Historical: explore for CSV or JSON endpoints

**Commands:**
```
irl water stations
irl water stations --county Galway
irl water level <STATION_ID>
irl water level <STATION_ID> --history 7d
irl water alerts
irl water search "Corrib"
```

### Tailte Module (irl-tailte)

**API Discovery targets:**
- `https://opendata.tailte.ie` or `https://api.valoff.ie`
- data.gov.ie datasets for Tailte Éireann

**Commands:**
```
irl tailte search --address "Main Street, Carlow"
irl tailte property <PROPERTY_NUMBER>
irl tailte area --rating-authority Dublin
irl tailte categories
```

### Geo Module (irl-geo)

**API Discovery targets:**
- `https://data-osi.opendata.arcgis.com/datasets/`
- ArcGIS REST: `https://services-eu1.arcgis.com/.../FeatureServer/0/query?where=1=1&outFields=*&f=json`

**Commands:**
```
irl geo boundaries --type county
irl geo boundaries --type electoral-district
irl geo search --lat 53.35 --lon -6.26
irl geo datasets
irl geo fetch <DATASET_ID> --format geojson
```

### Key Components

1. **irl-epa::airquality** — air quality index client
2. **irl-water::stations** — GeoJSON station parser, county filtering
3. **irl-water::readings** — water level data fetching and history
4. **irl-tailte::api** — valuation search client
5. **irl-geo::arcgis** — ArcGIS REST query client (reusable pattern)
6. **irl-geo::boundaries** — county/ED boundary fetcher

## Implementation Phases

### Phase 1: EPA Air Quality

**Goal**: `irl epa air-quality` shows current AQI data.

**Steps**:
1. API Discovery — explore airquality.ie and data.epa.ie for JSON endpoints
2. Implement air quality client based on discovered API
3. Implement station search and filtering
4. Add other EPA endpoints iteratively (water quality, facilities, emissions)

### Phase 2: OPW Water Levels

**Goal**: `irl water stations` lists stations; `irl water level <ID>` shows current level.

**Steps**:
1. API Discovery — fetch GeoJSON station list, explore data endpoints
2. Implement GeoJSON parsing for station list
3. Implement county filtering (extract from GeoJSON properties)
4. Implement water level reading for individual stations
5. Implement `--history` with duration parsing
6. Implement alerts (stations above threshold)

### Phase 3: Tailte Éireann Valuations

**Goal**: `irl tailte search --address "Main Street"` returns valuation data.

**Steps**:
1. API Discovery — test opendata.tailte.ie, api.valoff.ie, data.gov.ie
2. Implement based on discovered API shape
3. If no REST API, fall back to data.gov.ie bulk CSV download

### Phase 4: GeoHive / OSi Spatial Data

**Goal**: `irl geo boundaries --type county` returns GeoJSON boundaries.

**Steps**:
1. API Discovery — find ArcGIS FeatureServer URLs for counties and electoral districts
2. Implement ArcGIS REST query client
3. Implement boundary fetching with GeoJSON output
4. Implement point-in-polygon search
5. Implement dataset listing and download

## Testing Strategy

### Automated Tests
- [ ] EPA: air quality response parsing from fixtures
- [ ] Water: GeoJSON station list parsing
- [ ] Water: station filtering by county
- [ ] Water: reading data parsing
- [ ] Tailte: response deserialization from fixtures
- [ ] Geo: ArcGIS REST query response parsing
- [ ] Geo: GeoJSON boundary output formatting

### Manual Verification
- [ ] `irl epa air-quality` shows current data for Irish stations
- [ ] `irl water stations --county Galway` lists Galway stations
- [ ] `irl water level <ID>` shows current reading
- [ ] `irl tailte search --address "Main Street, Carlow"` returns results
- [ ] `irl geo boundaries --type county --format json` produces valid GeoJSON

## Success Criteria

- [ ] At least 3 of 4 modules have working primary commands
- [ ] Modules that hit unavailable APIs fail gracefully with clear messages
- [ ] All modules use the core HTTP client, cache, and output formatting
- [ ] GeoJSON output is valid and usable by downstream tools (e.g., geojson.io)
- [ ] `irl help` shows all 10 modules as implemented (no more "Coming soon")

## Risks & Mitigations

1. **EPA API may not exist as described**: Mitigation — fall back to web scraping or data.gov.ie datasets; clearly document API status in SOURCES.md
2. **waterlevel.ie data format unknown**: Mitigation — discovery-first; may need to parse HTML or CSV
3. **Tailte Éireann API may be gated**: Mitigation — fall back to data.gov.ie bulk data
4. **ArcGIS layer IDs are opaque**: Mitigation — dataset listing command helps users discover available layers
5. **GeoJSON boundaries are large files**: Mitigation — support `--simplify` flag in future; stream output rather than buffering

---

*Created: 2026-03-14*
*Status: todo*
*Ambiguity Score: 0.18*
*Jira: N/A*
