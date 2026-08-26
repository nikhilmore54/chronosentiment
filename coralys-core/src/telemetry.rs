use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchTelemetry {
    pub generation: usize,
    pub population: PopulationTelemetry,
    pub attachments: Vec<TelemetryAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PopulationTelemetry {
    pub best_fitness: f64,
    pub median_fitness: f64,
    pub worst_fitness: f64,
    pub diversity_score: f64,
    pub elite_diversity_score: f64,
    pub feasible_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TelemetryAttachment {
    pub namespace: String,
    pub version: u32,
    pub payload: serde_json::Value,
}
