use coralys_ecology::models::{MemoryState, CognitionGeometry};
use std::collections::HashMap;

/// Provides historical context (Fatigue) to the MOGA Engine using Coralys Ecology
#[derive(Debug, Clone)]
pub struct WorkforceEcology {
    // Maps worker_id to their MemoryState of past workload
    worker_history: HashMap<u64, MemoryState>,
}

impl WorkforceEcology {
    pub fn new() -> Self {
        Self {
            worker_history: HashMap::new(),
        }
    }

    pub fn record_historical_hours(&mut self, worker_id: u64, hours: f64) {
        let memory = self.worker_history.entry(worker_id).or_insert_with(|| {
            MemoryState::new(CognitionGeometry::RollingBounded { window: 4 })
        });
        
        memory.buffer.push(hours);
        if hours > memory.running_max {
            memory.running_max = hours;
        }
        if memory.buffer.len() > 4 { // Keep last 4 scheduling windows
            memory.buffer.remove(0);
        }
    }

    /// Returns normalized historical fatigue in [0.0, 1.0].
    ///
    /// Fatigue is defined as the worker's mean weekly hours over the recent
    /// history window, normalized against a fixed reference of 40h/week
    /// (standard full-time contract). This produces an absolute, workforce-wide
    /// comparable signal:
    ///
    ///   0.0 = no historical workload
    ///   0.5 = worker averaged 20h/week (half-time)
    ///   1.0 = worker averaged 40h/week (full-time) or above
    ///
    /// Using a fixed reference (not running_max) ensures that two workers with
    /// different absolute loads are distinguished correctly. A worker averaging
    /// 20h/week is less fatigued than one averaging 40h/week, regardless of
    /// their individual history peaks.
    ///
    /// SC2 = fatigue * assigned_hours * 2.0 then contributes 0–80 fitness units
    /// per worker per week, comparable in magnitude to SC1 = variance * 10.0.
    pub fn get_historical_fatigue(&self, worker_id: u64) -> f64 {
        const REFERENCE_HOURS: f64 = 40.0; // standard full-time weekly hours
        if let Some(mem) = self.worker_history.get(&worker_id) {
            if mem.buffer.is_empty() {
                return 0.0;
            }
            let mean: f64 = mem.buffer.iter().sum::<f64>() / mem.buffer.len() as f64;
            (mean / REFERENCE_HOURS).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

// ── INRC Ecology Alpha-Sweep Architecture ──────────────────────────────────
//
// State (EcologyState) and Policy (EcologyPolicy) are intentionally separated.
// EcologyState records cumulative workload facts. It contains NO search logic.
// EcologyPolicy holds the alpha parameter that controls search regularization.
// GA components (factory, mutator) consume BOTH and apply the interpolation:
//
//     interpolated = neutral + alpha * (aggressive - neutral)
//
// Endpoint invariants:
//     alpha = 0.0  →  exact STATE_ONLY behavior  (neutral only)
//     alpha = 1.0  →  exact FULL_ECOLOGY behavior (aggressive only)

/// Pure state: cumulative workload tracking across scheduling weeks.
/// Contains NO bias logic, NO probability computation, NO search heuristics.
#[derive(Clone, Debug)]
pub struct EcologyState {
    pub cumulative_assignments: Vec<usize>,
    pub cumulative_weekends: Vec<usize>,
}

impl EcologyState {
    pub fn new(num_nurses: usize) -> Self {
        Self {
            cumulative_assignments: vec![0; num_nurses],
            cumulative_weekends: vec![0; num_nurses],
        }
    }

    /// Mean cumulative assignments across all nurses. Returns 0.0 if empty.
    pub fn mean_assignments(&self) -> f64 {
        if self.cumulative_assignments.is_empty() {
            return 0.0;
        }
        self.cumulative_assignments.iter().sum::<usize>() as f64
            / self.cumulative_assignments.len() as f64
    }
}

/// Policy: controls how strongly ecology memory influences search.
/// alpha ∈ [0.0, 1.0] where:
///   0.0 = no ecology influence (STATE_ONLY equivalent)
///   1.0 = full ecology influence (FULL_ECOLOGY equivalent)
#[derive(Clone, Debug)]
pub struct EcologyPolicy {
    pub alpha: f64,
}

impl EcologyPolicy {
    pub fn new(alpha: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&alpha),
            "EcologyPolicy alpha must be in [0.0, 1.0], got {}",
            alpha
        );
        Self { alpha }
    }

    /// Core interpolation: blends between neutral (no ecology) and aggressive
    /// (full ecology) values. This is the ONLY place interpolation logic lives.
    ///
    /// - alpha = 0.0 → returns neutral exactly
    /// - alpha = 1.0 → returns aggressive exactly
    pub fn interpolate(&self, neutral: f64, aggressive: f64) -> f64 {
        neutral + self.alpha * (aggressive - neutral)
    }
}
