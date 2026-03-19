#!/bin/bash
# Bootstrap script for Market Research Grok skill
# Downloads platform-specific release binary from GitHub

set -e

REPO="second-state/market-research-grok"
SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPTS_DIR="${SKILL_DIR}/scripts"

detect_platform() {
    local os arch

    case "$(uname -s)" in
    Linux*) os="linux" ;;
    Darwin*) os="macos" ;;
    *)
        echo "Error: Unsupported operating system: $(uname -s)" >&2
        exit 1
        ;;
    esac

    case "$(uname -m)" in
    x86_64 | amd64) arch="x86_64" ;;
    aarch64 | arm64) arch="aarch64" ;;
    *)
        echo "Error: Unsupported architecture: $(uname -m)" >&2
        exit 1
        ;;
    esac

    echo "${os}-${arch}"
}

get_target() {
    local platform="$1"

    case "$platform" in
    linux-x86_64)
        echo "x86_64-unknown-linux-musl"
        ;;
    linux-aarch64)
        echo "aarch64-unknown-linux-musl"
        ;;
    macos-x86_64)
        echo "x86_64-apple-darwin"
        ;;
    macos-aarch64)
        echo "aarch64-apple-darwin"
        ;;
    *)
        echo "Error: Unsupported platform: ${platform}" >&2
        exit 1
        ;;
    esac
}

download_release() {
    local target="$1"

    echo "=== Downloading release for ${target} ===" >&2

    mkdir -p "${SCRIPTS_DIR}"

    # Get latest release tag
    local api_url="https://api.github.com/repos/${REPO}/releases/latest"
    local release_json
    release_json=$(curl -sL "$api_url")

    local tag
    tag=$(echo "$release_json" | grep -o '"tag_name": *"[^"]*"' | head -1 | sed 's/.*"tag_name": *"//;s/"//')

    if [ -z "$tag" ]; then
        echo "Error: Could not determine latest release tag." >&2
        echo "Check https://github.com/${REPO}/releases for available downloads." >&2
        exit 1
    fi

    local asset_name="market_research-${tag}-${target}.tar.gz"
    local download_url="https://github.com/${REPO}/releases/download/${tag}/${asset_name}"

    echo "Release tag: ${tag}" >&2
    echo "Fetching: ${download_url}" >&2

    local temp_dir
    temp_dir=$(mktemp -d)

    local http_code
    http_code=$(curl -sL -o "${temp_dir}/${asset_name}" -w "%{http_code}" "$download_url")

    if [ "$http_code" != "200" ]; then
        echo "Error: Download failed (HTTP ${http_code})." >&2
        echo "URL: ${download_url}" >&2
        echo "Check https://github.com/${REPO}/releases for available assets." >&2
        rm -rf "$temp_dir"
        exit 1
    fi

    echo "Extracting..." >&2
    tar xzf "${temp_dir}/${asset_name}" -C "${temp_dir}"

    cp "${temp_dir}/market_research" "${SCRIPTS_DIR}/market_research"
    chmod +x "${SCRIPTS_DIR}/market_research"

    rm -rf "$temp_dir"
    echo "Binary installed to ${SCRIPTS_DIR}/market_research" >&2
}

main() {
    local platform
    platform=$(detect_platform)
    echo "Detected platform: ${platform}" >&2

    local target
    target=$(get_target "$platform")
    echo "Target: ${target}" >&2

    # Check if already installed
    if [ -x "${SCRIPTS_DIR}/market_research" ]; then
        echo "Binary already exists at ${SCRIPTS_DIR}/market_research" >&2
        read -r -p "Re-download? [y/N] " confirm
        if [ "$confirm" != "y" ] && [ "$confirm" != "Y" ]; then
            echo "Skipping download." >&2
            exit 0
        fi
    fi

    download_release "$target"

    echo "" >&2
    echo "=== Installation complete ===" >&2
    echo "Binary: ${SCRIPTS_DIR}/market_research" >&2
    echo "" >&2
    echo "Usage:" >&2
    echo "  export GROK_API_KEY=\"xai-your-key\"" >&2
    echo "  ${SCRIPTS_DIR}/market_research --product \"Your product idea...\"" >&2
}

main "$@"
