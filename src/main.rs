use anyhow::{Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

// ── CLI ──────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "market-research",
    about = "AI-powered market research via Grok + X/web search"
)]
struct Cli {
    /// Product description: features, user profiles, price points, use cases
    #[arg(short, long)]
    product: String,

    /// Output file (defaults to stdout)
    #[arg(short, long)]
    output: Option<String>,

    /// Number of search terms to generate (10-20)
    #[arg(long, default_value_t = 15)]
    terms: u8,

    /// Skip image and video generation
    #[arg(long, default_value_t = false)]
    skip_media: bool,
}

// ── Grok Responses API types ────────────────────────────────────────────────

#[derive(Serialize)]
struct ResponsesRequest {
    model: String,
    input: Vec<InputMessage>,
    tools: Vec<Tool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Serialize, Clone)]
struct InputMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct Tool {
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Deserialize)]
struct ResponsesResponse {
    #[serde(default)]
    output: Vec<OutputItem>,
    #[serde(default)]
    error: Option<ApiError>,
}

#[derive(Deserialize)]
struct ApiError {
    message: Option<String>,
}

#[derive(Deserialize)]
struct OutputItem {
    #[serde(rename = "type")]
    _kind: Option<String>,
    #[serde(default)]
    content: Option<Vec<ContentBlock>>,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    kind: Option<String>,
    #[serde(default)]
    text: Option<String>,
}

// ── Image Generation API types ──────────────────────────────────────────────

#[derive(Serialize)]
struct ImageGenRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
struct ImageGenResponse {
    data: Vec<ImageData>,
}

#[derive(Deserialize)]
struct ImageData {
    url: String,
}

// ── Video Generation API types ──────────────────────────────────────────────

#[derive(Serialize)]
struct VideoGenRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<VideoImage>,
    duration: u32,
    aspect_ratio: String,
    resolution: String,
}

#[derive(Serialize)]
struct VideoImage {
    url: String,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Deserialize)]
struct VideoGenStartResponse {
    request_id: String,
}

#[derive(Deserialize)]
struct VideoGenPollResponse {
    status: String,
    #[serde(default)]
    video: Option<VideoResult>,
}

#[derive(Deserialize)]
struct VideoResult {
    url: String,
}

// ── Output schema ───────────────────────────────────────────────────────────

#[derive(Serialize)]
struct MarketReport {
    product_summary: String,
    search_terms: Vec<String>,
    findings: Vec<Finding>,
    synthesis: Synthesis,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    media: Vec<MediaAsset>,
}

#[derive(Serialize, Deserialize)]
struct Finding {
    term: String,
    positive_signals: Vec<String>,
    negative_signals: Vec<String>,
    notable_quotes: Vec<String>,
    sentiment: String,
}

#[derive(Serialize, Deserialize)]
struct Synthesis {
    market_need_score: u8,
    market_need_description: String,
    pain_points_solved: Vec<String>,
    pain_points_missed: Vec<String>,
    competitive_landscape: String,
    pricing_feedback: String,
    target_audience_fit: String,
    risks: Vec<String>,
    opportunities: Vec<String>,
    honest_assessment: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct MediaAsset {
    description: String,
    image_prompt: String,
    image_url: String,
    video_prompt: String,
    video_url: String,
}

// ── Grok client ─────────────────────────────────────────────────────────────

const GROK_RESPONSES_URL: &str = "https://api.x.ai/v1/responses";
const GROK_IMAGES_URL: &str = "https://api.x.ai/v1/images/generations";
const GROK_VIDEOS_URL: &str = "https://api.x.ai/v1/videos/generations";
const GROK_VIDEOS_POLL_URL: &str = "https://api.x.ai/v1/videos";
const MODEL: &str = "grok-4-0709";
const IMAGE_MODEL: &str = "grok-imagine-image-pro";
const VIDEO_MODEL: &str = "grok-imagine-video";

struct Grok {
    client: Client,
    api_key: String,
}

impl Grok {
    fn new() -> Result<Self> {
        let api_key = env::var("GROK_API_KEY").context("GROK_API_KEY env var not set")?;
        Ok(Self {
            client: Client::new(),
            api_key,
        })
    }

    async fn chat(&self, messages: Vec<InputMessage>, temp: Option<f32>) -> Result<String> {
        let req = ResponsesRequest {
            model: MODEL.to_string(),
            input: messages,
            tools: vec![
                Tool {
                    kind: "web_search".to_string(),
                },
                Tool {
                    kind: "x_search".to_string(),
                },
            ],
            temperature: temp,
        };

        let resp = self
            .client
            .post(GROK_RESPONSES_URL)
            .bearer_auth(&self.api_key)
            .json(&req)
            .send()
            .await
            .context("Grok API request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Grok API error {status}: {body}");
        }

