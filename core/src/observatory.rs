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
