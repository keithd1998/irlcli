# Weather (Met Éireann) & Statistics (CSO PxStat) Modules

## Ambiguity Assessment

| Dimension | Score (0-1) | Notes |
|-----------|-------------|-------|
| Goal Clarity | 0.90 | Clear commands; CSO JSON-stat format needs discovery |
| Constraint Clarity | 0.90 | XML parsing (quick-xml), no auth required |
| Success Criteria | 0.85 | Well-defined commands per module |
| Context Clarity | 0.95 | Builds on foundation from spec 01 |
| **Weighted Total** | **0.10** | Proceeding |

## Problem Statement

After establishing the CLI architecture and proving it with the Oireachtas module, the next highest-value modules are Met Éireann (immediately useful, no auth) and CSO PxStat (richest Irish dataset, thousands of statistical tables). These two modules demonstrate XML parsing and JSON-stat unpacking — two patterns not covered by the Oireachtas JSON module.

## Objectives

1. Implement `irl met` — weather forecasts, warnings, and station data from Met Éireann
2. Implement `irl cso` — statistical table browsing, metadata, and data querying from CSO PxStat
3. Demonstrate XML parsing pattern (Met Éireann) and JSON-stat unpacking pattern (CSO)

## Assumptions Exposed

1. **Met Éireann API is stable at documented URL** — needs verification via discovery; the metno-wdb2ts endpoint may have changed
2. **CSO PxStat serves JSON-stat** — the format is compact and requires dimension/value array unpacking; verify actual response structure
3. **No auth required for either API** — validated for Met Éireann; CSO is also open access
4. **Weather warnings available via RSS/XML** — the exact feed URL needs discovery

## Technical Approach

### Met Éireann Module

**API Discovery targets:**
- Point forecast: `https://openaccess.pf.api.met.ie/metno-wdb2ts/locationforecast?lat=53.35&long=-6.26`
- Warnings: `https://www.met.ie/warningsxml/warning_IRELAND.xml` (or similar)

**Built-in location lookup table** — hardcoded coordinates for ~20 major Irish cities/towns:
Dublin (53.35, -6.26), Cork (51.90, -8.47), Galway (53.27, -9.06), Limerick (52.66, -8.63), Waterford (52.26, -7.11), Belfast (54.60, -5.93), Killarney (52.06, -9.51), Sligo (54.27, -8.47), Athlone (53.42, -7.94), Letterkenny (54.95, -7.73), Wexford (52.34, -6.46), Dundalk (54.00, -6.42), Drogheda (53.72, -6.35), Kilkenny (52.65, -7.25), Ennis (52.84, -8.99), Tralee (52.27, -9.70), Carlow (52.84, -6.93), Tullamore (53.27, -7.49), Derry (55.00, -7.32), Newry (54.18, -6.34)

**Commands:**
```
irl met forecast --lat 53.35 --lon -6.26
irl met forecast --location dublin
irl met forecast --location dublin --hours 24
irl met warnings
irl met stations
```

**Display fields:** temperature, wind speed/direction, precipitation probability, weather description, time.

**Cache TTL:** 1 hour for forecasts, 15 minutes for warnings.

### CSO PxStat Module

**API Discovery targets:**
- Table of contents: `https://ws.cso.ie/public/api.restful/PxStat.Data.Cube_API.ReadCollection`
- Table metadata: `https://ws.cso.ie/public/api.restful/PxStat.Data.Cube_API.ReadDataset/{tableCode}?lang=en`
- Table data: `https://data.cso.ie/api/v1/en/table/{tableCode}`

**JSON-stat unpacking**: The format stores dimensions as arrays and values as a flat array indexed by dimension combination. Need to unpack into rows with named columns for table display.

**Commands:**
```
irl cso tables
irl cso tables --search "house prices"
irl cso info <TABLE_CODE>
irl cso query <TABLE_CODE>
irl cso query <TABLE_CODE> --dimension "Year=2024" --dimension "County=Dublin"
irl cso query CPM01 --last 12
```

**Cache TTL:** 24 hours for table of contents (~2MB), 1 hour for data queries.

### Key Components

1. **irl-met::locations** — hardcoded location lookup table, fuzzy name matching
2. **irl-met::forecast** — XML parsing of metno-wdb2ts forecast format via quick-xml
3. **irl-met::warnings** — RSS/XML warning feed parser
4. **irl-cso::catalog** — table of contents fetching, local search, caching
5. **irl-cso::jsonstat** — JSON-stat format unpacker (dimensions × values → tabular rows)
6. **irl-cso::query** — data querying with dimension filtering

