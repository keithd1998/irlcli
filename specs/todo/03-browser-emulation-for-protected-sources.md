# Spec: Browser Emulation for Protected Sources

## Problem Statement

Several Irish government data sources (CRO company search, potentially Tailte Éireann) are behind Cloudflare protection or require browser-like behavior. Currently, these commands fail with a fallback message directing users to the website. An LLM using `claude-in-chrome` browser tools can access these sites, but the CLI should be able to handle this without requiring the LLM to drive a browser manually.

## Objectives

1. **Identify which sources need browser-like access** and which are truly API-accessible
2. **Add headless browser capability** for Cloudflare-protected sources
3. **Maintain the CLI interface** — the LLM calls the same `irl cro search` command, the browser emulation is transparent

## Options Analysis

### Option A: Embed a headless browser (chromiumoxide / headless_chrome)
- **Pro**: Fully self-contained, no external dependencies
- **Con**: Adds ~50MB to binary, complex build, fragile
- **Verdict**: Too heavy for a CLI tool

### Option B: Use `reqwest` with realistic browser headers + cookie jar
- **Pro**: Lightweight, often sufficient for basic Cloudflare challenges
- **Con**: Won't pass JS challenges (Cloudflare Managed Challenge)
- **Verdict**: Good first attempt, covers many cases

### Option C: Scraping via external browser (delegate to `claude-in-chrome`)
- **Pro**: The LLM already has browser access; CLI could output structured guidance
- **Con**: Couples the CLI to a specific LLM tool
- **Verdict**: Not suitable for a standalone CLI

### Option D: Use `playwright` or `selenium` as an optional dependency
- **Pro**: Proven browser automation, handles JS challenges
- **Con**: Requires Node.js/Python runtime, not a single binary
- **Verdict**: Could work as an optional feature flag

### Recommended: Option B first, with structured fallback

## Detailed Changes

### Phase 1: Realistic browser HTTP client

- Add a `BrowserLikeClient` to `irl-core` that:
  - Sets a realistic Chrome User-Agent
  - Maintains a cookie jar across requests
  - Follows redirects through Cloudflare's initial cookie-setting flow
  - Sends `Accept`, `Accept-Language`, `Accept-Encoding` headers matching Chrome
  - Handles `cf_clearance` cookie if returned

### Phase 2: CRO integration fix

- Switch `irl-cro` from the direct API endpoint to scraping `core.cro.ie` search results
- Parse the HTML search results page into structured data
- If Cloudflare blocks, return a structured error with:
  - The URL that was attempted
  - What data was expected
  - Suggestion: "This source requires browser access. Use the CRO website directly at [URL]"

### Phase 3: Tailte Éireann integration

- Investigate the actual Tailte Éireann data source and determine access method
- Implement either API or browser-like scraping as appropriate

## Success Criteria

1. `irl cro search --name "Stripe"` returns company results without manual browser intervention
2. If Cloudflare blocks the request, the error message is structured and actionable
3. No additional runtime dependencies required (no Node.js, no browser binary)

## Files Affected

- `crates/irl-core/src/http.rs` — new `BrowserLikeClient` or extend existing `HttpClient`
- `crates/irl-cro/src/api.rs` — switch to browser-like client
- `crates/irl-cro/src/scraper.rs` — new file: HTML parsing for CRO results
- `crates/irl-tailte/src/api.rs` — investigate and fix

## Complexity

Medium-High — browser emulation is unpredictable, Cloudflare changes frequently.

## Risks

- Cloudflare may upgrade protection, breaking the browser-like approach
- Scraping HTML is fragile if CRO redesigns their page
- Legal/ToS considerations for scraping government sites (likely fine for public data)

## Dependencies

- None — independent of specs 01 and 02
