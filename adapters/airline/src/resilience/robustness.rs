//! Robustness scoring for crew rosters.
//!
//! A [`RobustnessScore`] quantifies how well a roster can absorb disruptions
//! without requiring re-planning.  [`RobustnessEvaluator`] computes the score
//! by examining buffer times, crew slack, and pairing density.
//!
//! # Metrics
//! - **Rest buffer ratio**: fraction of rest periods that exceed the minimum
//!   rest requirement by at least `buffer_threshold` minutes.
//! - **Crew slack ratio**: fraction of rotations that have fewer than
//!   `max_pairings_per_rotation` pairings (i.e. room to absorb extra work).
//! - **Overall score**: weighted average of the two ratios, in [0.0, 1.0].
//!   Higher is more robust.

use crate::domain::roster::Roster;

// ── Robustness score ──────────────────────────────────────────────────────────

/// A robustness score for a roster, in [0.0, 1.0].
#[derive(Debug, Clone, PartialEq)]
pub struct RobustnessScore {
    /// Fraction of rest periods with adequate buffer (≥ threshold above minimum).
    pub rest_buffer_ratio: f64,
    /// Fraction of rotations with spare capacity.
    pub crew_slack_ratio: f64,
    /// Weighted overall score.
    pub overall: f64,
}

impl RobustnessScore {
    fn new(rest_buffer_ratio: f64, crew_slack_ratio: f64, rest_weight: f64) -> Self {
        let slack_weight = 1.0 - rest_weight;
        let overall = rest_buffer_ratio * rest_weight + crew_slack_ratio * slack_weight;
        Self { rest_buffer_ratio, crew_slack_ratio, overall }
    }
}

// ── Evaluator ─────────────────────────────────────────────────────────────────

/// Computes a [`RobustnessScore`] for a roster.
pub struct RobustnessEvaluator {
    /// Minimum rest period in minutes (below which a violation would occur).
    min_rest_minutes: f64,
    /// Additional buffer above the minimum that counts as "robust" rest.
    buffer_threshold_minutes: f64,
    /// Maximum pairings per rotation before it is considered "full".
    max_pairings_per_rotation: usize,
    /// Weight given to the rest buffer ratio (crew slack gets 1 - rest_weight).
    rest_weight: f64,
}

impl RobustnessEvaluator {
    /// Create a new evaluator with the given parameters.
    ///
    /// # Parameters
    /// - `min_rest_minutes`: regulatory minimum rest (e.g. 480 for 8 h).
    /// - `buffer_threshold_minutes`: extra rest above minimum to count as buffered.
    /// - `max_pairings_per_rotation`: rotations with fewer pairings have slack.
    /// - `rest_weight`: weight for rest buffer ratio in [0.0, 1.0].
    pub fn new(
        min_rest_minutes: f64,
        buffer_threshold_minutes: f64,
        max_pairings_per_rotation: usize,
        rest_weight: f64,
    ) -> Self {
        assert!((0.0..=1.0).contains(&rest_weight), "rest_weight must be in [0, 1]");
        Self {
            min_rest_minutes,
            buffer_threshold_minutes,
            max_pairings_per_rotation,
            rest_weight,
        }
    }

    /// Compute the robustness score for `roster`.
    pub fn evaluate(&self, roster: &Roster) -> RobustnessScore {
        let rest_buffer_ratio = self.compute_rest_buffer_ratio(roster);
        let crew_slack_ratio = self.compute_crew_slack_ratio(roster);
        RobustnessScore::new(rest_buffer_ratio, crew_slack_ratio, self.rest_weight)
    }

    fn compute_rest_buffer_ratio(&self, roster: &Roster) -> f64 {
        let mut total_rests = 0usize;
        let mut buffered_rests = 0usize;
        let threshold = self.min_rest_minutes + self.buffer_threshold_minutes;

        for rotation in roster.rotations() {
            for pairing in rotation.pairings().iter() {
                let duties = pairing.duties();
                for window in duties.windows(2) {
                    let rest_start = window[0].end();
                    let rest_end = window[1].start();
                    let rest_minutes = (rest_end - rest_start).num_minutes() as f64;
                    total_rests += 1;
                    if rest_minutes >= threshold {
                        buffered_rests += 1;
                    }
                }
            }
        }

        if total_rests == 0 {
            1.0 // no rest periods to evaluate → trivially robust
        } else {
            buffered_rests as f64 / total_rests as f64
        }
    }

