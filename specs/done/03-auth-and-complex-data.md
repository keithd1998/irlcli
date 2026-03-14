# Transport (GTFS), Companies (CRO) & Property Price Register Modules

## Ambiguity Assessment

| Dimension | Score (0-1) | Notes |
|-----------|-------------|-------|
| Goal Clarity | 0.85 | Commands clear; CRO API needs exploration |
| Constraint Clarity | 0.80 | Transport requires API key + protobuf; Property needs SQLite |
| Success Criteria | 0.80 | Some API endpoints unverified |
| Context Clarity | 0.95 | Builds on established patterns from specs 01-02 |
| **Weighted Total** | **0.14** | Proceeding |

## Problem Statement

Phase 2 introduces modules with higher complexity: API key authentication (Transport), unverified API endpoints (CRO), and local data storage with CSV-to-SQLite ingest (Property). These patterns expand the CLI's capabilities significantly and demonstrate that the core architecture handles diverse data source patterns.

## Objectives

1. Implement `irl transport` — real-time GTFS data from NTA (requires API key, protobuf decoding)
2. Implement `irl cro` — Companies Registration Office search and company details
3. Implement `irl property` — Property Price Register with local SQLite database

## Assumptions Exposed

1. **NTA GTFS-R endpoints support JSON output** — the `?format=json` parameter may work, avoiding protobuf. Discovery required.
2. **CRO API is publicly accessible** — may require API key or specific headers. Discovery required.
3. **PSRA CSV download URLs are stable** — the propertypriceregister.ie site uses Lotus Notes/Domino which has unstable URLs. The civictech.ie wrapper is a fallback.
4. **rusqlite compiles on macOS without issues** — should be fine with bundled SQLite feature

## Technical Approach

### Transport Module (irl-transport)

**Requires:** NTA API key via `irl config set transport.api_key <KEY>`

**API Discovery targets:**
- Trip updates: `https://api.nationaltransport.ie/gtfsr/v2/TripUpdates?format=json`
- Vehicle positions: `https://api.nationaltransport.ie/gtfsr/v2/Vehicles?format=json`
- GTFS Static: `https://www.transportforireland.ie/transitData/Data/GTFS_Realtime.zip`

**Strategy:** Try JSON format first. If unavailable, use `prost-build` to compile GTFS-realtime.proto in a build script. Cache static GTFS data (stops.txt, routes.txt) locally for 1 week.

**Commands:**
```
irl transport departures --stop <STOP_ID>
irl transport departures --stop <STOP_ID> --route 46a
irl transport vehicles --route 46a
irl transport stops --search "O'Connell"
irl transport routes
irl transport routes --operator "Dublin Bus"
```

**Cache TTL:** 5 minutes for real-time data, 1 week for static GTFS.

### CRO Module (irl-cro)

**API Discovery targets (verify all):**
- Company search: `https://services.cro.ie/cro/company/search?name=<NAME>&status=<STATUS>`
- Company details: `https://services.cro.ie/cro/company/<NUMBER>`
- Company filings: `https://services.cro.ie/cro/company/<NUMBER>/filings`

**Commands:**
```
irl cro search "Marino Software"
irl cro search --status active "Tech"
irl cro company <NUMBER>
irl cro filings <NUMBER>
irl cro filings <NUMBER> --type annual-return
```

**Cache TTL:** 24 hours for search results, 1 hour for company details.

### Property Module (irl-property)

**Data Source:** CSV files from PSRA, CP1252 encoded.

**Local Storage:** SQLite database at `~/.irl/data/property.db`

**Ingest Pipeline:** Download CSV → decode CP1252 → parse → insert into SQLite → query locally.

**Additional dependency:** `rusqlite` with `bundled` feature.

**Commands:**
```
irl property search --county Dublin --year 2025
irl property search --county Cork --min 200000 --max 500000
irl property search --address "Marino"
irl property stats --county Dublin --year 2024
irl property stats --compare Dublin,Cork,Galway
irl property update
```

**Schema:**
```sql
CREATE TABLE sales (
    id INTEGER PRIMARY KEY,
    date_of_sale TEXT,
    address TEXT,
    county TEXT,
    eircode TEXT,
    price REAL,
    not_full_market_price INTEGER,
    vat_exclusive INTEGER,
    description TEXT,
    property_size TEXT
);
CREATE INDEX idx_county ON sales(county);
CREATE INDEX idx_date ON sales(date_of_sale);
CREATE INDEX idx_price ON sales(price);
```

