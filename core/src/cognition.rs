use serde::{Serialize, Deserialize};

/// Defines the deterministic state evolution constraints for a chronological container.
/// This strictly models memory physics, not alpha generation or strategy logic.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CognitionGeometry {
    RollingBounded { window: usize },
    EventReset { drop_threshold_pct: f64 },
    Accumulator,
}

/// A deterministic state container that evolves under a specific CognitionGeometry.
#[derive(Debug, Clone)]
pub struct MemoryState {
    pub geometry: CognitionGeometry,
    pub buffer: Vec<f64>,
    pub running_max: f64,
}

impl MemoryState {
    pub fn new(geometry: CognitionGeometry) -> Self {
        Self {
            geometry,
            buffer: Vec::new(),
            running_max: 0.0,
        }
    }

    /// Progresses the memory state by integrating a single chronological observation.
    pub fn ingest(&mut self, value: f64) {
        // Update running maximum for event-reset logic
        if self.buffer.is_empty() || value > self.running_max {
            self.running_max = value;
        }

        match self.geometry {
            CognitionGeometry::RollingBounded { window } => {
                self.buffer.push(value);
                if self.buffer.len() > window {
                    self.buffer.remove(0);
                }
            }
            CognitionGeometry::EventReset { drop_threshold_pct } => {
                // If the value drops below the threshold percentage of the running max, the memory violently flushes.
                let threshold_val = self.running_max * (1.0 - drop_threshold_pct);
                if !self.buffer.is_empty() && value < threshold_val {
                    self.buffer.clear();
                    self.running_max = value;
                }
                self.buffer.push(value);
            }
            CognitionGeometry::Accumulator => {
                // Infinite state accumulation. Memory is never forgotten or purged.
                self.buffer.push(value);
            }
        }
    }

    /// Computes the overlap ratio between this fragmented state and a canonical baseline state.
    /// This is the core raw trace of chronological deformation.
    pub fn overlap_ratio(&self, baseline: &MemoryState) -> f64 {
        if baseline.buffer.is_empty() {
            return 0.0;
        }
        
        // We align the buffers from the most recent tick (right-aligned).
        let base_len = baseline.buffer.len();
        let frag_len = self.buffer.len();
        
        let mut overlap_count = 0;
        
        for i in 0..base_len {
            let base_val = baseline.buffer[base_len - 1 - i];
            // If the fragmented buffer is shorter, we assume 0.0 (or default) for missing historical states.
            let frag_val = if i < frag_len {
                self.buffer[frag_len - 1 - i]
            } else {
                // Missing state due to cognitive boundary (e.g. event reset purged it)
                // If base has a value but frag is completely empty at this index, they don't overlap.
                // Assuming prices are > 0, 0.0 will never match a valid price.
                0.0 
            };
            
            // Exact floating-point equivalence for chronological traces 
            // (safe here as they are deterministic copies of the same market telemetry)
            if (base_val - frag_val).abs() < f64::EPSILON {
                overlap_count += 1;
            }
        }
        
        overlap_count as f64 / base_len as f64
    }
}
