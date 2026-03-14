# Spec: CSO + Location — Statistical Context for Places

## Problem Statement

The CSO has detailed statistics by county and electoral division (population, housing, income, deprivation). The `nearby` command currently shows weather, TDs, and water stations but no socioeconomic context. An LLM answering "what's life like in Galway?" needs this data.

## Objectives

1. Add CSO data to `irl nearby` output — population, housing stats for the county
2. Add `irl cso local --county Dublin` for county-level statistical profile
3. Identify the most useful CSO tables and pre-wire them for geographic queries

## Detailed Changes

### Phase 1: Identify key CSO tables

- Census population by county
- Average house prices by county (from CSO, not PPR)
- Unemployment rate by region
- Map table codes and dimension filters for county-level extraction

### Phase 2: `irl cso local` command

- `irl cso local --county Dublin` returns a JSON profile:
  - Population (latest census)
  - Housing stock
  - Average earnings
  - Unemployment rate
- Queries the relevant CSO tables with county dimension filters

### Phase 3: Integrate into `nearby`

- Add a `cso_profile` section to the `NearbyResult` struct
- Map the Met station county to CSO county names
- Include population and key stats in nearby output

## Success Criteria

1. `irl cso local --county Dublin` returns population and economic stats
2. `irl nearby --location galway` includes CSO data in the output
3. LLM can answer "what's the population of Cork?" from a single command

## Complexity

Medium — CSO API is already working, main challenge is identifying the right tables and dimension filters.