## Implementation Phases

### Phase 1: Met Éireann API Discovery & Forecast

**Goal**: `irl met forecast --location dublin` returns real weather data.

**Steps**:
1. API Discovery — hit forecast endpoint, save XML response as fixture
2. Define XML response structs for quick-xml deserialization
3. Implement location lookup table
4. Implement forecast command with lat/lon and named location support
5. Implement `--hours` filtering
6. Wire into main.rs (replace stub)
7. Unit tests against XML fixtures

**Files to create/modify**:
- `crates/irl-met/src/lib.rs` — module root
- `crates/irl-met/src/locations.rs` — city/town coordinate lookup
- `crates/irl-met/src/forecast.rs` — XML parsing and forecast display
- `crates/irl-met/src/commands.rs` — clap subcommands
- `crates/irl-met/tests/fixtures/` — saved XML responses
- `src/main.rs` — wire in met commands

### Phase 2: Met Éireann Warnings & Stations

**Goal**: `irl met warnings` and `irl met stations` working.

**Steps**:
1. Discover warnings XML feed URL, save fixture
2. Implement warnings parser
3. Implement stations list (may be from hardcoded data or API discovery)
4. Tests for warning parsing

### Phase 3: CSO API Discovery & Catalog

**Goal**: `irl cso tables` and `irl cso tables --search` working.

**Steps**:
1. API Discovery — fetch table of contents, examine JSON structure, save fixture
2. Define response structs for catalog
3. Implement local search (search both table codes and descriptions)
4. Cache TOC with 24h TTL
5. Wire into main.rs

### Phase 4: CSO Data Querying

**Goal**: `irl cso query <TABLE_CODE>` returns formatted statistical data.

**Steps**:
1. API Discovery — fetch a known table (e.g., CPM01), examine JSON-stat structure
2. Implement JSON-stat unpacker
3. Implement dimension filtering (`--dimension "Key=Value"`)
4. Implement `--last N` for time period limiting
5. Implement `irl cso info <TABLE_CODE>` for metadata display
6. Unit tests against JSON-stat fixtures

## Testing Strategy

### Automated Tests
- [ ] Met forecast XML parsing — valid forecast, empty response, malformed XML
- [ ] Met location lookup — exact match, case insensitive, not found
- [ ] Met warnings parsing — active warning, no warnings, multiple warnings
- [ ] CSO catalog deserialization from fixture
- [ ] CSO local search — keyword match in title, in code, no match
- [ ] CSO JSON-stat unpacking — single dimension, multi-dimension, filtered
- [ ] CSO `--last N` time period filtering
- [ ] Integration: `irl met forecast --location dublin` returns data (feature-flagged)
- [ ] Integration: `irl cso tables` returns catalog (feature-flagged)

### Manual Verification
- [ ] `irl met forecast --location dublin` shows temperature, wind, precipitation
- [ ] `irl met forecast --location dublin --hours 24` limits output
- [ ] `irl met warnings` shows current warnings or "No active warnings"
- [ ] `irl cso tables --search "population"` returns relevant results
- [ ] `irl cso query CPM01 --last 6 --format json` produces valid JSON
- [ ] Large CSO table query shows spinner during download

## Success Criteria

- [ ] `irl met forecast --location dublin` displays formatted weather data
- [ ] `irl met warnings` shows current Met Éireann weather warnings
- [ ] `irl cso tables` lists available statistical tables with search
- [ ] `irl cso query <TABLE_CODE>` returns and formats JSON-stat data correctly
- [ ] CSO dimension filtering works (`--dimension "Year=2024"`)
- [ ] Both modules respect cache TTLs and `--no-cache` flag
- [ ] Fadas in place names display correctly (e.g., Dún Laoghaire)
- [ ] All 3 output formats work for both modules

## Risks & Mitigations

1. **Met Éireann XML format undocumented**: Mitigation — discovery-first approach, save real responses as fixtures
2. **CSO JSON-stat unpacking complexity**: Mitigation — start with simple single-dimension tables, add multi-dimension iteratively
3. **CSO TOC is ~2MB**: Mitigation — aggressive caching (24h TTL), local search after first fetch
4. **Met Éireann warnings feed URL may change**: Mitigation — make the URL configurable in config.toml

---

*Created: 2026-03-14*
*Status: todo*
*Ambiguity Score: 0.10*
*Jira: N/A*
