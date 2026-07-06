use serde::{Serialize, Deserialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct InnovationTelemetry {
    pub novel_signatures_discovered: usize,
    pub novelty_ratio: f64,
    pub persistence_ratio: f64,
    pub rediscovery_ratio: f64,
    pub extinction_ratio: f64,
    pub active_memory_size: usize,
}

pub struct InnovationTracker {
    global_memory: HashSet<u64>,
    previous_step_memory: HashSet<u64>,
}

impl InnovationTracker {
    pub fn new() -> Self {
        Self {
            global_memory: HashSet::new(),
            previous_step_memory: HashSet::new(),
        }
    }

    pub fn observe(&mut self, current_signatures: &[u64]) -> InnovationTelemetry {
        let mut novel_discovered = 0;
        let mut rediscovered = 0;
        let mut persisted = 0;

        let current_set: HashSet<u64> = current_signatures.iter().copied().collect();
        let total_current = current_set.len();
        let total_previous = self.previous_step_memory.len();

        for &sig in &current_set {
            if !self.global_memory.contains(&sig) {
                novel_discovered += 1;
                self.global_memory.insert(sig);
            } else if !self.previous_step_memory.contains(&sig) {
                rediscovered += 1;
            } else {
                persisted += 1;
            }
        }

        let mut extinct = 0;
        for &sig in &self.previous_step_memory {
            if !current_set.contains(&sig) {
                extinct += 1;
            }
        }

        self.previous_step_memory = current_set;

        InnovationTelemetry {
            novel_signatures_discovered: novel_discovered,
            novelty_ratio: if total_current > 0 { novel_discovered as f64 / total_current as f64 } else { 0.0 },
            persistence_ratio: if total_previous > 0 { persisted as f64 / total_previous as f64 } else { 0.0 },
            rediscovery_ratio: if total_current > 0 { rediscovered as f64 / total_current as f64 } else { 0.0 },
            extinction_ratio: if total_previous > 0 { extinct as f64 / total_previous as f64 } else { 0.0 },
            active_memory_size: self.global_memory.len(),
        }
    }
}
impl Default for InnovationTracker {
    fn default() -> Self {
        Self::new()
    }
}
