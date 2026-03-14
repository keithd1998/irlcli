# Spec: LLM Data Fidelity — Full Data, No Truncation

## Problem Statement

The `irl` CLI is consumed by LLMs to answer natural language questions about Irish public data. Currently, the tool silently destroys information through truncation, incomplete JSON output, and client-side filtering on partial result sets. This means the LLM gives wrong or incomplete answers to users — the worst possible failure mode.

## Objectives

1. **JSON output must contain full API data**, not truncated display rows
2. **Remove all string truncation** from data fields (truncation is only for table display, never for JSON/CSV)
3. **Fix client-side filtering** so filtered queries scan all available data, not just the first page
4. **Remove silent `.truncate()` calls** that drop results without warning

## Detailed Changes

### Phase 1: Dual-path output (JSON=full, table=truncated)

**Core change in `irl-core/output.rs`:**
- Add a second serialization path: when format is JSON or CSV, serialize the **full API response models** (not the `*Row` display structs)
- Table format continues to use `*Row` structs with truncation for human readability
- Each crate's `handle_command` must pass both the raw API response and the display rows to the output system

**Per-crate changes:**
- All `*Row::from_result()` methods stay as-is (table display)
- Add `Serialize` derive to all API response models (`MemberResult`, `QuestionResult`, etc.)
- Modify `handle_command` to call `output.render_full(&api_results, &display_rows)`

**Affected crates:** irl-oireachtas, irl-met, irl-transport, irl-cso, irl-property, irl-cro, irl-geo, irl-tailte, irl-epa, irl-water

### Phase 2: Fix client-side filtering with auto-pagination

**irl-oireachtas:**
- When a `--member`, `--party`, or `--constituency` filter is provided, fetch **all pages** (loop until results exhausted) before applying the client-side filter
- Add a `--limit` that caps the *output* count, not the *fetch* count
- API calls should page through with skip increments until `results.len() < limit`

**irl-transport:**
- Remove hard-coded `rows.truncate(50)` from departures and stops commands
- Add `--limit` flag (default: 50 for table, unlimited for JSON) so the LLM can request all results

### Phase 3: UTF-8 safe truncation

- Replace all `&str[..N]` truncation with a helper that truncates at char boundaries:
  ```rust
  fn truncate_display(s: &str, max: usize) -> String {
      if s.chars().count() <= max { s.to_string() }
      else { format!("{}...", s.chars().take(max - 3).collect::<String>()) }
  }
  ```
- Add this to `irl-core` and use across all crates

## Success Criteria

1. `irl oireachtas questions --member "Tom Brabazon" --format json` returns full question text (not truncated to 50 chars)
2. `irl oireachtas members --constituency "Dublin Central" --format json` returns all matching members regardless of pagination
3. `irl transport departures --stop 8220DB000769 --format json` returns all departures, not silently capped at 50
4. No panics on Irish-language names with fadas (é, á, ó, etc.)
5. LLM can answer "what has Tom Brabazon spoken about?" without needing to curl the API directly

## Files Affected

- `crates/irl-core/src/output.rs` — dual-path render method
- `crates/irl-core/src/lib.rs` — add `truncate_display` helper
- `crates/irl-oireachtas/src/commands.rs` — auto-pagination, pass full data
- `crates/irl-oireachtas/src/models.rs` — add Serialize to API models, use safe truncation
- `crates/irl-met/src/models.rs` — safe truncation
- `crates/irl-transport/src/commands.rs` — remove `.truncate(50)`, add `--limit`
- `crates/irl-property/src/models.rs` — safe truncation
- `crates/irl-cro/src/models.rs` — safe truncation
- `crates/irl-geo/src/models.rs` — safe truncation
- `crates/irl-tailte/src/models.rs` — safe truncation

## Complexity

High — touches all crates, but changes are mechanical and repetitive.

## Out of Scope

- Cross-source queries (separate spec)
- Browser emulation for Cloudflare-protected sources (separate spec)
- New commands or data sources
- Static GTFS data integration
