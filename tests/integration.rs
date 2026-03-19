//! Integration tests — require GROK_API_KEY to be set.
//! Skipped gracefully if the key is missing.

use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_market_research"))
}

fn has_api_key() -> bool {
    std::env::var("GROK_API_KEY").is_ok()
}

#[test]
fn missing_api_key_returns_error() {
    let out = Command::new(env!("CARGO_BIN_EXE_market_research"))
        .env_remove("GROK_API_KEY")
        .args(["--product", "test product"])
        .output()
        .expect("failed to run binary");

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("GROK_API_KEY"),
        "should mention missing env var, got: {stderr}"
    );
}

#[test]
fn help_flag_works() {
    let out = bin().arg("--help").output().expect("failed to run binary");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("market-research"));
    assert!(stdout.contains("--product"));
}

#[test]
fn product_flag_is_required() {
    let out = bin().output().expect("failed to run binary");
    assert!(!out.status.success());
}

/// Live API test — generates search terms for a simple product.
/// Only runs when GROK_API_KEY is available (CI or local dev).
#[test]
fn live_api_generates_report() {
    if !has_api_key() {
        eprintln!("GROK_API_KEY not set — skipping live API test");
        return;
    }

    let out = bin()
        .args([
            "--product",
            "A simple CLI tool that converts markdown files to PDF. Target: developers. Price: free and open source.",
            "--terms",
            "3", // keep it small for CI cost/time
        ])
        .output()
        .expect("failed to run binary");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        out.status.success(),
        "binary failed.\nstdout: {stdout}\nstderr: {stderr}"
    );

    // Should be valid JSON
    let report: serde_json::Value =
        serde_json::from_str(&stdout).expect("output is not valid JSON");

    // Verify top-level keys
    assert!(report.get("product_summary").is_some());
    assert!(report.get("search_terms").is_some());
    assert!(report.get("findings").is_some());
    assert!(report.get("synthesis").is_some());

    // search_terms should be a non-empty array
    let terms = report["search_terms"].as_array().expect("search_terms is array");
    assert!(!terms.is_empty(), "should have generated search terms");

    // synthesis should have market_need_score
    let score = report["synthesis"]["market_need_score"]
        .as_u64()
        .expect("market_need_score is a number");
    assert!((1..=10).contains(&score), "score should be 1-10, got {score}");
}
