# irl — Irish Public Sector Open Data CLI

A unified command-line tool for accessing Irish public sector open data. Single binary, subcommand-per-source architecture. Designed to be used by LLMs to answer natural language questions about Ireland — `--format json` returns full, untruncated API data suitable for machine consumption.

## Quick Start

```bash
# Build from source
cargo install --path .

# See all available data sources
irl help

# Create config file
irl config init

# Get weather for Dublin
irl met forecast --location dublin

# List current TDs and Senators
irl oireachtas members

# Get a TD's full profile — questions, bills, party, constituency
irl oireachtas td --name "Paul Murphy"

# National snapshot — warnings, votes, legislation in one call
irl snapshot

# Cross-source: what's near Galway?
irl nearby --location galway

# Output as JSON (full untruncated API data)
irl oireachtas members --format json

# Fuzzy matching corrects typos
irl met forecast --location dubln

# Historical constituency names auto-resolve
irl oireachtas members --constituency "Dublin North Central"
```

## Data Sources

| Module | Source | Auth | Status |
|--------|--------|------|--------|
| `oireachtas` | Houses of the Oireachtas | None | Working |
| `met` | Met Éireann | None | Working |
| `cso` | Central Statistics Office (PxStat) | None | Working |
| `transport` | Transport for Ireland (NTA GTFS) | API Key | Requires registration |
| `cro` | Companies Registration Office | None | API under investigation |
| `property` | Property Price Register (PSRA) | None | Local CSV/SQLite |
| `epa` | Environmental Protection Agency | None | API under investigation |
| `water` | OPW Water Levels | None | Station list working |
| `tailte` | Tailte Éireann (Valuation Office) | None | API under investigation |
| `geo` | GeoHive / OSi (ArcGIS) | None | FeatureServer URLs needed |

### Cross-Source Commands

| Command | Sources Combined | What it does |
|---------|-----------------|--------------|
| `nearby` | Met + Water + Oireachtas + Property | Weather, TDs, water stations, property stats for a location |
| `snapshot` | Met + Oireachtas | National overview: warnings, recent votes, latest bills |
| `flood-risk` | Met + Water | Rainfall near water monitoring stations with risk levels |
| `watch` | Any | Baseline-and-diff change detection for warnings, legislation, votes |

## Installation

### From source

```bash
git clone https://github.com/your-org/irl-cli.git
cd irl-cli
cargo install --path .
```

### Requirements

- Rust 2021 edition (1.56+)

## Configuration

Config file location: `~/.irl/config.toml`

```bash
# Create default config
irl config init

# Set Transport for Ireland API key
irl config set transport.api_key YOUR_KEY

# Set default output format
irl config set general.default_format json

# View current config
irl config show
```

### API Keys

**Transport for Ireland (NTA):** Register at https://developer.nationaltransport.ie/ to get an API key. Then:
```bash
irl config set transport.api_key YOUR_NTA_API_KEY
```

## Global Options

Every subcommand supports:

```
--format <table|json|csv>   Output format (default: table)
--no-colour                 Disable coloured output
--no-cache                  Bypass local response cache
-v, --verbose               Show HTTP request/response details
-q, --quiet                 Suppress non-data output (good for piping)
```

The CLI also respects the `NO_COLOR` environment variable and automatically disables colour when stdout is not a TTY (e.g., when piping).

## Module Examples

### Oireachtas (Parliament)

```bash
irl oireachtas members                                  # All current TDs/Senators
irl oireachtas members --party "Sinn Féin"              # Filter by party
irl oireachtas members --constituency "Dublin Central"  # Filter by constituency
irl oireachtas members --constituency "Dublin North Central"  # Historical names auto-resolve
irl oireachtas legislation --search "planning"          # Search bills
irl oireachtas divisions                                # Recent votes with debate topics
irl oireachtas questions --member "Mary Lou McDonald"   # Parliamentary questions (auto-paginates)
irl oireachtas debates --date 2026-03-05 --chamber dail # Dáil debates with section topics
irl oireachtas td --name "Paul Murphy"                  # Full TD profile
irl oireachtas td --name "mcdonald"                     # Fuzzy name matching
```

The `td` command returns a unified JSON profile: party, constituency, recent questions (full text), and all sponsored bills. Filtered queries automatically paginate through all API results to ensure nothing is missed.

### Met Éireann (Weather)

