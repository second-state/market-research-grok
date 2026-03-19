# Market Research via Grok — X/Web Sentiment Analysis + Product Visuals

Analyze market demand for a product idea by searching X (Twitter) and the web for real user opinions, pain points, and competitive signals. Generates an honest market assessment plus 5 AI-rendered product concept images and 5 animated videos.

## Binary

- `{baseDir}/scripts/market_research` — Market research CLI.

## Prerequisites

- `GROK_API_KEY` environment variable set with a valid xAI API key.

## Usage

### Full run (research + images + videos)

```shell
GROK_API_KEY="$GROK_API_KEY" {baseDir}/scripts/market_research \
  --product "PRODUCT_DESCRIPTION" \
  --output /tmp/market-report.json
```

### Research only (skip media generation)

```shell
GROK_API_KEY="$GROK_API_KEY" {baseDir}/scripts/market_research \
  --product "PRODUCT_DESCRIPTION" \
  --skip-media \
  --output /tmp/market-report.json
```

### Parameters

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--product` | Yes | Product description: features, user profiles, price points, use cases |
| `--output` | No | Output file path (defaults to stdout) |
| `--terms` | No | Number of search terms to generate, 10-20 (default: 15) |
| `--skip-media` | No | Skip image and video generation (research only) |

### Output

A JSON report containing:

- `product_summary` — The input product description
- `search_terms` — Generated multi-dimensional search terms
- `findings[]` — Per-term analysis with positive/negative signals, quotes, and sentiment
- `synthesis` — Final assessment with market need score (1-10), pain points, risks, and opportunities
- `media[]` — 5 product concept images and 5 animated videos (unless `--skip-media`)

Each media entry contains:
- `description` — What the image/video shows
- `image_prompt` / `video_prompt` — The prompts used for generation
- `image_url` — URL to the generated product concept image
- `video_url` — URL to the 5-second animated video

> ⚠️ Image and video URLs are temporary. Download or present them immediately.

## Workflow

### 1. Gather the Product Description

Ask the user to describe their product idea. Encourage them to include:
- Key features and differentiators
- Target user profiles / personas
- Proposed price points
- Example use cases or problem scenarios

### 2. Run the Analysis

For a full analysis with visuals:
```shell
GROK_API_KEY="$GROK_API_KEY" {baseDir}/scripts/market_research \
  --product "USER_PRODUCT_DESCRIPTION" \
  --output /tmp/market-report.json
```

For quick research without media (faster, cheaper):
```shell
GROK_API_KEY="$GROK_API_KEY" {baseDir}/scripts/market_research \
  --product "USER_PRODUCT_DESCRIPTION" \
  --skip-media \
  --output /tmp/market-report.json
```

The CLI runs six phases:
1. **Term Generation** — Grok generates 10-20 search terms covering features, competitors, personas, pricing, and pain points.
2. **Market Research** — Each term is searched on X and the web via Grok with live search enabled.
3. **Synthesis** — All findings are synthesized into an honest market assessment.
4. **Image Prompts** — Grok creates 5 polished image concepts based on the product and synthesis.
5. **Image Generation** — Each concept is rendered via grok-imagine-image-pro.
6. **Video Generation** — Each image is animated into a 5-second video via grok-imagine-video.

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
- **Product Visuals** — Share the image and video URLs with the user

### 4. Download Media (Important)

Image and video URLs are temporary. If the user wants to keep them:
```shell
# Download all images and videos
cat /tmp/market-report.json | jq -r '.media[].image_url' | xargs -I{} curl -sLO {}
cat /tmp/market-report.json | jq -r '.media[].video_url' | xargs -I{} curl -sLO {}
```

## How It Works

- Uses **Grok (grok-4-0709)** for research/synthesis — most advanced model, always reasons at maximum effort
- Uses **grok-imagine-image-pro** for images — highest quality ($0.07/image)
- Uses **grok-imagine-video** for videos — 5 seconds, 16:9, 720p, image-to-video animation
- All research calls have X search, web search, and news search enabled
- The synthesis is designed to be brutally honest, not a cheerleading exercise

## Cost Per Run

- **Research only** (`--skip-media`): ~17 Grok API calls (variable with token usage)
- **Full run**: research + 5 images ($0.35) + 5 videos ($1.25) = **~$1.60 extra for media**

## Supported Platforms

- macOS Apple Silicon (aarch64)
- macOS Intel (x86_64)
- Linux x86_64 (static musl)
- Linux aarch64 (static musl)