    fn compute_crew_slack_ratio(&self, roster: &Roster) -> f64 {
        let rotations: Vec<_> = roster.rotations().collect();
        if rotations.is_empty() {
            return 1.0;
        }
        let slack_count = rotations
            .iter()
            .filter(|r| r.pairings().len() < self.max_pairings_per_rotation)
            .count();
        slack_count as f64 / rotations.len() as f64
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legality::test_helpers::*;

    fn make_evaluator() -> RobustnessEvaluator {
        RobustnessEvaluator::new(
            480.0,  // 8 h minimum rest
            60.0,   // 1 h buffer threshold
            3,      // max 3 pairings per rotation
            0.5,    // equal weights
        )
    }

    #[test]
    fn empty_roster_scores_one() {
        let roster = make_roster(vec![], vec![]);
        let evaluator = make_evaluator();
        let score = evaluator.evaluate(&roster);
        assert_eq!(score.rest_buffer_ratio, 1.0);
        assert_eq!(score.crew_slack_ratio, 1.0);
        assert_eq!(score.overall, 1.0);
    }

    #[test]
    fn single_duty_pairing_has_no_rest_periods() {
        // A pairing with one duty has no inter-duty rest → rest_buffer_ratio = 1.0.
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 8, 10)]);
        let d2 = make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 22, 24)]);
        let p1 = make_pairing("P1", "LHR", vec![d1, d2]);
        let r1 = make_rotation("R1", "C1", vec![p1]);
        let roster = make_roster(vec![], vec![r1]);

        let evaluator = make_evaluator();
        let score = evaluator.evaluate(&roster);
        // One rest period between D1 and D2: 22h - 10h = 12h = 720 min.
        // Threshold = 480 + 60 = 540 min.  720 >= 540 → buffered.
        assert_eq!(score.rest_buffer_ratio, 1.0);
    }

    #[test]
    fn short_rest_reduces_buffer_ratio() {
        // Rest of only 30 min — well below threshold.
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 8, 10)]);
        let d2 = make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 10, 11)]);
        // 11h - 10h = 1h = 60 min rest; threshold = 540 min → not buffered.
        // But wait: d2 starts at 10h and d1 ends at 10h → 0 min rest.
        // Use hours 8→9 and 9→10 to get 0 rest.
        let d3 = make_duty("D3", vec![make_leg("L3", "LHR", "CDG", 8, 9)]);
        let d4 = make_duty("D4", vec![make_leg("L4", "CDG", "LHR", 9, 10)]);
        let p1 = make_pairing("P1", "LHR", vec![d3, d4]);
        let r1 = make_rotation("R1", "C1", vec![p1]);
        let roster = make_roster(vec![], vec![r1]);

        let evaluator = make_evaluator();
        let score = evaluator.evaluate(&roster);
        // Rest = 9h - 9h = 0 min < 540 → not buffered.
        assert_eq!(score.rest_buffer_ratio, 0.0);
    }

    #[test]
    fn crew_slack_ratio_reflects_pairing_count() {
        // Two rotations: one with 1 pairing (slack), one with 3 (full).
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 8, 10)]);
        let d2 = make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 22, 24)]);
        let p1 = make_pairing("P1", "LHR", vec![d1, d2]);
        let r1 = make_rotation("R1", "C1", vec![p1]); // 1 pairing < 3 → slack

        let d3 = make_duty("D3", vec![make_leg("L3", "LHR", "CDG", 8, 10)]);
        let d4 = make_duty("D4", vec![make_leg("L4", "CDG", "LHR", 22, 24)]);
        let p2 = make_pairing("P2", "LHR", vec![d3, d4]);
        let d5 = make_duty("D5", vec![make_leg("L5", "LHR", "CDG", 32, 34)]);
        let d6 = make_duty("D6", vec![make_leg("L6", "CDG", "LHR", 46, 48)]);
        let p3 = make_pairing("P3", "LHR", vec![d5, d6]);
        let d7 = make_duty("D7", vec![make_leg("L7", "LHR", "CDG", 56, 58)]);
        let d8 = make_duty("D8", vec![make_leg("L8", "CDG", "LHR", 70, 72)]);
        let p4 = make_pairing("P4", "LHR", vec![d7, d8]);
        let r2 = make_rotation("R2", "C2", vec![p2, p3, p4]); // 3 pairings = max → no slack

        let roster = make_roster(vec![], vec![r1, r2]);
        let evaluator = make_evaluator();
        let score = evaluator.evaluate(&roster);
        // 1 of 2 rotations has slack → 0.5
        assert!((score.crew_slack_ratio - 0.5).abs() < 1e-9);
    }

    #[test]
    fn overall_is_weighted_average() {
        let evaluator = RobustnessEvaluator::new(480.0, 60.0, 3, 0.5);
        let score = RobustnessScore::new(0.8, 0.6, 0.5);
        assert!((score.overall - 0.7).abs() < 1e-9);
    }
}