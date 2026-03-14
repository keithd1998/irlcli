# Spec: Debate Transcripts — What TDs Actually Said

## Problem Statement

The Oireachtas debates endpoint returns section titles and counts but not the actual transcript text. An LLM cannot answer "what did Mary Lou McDonald say about housing?" without the debate content. The API provides full debate section text — we just don't expose it.

## Objectives

1. Add `irl oireachtas debates --date 2026-03-04 --format json` that returns full debate section titles and text
2. Add `irl oireachtas speeches --member "Mary Lou McDonald"` to search for a TD's contributions across debates
3. Ensure JSON output contains full untruncated speech text

## Detailed Changes

### Phase 1: Expose debate sections

- The Oireachtas API `/debates` endpoint returns `debateRecord.sections[]` with `showAs` titles
- Each section has a URI that can be fetched for full content
- Add section titles to `DebateRow` display and full section data to JSON output
- Add `--sections` flag to show section detail in table mode

### Phase 2: Debate detail endpoint

- Add `irl oireachtas debate <debate-uri>` to fetch and display a specific debate's full text
- Parse the debate XML/JSON for speaker contributions
- Return structured JSON with speaker name, party, and text for each contribution

### Phase 3: Member speech search

- Add `irl oireachtas speeches --member "name" --date 2026-03-04`
- Fetch debates for the date, then filter sections/speeches by member name
- Return the full text of what that member said

## Success Criteria

1. `irl oireachtas debates --date 2026-03-04 --format json` returns section titles
2. `irl oireachtas speeches --member "McDonald" --date 2026-03-04` returns her speech text
3. LLM can answer "what did [TD] say about [topic]?" from a single command

## Complexity

Medium-High — requires investigating the debate detail API format (may be XML).
