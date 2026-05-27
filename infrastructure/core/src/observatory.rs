use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservatoryManifestV1 {
    pub replay_version: String,
    pub topology_version: String,
    pub cognition_version: String,
    pub commit_hash: String,
    pub artifact_hash: String,
    pub generation_timestamp: u64,
    pub chronology_bounds: ChronologyBounds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronologyBounds {
    pub start_tick: u64,
    pub end_tick: u64,
    pub total_ticks: usize,
}

/// Resolve the current git HEAD for observatory provenance (V-010).
/// Falls back to `"unknown"` when git is unavailable — never a placeholder literal.
pub fn resolve_git_commit_hash() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}
