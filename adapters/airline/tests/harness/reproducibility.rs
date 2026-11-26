//! WP7 — Reproducibility Layer
//!
//! Captures the environment at experiment runtime so any published result can
//! be independently reproduced. Written to `reproducibility.json` alongside
//! `experiment.json`.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Snapshot of the environment at experiment runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReproducibilityInfo {
    /// Rust compiler version (from `rustc --version`).
    pub rust_version: String,
    /// Cargo version (from `cargo --version`).
    pub cargo_version: String,
    /// Compiler profile: "debug" or "release".
    pub compiler_profile: String,
    /// Git commit hash (short), or "unknown" if not in a git repo.
    pub git_commit: String,
    /// Git branch name, or "unknown".
    pub git_branch: String,
    /// Operating system (e.g. "macos", "linux").
    pub os: String,
    /// Random seed used for this run.
    pub random_seed: u64,
    /// SHA-256 hex digest of the benchmark flights.csv, or "unknown".
    pub benchmark_checksum: String,
    /// ISO 8601 timestamp when this info was captured.
    pub captured_at_utc: String,
}

impl ReproducibilityInfo {
    /// Capture reproducibility info at runtime.
    ///
    /// `benchmark_flights_path` is the path to the flights.csv for the instance
    /// being run. Pass `None` to skip checksum computation.
    pub fn capture(seed: u64, benchmark_flights_path: Option<&Path>) -> Self {
        let rust_version = run_command("rustc", &["--version"])
            .unwrap_or_else(|| "unknown".to_string());
        let cargo_version = run_command("cargo", &["--version"])
            .unwrap_or_else(|| "unknown".to_string());
        let git_commit = run_command("git", &["rev-parse", "--short", "HEAD"])
            .unwrap_or_else(|| "unknown".to_string());
        let git_branch = run_command("git", &["rev-parse", "--abbrev-ref", "HEAD"])
            .unwrap_or_else(|| "unknown".to_string());

        let compiler_profile = if cfg!(debug_assertions) {
            "debug".to_string()
        } else {
            "release".to_string()
        };

        let os = std::env::consts::OS.to_string();

        let benchmark_checksum = benchmark_flights_path
            .and_then(|p| sha256_file(p).ok())
            .unwrap_or_else(|| "unknown".to_string());

        let captured_at_utc = chrono::Utc::now().to_rfc3339();

        Self {
            rust_version: rust_version.trim().to_string(),
            cargo_version: cargo_version.trim().to_string(),
            compiler_profile,
            git_commit: git_commit.trim().to_string(),
            git_branch: git_branch.trim().to_string(),
            os,
            random_seed: seed,
            benchmark_checksum,
            captured_at_utc,
        }
    }

    /// Write to `reproducibility.json` in the given directory.
    pub fn write_to(&self, dir: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .expect("ReproducibilityInfo must be serializable");
        std::fs::write(dir.join("reproducibility.json"), json)?;
        Ok(())
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Run a command and return its stdout as a String, or None on failure.
fn run_command(cmd: &str, args: &[&str]) -> Option<String> {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
}

/// Compute SHA-256 hex digest of a file.
fn sha256_file(path: &Path) -> std::io::Result<String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    // Simple FNV-1a 64-bit hash as a lightweight stand-in.
    // Replace with sha2 crate if cryptographic strength is needed.
    let mut hash: u64 = 14695981039346656037;
    for byte in &buf {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    Ok(format!("fnv1a64:{hash:016x}"))
}