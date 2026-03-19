# CLAUDE.md — Project Guide for AI Agents

This file provides context for AI coding agents (Claude, Cursor, Copilot, etc.) working on this project.

## Project Overview

**market_research_grok** is a Rust CLI that performs market research on product ideas using the Grok API with live X (Twitter) and web search. It takes a product description as input, generates search terms, researches each term via Grok's live search, and produces a JSON report with an honest market assessment.

**Repository:** `second-state/market-research-grok`
**Language:** Rust (edition 2024)
**Binary name:** `market_research`

## Architecture

This is intentionally a single-file binary (`src/main.rs`, ~300 lines). Do not split it into multiple files unless it exceeds ~500 lines.

### Code Structure

```
src/main.rs
├── CLI definition (clap derive)
├── API types (ResponsesRequest, ResponsesResponse, etc.)
├── Output types (MarketReport, Finding, Synthesis)
├── Grok client (new, chat)
├── Phase 1: generate_terms() — creates search queries from product description
├── Phase 2: research_term() — searches X/web for each term, extracts signals
├── Phase 3: synthesize() — combines all findings into honest assessment
├── JSON extraction helpers
└── main() — orchestrates the three phases
```

### Key Dependencies

| Crate | Purpose | Notes |
|-------|---------|-------|
| `clap` | CLI parsing | derive feature |
| `reqwest` | HTTP client | `default-features = false`, `rustls-tls` — **NO OpenSSL** |
| `serde` / `serde_json` | JSON serialization | |
| `tokio` | Async runtime | |
| `anyhow` | Error handling | |
| `indicatif` | Progress bar | |

**Critical: reqwest must use `rustls-tls` with `default-features = false`.** This ensures zero OpenSSL dependency, enabling static musl builds on Linux.

## Build & Test

```bash
# Build
cargo build --release

# Quick tests (no API key)
cargo test -- --skip live_api

# Full tests (needs GROK_API_KEY)
GROK_API_KEY="..." cargo test --release -- --nocapture

# Lint
cargo fmt -- --check
cargo clippy -- -D warnings
```

### ⚠️ Pre-Commit Checklist (MANDATORY)

**Before every commit, run all three and fix any issues:**

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test -- --skip live_api
```

Do NOT commit if any of these fail. CI will reject it. Common gotchas:

- **`cargo fmt`** — Always run it, not just `--check`. The CI runner's formatting may differ from your local defaults.
- **`cargo clippy`** — Treats all warnings as errors (`-D warnings`). Fix them, don't suppress with `#[allow]` unless there's a good reason.
- **Test assertions** — The binary name is `market_research` (underscore), not `market-research` (hyphen). The repo/skill name uses hyphens, the Cargo binary uses underscores. Don't mix them up in assertions or docs.

## CI/CD

### CI (`ci.yml`)
- **lint** — fmt + clippy (runs on every push and PR)
- **build** — cargo build --release (runs on every push and PR)
- **test (unit)** — unit tests run on every push and PR
- **test (live API)** — integration tests with real Grok API calls run **only on PR merges to main** (not on every push). These cost real money per run.

### Release (`release.yml`)
- Triggered by pushing a `v*` tag
- Builds 4 targets:
  - `aarch64-apple-darwin` (macOS ARM) — runs on `macos-latest`
  - `x86_64-apple-darwin` (macOS Intel) — runs on `macos-latest` (cross-compiles)
  - `x86_64-unknown-linux-musl` (Linux x86, **static**)
  - `aarch64-unknown-linux-musl` (Linux ARM, **static**)
- Packages as `.tar.gz` and attaches to GitHub Release

To cut a release:
```bash
git tag v0.1.0
git push --tags
```

## API Integration

### Grok API
- **Endpoint:** `https://api.x.ai/v1/responses` (Responses API — NOT the old `/v1/chat/completions`)
- **Model:** `grok-4-0709` — the most expensive/advanced model ($3.00 in / $15.00 out per MTok), always reasons at maximum effort
- **Auth:** Bearer token from `GROK_API_KEY` env var
- **Search:** Always enabled via tools: `[{"type": "web_search"}, {"type": "x_search"}]`
- **Request format:** Uses `input` (not `messages`) and `tools` (not `search_parameters`)

