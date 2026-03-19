# Market Research via Grok

A Rust CLI that analyzes market demand for product ideas using real-time data from X (Twitter) and the web, powered by [Grok](https://x.ai). Generates an honest market assessment plus AI-created product concept images and videos.

Instead of guessing whether your product idea has legs, this tool searches for what real people are actually saying — their frustrations, desires, complaints about competitors, and willingness to pay — then synthesizes it into an honest market assessment with visual concept art.

## How It Works

```
Product Idea → Search terms → Live X/web research → Honest synthesis → Product images → Animated videos
```

**Six phases, one command:**

1. **Term Generation** — Your product description is analyzed by Grok to produce 10-20 search queries covering features, competitors, user personas, pricing sensitivity, and pain points.

2. **Market Research** — Each term is searched live on X and the web. For every term, the tool extracts positive signals, negative signals, notable quotes from real posts, and an overall sentiment rating.

3. **Synthesis** — All findings are fed back to Grok for a brutally honest final assessment: market need score (1-10), pain points solved and missed, competitive landscape, pricing feedback, risks, opportunities, and a frank "would you invest?" verdict.

4. **Image Prompt Generation** — Grok creates 5 polished image concepts based on the product and synthesis: hero shot, user in context, pain point visualization, transformation scene, and aspirational outcome.

5. **Image Generation** — Each concept is rendered via `grok-imagine-image-pro` ($0.07/image, highest quality).

6. **Video Generation** — Each image is animated into a 5-second cinematic video via `grok-imagine-video` (16:9, 720p). Videos are generated asynchronously with polling.

Every API call uses live search and the most advanced models available.

## Quick Start

```bash
# Set your xAI API key
export GROK_API_KEY="xai-your-key-here"

# Full run with images and videos
./market_research \
  --product "A CLI tool for developers that auto-generates API docs from code comments. Supports Rust, Go, Python. Outputs OpenAPI. $19/mo for teams, free for OSS." \
  --output report.json

# Research only (skip expensive media generation)
./market_research \
  --product "Your product idea..." \
  --skip-media \
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

Linux binaries are fully statically linked — no glibc, no OpenSSL, no runtime dependencies.

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
      --skip-media         Skip image and video generation
  -h, --help               Print help
```

### Writing a Good Product Description

The quality of your research depends on the input. Include:

- **Features** — What does it do? What's the core differentiator?
- **Target users** — Who specifically would use this?
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

### Adjusting Search Depth

- `--terms 10` — Faster, cheaper, good for quick validation
- `--terms 15` — Default, balanced coverage
- `--terms 20` — Maximum depth, broader signal capture

## Output Format

```json
{
  "product_summary": "Your input description",
  "search_terms": ["term1", "term2", "..."],
  "findings": [
    {
      "term": "freelance designer invoice automation",
      "positive_signals": ["Many freelancers express frustration with manual invoicing"],
      "negative_signals": ["Some prefer all-in-one platforms over point solutions"],
      "notable_quotes": ["@designer: 'I spend 2 hours every Friday doing invoices'"],
      "sentiment": "positive"
    }
  ],
  "synthesis": {
    "market_need_score": 7,
    "market_need_description": "Genuine frustration among freelance designers...",
    "pain_points_solved": ["Manual time-to-invoice conversion"],
    "pain_points_missed": ["Contract/proposal generation"],
    "competitive_landscape": "Crowded. FreshBooks, Harvest, Wave...",
    "pricing_feedback": "Solo freelancers resist $12/mo for invoicing alone.",
    "target_audience_fit": "Good fit for design studios.",
    "risks": ["Figma could build native invoicing"],
    "opportunities": ["No one owns the Figma→invoice pipeline yet"],
    "honest_assessment": "The pain point is real but the moat is thin..."
  },
  "media": [
    {
      "description": "Hero product shot showing the app dashboard",
      "image_prompt": "Photorealistic screenshot of a modern invoicing app...",
      "image_url": "https://...",
      "video_prompt": "Slow zoom into the dashboard with subtle UI animations...",
      "video_url": "https://..."
    }
  ]
}
```

The `media` array contains 5 entries (omitted when `--skip-media` is used). Each has:
- **description** — Human-readable concept summary
- **image_prompt** / **video_prompt** — The prompts used for generation
- **image_url** — Temporary URL to the generated image (download promptly)
- **video_url** — Temporary URL to the 5-second video (download promptly)

> ⚠️ Image and video URLs are temporary. Download them immediately after generation.

## Architecture

```
src/main.rs          Single-file Rust binary (~500 lines)
├── CLI parsing      clap with derive macros
├── Grok client      reqwest + rustls (zero OpenSSL)
│   ├── chat()             Responses API with web_search + x_search
│   ├── generate_image()   Image generation API
│   └── generate_video()   Video generation API with async polling
├── Phase 1          Term generation via structured prompts
├── Phase 2          Per-term live search + sentiment extraction
├── Phase 3          Synthesis with enforced honesty
├── Phase 4          Image prompt generation from product + synthesis
├── Phase 5          Image rendering via grok-imagine-image-pro
├── Phase 6          Video generation from images via grok-imagine-video
└── JSON output      serde_json pretty-print
```

**Key design decisions:**

- **Single binary, zero runtime deps** — No Python, no Node, no Docker.
- **rustls, not OpenSSL** — Pure Rust TLS. Static musl builds on Linux.
- **Sequential API calls** — Intentionally not parallelized for rate limit safety.
- **Honest-by-design prompts** — Synthesis explicitly penalizes cheerleading.
- **Image-to-video pipeline** — Each video is animated from its corresponding image for visual consistency.

## API Details

| Endpoint | Model | Purpose |
|----------|-------|---------|
| `POST /v1/responses` | `grok-4-0709` | Research + synthesis (with web_search + x_search tools) |
| `POST /v1/images/generations` | `grok-imagine-image-pro` | Product concept images ($0.07/image) |
| `POST /v1/videos/generations` | `grok-imagine-video` | 5-sec animated videos from images ($0.05/sec) |
| `GET /v1/videos/{request_id}` | — | Poll video generation status |

## Cost Estimation

**Research only** (`--skip-media`): `2 + N` API calls where N = number of search terms.
With `--terms 15`, that's 17 Grok API calls.

**Full run** (with media): adds 1 prompt generation call + 5 image generations + 5 video generations.
- Research: ~17 × grok-4-0709 calls
- Images: 5 × $0.07 = $0.35
- Videos: 5 × 5sec × $0.05/sec = $1.25
- **Total media cost: ~$1.60 per run**

Check [xAI pricing](https://x.ai/pricing) for current rates.

## Testing

```bash
# Unit tests (no API key needed)
cargo test -- --skip live_api

# Research test only (needs GROK_API_KEY)
GROK_API_KEY="..." cargo test --release -- live_api_generates_report --nocapture

# Full test with media (expensive, ~10 min)
GROK_API_KEY="..." cargo test --release -- live_api_generates_media --nocapture
```

CI runs lint/build on every push. Live API tests (research + media) only run on merges to main. Doc-only changes skip CI entirely.

## Contributing

1. Fork the repo
2. Create a feature branch
3. Run `cargo fmt && cargo clippy -- -D warnings && cargo test -- --skip live_api`
4. Open a PR

## License

Apache-2.0
