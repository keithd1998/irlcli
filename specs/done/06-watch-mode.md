# Spec: Watch Mode — Poll for Changes

## Problem Statement

Users and LLMs may want to monitor for changes: new weather warnings, new legislation, or specific events. Currently every query is a one-shot — there's no way to say "tell me when something changes."

## Objectives

1. Add `irl watch <subcommand> [--interval <seconds>]` that polls and reports changes
2. Output only the diff (new items) on each poll, not the full result set
3. Default interval: 5 minutes for weather, 1 hour for legislation

## Detailed Changes

### Phase 1: Watch wrapper

- Add a top-level `irl watch` command that wraps any existing subcommand
- On first run, capture the full result as the baseline
- On subsequent runs, diff against baseline and output only new items
- Use cache TTL as the default poll interval per source

### Phase 2: Useful defaults

- `irl watch met warnings` — poll for new weather warnings (5 min)
- `irl watch oireachtas legislation --search "housing"` — poll for new housing bills (1 hour)
- `irl watch oireachtas divisions` — poll for new votes (1 hour)

### Phase 3: Output format

- JSON output with `{"event": "new_item", "timestamp": "...", "data": {...}}`
- Table output shows only new rows with a timestamp prefix
- `--once` flag to check once and exit with code 0 (changes found) or 1 (no changes)

## Success Criteria

1. `irl watch met warnings --once` exits 0 if warnings appeared since last check
2. `irl watch oireachtas legislation --search "planning"` outputs new planning bills as they appear
3. Ctrl+C cleanly exits the watch loop

## Complexity

Medium — the polling loop is simple, the diffing logic needs thought.