> ⚠️ **API Migration History (March 2026):**
> - The old `/v1/chat/completions` with `search_parameters` was deprecated (410 Gone).
> - The new Responses API uses `/v1/responses` with `tools` array. See https://docs.x.ai/developers/tools/overview
> - `grok-3` does NOT support server-side tools (web_search, x_search) — only grok-4 family does.
> - `grok-4-0709` does NOT accept `reasoning_effort` parameter — it always thinks at max. Only `grok-3-mini` supports `reasoning_effort`.

### Response Format

The Responses API returns an `output` array containing items with `content` blocks:
```json
{
  "output": [
    {
      "type": "message",
      "content": [
        { "type": "output_text", "text": "..." }
      ]
    }
  ]
}
```
The `extract_text_from_output()` function walks this structure to get the text.

### Prompt Design

The prompts are critical to output quality. Key principles:
1. **Structured JSON output** — Every prompt requests specific JSON schemas with no markdown wrapping
2. **Honesty enforcement** — The synthesis prompt explicitly asks for brutal honesty and penalizes cheerleading
3. **Temperature tuning** — 0.7 for term generation (needs creativity), 0.3 for research/synthesis (needs precision)
4. **Dimension coverage** — Term generation covers: features, personas, pricing, competitors, problem domains, emotional triggers

### JSON Extraction

Grok sometimes wraps JSON in markdown code blocks. The `extract_json_array()` and `extract_json_object()` helpers handle this by finding the first `[`/`{` and last `]`/`}` in the response. This is intentionally simple — don't over-engineer it.

## OpenClaw Skill

The `skill/` directory contains an [OpenClaw](https://openclaw.ai) agent skill:

```
skill/
├── SKILL.md        — Agent instructions (uses {baseDir} placeholder)
├── README.md       — Human-facing docs
├── install.md      — Installation guide
└── bootstrap.sh    — Auto-downloads platform binary from GitHub Releases
```

Installed to `~/.openclaw/skills/market-research-grok/`. The bootstrap script detects OS/arch and downloads the matching release binary.

## Design Principles

1. **Single binary** — No Python, Node, Docker, or runtime dependencies
2. **Honest output** — The tool is designed to tell you your idea might be bad. That's the point.
3. **Real data** — Every finding comes from live X/web search, not training data
4. **Simple code** — One file, straightforward flow, no abstractions that don't pay for themselves
5. **Cross-platform** — macOS + Linux, ARM + x86, static musl for Linux

## Lessons Learned

- **`macos-13` GitHub Actions runner is retired.** Use `macos-latest` for all macOS builds. Cross-compile x86_64-apple-darwin from ARM runners.
- **Grok API changed significantly in 2026.** The `search_parameters` field on `/v1/chat/completions` is gone (410). Must use `/v1/responses` with `tools` array.
- **Server-side tools require grok-4 family.** `grok-3` returns 400 if you pass `web_search` or `x_search` tools.
- **Don't run live API tests on every push.** They cost real money (grok-4 at $3/$15 per MTok). Run them only on PR merges.
- **Naming convention:** repo name uses hyphens (`market-research-grok`), Cargo/binary uses underscores (`market_research`). Be consistent in each context.

## Common Tasks

### Adding a new output field to the report
1. Add the field to the relevant struct (`Finding` or `Synthesis`)
2. Update the corresponding prompt to request the new field
3. Update the example in README.md

### Changing the model
Update the `MODEL` constant in `src/main.rs`. Only grok-4 family models support server-side tools.

### Adding a new search tool
Add a new `Tool { kind: "..." }` entry in the `Grok::chat()` method's `tools` vec. Available: `web_search`, `x_search`, `code_interpreter`.

### Adjusting prompt behavior
All prompts are inline in `generate_terms()`, `research_term()`, and `synthesize()`. Edit directly — they're format strings with clear structure.

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `GROK_API_KEY` | Yes | xAI API key from [console.x.ai](https://console.x.ai) |

## Repo Secrets (GitHub Actions)

| Secret | Used in | Purpose |
|--------|---------|---------|
| `GROK_API_KEY` | `ci.yml` | Live API integration tests (PR merges only) |
