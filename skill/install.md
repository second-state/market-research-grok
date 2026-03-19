# Market Research Grok Skill — Installation Guide

Install the Market Research skill for analyzing product-market fit via Grok + X/web search.

## Prerequisites

- `curl` (for downloading)
- `unzip` or `tar` (for extraction)
- `bash` (shell)
- A valid [xAI API key](https://console.x.ai) set as `GROK_API_KEY`

## Quick Install (Recommended)

```bash
SKILL_DIR="${HOME}/.openclaw/skills/market-research-grok"
mkdir -p "$SKILL_DIR"

# Clone the repo
git clone --depth 1 https://github.com/second-state/market-research-grok.git /tmp/mr-repo
cp -r /tmp/mr-repo/skill/* "$SKILL_DIR/"
rm -rf /tmp/mr-repo

# Download platform-specific release binary
"${SKILL_DIR}/bootstrap.sh"
```

After installation, verify it works:

```bash
export GROK_API_KEY="xai-your-key-here"
~/.openclaw/skills/market-research-grok/scripts/market_research \
  --product "A todo app for cats" \
  --terms 10
```

## Manual Installation

If the automatic download fails, manually install:

1. Go to https://github.com/second-state/market-research-grok/releases/latest
2. Download the archive for your platform:
   - `market_research-v*-aarch64-apple-darwin.tar.gz` (macOS Apple Silicon)
   - `market_research-v*-x86_64-apple-darwin.tar.gz` (macOS Intel)
   - `market_research-v*-x86_64-unknown-linux-musl.tar.gz` (Linux x86_64, static)
   - `market_research-v*-aarch64-unknown-linux-musl.tar.gz` (Linux ARM64, static)
3. Extract and install:
   ```bash
   SCRIPTS="${HOME}/.openclaw/skills/market-research-grok/scripts"
   mkdir -p "$SCRIPTS"
   tar xzf market_research-v*-<platform>.tar.gz
   cp market_research "$SCRIPTS/"
   chmod +x "$SCRIPTS/market_research"
   ```

## Environment Setup

The CLI requires a Grok API key. Set it in your shell profile or OpenClaw config:

```bash
# In ~/.zshrc or ~/.bashrc
export GROK_API_KEY="xai-your-key-here"
```

Or pass it per-invocation:

```bash
GROK_API_KEY="xai-..." ~/.openclaw/skills/market-research-grok/scripts/market_research \
  --product "..."
```

## Troubleshooting

### "GROK_API_KEY env var not set"

Ensure the environment variable is exported:

```bash
echo $GROK_API_KEY
```

### Download Failed

Check network connectivity and GitHub access:

```bash
curl -I "https://github.com/second-state/market-research-grok/releases/latest"
```

### Unsupported Platform

Check your platform:

```bash
echo "OS: $(uname -s), Arch: $(uname -m)"
```

Supported: Linux (x86_64, aarch64) and macOS (x86_64, aarch64/arm64).

## Uninstall

```bash
rm -rf ~/.openclaw/skills/market-research-grok
```
