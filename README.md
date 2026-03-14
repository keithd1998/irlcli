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

# Search CSO statistical tables
irl cso tables --search "house prices"

# Output as JSON (full untruncated API data)
irl oireachtas members --format json

# Output as CSV for spreadsheets
irl met forecast --location cork --format csv

# Cross-source: what's near Dublin?
irl nearby --location dublin

# Handles historical constituency names
irl oireachtas members --constituency "Dublin North Central"

# Fuzzy matching corrects typos
irl met forecast --location dubln
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
| `nearby` | Cross-source geographic view | None | Working |

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
irl oireachtas divisions                                # Recent votes
irl oireachtas questions --member "Mary Lou McDonald"   # Parliamentary questions (auto-paginates)
```

Filtered queries (party, constituency, member) automatically paginate through all API results to ensure nothing is missed.

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
```

### Transport

```bash
irl transport departures --stop 767         # Next departures (requires API key)
irl transport vehicles --route 46a          # Live vehicle positions
irl transport routes                        # List routes
```

### Property Price Register

```bash
irl property update                                    # Download/import data
irl property search --county Dublin --year 2024        # Search sales
irl property search --min 200000 --max 500000          # Price range
irl property stats --county Dublin --year 2024         # Statistics
irl property stats --compare Dublin,Cork,Galway        # Compare counties
```

### Water Levels

```bash
irl water stations                          # All monitoring stations
irl water stations --county Galway          # Filter by county
irl water search "Corrib"                   # Search by name
```

### Nearby (Cross-Source)

```bash
irl nearby --location dublin                # Weather + water stations near Dublin
irl nearby --location cork                  # Weather + water stations near Cork
irl nearby --lat 53.35 --lon -6.26          # Custom coordinates
```

Returns a composite JSON view combining data from multiple sources for a geographic area — weather conditions from the nearest Met Éireann station and the 5 closest OPW water monitoring stations within 50km.

## LLM Integration

This tool is designed to be called by LLMs (like Claude) to answer questions about Irish public data. Key features for LLM usage:

- **`--format json`** returns full, untruncated API data (table output is truncated for human readability, JSON never is)
- **`--quiet`** suppresses headers and info messages, outputting only data
- **Fuzzy matching** auto-corrects typos in location names, constituency names, and member names
- **Historical awareness** maps redistricted constituency names to current equivalents
- **Auto-pagination** ensures filtered queries return complete results
- **Cross-source queries** (`irl nearby`) answer geographic questions in a single call
- **Structured errors** include "Did you mean?" suggestions and actionable next steps

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
