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
├── API types (ChatRequest, ChatResponse, etc.)
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
- **lint** — fmt + clippy
- **build** — cargo build --release
- **test** — unit tests always run; live API tests run only when `GROK_API_KEY` secret is available

### Release (`release.yml`)
- Triggered by pushing a `v*` tag
- Builds 4 targets:
  - `aarch64-apple-darwin` (macOS ARM)
  - `x86_64-apple-darwin` (macOS Intel)
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
- **Endpoint:** `https://api.x.ai/v1/chat/completions`
- **Model:** `grok-3` (hardcoded as `MODEL` constant)
- **Auth:** Bearer token from `GROK_API_KEY` env var
- **Search:** Always enabled — `mode: "on"`, sources: `["x", "web", "news"]`
- **Citations:** Enabled

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

## Common Tasks

### Adding a new output field to the report
1. Add the field to the relevant struct (`Finding` or `Synthesis`)
2. Update the corresponding prompt to request the new field
3. Update the example in README.md

### Changing the model
Update the `MODEL` constant in `src/main.rs`. The API format is OpenAI-compatible.

### Adding a new search source
Add a new `Source { kind: "..." }` entry in the `Grok::chat()` method's `sources` vec.

### Adjusting prompt behavior
All prompts are inline in `generate_terms()`, `research_term()`, and `synthesize()`. Edit directly — they're format strings with clear structure.

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `GROK_API_KEY` | Yes | xAI API key from [console.x.ai](https://console.x.ai) |

## Repo Secrets (GitHub Actions)

| Secret | Used in | Purpose |
|--------|---------|---------|
| `GROK_API_KEY` | `ci.yml` | Live API integration tests |
