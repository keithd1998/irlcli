# Spec: TD Accountability Profile

## Problem Statement

Information about a TD is scattered across three Oireachtas endpoints: members (party, constituency), questions (what they've asked), and divisions (how they voted). There's no single command to get a complete picture of what a TD has been doing. An LLM answering "tell me about my TD" needs to make 3+ calls and join the data.

## Objectives

1. Add `irl oireachtas td --name "Paschal Donohoe"` that returns a unified profile
2. Include: party, constituency, recent questions, voting record, and sponsored bills
3. Single JSON output that gives an LLM everything it needs

## Detailed Changes

### Phase 1: TD profile command

- Add `Td` subcommand to OireachtasCommands
- Accept `--name` with fuzzy matching
- Fetch member data, then parallel-fetch questions and legislation filtered by member

### Phase 2: Profile structure

- JSON output includes:
  - `member`: name, party, constituency, house
  - `recent_questions`: last 10 questions with full text
  - `sponsored_bills`: bills where this TD is a sponsor
  - `summary`: counts (total questions asked, bills sponsored)

### Phase 3: Voting record (if API supports)

- Check if the divisions endpoint allows member-level filtering
- If so, include recent votes with the TD's position (Tá/Níl)
- If not, note this as a limitation

## Success Criteria

1. `irl oireachtas td --name "Paschal Donohoe"` returns a complete profile
2. `irl oireachtas td --name "mcdonald"` fuzzy-matches to Mary Lou McDonald
3. LLM can answer "what has my TD been doing?" from a single command

## Complexity

Medium — combines existing API calls with member-name joining. Fuzzy matching infrastructure already exists.

## Dependencies

- Spec 01 (data fidelity) — auto-pagination for questions
- Spec 02 (smart resolution) — fuzzy matching for member names