        let resp: ResponsesResponse = resp.json().await.context("Failed to parse Grok response")?;

        if let Some(err) = resp.error {
            anyhow::bail!(
                "Grok API error: {}",
                err.message.unwrap_or_else(|| "unknown".to_string())
            );
        }

        extract_text_from_output(&resp.output)
    }

    async fn generate_image(&self, prompt: &str) -> Result<String> {
        let req = ImageGenRequest {
            model: IMAGE_MODEL.to_string(),
            prompt: prompt.to_string(),
        };

        let resp = self
            .client
            .post(GROK_IMAGES_URL)
            .bearer_auth(&self.api_key)
            .json(&req)
            .send()
            .await
            .context("Image generation request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Image generation error {status}: {body}");
        }

        let resp: ImageGenResponse = resp
            .json()
            .await
            .context("Failed to parse image generation response")?;

        resp.data
            .first()
            .map(|d| d.url.clone())
            .context("No image URL in response")
    }

    async fn generate_video(&self, prompt: &str, image_url: Option<&str>) -> Result<String> {
        let req = VideoGenRequest {
            model: VIDEO_MODEL.to_string(),
            prompt: prompt.to_string(),
            image: image_url.map(|url| VideoImage {
                url: url.to_string(),
                kind: "image_url".to_string(),
            }),
            duration: 5,
            aspect_ratio: "16:9".to_string(),
            resolution: "720p".to_string(),
        };

        let resp = self
            .client
            .post(GROK_VIDEOS_URL)
            .bearer_auth(&self.api_key)
            .json(&req)
            .send()
            .await
            .context("Video generation request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Video generation error {status}: {body}");
        }

        let start: VideoGenStartResponse = resp
            .json()
            .await
            .context("Failed to parse video generation start response")?;

        // Poll for completion
        let poll_url = format!("{}/{}", GROK_VIDEOS_POLL_URL, start.request_id);
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            let resp = self
                .client
                .get(&poll_url)
                .bearer_auth(&self.api_key)
                .send()
                .await
                .context("Video poll request failed")?;

            let status = resp.status();
            if !status.is_success() {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("Video poll error {status}: {body}");
            }

            let poll: VideoGenPollResponse = resp
                .json()
                .await
                .context("Failed to parse video poll response")?;

            match poll.status.as_str() {
                "done" => {
                    return poll
                        .video
                        .map(|v| v.url)
                        .context("Video done but no URL returned");
                }
                "failed" => anyhow::bail!("Video generation failed"),
                "expired" => anyhow::bail!("Video generation request expired"),
                _ => continue, // "pending" — keep polling
            }
        }
    }
}

/// Walk the output array and concatenate all text content from message items.
fn extract_text_from_output(output: &[OutputItem]) -> Result<String> {
    let mut text_parts: Vec<String> = Vec::new();

    for item in output {
        if let Some(text) = &item.text {
            text_parts.push(text.clone());
        }

        if let Some(content) = &item.content {
            for block in content {
                if (block.kind.as_deref() == Some("output_text") || block.kind.is_none())
                    && block.text.is_some()
                {
                    text_parts.push(block.text.clone().unwrap());
                }
            }
        }
    }

    if text_parts.is_empty() {
        anyhow::bail!("No text content in Grok response");
    }

    Ok(text_parts.join(""))
}

// ── Phase 1: Generate search terms ──────────────────────────────────────────

async fn generate_terms(grok: &Grok, product: &str, count: u8) -> Result<Vec<String>> {
    let prompt = format!(
        r#"You are a market research analyst. Given the following product description, generate exactly {count} search terms/phrases that would help discover real user opinions, pain points, competitor mentions, and market sentiment on X (Twitter) and the web.

Cover multiple dimensions:
- Core features and their alternatives
- Target user personas and their language
- Price sensitivity / willingness to pay
- Competitor products and comparisons
- Problem domains the product addresses
- Emotional triggers (frustration, delight)

Product description:
---
{product}
---

Return ONLY a JSON array of strings. No markdown, no explanation. Example:
["term one", "term two", ...]"#
    );

    let messages = vec![InputMessage {
        role: "user".to_string(),
        content: prompt,
    }];

    let raw = grok.chat(messages, Some(0.7)).await?;
    let json_str = extract_json_array(&raw)?;
    let terms: Vec<String> =
        serde_json::from_str(&json_str).context("Failed to parse search terms JSON")?;

    Ok(terms)
}

// ── Phase 2: Research each term ─────────────────────────────────────────────

