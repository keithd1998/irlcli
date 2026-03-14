# Spec: Smart Resolution and Contextual Guidance

## Problem Statement

When an LLM queries the CLI on behalf of a user, it often doesn't know the exact identifiers the API expects. "Dublin North-Central" returns 0 results with no hint that the constituency was redistricted. A bus stop search by name fails because only real-time stop IDs are available. The LLM has to guess, fail, and retry — or fall back to raw API calls.

## Objectives

1. **Fuzzy matching** for constituency names, member names, station names, and stop names
2. **"Did you mean?"** suggestions when 0 results are found
3. **Constituency redistricting awareness** — map historical names to current ones
4. **Contextual hints** in error output that help the LLM self-correct

## Detailed Changes

### Phase 1: Fuzzy matching infrastructure

- Add a `fuzzy_match(query: &str, candidates: &[&str], threshold: f64) -> Vec<(String, f64)>` function to `irl-core`
- Use simple normalized Levenshtein distance or Jaro-Winkler (no heavy deps — implement inline or use `strsim` crate)
- When a filter returns 0 results, re-run against all known values and suggest the top 3 matches

### Phase 2: Constituency mapping

- Add a static lookup table in `irl-oireachtas` mapping historical constituency names to current ones:
  - "Dublin North-Central" → "Dublin Central" + "Dublin Bay North"
  - "Dublin South-East" → "Dublin Bay South"
  - "Dublin North-East" → "Dublin Bay North"
  - (and other 2016/2020 boundary changes)
- When `--constituency` matches a historical name, automatically expand to current constituencies with a note

### Phase 3: Transport stop name resolution

- Fetch and cache static GTFS stop data (stops.txt) from NTA
- Enable `irl transport stops --search "o'connell"` to search by stop name, not just ID
- Cache with 24h TTL (stop names don't change often)

### Phase 4: Helpful zero-result messages

- When any command returns 0 results after filtering:
  1. State what was searched and what filter was applied
  2. If fuzzy matches exist, suggest them
  3. If the filter field has known valid values, mention how to list them (e.g., "Use `irl oireachtas members` to see all constituencies")

## Success Criteria

1. `irl oireachtas members --constituency "Dublin North Central"` returns TDs from Dublin Central + Dublin Bay North with a note about redistricting
2. `irl transport stops --search "connolly"` finds Connolly Station stops by name
3. `irl met forecast --location "dubln"` suggests "dublin" (typo correction)
4. Zero-result output includes actionable next steps, not just "No results found"

## Files Affected

- `crates/irl-core/src/lib.rs` — fuzzy matching utility
- `crates/irl-oireachtas/src/commands.rs` — constituency mapping, fuzzy member names
- `crates/irl-oireachtas/src/constituencies.rs` — new file: historical→current mapping
- `crates/irl-transport/src/commands.rs` — stop name search
- `crates/irl-transport/src/api.rs` — static GTFS stops.txt fetch
- `crates/irl-met/src/locations.rs` — fuzzy location matching
- `crates/irl-core/src/output.rs` — enhanced zero-result messages

## Complexity

Medium — fuzzy matching is self-contained, constituency mapping is a static table, GTFS stops is a new data fetch.

## Dependencies

- Spec 01 (data fidelity) should land first so error messages include full context
