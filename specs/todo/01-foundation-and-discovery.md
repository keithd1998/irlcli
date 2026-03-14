# Foundation, Core Infrastructure & Oireachtas Module

## Ambiguity Assessment

| Dimension | Score (0-1) | Notes |
|-----------|-------------|-------|
| Goal Clarity | 0.95 | Exhaustive prompt with per-module commands |
| Constraint Clarity | 0.90 | Rust 2021, ~/.irl/, latest deps, stubs upfront |
| Success Criteria | 0.85 | Clear CLI commands; Oireachtas API well-documented |
| Context Clarity | 1.0 | Greenfield in empty directory |
| **Weighted Total** | **0.095** | Proceeding |

## Problem Statement

Irish public sector data is scattered across dozens of APIs with inconsistent formats, authentication requirements, and documentation quality. There is no unified CLI tool that provides structured access to this data. Developers, journalists, data scientists, and civic tech enthusiasts must write bespoke scripts for each source.

`irl` solves this by providing a single, polished CLI binary with subcommand-per-source architecture — like the `aws` CLI but for Irish government data.

## Objectives

1. Establish a Rust workspace with the full module structure (10 crates as stubs)
2. Build `irl-core` with shared config, HTTP client, output formatting, and caching
3. Implement `irl-oireachtas` as the first fully working module (best-documented API, proves architecture)
4. Deliver a working CLI with `irl help`, `irl config`, `irl oireachtas` commands, and 9 stubbed subcommands showing "coming soon"

## Assumptions Exposed

1. **`~/.irl/` config path** — validated: user prefers simple dotdir over XDG paths for discoverability
2. **All 10 crate stubs upfront** — validated: user wants the full vision visible from day one, even if modules aren't implemented
3. **Rust 2021 edition** — validated: broadest compatibility over bleeding-edge features
4. **Latest compatible deps** — validated: use broad version ranges, let cargo resolve
5. **Oireachtas API is stable** — assumption: the API at api.oireachtas.ie is well-documented and returns JSON; should be verified with discovery requests during build
6. **Split specs** — validated: 4 independent specs, each buildable with `/oibri-build`

## Technical Approach

### Overview