async fn research_term(grok: &Grok, term: &str, product: &str) -> Result<Finding> {
    let prompt = format!(
        r#"You are conducting market research. Search X (Twitter) and the web for real opinions related to: "{term}"

Context — this is about the following product:
---
{product}
---

Analyze what real people are saying. Extract:
1. **positive_signals**: Real positive comments, desires, or enthusiasm (quote or paraphrase actual posts/articles)
2. **negative_signals**: Real complaints, skepticism, or concerns
3. **notable_quotes**: Direct quotes or close paraphrases from X posts or web sources (max 5)
4. **sentiment**: Overall sentiment for this term — one of: "positive", "negative", "mixed", "neutral"

Be honest. If there's no real signal, say so. Don't fabricate sentiment.

Return ONLY valid JSON (no markdown) matching this schema:
{{
  "term": "{term}",
  "positive_signals": ["..."],
  "negative_signals": ["..."],
  "notable_quotes": ["..."],
  "sentiment": "mixed"
}}"#
    );

    let messages = vec![InputMessage {
        role: "user".to_string(),
        content: prompt,
    }];

    let raw = grok.chat(messages, Some(0.3)).await?;
    let json_str = extract_json_object(&raw)?;
    let finding: Finding = serde_json::from_str(&json_str)
        .with_context(|| format!("Failed to parse finding for term: {term}"))?;

    Ok(finding)
}

// ── Phase 3: Synthesize ─────────────────────────────────────────────────────

