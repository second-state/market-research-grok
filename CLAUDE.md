# CLAUDE.md — Project Guide for AI Agents

This file provides context for AI coding agents (Claude, Cursor, Copilot, etc.) working on this project.

## Project Overview

**market_research_grok** is a Rust CLI that performs market research on product ideas using the Grok API with live X (Twitter) and web search, then generates product concept images and animated videos. It takes a product description as input, generates search terms, researches each term via Grok's live search, produces a JSON report with an honest market assessment, and creates 5 AI-generated product images + 5 animated videos.

**Repository:** `second-state/market-research-grok`
**Language:** Rust (edition 2024)
**Binary name:** `market_research`

## Architecture

This is intentionally a single-file binary (`src/main.rs`, ~500 lines). Do not split it into multiple files unless it exceeds ~800 lines.

### Code Structure

```
src/main.rs
├── CLI definition (clap derive)
├── Grok Responses API types (ResponsesRequest, ResponsesResponse, etc.)
├── Image Generation API types (ImageGenRequest, ImageGenResponse)
├── Video Generation API types (VideoGenRequest, VideoGenPollResponse)
├── Output types (MarketReport, Finding, Synthesis, MediaAsset)
├── Grok client
│   ├── chat()             — Responses API with web_search + x_search tools
│   ├── generate_image()   — POST /v1/images/generations
│   └── generate_video()   — POST /v1/videos/generations + polling GET /v1/videos/{id}
├── Phase 1: generate_terms() — search query generation
├── Phase 2: research_term() — per-term live search + sentiment extraction
├── Phase 3: synthesize() — honest market assessment
├── Phase 4: generate_image_prompts() — 5 concept briefs from product + synthesis
├── Phase 5-6: generate_media() — image rendering + video animation
├── JSON extraction helpers
└── main() — orchestrates all six phases
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

**Dev-only:** `reqwest` with `blocking` feature for integration test URL checks.

**Critical: reqwest must use `rustls-tls` with `default-features = false`.** This ensures zero OpenSSL dependency, enabling static musl builds on Linux.

## Build & Test

```bash
# Build
cargo build --release

# Quick tests (no API key)
cargo test -- --skip live_api

# Research test (needs GROK_API_KEY, uses --skip-media)
GROK_API_KEY="..." cargo test --release -- live_api_generates_report --nocapture

# Full test with media generation (expensive, ~10 min)
GROK_API_KEY="..." cargo test --release -- live_api_generates_media --nocapture

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
- **test (research)** — live API research test (--skip-media), runs **only on PR merges to main**
- **test (media)** — full media generation test, runs **only on PR merges to main**, 20min timeout
- **paths-ignore** — CI is **skipped entirely** for doc-only changes (`*.md`, `skill/**`, `LICENSE`, `.gitignore`)

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
git tag v0.2.0
git push --tags
```

## API Integration

### Models Used

| API | Model | Price | Purpose |
|-----|-------|-------|---------|
| Responses | `grok-4-0709` | $3.00/$15.00 per MTok | Research, synthesis, prompt generation |
| Image Gen | `grok-imagine-image-pro` | $0.07/image | Product concept images (highest quality) |
| Video Gen | `grok-imagine-video` | $0.05/sec | 5-sec animated videos from images |

### Grok Responses API
- **Endpoint:** `https://api.x.ai/v1/responses`
- **Model:** `grok-4-0709` — most expensive, always reasons at max effort
- **Tools:** `[{"type": "web_search"}, {"type": "x_search"}]`
- **Request format:** Uses `input` (not `messages`) and `tools` (not `search_parameters`)

### Image Generation API
- **Endpoint:** `https://api.x.ai/v1/images/generations`
- **Model:** `grok-imagine-image-pro` (not the cheaper `grok-imagine-image`)
- **Response:** `data[0].url` — temporary URL, download promptly

### Video Generation API
- **Endpoint:** `https://api.x.ai/v1/videos/generations` (start) + `GET /v1/videos/{request_id}` (poll)
- **Model:** `grok-imagine-video`
- **Config:** 5 seconds, 16:9, 720p
- **Flow:** Submit → get `request_id` → poll every 5s → status `done` returns `video.url`
- **Statuses:** `pending` | `done` | `failed` | `expired`
- **Image-to-video:** Pass `image: {url, type: "image_url"}` to animate from a source image

> ⚠️ **API Migration History (March 2026):**
> - The old `/v1/chat/completions` with `search_parameters` was deprecated (410 Gone).
> - The new Responses API uses `/v1/responses` with `tools` array. See https://docs.x.ai/developers/tools/overview
> - `grok-3` does NOT support server-side tools (web_search, x_search) — only grok-4 family does.
> - `grok-4-0709` does NOT accept `reasoning_effort` parameter — it always thinks at max.

### Prompt Design

The prompts are critical to output quality. Key principles:
1. **Structured JSON output** — Every prompt requests specific JSON schemas with no markdown wrapping
2. **Honesty enforcement** — The synthesis prompt explicitly asks for brutal honesty and penalizes cheerleading
3. **Temperature tuning** — 0.7 for creative tasks (terms, image prompts), 0.3 for analytical tasks (research, synthesis)
4. **Dimension coverage** — Term generation covers: features, personas, pricing, competitors, problem domains, emotional triggers
5. **Image diversity** — 5 concepts cover: hero shot, user context, pain point, transformation, aspiration

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
6. **Visual output** — Product concepts rendered as images and animated videos for pitch decks

## Lessons Learned

- **`macos-13` GitHub Actions runner is retired.** Use `macos-latest` for all macOS builds.
- **Grok API changed in 2026.** `search_parameters` on `/v1/chat/completions` is gone (410). Must use `/v1/responses` with `tools`.
- **Server-side tools require grok-4 family.** `grok-3` returns 400 with web_search/x_search.
- **Don't run live API tests on every push.** They cost real money. Run on PR merges only.
- **Skip CI for doc changes.** Use `paths-ignore` in workflow triggers.
- **Video generation is async.** Submit, get request_id, poll every 5s. Can take several minutes.
- **Image/video URLs are temporary.** Download immediately after generation.
- **Naming convention:** repo uses hyphens (`market-research-grok`), Cargo/binary uses underscores (`market_research`).

## Common Tasks

### Adding a new output field to the report
1. Add the field to the relevant struct (`Finding`, `Synthesis`, or `MediaAsset`)
2. Update the corresponding prompt to request the new field
3. Update the example in README.md

### Changing the model
Update the relevant constant in `src/main.rs`:
- `MODEL` — for research/synthesis (must be grok-4 family for tools)
- `IMAGE_MODEL` — for image generation
- `VIDEO_MODEL` — for video generation

### Adjusting video parameters
Edit the `generate_video()` method: `duration`, `aspect_ratio`, `resolution`.

### Adding a new media concept
Update the prompt in `generate_image_prompts()` — it currently requests 5 concepts.

### Adjusting prompt behavior
All prompts are inline in their respective functions. Edit directly — they're format strings.

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `GROK_API_KEY` | Yes | xAI API key from [console.x.ai](https://console.x.ai) |

## Repo Secrets (GitHub Actions)

| Secret | Used in | Purpose |
|--------|---------|---------|
| `GROK_API_KEY` | `ci.yml` | Live API integration tests (PR merges only) |