Create a Cargo workspace rooted at `cliirl/` (not a nested `irl-cli/` subdirectory — we're already in the project directory). The workspace contains `irl-core` (shared infrastructure) and 10 module crates. The main binary in `src/main.rs` uses clap derive macros for subcommand routing.

### Architecture

```
cliirl/
├── Cargo.toml              # Workspace root
├── README.md
├── LICENSE                  # MIT
├── SOURCES.md              # API endpoints, licences, registration requirements
├── crates/
│   ├── irl-core/           # Config, HTTP, output, cache, error
│   ├── irl-oireachtas/     # IMPLEMENTED in this spec
│   ├── irl-cso/            # Stub
│   ├── irl-met/            # Stub
│   ├── irl-transport/      # Stub
│   ├── irl-cro/            # Stub
│   ├── irl-property/       # Stub
│   ├── irl-epa/            # Stub
│   ├── irl-water/          # Stub
│   ├── irl-tailte/         # Stub
│   └── irl-geo/            # Stub
├── src/
│   └── main.rs             # CLI entry point
└── specs/
```

### Key Components

1. **irl-core::config** — Config file management at `~/.irl/config.toml`. `irl config init` creates interactively, `irl config set` for programmatic updates. Serde-based config struct.
2. **irl-core::http** — Shared `reqwest` client with retry (exponential backoff, max 3 retries), rate limiting (token bucket), and verbose request logging. Uses `rustls-tls`.
3. **irl-core::output** — Output formatting: table (via `tabled` + `colored`), JSON (pretty-printed), CSV. Auto-detects TTY for colour/format. Respects `NO_COLOR` env var.
4. **irl-core::cache** — File-based response cache in `~/.irl/cache/`. Key = hash of URL + params. Configurable TTL per request. `--no-cache` bypasses.
5. **irl-core::error** — `thiserror` for library errors, `anyhow` for application. Actionable error messages (e.g., "API key missing — run: irl config set transport.api_key <YOUR_KEY>").
6. **irl-oireachtas** — Full implementation against `https://api.oireachtas.ie/v1/`. Members, legislation, debates, questions, divisions, parties. Pagination support.

### Global CLI Flags

Every subcommand inherits these via clap:
- `--format <table|json|csv>` — override default output format
- `--no-colour` — disable coloured output (also respect `NO_COLOR` env var)
- `--no-cache` — bypass local cache
- `--verbose` / `-v` — show HTTP request/response details
- `--quiet` / `-q` — suppress everything except data output

### Oireachtas Module Commands

```
irl oireachtas members                                    # List current TDs and Senators
irl oireachtas members --party "Sinn Féin"                # Filter by party
irl oireachtas members --constituency "Dublin Central"    # Filter by constituency
irl oireachtas legislation                                # Recent legislation
irl oireachtas legislation --search "planning"            # Search bills
irl oireachtas legislation --status enacted --year 2025   # Filter
irl oireachtas debates --date 2025-03-01                  # Debates on a date
irl oireachtas questions --member "Mary Lou McDonald"     # PQs by member
irl oireachtas divisions --recent                         # Recent Dáil/Seanad votes
```

API endpoints (documented at https://api.oireachtas.ie/):
- `/v1/members?limit=50&skip=0`
- `/v1/legislation?limit=20&skip=0`
- `/v1/debates?date_start=YYYY-MM-DD`
- `/v1/questions?member_id=...`
- `/v1/divisions?limit=20`
- `/v1/parties`

Pagination: API uses `limit` and `skip` params. Default limit=20, support `--limit` and `--page` flags.

## Implementation Phases

### Phase 1: Workspace Skeleton & Build Verification

**Goal**: Cargo workspace compiles, `irl --help` shows all subcommands, `irl --version` works.

**Steps**:
1. Create root `Cargo.toml` with workspace members and shared dependencies
2. Create `src/main.rs` with clap derive-based CLI struct, all subcommands defined
3. Create all 10 module crate directories with minimal `Cargo.toml` and `lib.rs` (empty public module)
4. Stub subcommands show "Coming soon: <module name>" message
5. Implement `--version` with version from Cargo.toml
6. Create `LICENSE` (MIT) and skeleton `README.md`
7. Verify `cargo build` and `cargo test` pass

**Files to create**:
- `Cargo.toml` — workspace root
- `src/main.rs` — CLI entry point with all subcommand routing
- `crates/irl-core/Cargo.toml` + `crates/irl-core/src/lib.rs`
- `crates/irl-oireachtas/Cargo.toml` + `crates/irl-oireachtas/src/lib.rs`
- `crates/irl-cso/Cargo.toml` + `crates/irl-cso/src/lib.rs` (stub)
- `crates/irl-met/Cargo.toml` + `crates/irl-met/src/lib.rs` (stub)
- `crates/irl-transport/Cargo.toml` + `crates/irl-transport/src/lib.rs` (stub)
- `crates/irl-cro/Cargo.toml` + `crates/irl-cro/src/lib.rs` (stub)
- `crates/irl-property/Cargo.toml` + `crates/irl-property/src/lib.rs` (stub)
- `crates/irl-epa/Cargo.toml` + `crates/irl-epa/src/lib.rs` (stub)
- `crates/irl-water/Cargo.toml` + `crates/irl-water/src/lib.rs` (stub)
- `crates/irl-tailte/Cargo.toml` + `crates/irl-tailte/src/lib.rs` (stub)
- `crates/irl-geo/Cargo.toml` + `crates/irl-geo/src/lib.rs` (stub)
- `LICENSE`
- `README.md`

### Phase 2: Core Infrastructure (irl-core)

**Goal**: Config loading, HTTP client with retry/rate-limiting, output formatting, and caching all working with unit tests.

**Steps**:
1. Implement `config.rs` — load/save `~/.irl/config.toml`, `Config` struct with serde, `irl config init` and `irl config set` commands
2. Implement `http.rs` — shared reqwest client builder, retry with exponential backoff (max 3 attempts, 1s/2s/4s), rate limiter (token bucket), verbose logging, `User-Agent: irl-cli/<version>`
3. Implement `output.rs` — `OutputFormat` enum, `Outputable` trait, table renderer (tabled + colored), JSON renderer (serde_json pretty), CSV renderer. Auto-detect TTY via `atty` or `std::io::IsTerminal`. Respect `NO_COLOR` env var and `--no-colour` flag.
4. Implement `cache.rs` — file-based cache in `~/.irl/cache/`, SHA256 hash of URL+params as filename, JSON metadata sidecar with TTL and timestamp, `--no-cache` bypass
5. Implement `error.rs` — `IrlError` enum with `thiserror`, variants for HttpError, ConfigError, CacheError, ParseError, ApiKeyMissing. Actionable Display impls.
6. Add `indicatif` spinner integration — wrap HTTP requests with spinner that appears after 500ms

**Files to create/modify**:
- `crates/irl-core/src/lib.rs` — re-export modules
- `crates/irl-core/src/config.rs`
- `crates/irl-core/src/http.rs`
- `crates/irl-core/src/output.rs`
- `crates/irl-core/src/cache.rs`
- `crates/irl-core/src/error.rs`
- `crates/irl-core/Cargo.toml` — add dependencies

### Phase 3: Oireachtas API Discovery & Implementation

**Goal**: Fully working `irl oireachtas` with all subcommands hitting the live API.

**Steps**:
1. **API Discovery** — make exploratory requests to `https://api.oireachtas.ie/v1/members`, `/v1/legislation`, `/v1/debates`, `/v1/questions`, `/v1/divisions`, `/v1/parties`. Save response samples as test fixtures in `crates/irl-oireachtas/tests/fixtures/`.
2. Define response structs based on actual API responses (not assumed shapes)
3. Implement API client functions: `list_members()`, `list_legislation()`, `list_debates()`, `list_questions()`, `list_divisions()`, `list_parties()`
4. Implement clap subcommand enums and handler functions
5. Wire into `src/main.rs` subcommand routing
6. Implement pagination with `--limit` and `--page` flags
7. Implement search/filter flags per subcommand
8. Add cache TTL of 1 hour for Oireachtas data
9. Write unit tests against saved fixtures
10. Write integration tests behind `#[cfg(feature = "integration-tests")]`

**Files to create/modify**:
- `crates/irl-oireachtas/src/lib.rs` — main module with API client and subcommands
- `crates/irl-oireachtas/src/api.rs` — API client functions
- `crates/irl-oireachtas/src/models.rs` — response structs
- `crates/irl-oireachtas/src/commands.rs` — clap subcommand definitions and handlers
- `crates/irl-oireachtas/tests/fixtures/` — saved API responses
- `crates/irl-oireachtas/tests/` — unit tests
- `src/main.rs` — wire in oireachtas commands

### Phase 4: Documentation & Polish

**Goal**: README, SOURCES.md, help text on every subcommand, final verification.

**Steps**:
1. Write `README.md` with installation instructions, quick start, module overview, contributing guide
2. Write `SOURCES.md` documenting each API endpoint, data licence, and registration requirements
3. Ensure every subcommand has `/// Long help text with examples` via clap attributes
4. Ensure `irl help` shows all modules with one-line descriptions
5. Verify coloured output, table formatting, JSON/CSV output modes
6. Test pipe detection (stdout not TTY → plain format, no colour)
7. Final `cargo clippy` and `cargo fmt` pass

**Files to create/modify**:
- `README.md` — full documentation
- `SOURCES.md` — API endpoint documentation
- Various `--help` text in clap derive attributes

## Testing Strategy

### Automated Tests
- [ ] `irl-core::config` — load, save, create default, set values, handle missing file
- [ ] `irl-core::http` — retry logic (mock server returning 500 then 200), rate limiter timing
- [ ] `irl-core::output` — table, JSON, CSV rendering from sample data structs
- [ ] `irl-core::cache` — write, read, TTL expiry, bypass flag
- [ ] `irl-oireachtas::models` — deserialize saved API fixtures
- [ ] `irl-oireachtas::commands` — verify clap parsing for all flag combinations
- [ ] Integration: `irl oireachtas members` returns data (behind feature flag)
- [ ] Integration: `irl oireachtas members --format json | jq .` produces valid JSON

### Manual Verification
- [ ] `irl --help` shows all modules with descriptions
- [ ] `irl --version` shows version
- [ ] `irl oireachtas members` displays formatted table with colour
- [ ] `irl oireachtas members --format json` outputs valid JSON
- [ ] `irl oireachtas members --format csv` outputs valid CSV
- [ ] `irl config init` creates config file interactively
- [ ] Piping `irl oireachtas members | head` disables colour automatically
- [ ] `NO_COLOR=1 irl oireachtas members` disables colour
- [ ] `irl cso tables` shows "Coming soon" message
- [ ] Spinner appears on slow requests

## Success Criteria

- [ ] `cargo build --release` produces a single `irl` binary
- [ ] `cargo test` passes all unit tests
- [ ] `irl --help` shows 10+ subcommands (1 working, 9 stubbed)
- [ ] `irl oireachtas members` returns and displays real parliamentary data
- [ ] All 3 output formats (table, json, csv) work correctly
- [ ] Config file at `~/.irl/config.toml` is created and respected
- [ ] Cache reduces repeated API calls (verified by --verbose showing cache hits)
- [ ] Error messages are actionable, not raw HTTP errors
- [ ] `cargo clippy` produces no warnings
- [ ] Irish characters (fadas) display correctly in all output formats

## Risks & Mitigations

1. **Oireachtas API response format differs from docs**: Mitigation — API discovery phase saves real responses as fixtures before writing parsing code
2. **Large workspace slow to compile**: Mitigation — stub crates are minimal; only irl-core and irl-oireachtas have real code
3. **Rate limiting by Oireachtas API**: Mitigation — built-in rate limiter in irl-core::http, cache layer reduces repeat requests
4. **tabled/colored crate compatibility**: Mitigation — using latest compatible versions; table formatting tested independently

---

*Created: 2026-03-14*
*Status: todo*
*Ambiguity Score: 0.095*
*Jira: N/A*