async fn synthesize(grok: &Grok, product: &str, findings: &[Finding]) -> Result<Synthesis> {
    let findings_json = serde_json::to_string_pretty(findings)?;

    let prompt = format!(
        r#"You are a brutally honest market research analyst. Given a product description and real market research findings from X and the web, produce a final synthesis.

Product:
---
{product}
---

Research findings:
---
{findings_json}
---

Produce an honest, unvarnished assessment. No cheerleading. Include:
- market_need_score (1-10, where 10 = desperate unmet need)
- market_need_description (2-3 sentences)
- pain_points_solved (list of real problems this addresses)
- pain_points_missed (problems users have that this doesn't solve)
- competitive_landscape (who else is here, how crowded)
- pricing_feedback (what people say about pricing in this space)
- target_audience_fit (does the described audience actually want this?)
- risks (honest risks to building this)
- opportunities (genuine white space found)
- honest_assessment (3-5 sentence frank assessment — would you invest?)

Return ONLY valid JSON (no markdown). Schema:
{{
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
}}"#
    );

    let messages = vec![InputMessage {
        role: "user".to_string(),
        content: prompt,
    }];

    let raw = grok.chat(messages, Some(0.3)).await?;
    let json_str = extract_json_object(&raw)?;
    let synthesis: Synthesis =
        serde_json::from_str(&json_str).context("Failed to parse synthesis JSON")?;

    Ok(synthesis)
}

// ── Phase 4: Generate image prompts ─────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct ImagePromptSet {
    prompts: Vec<ImagePromptEntry>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ImagePromptEntry {
    description: String,
    image_prompt: String,
    video_prompt: String,
}

async fn generate_image_prompts(
    grok: &Grok,
    product: &str,
    synthesis: &Synthesis,
) -> Result<Vec<ImagePromptEntry>> {
    let prompt = format!(
        r#"You are a creative director for a product marketing team. Based on the product description and market research synthesis below, create exactly 5 compelling image concepts for this hypothetical product.

For each concept, provide:
1. "description" — A short human-readable description of what the image shows (1 sentence)
2. "image_prompt" — A detailed, polished prompt for an AI image generator. Be specific about style, composition, lighting, colors, and mood. Make it photorealistic or high-quality illustration style.
3. "video_prompt" — A short prompt describing a 5-second animation or motion based on the image. Focus on subtle, cinematic movement (camera pan, zoom, parallax, element animation).

Cover diverse angles:
- Hero product shot / UI mockup
- Target user in their environment using the product
- Problem visualization (the pain point being solved)
- Before/after or transformation scene
- Aspirational outcome / success state

Product description:
---
{product}
---

Market research synthesis:
---
Market need: {need}
Pain points solved: {pain}
Target audience: {audience}
---

Return ONLY valid JSON (no markdown):
{{
  "prompts": [
    {{
      "description": "...",
      "image_prompt": "...",
      "video_prompt": "..."
    }}
  ]
}}"#,
        need = synthesis.market_need_description,
        pain = synthesis.pain_points_solved.join(", "),
        audience = synthesis.target_audience_fit,
    );

    let messages = vec![InputMessage {
        role: "user".to_string(),
        content: prompt,
    }];

    let raw = grok.chat(messages, Some(0.7)).await?;
    let json_str = extract_json_object(&raw)?;
    let set: ImagePromptSet =
        serde_json::from_str(&json_str).context("Failed to parse image prompts JSON")?;

    Ok(set.prompts)
}

// ── Phase 5 & 6: Generate images and videos ─────────────────────────────────

async fn generate_media(grok: &Grok, prompts: &[ImagePromptEntry]) -> Vec<MediaAsset> {
    let mut assets = Vec::new();
    let pb = ProgressBar::new(prompts.len() as u64 * 2); // images + videos
    pb.set_style(
        ProgressStyle::default_bar()
            .template("   [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓░"),
    );

    for (i, entry) in prompts.iter().enumerate() {
        // Generate image
        pb.set_message(format!("image {}/5", i + 1));
        let image_url = match grok.generate_image(&entry.image_prompt).await {
            Ok(url) => url,
            Err(e) => {
                eprintln!("\n   ⚠️  Image {}: {e}", i + 1);
                pb.inc(2);
                continue;
            }
        };
        pb.inc(1);

        // Generate video from image
        pb.set_message(format!("video {}/5", i + 1));
        let video_url = match grok
            .generate_video(&entry.video_prompt, Some(&image_url))
            .await
        {
            Ok(url) => url,
            Err(e) => {
                eprintln!("\n   ⚠️  Video {}: {e}", i + 1);
                pb.inc(1);
                // Still include the image even if video fails
                assets.push(MediaAsset {
                    description: entry.description.clone(),
                    image_prompt: entry.image_prompt.clone(),
                    image_url,
                    video_prompt: entry.video_prompt.clone(),
                    video_url: String::new(),
                });
                continue;
            }
        };
        pb.inc(1);

        assets.push(MediaAsset {
            description: entry.description.clone(),
            image_prompt: entry.image_prompt.clone(),
            image_url,
            video_prompt: entry.video_prompt.clone(),
            video_url,
        });
    }
    pb.finish_with_message("done");

    assets
}

// ── JSON extraction helpers ─────────────────────────────────────────────────

fn extract_json_array(raw: &str) -> Result<String> {
    if let (Some(start), Some(end)) = (raw.find('['), raw.rfind(']')) {
        return Ok(raw[start..=end].to_string());
    }
    anyhow::bail!(
        "No JSON array found in response: {}",
        &raw[..raw.len().min(200)]
    )
}

fn extract_json_object(raw: &str) -> Result<String> {
    if let (Some(start), Some(end)) = (raw.find('{'), raw.rfind('}')) {
        return Ok(raw[start..=end].to_string());
    }
    anyhow::bail!(
        "No JSON object found in response: {}",
        &raw[..raw.len().min(200)]
    )
}

// ── Main ────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let grok = Grok::new()?;

    // Phase 1: Generate search terms
    eprintln!("🔍 Phase 1: Generating search terms...");
    let terms = generate_terms(&grok, &cli.product, cli.terms).await?;
    eprintln!("   Generated {} terms", terms.len());
    for (i, t) in terms.iter().enumerate() {
        eprintln!("   {:2}. {}", i + 1, t);
    }

    // Phase 2: Research each term
    eprintln!("\n📊 Phase 2: Researching market signals...");
    let pb = ProgressBar::new(terms.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("   [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓░"),
    );

    let mut findings = Vec::new();
    for term in &terms {
        pb.set_message(format!(
            "\"{}\"",
            if term.len() > 30 { &term[..30] } else { term }
        ));
        match research_term(&grok, term, &cli.product).await {
            Ok(finding) => findings.push(finding),
            Err(e) => eprintln!("\n   ⚠️  Skipping \"{term}\": {e}"),
        }
        pb.inc(1);
    }
    pb.finish_with_message("done");

    // Phase 3: Synthesize
    eprintln!("\n🧠 Phase 3: Synthesizing findings...");
    let synthesis = synthesize(&grok, &cli.product, &findings).await?;

    // Phase 4-6: Media generation (unless skipped)
    let media = if cli.skip_media {
        eprintln!("\n⏭️  Skipping media generation (--skip-media)");
        Vec::new()
    } else {
        eprintln!("\n🎨 Phase 4: Generating image & video prompts...");
        let prompts = generate_image_prompts(&grok, &cli.product, &synthesis).await?;
        eprintln!("   Created {} concept briefs", prompts.len());
        for (i, p) in prompts.iter().enumerate() {
            eprintln!("   {:2}. {}", i + 1, p.description);
        }

        eprintln!("\n🖼️  Phase 5-6: Generating images & videos...");
        generate_media(&grok, &prompts).await
    };

    // Build report
    let report = MarketReport {
        product_summary: cli.product.clone(),
        search_terms: terms,
        findings,
        synthesis,
        media,
    };

    let json = serde_json::to_string_pretty(&report)?;

    // Output
    if let Some(path) = &cli.output {
        std::fs::write(path, &json)?;
        eprintln!("\n✅ Report written to {path}");
    } else {
        println!("{json}");
    }

    Ok(())
}