```bash
irl met forecast --location dublin          # Today's observations
irl met forecast --location galway --hours 6  # Last 6 hours
irl met forecast --location dubln           # Typos auto-corrected via fuzzy matching
irl met warnings                            # Active weather warnings
irl met stations                            # List available stations
```

### CSO (Statistics)

```bash
irl cso tables                              # All statistical tables
irl cso tables --search "population"        # Search tables
irl cso info CPM01                          # Table metadata
irl cso query CPM01 --last 12               # Last 12 time periods
irl cso query CPM01 --dimension "Year=2024" # Filter by dimension
irl cso local --county Dublin               # County profile (population, house prices)
irl cso local --county Cork                 # Works for any county
```

The `local` command queries multiple CSO tables automatically to build a county statistical profile including Census 2022 population and house price data.

### Transport

```bash
irl transport departures --stop 767         # Next departures (requires API key)
irl transport vehicles --route 46a          # Live vehicle positions
irl transport routes                        # List routes
irl transport stops --search 8220DB         # Search stops by ID
```

### Property Price Register

```bash
irl property update                                    # Download/import data
irl property search --county Dublin --year 2024        # Search sales
irl property search --min 200000 --max 500000          # Price range
irl property stats --county Dublin --year 2024         # Statistics
irl property stats --compare Dublin,Cork,Galway        # Compare counties
irl property trends --county Dublin                    # Year-over-year price trends
irl property trends --county Dublin --from 2020        # Trends from 2020 onwards
```

The `trends` command shows average and median prices by year with percentage changes — useful for answering "are house prices going up?"

### Water Levels

```bash
irl water stations                          # All monitoring stations
irl water stations --county Galway          # Filter by county
irl water search "Corrib"                   # Search by name
```

### Cross-Source Commands

```bash
# What's near a location? (weather + TDs + water stations)
irl nearby --location dublin
irl nearby --location galway
irl nearby --lat 51.90 --lon -8.47          # Custom coordinates (Cork city)

# National overview
irl snapshot                                # Warnings + recent votes + latest bills

# Flood risk assessment (rainfall × water stations)
irl flood-risk                              # National
irl flood-risk --county Galway              # County-filtered

# Watch for changes
irl watch --source warnings                 # First run saves baseline
irl watch --source warnings                 # Subsequent runs report new items
irl watch --source legislation              # Watch for new bills
irl watch --source divisions                # Watch for new votes
irl watch --source warnings --reset         # Reset the baseline
```

## LLM Integration

This tool is designed to be called by LLMs (like Claude) to answer questions about Irish public data. Key features for LLM usage:

- **`--format json`** returns full, untruncated API data (table output is truncated for human readability, JSON never is)
- **`--quiet`** suppresses headers and info messages, outputting only data
- **Fuzzy matching** auto-corrects typos in location names, constituency names, and member names
- **Historical awareness** maps redistricted constituency names (e.g., "Dublin North-Central") to current equivalents with an explanatory note
- **Auto-pagination** ensures filtered queries return complete results, not just the first page
- **TD profiles** (`irl oireachtas td`) give an LLM everything about a member in one call
- **Cross-source queries** (`nearby`, `snapshot`, `flood-risk`) answer compound questions in a single call
- **Change detection** (`watch`) lets an LLM check if anything has changed since it last looked
- **Structured errors** include "Did you mean?" suggestions and actionable next steps

### Example LLM workflow

```
User: "What's my TD doing about housing?"
LLM:  1. irl nearby --location dublin --quiet  → find TDs for Dublin
      2. irl oireachtas td --name "Gary Gannon" --quiet  → get profile
      → Answer from questions and sponsored bills about housing
```

Or in a single call:
```
User: "What's happening in Ireland today?"
LLM:  irl snapshot --quiet
      → Answer from warnings, votes, and legislation
```

## Caching

Responses are cached locally at `~/.irl/cache/` with these default TTLs:

- CSO table of contents: 24 hours
- CSO data queries: 1 hour
- Met Éireann observations: 1 hour
- Transport real-time data: 5 minutes
- CRO search results: 24 hours
- Water station list: 1 hour
- Oireachtas data: 1 hour

Use `--no-cache` to bypass caching for any request.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Run `cargo test --workspace` and `cargo clippy --workspace`
4. Submit a pull request

## Licence

MIT. See [LICENSE](LICENSE).

Data sourced from Irish public sector APIs. Most data is published under CC BY 4.0. See [SOURCES.md](SOURCES.md) for details.
