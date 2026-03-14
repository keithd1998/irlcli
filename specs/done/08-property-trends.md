# Spec: Property Price Trends

## Problem Statement

The PPR data in the local SQLite DB contains every residential sale since 2010 with date, county, and price. The `stats` command shows a single point-in-time snapshot, but there's no way to see trends over time. An LLM can't answer "are house prices going up in Dublin?"

## Objectives

1. Add `irl property trends --county Dublin` showing year-over-year price changes
2. Add `irl property trends --county Dublin --compare Cork` for multi-county comparison
3. Output structured JSON with yearly averages, medians, and percentage changes

## Detailed Changes

### Phase 1: Year-over-year trends

- Add `Trends` subcommand to PropertyCommands
- Query the SQLite DB for median/average prices grouped by year for a county
- Calculate year-over-year percentage change
- Output as JSON array of `{year, median, average, sales_count, yoy_change_pct}`

### Phase 2: Multi-county comparison

- `--compare` flag accepts comma-separated counties
- Returns parallel trend data for each county
- Enables "which county had the biggest price increase?"

### Phase 3: Time-range filtering

- `--from 2020 --to 2024` to limit the trend window
- Default: all available years

## Success Criteria

1. `irl property trends --county Dublin --format json` returns yearly price data with YoY changes
2. `irl property trends --county Dublin --compare Cork` shows both counties
3. LLM can answer "how have house prices changed in Galway since 2020?"

## Complexity

Low — all data is local in SQLite, just needs GROUP BY year queries.

## Dependencies

- Requires PPR data to be imported (`irl property update`)
