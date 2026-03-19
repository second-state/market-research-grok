# Market Research via Grok

An AI-powered CLI that analyzes market demand for product ideas by searching X (Twitter) and the web through [Grok](https://x.ai).

## What It Does

Given a product description (features, user profiles, price points, use cases), this tool:

1. **Generates search terms** — Grok creates 10-20 multi-dimensional queries covering features, competitors, user personas, pricing sensitivity, and pain points.
2. **Researches each term** — Each term is searched live on X and the web. Real user opinions, positive/negative signals, and direct quotes are extracted.
3. **Synthesizes findings** — All research is combined into an honest market assessment with a 1-10 market need score, competitive landscape analysis, and a frank "would you invest?" verdict.

## Quick Start

### Install as OpenClaw Skill

```bash
SKILL_DIR="${HOME}/.openclaw/skills/market-research-grok"
mkdir -p "$SKILL_DIR"
git clone --depth 1 https://github.com/second-state/market-research-grok.git /tmp/mr-repo
cp -r /tmp/mr-repo/skill/* "$SKILL_DIR/"
rm -rf /tmp/mr-repo
"${SKILL_DIR}/bootstrap.sh"
```

### Manual Install

Download the latest release for your platform from [Releases](https://github.com/second-state/market-research-grok/releases), extract, and run:

```bash
export GROK_API_KEY="xai-your-key-here"
./market_research --product "Your product idea..." --output report.json
```

### Build from Source

```bash
cargo build --release
```

## Usage

```
market_research [OPTIONS] --product <PRODUCT>

Options:
  -p, --product <PRODUCT>  Product description (features, users, pricing, use cases)
  -o, --output <OUTPUT>    Output file path (default: stdout)
      --terms <TERMS>      Number of search terms to generate, 10-20 [default: 15]
  -h, --help               Print help
```

## Output Format

```json
{
  "product_summary": "...",
  "search_terms": ["term1", "term2", "..."],
  "findings": [
    {
      "term": "...",
      "positive_signals": ["..."],
      "negative_signals": ["..."],
      "notable_quotes": ["..."],
      "sentiment": "mixed"
    }
  ],
  "synthesis": {
    "market_need_score": 7,
    "market_need_description": "...",
    "pain_points_solved": ["..."],
    "pain_points_missed": ["..."],
    "competitive_landscape": "...",
    "pricing_feedback": "...",
    "target_audience_fit": "...",
    "risks": ["..."],
    "opportunities": ["..."],
    "honest_assessment": "..."
  }
}
```

## Requirements

- **Grok API key** — Set `GROK_API_KEY` environment variable. Get one at [console.x.ai](https://console.x.ai).
- No other dependencies. The binary is self-contained (rustls for TLS, no OpenSSL).

## Platforms

| Platform | Binary | Linking |
|----------|--------|---------|
| macOS Apple Silicon | `market_research-macos-aarch64` | Dynamic (system libs) |
| macOS Intel | `market_research-macos-x86_64` | Dynamic (system libs) |
| Linux x86_64 | `market_research-linux-x86_64` | Static (musl) |
| Linux aarch64 | `market_research-linux-aarch64` | Static (musl) |

## License

Apache-2.0