### Key Components

1. **irl-transport::gtfs** — GTFS static data manager (download, cache, parse stops.txt/routes.txt)
2. **irl-transport::realtime** — GTFS-R client (JSON or protobuf)
3. **irl-cro::api** — CRO REST client (needs discovery)
4. **irl-property::ingest** — CSV download, CP1252 decode, SQLite import
5. **irl-property::query** — SQL query builder for search and stats
6. **irl-property::stats** — Statistical aggregations (median, mean, count, percentiles)

## Implementation Phases

### Phase 1: Transport — Static GTFS & API Discovery

**Goal**: Download and cache static GTFS data; discover real-time endpoint format.

**Steps**:
1. API Discovery — test JSON format on GTFS-R endpoints with API key
2. Download GTFS static ZIP, extract stops.txt and routes.txt
3. Parse and cache static data locally
4. Implement stop search and route listing

### Phase 2: Transport — Real-time Data

**Goal**: `irl transport departures --stop 767` shows live departures.

**Steps**:
1. Implement GTFS-R client (JSON or protobuf based on discovery)
2. Implement departures and vehicle position commands
3. Cross-reference real-time data with static stop/route names
4. Handle missing API key with actionable error message

### Phase 3: CRO — Discovery & Implementation

**Goal**: `irl cro search "Marino Software"` returns company results.

**Steps**:
1. API Discovery — test endpoints, check auth requirements, save fixtures
2. Implement based on discovered API shape
3. If API requires key, add to config; if open, proceed
4. Implement all search/company/filings commands

### Phase 4: Property — Ingest & Query

**Goal**: `irl property update` downloads data; `irl property search` queries locally.

**Steps**:
1. Discover PSRA CSV download URLs (may need to scrape the download page)
2. Implement CSV download with CP1252 → UTF-8 conversion
3. Create SQLite database and import pipeline
4. Implement search with county, year, price range, address filters
5. Implement stats with median, mean, count aggregations
6. Implement `--compare` for multi-county comparison

## Testing Strategy

### Automated Tests
- [ ] Transport: static GTFS CSV parsing (stops.txt, routes.txt fixtures)
- [ ] Transport: GTFS-R response parsing (JSON or protobuf fixture)
- [ ] Transport: stop search by name, route filtering
- [ ] CRO: response deserialization from fixtures
- [ ] CRO: company number validation
- [ ] Property: CP1252 CSV decoding
- [ ] Property: SQLite import pipeline (in-memory DB)
- [ ] Property: search queries with various filters
- [ ] Property: stats calculation (median, mean, count)

### Manual Verification
- [ ] `irl transport departures --stop 767` shows Dublin Bus departures
- [ ] `irl transport stops --search "O'Connell"` finds stops
- [ ] `irl cro search "Google"` returns company results
- [ ] `irl property update` downloads and imports data (first run may take a while)
- [ ] `irl property search --county Dublin --year 2024` returns results
- [ ] `irl property stats --compare Dublin,Cork` shows comparison table

## Success Criteria

- [ ] Transport module works with NTA API key
- [ ] Missing API key produces clear setup instructions
- [ ] CRO search and company detail retrieval work
- [ ] Property data downloads, converts from CP1252, and imports into SQLite
- [ ] Property search responds quickly from local SQLite (<100ms)
- [ ] Property stats show accurate median/mean calculations
- [ ] All modules respect cache TTLs and output format flags

## Risks & Mitigations

1. **NTA API key registration may take time**: Mitigation — all transport tests use saved fixtures; live testing is feature-flagged
2. **CRO API may not be publicly accessible**: Mitigation — fall back to open data portal CSV snapshots if REST API is gated
3. **PSRA download URLs are unstable (Lotus Notes)**: Mitigation — try civictech.ie JSON API as fallback; make download URL configurable
4. **Property SQLite database could be large (~50MB)**: Mitigation — use proper indexes; `irl property update` shows progress bar during import
5. **Protobuf compilation in build script**: Mitigation — try JSON format first; only add prost-build if needed

---

*Created: 2026-03-14*
*Status: todo*
*Ambiguity Score: 0.14*
*Jira: N/A*
