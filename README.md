# Market Research via Grok

A Rust CLI that analyzes market demand for product ideas using real-time data from X (Twitter) and the web, powered by [Grok](https://x.ai).

Instead of guessing whether your product idea has legs, this tool searches for what real people are actually saying — their frustrations, desires, complaints about competitors, and willingness to pay — then synthesizes it into an honest market assessment.

## How It Works

```
Product Idea → Grok generates search terms → Live X/web search → Sentiment extraction → Honest synthesis
```

**Three phases, one command:**

1. **Term Generation** — Your product description is analyzed by Grok to produce 10-20 search queries covering features, competitors, user personas, pricing sensitivity, and pain points.

2. **Market Research** — Each term is searched live on X and the web. For every term, the tool extracts positive signals, negative signals, notable quotes from real posts, and an overall sentiment rating.

3. **Synthesis** — All findings are fed back to Grok for a brutally honest final assessment: market need score (1-10), pain points solved and missed, competitive landscape, pricing feedback, risks, opportunities, and a frank "would you invest?" verdict.

Every API call uses **Grok (grok-3)** with X search, web search, and news search enabled. Results reflect real-time conversations and sentiment.

## Quick Start

```bash
# Set your xAI API key
export GROK_API_KEY="xai-your-key-here"

# Run analysis
./market_research \
  --product "A CLI tool for developers that auto-generates API docs from code comments. Supports Rust, Go, Python. Outputs OpenAPI. $19/mo for teams, free for OSS." \
  --output report.json
```

## Installation

### Pre-built Binaries

Download from [Releases](https://github.com/second-state/market-research-grok/releases/latest):

| Platform | Binary | Notes |
|----------|--------|-------|
| macOS Apple Silicon | `market_research-v*-aarch64-apple-darwin.tar.gz` | Dynamic |
| macOS Intel | `market_research-v*-x86_64-apple-darwin.tar.gz` | Dynamic |
| Linux x86_64 | `market_research-v*-x86_64-unknown-linux-musl.tar.gz` | **Static** (musl) |
| Linux ARM64 | `market_research-v*-aarch64-unknown-linux-musl.tar.gz` | **Static** (musl) |

Linux binaries are fully statically linked — no glibc, no OpenSSL, no runtime dependencies. Drop the binary anywhere and run it.

```bash
# Example: Linux x86_64
curl -sL https://github.com/second-state/market-research-grok/releases/latest/download/market_research-v0.1.0-x86_64-unknown-linux-musl.tar.gz | tar xz
chmod +x market_research
./market_research --help
```

### Build from Source

```bash
git clone https://github.com/second-state/market-research-grok.git
cd market-research-grok
cargo build --release
# Binary at ./target/release/market_research
```

### Install as OpenClaw Skill

```bash
SKILL_DIR="${HOME}/.openclaw/skills/market-research-grok"
mkdir -p "$SKILL_DIR"
git clone --depth 1 https://github.com/second-state/market-research-grok.git /tmp/mr-repo
cp -r /tmp/mr-repo/skill/* "$SKILL_DIR/"
rm -rf /tmp/mr-repo
"${SKILL_DIR}/bootstrap.sh"
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

### Writing a Good Product Description

The quality of your research depends on the input. Include:

- **Features** — What does it do? What's the core differentiator?
- **Target users** — Who specifically would use this? (developers, small business owners, students, etc.)
- **Price points** — Free? Freemium? $X/mo? Enterprise?
- **Use cases** — Concrete scenarios where someone would reach for this.

**Good example:**
```
A desktop app for freelance designers that automatically generates
invoices from time-tracking data. Integrates with Figma and Toggl.
Features: auto-tax calculation, multi-currency, PDF export, Stripe
payments. Target: solo freelancers and small design studios (2-5 people).
Price: $12/mo or $99/year. Use case: designer finishes a project,
clicks one button, client gets a professional invoice with tracked hours.
```

**Weak example:**
```
An invoicing app.
```

### Adjusting Search Depth

- `--terms 10` — Faster, cheaper, good for quick validation
- `--terms 15` — Default, balanced coverage
- `--terms 20` — Maximum depth, broader signal capture

Each term generates one Grok API call, so more terms = more cost and time.

## Output Format

```json
{
  "product_summary": "Your input description",
  "search_terms": [
    "freelance designer invoice automation",
    "Figma time tracking invoice",
    "Toggl invoice integration complaints",
    "..."
  ],
  "findings": [
    {
      "term": "freelance designer invoice automation",
      "positive_signals": [
        "Many freelancers on X express frustration with manual invoicing",
        "Strong desire for tools that connect design work to billing"
      ],
      "negative_signals": [
        "Some prefer all-in-one platforms over point solutions",
        "Price sensitivity — many freelancers expect free tools"
      ],
      "notable_quotes": [
        "@designer: 'I spend 2 hours every Friday doing invoices instead of designing'",
        "@studio_owner: 'We tried 4 different invoicing tools, none talk to Figma'"
      ],
      "sentiment": "positive"
    }
  ],
  "synthesis": {
    "market_need_score": 7,
    "market_need_description": "There is genuine frustration among freelance designers about the disconnect between time tracking and invoicing. The pain is real but the market has several incumbents.",
    "pain_points_solved": [
      "Manual time-to-invoice conversion",
      "Multi-tool workflow friction"
    ],
    "pain_points_missed": [
      "Contract/proposal generation",
      "Client communication and follow-ups"
    ],
    "competitive_landscape": "Crowded. FreshBooks, Harvest, and Wave all serve this space. Figma-specific integration is a differentiator but narrow.",
    "pricing_feedback": "Solo freelancers resist $12/mo for invoicing alone. Bundling with time tracking would increase perceived value.",
    "target_audience_fit": "Good fit for design studios. Solo freelancers may churn — they want free or very cheap.",
    "risks": [
      "Figma could build native invoicing",
      "Low switching costs — easy to leave for a cheaper alternative"
    ],
    "opportunities": [
      "No one owns the Figma→invoice pipeline yet",
      "Design agencies (5-20 people) are underserved and less price-sensitive"
    ],
    "honest_assessment": "The pain point is real but the moat is thin. Figma integration is clever but defensible only until Figma or a larger player copies it. I'd validate with 50 paying design studios before building beyond MVP. The solo freelancer market is a trap — high churn, low willingness to pay."
  }
}
```

## Architecture

```
src/main.rs          Single-file Rust binary (~300 lines)
├── CLI parsing      clap with derive macros
├── Grok client      reqwest + rustls (zero OpenSSL)
├── Phase 1          Term generation via structured prompts
├── Phase 2          Per-term live search + sentiment extraction
├── Phase 3          Synthesis with enforced honesty
└── JSON output      serde_json pretty-print
```

**Key design decisions:**

- **Single binary, zero runtime deps** — No Python, no Node, no Docker. Download and run.
- **rustls, not OpenSSL** — Pure Rust TLS. Static linking works on Linux (musl) without fighting OpenSSL cross-compilation.
- **Sequential API calls** — Intentionally not parallelized. Grok rate limits are per-key, and sequential calls produce more reliable results with search enabled.
- **Honest-by-design prompts** — The synthesis prompt explicitly asks for unvarnished assessment and penalizes cheerleading. This is a feature, not a bug.

## API Details

All calls go to `https://api.x.ai/v1/responses` (the Responses API) with:

- **Model:** `grok-3`
- **Tools:** `[{"type": "web_search"}, {"type": "x_search"}]` — live search on every call
- **Temperature:** 0.7 for term generation (creative), 0.3 for research and synthesis (precise)

## Cost Estimation

Each run makes `2 + N` API calls where N = number of search terms:
- 1 call for term generation
- N calls for research (one per term)
- 1 call for synthesis

With `--terms 15` (default), that's 17 Grok API calls. Check [xAI pricing](https://x.ai/pricing) for current rates.

## Testing

```bash
# Unit tests (no API key needed)
cargo test -- --skip live_api

# Full test suite (requires GROK_API_KEY)
export GROK_API_KEY="xai-..."
cargo test --release -- --nocapture
```

CI runs both — unit tests always, live API tests when `GROK_API_KEY` is available as a repo secret.

## Contributing

1. Fork the repo
2. Create a feature branch
3. Make changes — keep it simple, it's a single-file binary for a reason
4. Run `cargo fmt && cargo clippy -- -D warnings && cargo test`
5. Open a PR

## License

Apache-2.0
