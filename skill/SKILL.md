# Market Research via Grok — X/Web Sentiment Analysis

Analyze market demand for a product idea by searching X (Twitter) and the web for real user opinions, pain points, and competitive signals.

## Binary

- `{baseDir}/scripts/market_research` — Market research CLI.

## Prerequisites

- `GROK_API_KEY` environment variable set with a valid xAI API key.

## Usage

Run a market research analysis on a product idea:

```shell
GROK_API_KEY="$GROK_API_KEY" {baseDir}/scripts/market_research \
  --product "PRODUCT_DESCRIPTION" \
  --output /tmp/market-report.json
```

### Parameters

| Parameter   | Required | Description                                              |
|-------------|----------|----------------------------------------------------------|
| `--product` | Yes      | Product description: features, user profiles, price points, use cases |
| `--output`  | No       | Output file path (defaults to stdout)                    |
| `--terms`   | No       | Number of search terms to generate, 10-20 (default: 15)  |

### Output

A JSON report written to the specified file (or stdout), containing:

- `product_summary` — The input product description
- `search_terms` — Generated multi-dimensional search terms
- `findings[]` — Per-term analysis with positive/negative signals, quotes, and sentiment
- `synthesis` — Final assessment with market need score (1-10), pain points, risks, and opportunities

### Example

```shell
GROK_API_KEY="$GROK_API_KEY" {baseDir}/scripts/market_research \
  --product "A CLI tool for developers that auto-generates API documentation from code comments. Features: multi-language support (Rust, Go, Python), OpenAPI output, CI integration. Target users: backend developers, DevOps teams. Price: $19/mo for teams, free for OSS. Use case: reducing doc drift in fast-moving codebases." \
  --output /tmp/api-doc-tool-report.json
```

## Workflow

### 1. Gather the Product Description

Ask the user to describe their product idea. Encourage them to include:
- Key features and differentiators
- Target user profiles / personas
- Proposed price points
- Example use cases or problem scenarios

### 2. Run the Analysis

```shell
GROK_API_KEY="$GROK_API_KEY" {baseDir}/scripts/market_research \
  --product "USER_PRODUCT_DESCRIPTION" \
  --output /tmp/market-report.json
```

The CLI runs three phases:
1. **Term Generation** — Grok generates 10-20 search terms covering features, competitors, personas, pricing, and pain points.
2. **Market Research** — Each term is searched on X and the web via Grok with live search enabled. Real quotes, positive/negative signals, and sentiment are extracted.
3. **Synthesis** — All findings are synthesized into an honest market assessment.

### 3. Present the Results

Read the JSON output and present a summary to the user:

```shell
cat /tmp/market-report.json
```

Key sections to highlight:
- **Market Need Score** (1-10) and description
- **Pain Points Solved** vs **Pain Points Missed**
- **Competitive Landscape**
- **Honest Assessment** — the frank "would you invest?" verdict
- **Risks** and **Opportunities**

## How It Works

- Uses **Grok (grok-3)** with X search, web search, and news search enabled on every call.
- All search is live — results reflect current conversations and sentiment on X/Twitter.
- The synthesis is designed to be brutally honest, not a cheerleading exercise.
- No OpenSSL dependency — uses rustls for TLS. Linux builds are statically linked with musl.

## Supported Platforms

- macOS Apple Silicon (aarch64)
- macOS Intel (x86_64)
- Linux x86_64 (static musl)
- Linux aarch64 (static musl)
