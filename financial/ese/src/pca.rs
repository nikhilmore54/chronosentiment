use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct PcaWeights {
    pub mean: Vec<f64>,
    pub std: Vec<f64>,
    pub pc1_vector: Vec<f64>,
    pub pc2_vector: Vec<f64>,
    pub centroids: Vec<[f64; 2]>,
}

impl PcaWeights {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("read pca weights {}", path.display()))?;
        serde_json::from_str(&raw).context("parse pca weights")
    }

    pub fn project_and_classify(&self, features: &[f64; 5]) -> (f64, f64, usize, f64) {
        let norm: Vec<f64> = features
            .iter()
            .enumerate()
            .map(|(i, f)| (f - self.mean[i]) / self.std[i])
            .collect();
        let pc1: f64 = norm
            .iter()
            .enumerate()
            .map(|(i, n)| n * self.pc1_vector[i])
            .sum();
        let pc2: f64 = norm
            .iter()
            .enumerate()
            .map(|(i, n)| n * self.pc2_vector[i])
            .sum();
        let mut best_id = 0usize;
        let mut best_dist = f64::MAX;
        for (i, c) in self.centroids.iter().enumerate() {
            let dx = pc1 - c[0];
            let dy = pc2 - c[1];
            let d = (dx * dx + dy * dy).sqrt();
            if d < best_dist {
                best_dist = d;
                best_id = i;
            }
        }
        (pc1, pc2, best_id, best_dist)
    }
}

pub const STATE_NAMES: [&str; 3] = [
    "LIQUIDITY_EXHAUSTION",
    "NARRATIVE_PERSISTENCE",
    "NOISE_TRANSITIONAL",
];
