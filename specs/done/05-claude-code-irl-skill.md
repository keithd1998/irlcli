# Claude Code Skill for irl CLI

## Ambiguity Assessment

| Dimension | Score (0-1) | Notes |
|-----------|-------------|-------|
| Goal Clarity | 0.95 | Claude Code skill wrapping irl CLI with auto-execute |
| Constraint Clarity | 0.85 | Single /irl command, JSON output, smart markdown formatting |
| Success Criteria | 0.85 | Natural language → irl command routing, readable output |
| Context Clarity | 1.0 | Builds on existing irl-cli at cliirl/ |
| **Weighted Total** | **0.10** | Proceeding |

## Problem Statement

The `irl` CLI is a powerful tool for accessing Irish public sector data, but it requires the user to know the exact subcommand syntax. A Claude Code skill would let users ask natural language questions like "what's the weather in Dublin?" or "list current TDs" and have Claude automatically route to the right `irl` command, execute it, and format the results as readable markdown.

## Objectives

1. Create a Claude Code plugin with a single `/irl` skill that accepts natural language queries
2. Claude auto-routes queries to the correct `irl` module and subcommand
3. Execute `irl` commands with `--format json --quiet` for machine-readable output
4. Format JSON results into markdown tables and natural language summaries
5. Ensure the `irl` binary is found (from PATH or a configurable path)

## Assumptions Exposed

1. **irl binary is installed and in PATH** — validated: skill should check for binary and give install instructions if missing
2. **Single /irl entry point** — validated: one command, Claude figures out routing
3. **Auto-execute** — validated: all irl commands are read-only data queries, safe to run without confirmation
4. **JSON output** — validated: use `--format json -q` for all commands, then format as markdown
5. **Plugin structure** — assumption: use standard Claude Code plugin layout with skill markdown file

## Technical Approach

### Overview

Create a Claude Code plugin at `cliirl/.claude-plugin/` (or a standalone directory) containing a skill definition that:
1. Accepts a natural language query via `/irl <query>`
2. Instructs Claude to map the query to the appropriate `irl` subcommand
3. Runs the command via Bash with `--format json -q`
4. Formats the JSON output as a markdown table with a brief summary

### Plugin Structure

```
cliirl/.claude-plugin/
├── plugin.json           # Plugin manifest
└── skills/
    └── irl.md            # The /irl skill definition
```

### Key Components

1. **plugin.json** — Plugin manifest declaring the skill, name, description
2. **skills/irl.md** — Skill definition with:
   - YAML frontmatter (name, description, trigger patterns)
   - Command routing logic — maps natural language to irl subcommands
   - Execution instructions — run via Bash with --format json -q
   - Output formatting instructions — parse JSON, render markdown tables
   - Error handling — missing binary, API errors, missing config

### Skill Routing Logic

The skill prompt should include a routing table mapping query patterns to irl commands:

| Query Pattern | irl Command |
|---------------|-------------|
| weather/forecast + location | `irl met forecast --location <loc>` |
| weather warnings | `irl met warnings` |
| TDs/senators/members + optional party/constituency | `irl oireachtas members [--party X] [--constituency Y]` |
| legislation/bills + optional search | `irl oireachtas legislation [--search X]` |
| votes/divisions | `irl oireachtas divisions` |
| questions + optional member | `irl oireachtas questions [--member X]` |
| debates + optional date | `irl oireachtas debates [--date X]` |
| statistics/CSO + search term | `irl cso tables --search X` |
| CSO table data + code | `irl cso query <CODE> [--last N]` |
| company/CRO + name | `irl cro search <name>` |
| property/house prices + county/year | `irl property search [--county X] [--year Y]` |
| property stats + county | `irl property stats [--county X]` |
| transport/bus/departures + stop | `irl transport departures --stop <ID>` |
| water/river levels + search | `irl water stations [--county X]` |
| air quality/EPA | `irl epa air-quality` |
| what modules/sources available | `irl help` |

### Output Formatting

The skill should instruct Claude to:
1. Run the command with `--format json -q`
2. Parse the JSON array/object
3. Render as a markdown table with headers
4. Add a 1-2 sentence natural language summary above the table
5. If the result is large (>20 rows), show top 20 with "showing 20 of N results"
6. For errors, show the error message clearly and suggest fixes

### Binary Detection

Before running any command, check if `irl` is available:
```bash
which irl || echo "NOT_FOUND"
```
If not found, instruct the user to install it:
```
The irl CLI is not installed. Install with:
  cd /path/to/cliirl && cargo install --path .
```

Alternatively, allow configuring the path in `.claude/irl.local.md` or similar.

## Implementation Phases

### Phase 1: Plugin Scaffold & Skill Definition

**Goal**: Working `/irl` skill that maps queries to commands and executes them.

**Steps**:
1. Create `plugin.json` with plugin metadata
2. Create `skills/irl.md` with:
   - YAML frontmatter (name: "irl", description, user_invocable: true)
   - Full routing table in the prompt body
   - Execution instructions (Bash with --format json -q)
   - Output formatting instructions (markdown tables)
   - Error handling instructions
   - Binary detection step
3. Test with sample queries

**Files to create**:
- `.claude-plugin/plugin.json`
- `.claude-plugin/skills/irl.md`

### Phase 2: Polish & Edge Cases

**Goal**: Handle all edge cases gracefully.

**Steps**:
1. Add handling for ambiguous queries (ask for clarification)
2. Add handling for commands that need config (transport API key)
3. Add `irl config init` and `irl config set` guidance
4. Test with edge cases: fadas in names, empty results, API errors
5. Add examples in the skill prompt for common queries

## Testing Strategy

### Manual Verification
- [ ] `/irl what's the weather in Dublin` → runs `irl met forecast --location dublin --format json -q`, shows markdown table
- [ ] `/irl list current TDs` → runs `irl oireachtas members --format json -q`, shows member table
- [ ] `/irl search CSO for house prices` → runs `irl cso tables --search "house prices" --format json -q`
- [ ] `/irl help` → shows available modules
- [ ] `/irl` with no query → shows usage examples
- [ ] Running when irl binary not installed → clear install instructions
- [ ] Running transport command without API key → clear config instructions

## Success Criteria

- [ ] `/irl` skill is loadable by Claude Code
- [ ] Natural language queries correctly route to irl subcommands
- [ ] Results display as formatted markdown tables
- [ ] Missing binary detected with install instructions
- [ ] Missing API key detected with config instructions
- [ ] Fadas (á, é, í, ó, ú) display correctly in output
- [ ] Empty results show "No results found" message
- [ ] All 10 data modules are routable via the skill

## Risks & Mitigations

1. **Query ambiguity**: Mitigation — skill prompt includes a detailed routing table with examples; Claude's NLU handles fuzzy matching
2. **JSON parsing failures**: Mitigation — skill instructs Claude to handle non-JSON output gracefully (e.g., error messages from irl)
3. **Large result sets**: Mitigation — instruct Claude to truncate to top 20 rows with count
4. **Binary not in PATH**: Mitigation — check `which irl` first, provide install instructions

---

*Created: 2026-03-14*
*Status: todo*
*Ambiguity Score: 0.10*
*Jira: N/A*
