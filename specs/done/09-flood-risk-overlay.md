# Spec: Flood Risk Overlay — Rainfall + Water Levels

## Problem Statement

Ireland has detailed real-time water level monitoring (OPW) and hourly rainfall data (Met Éireann). Combining these could flag potential flood risk — if it's been raining heavily near a river station with already elevated levels. Currently these data sources are completely siloed.

## Objectives

1. Add `irl water risk` that checks nearby rainfall against water station locations
2. Highlight stations where heavy rainfall has occurred in the last 6 hours
3. Cross-reference with active Met weather warnings for rain/flood

## Detailed Changes

### Phase 1: Rainfall near water stations

- For each water monitoring station (OPW), find the nearest Met station
- Fetch recent rainfall data from that Met station
- Calculate total rainfall in the last 6 hours
- Flag stations where rainfall exceeds thresholds (e.g., > 5mm in 6h)

### Phase 2: Risk assessment

- `irl water risk` returns a JSON list of stations with:
  - Station name and location
  - Nearest Met station and distance
  - Recent rainfall total (6h)
  - Risk level: low/moderate/high based on rainfall thresholds
- `irl water risk --county Galway` to filter by area

### Phase 3: Integration with warnings

- If Met warnings include rain/flood warnings, cross-reference affected regions
- Include active warnings in the risk output

## Success Criteria

1. `irl water risk` returns stations with rainfall data and risk levels
2. `irl water risk --county Cork` filters to Cork-area stations
3. LLM can answer "is there flood risk in Galway?" from a single command

## Complexity

Medium — geographic joining (haversine) is already built, main work is the rainfall aggregation and threshold logic.

## Dependencies

- Spec 04 (cross-source queries) — haversine and location infrastructure already exist
