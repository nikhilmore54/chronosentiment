//! # observer
//!
//! Canonical external observer for INRC schedules.
//!
//! ## Canonical Formula (validated in Sprint 3.5D)
//!
//! ```text
//! official_total = hc_coverage + hc_skills + hc_one_shift_per_day
//!                + hc_forbidden_successions + soft_total
//! ```
//!
//! All HC components are already penalty-weighted by the INRC evaluator
//! (each violation × 1000 internally). No additional multiplier is applied here.
//!
//! This formula is the one used by `InrcExternalScorer::score_with_components()`
//! in `validation_pass.rs` (line 70), which produced:
//!   - Soft alignment Spearman ≈ +0.80 to +0.85
//!   - Best Ever External ≈ 37k–43k
//!   - Champion Retention Error = 1
//!
//! Both `validation_pass.rs` and `inrc_archive_forensics.rs` must call
//! `score_inrc_official()` from this module. No other scoring formula
//! for `observer_id = "inrc_official_total"` is permitted.
//!
//! ## Observer ID
//! ```ignore
//! pub const OBSERVER_ID: &str = "inrc_official_total";
//! ```

use crate::inrc::models::InrcScenario;
use crate::inrc::optimization::{InrcGenome, InrcOptimizer};
use crate::inrc::schedule_optimizer::ScheduleGenome;
use coralys_moga::traits::FitnessEvaluator;
use std::collections::HashMap;

/// The canonical observer ID for the INRC official total metric.
pub const OBSERVER_ID: &str = "inrc_official_total";

/// Decomposed INRC score components.
#[derive(Debug, Clone)]
pub struct InrcScoreComponents {
    /// HC1: Coverage violations (penalty-weighted, ×1000 per violation in evaluator)
    pub hc_coverage: usize,
    /// HC2: Skills violations (penalty-weighted)
    pub hc_skills: usize,
    /// HC3: One-shift-per-day violations (penalty-weighted)
    pub hc_one_shift_per_day: usize,
    /// HC4: Forbidden succession violations (penalty-weighted)
    pub hc_forbidden_successions: usize,
    /// Sum of all HC penalty components
    pub total_hc_penalty: usize,
    /// Soft constraint total penalty
    pub soft_total: i32,
    /// Canonical official total = total_hc_penalty + soft_total
    pub official_total: f64,
    /// True iff total_hc_penalty == 0
    pub feasible: bool,
}

/// Convert a `ScheduleGenome` to an `InrcGenome` for evaluation.
pub fn to_inrc_genome(genome: &ScheduleGenome, scenario: &InrcScenario) -> InrcGenome {
    let num_nurses = scenario.nurses.len();
    let num_days = scenario.number_of_weeks * 7;
    let num_shift_types = scenario.shift_types.len();
    let size = num_nurses * num_days * num_shift_types;
    let mut bits = vec![false; size];

    let nurse_map: HashMap<String, usize> = scenario
        .nurses
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.clone(), i))
        .collect();
    let shift_map: HashMap<String, usize> = scenario
        .shift_types
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id.clone(), i))
        .collect();

    for slot in &genome.slots {
        if slot.assigned_nurse.is_empty() {
            continue;
        }
        if let Some(&nurse_idx) = nurse_map.get(&slot.assigned_nurse) {
            if let Some(&shift_idx) = shift_map.get(&slot.shift_type) {
                let index = nurse_idx * (num_days * num_shift_types)
                    + slot.day * num_shift_types
                    + shift_idx;
                bits[index] = true;
            }
        }
    }
    InrcGenome { bits }
}

/// Compute the canonical INRC official total score and its decomposition.
///
/// Formula (canonical — do not change without updating forensics_manifest.txt):
///   official_total = hc_coverage + hc_skills + hc_one_shift_per_day
///                  + hc_forbidden_successions + soft_total
///
/// This matches `InrcExternalScorer::score_with_components()` line 70 in
/// `validation_pass.rs`, which is the source of the validated SD-003 observation.
pub fn score_inrc_official(
    genome: &ScheduleGenome,
    scenario: &InrcScenario,
    inrc_optimizer: &InrcOptimizer,
) -> InrcScoreComponents {
    let i_genome = to_inrc_genome(genome, scenario);
    let empty_metrics = coralys_moga::runtime::optimization::metric::MetricReport::default();
    let ev = inrc_optimizer.evaluate(&i_genome, &empty_metrics);
    let total_hc =
        ev.hc_coverage + ev.hc_skills + ev.hc_one_shift_per_day + ev.hc_forbidden_successions;
    // Canonical formula: no additional multiplier.
    // HC components are already penalty-weighted (×1000) by the evaluator.
    let official_total = (total_hc as i64 + ev.soft_report.total_penalty as i64) as f64;
    let feasible = total_hc == 0;
    InrcScoreComponents {
        hc_coverage: ev.hc_coverage,
        hc_skills: ev.hc_skills,
        hc_one_shift_per_day: ev.hc_one_shift_per_day,
        hc_forbidden_successions: ev.hc_forbidden_successions,
        total_hc_penalty: total_hc,
        soft_total: ev.soft_report.total_penalty,
        official_total,
        feasible,
    }
}
