// Decision Intelligence module for UltraCrew.
//
// Provides:
//   - `analyze_solution` / `generate_insights` — per-solution metrics and insights
//   - `OperationalLearningLoop` — full end-to-end learning loop workflow:
//       cycle completed → outcomes reviewed → patterns identified →
//       insights added to Operational Knowledge Graph

use crate::schedule_solution::ScheduleSolution;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Per-solution analysis ─────────────────────────────────────────────────────

/// Analyzes a `ScheduleSolution` and returns key metrics as a map.
/// The keys are descriptive metric names and the values are numeric scores.
pub fn analyze_solution(solution: &ScheduleSolution) -> HashMap<String, f64> {
    let mut metrics = HashMap::new();
    metrics.insert("fitness".to_string(), solution.fitness);
    metrics.insert(
        "hard_violations".to_string(),
        solution.hard_violations as f64,
    );
    metrics.insert("fairness_penalty".to_string(), solution.fairness_penalty);
    metrics.insert("fatigue_penalty".to_string(), solution.fatigue_penalty);
    metrics.insert(
        "rest_violations".to_string(),
        solution.rest_violations as f64,
    );
    metrics
}

/// Generates human‑readable insights from a `ScheduleSolution`.
/// Returns a vector of strings, each describing an aspect of the schedule.
pub fn generate_insights(solution: &ScheduleSolution) -> Vec<String> {
    let mut insights = Vec::new();
    insights.push(format!("Overall fitness: {:.2}", solution.fitness));
    if solution.hard_violations == 0 {
        insights.push("No hard‑constraint violations detected.".to_string());
    } else {
        insights.push(format!(
            "Hard‑constraint violations: {}",
            solution.hard_violations
        ));
    }
    if solution.fairness_penalty > 0.0 {
        insights.push(format!(
            "Fairness penalty: {:.2}",
            solution.fairness_penalty
        ));
    }
    if solution.fatigue_penalty > 0.0 {
        insights.push(format!("Fatigue penalty: {:.2}", solution.fatigue_penalty));
    }
    if solution.rest_violations > 0 {
        insights.push(format!(
            "Rest period violations: {}",
            solution.rest_violations
        ));
    }
    insights
}

// ── Operational Learning Loop ─────────────────────────────────────────────────

/// A completed scheduling cycle — the raw material for the learning loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingCycleOutcome {
    pub cycle_id: String,
    pub workspace_id: String,
    pub solution: ScheduleSolution,
    pub cycle_metrics: HashMap<String, f64>,
    pub disruption_count: usize,
    pub disruptions_resolved: usize,
    pub timestamp: u64,
}

impl SchedulingCycleOutcome {
    pub fn new(
        cycle_id: impl Into<String>,
        workspace_id: impl Into<String>,
        solution: ScheduleSolution,
        disruption_count: usize,
        disruptions_resolved: usize,
        timestamp: u64,
    ) -> Self {
        let cycle_metrics = analyze_solution(&solution);
        Self {
            cycle_id: cycle_id.into(),
            workspace_id: workspace_id.into(),
            solution,
            cycle_metrics,
            disruption_count,
            disruptions_resolved,
            timestamp,
        }
    }
}

/// A workforce behaviour pattern extracted from multiple scheduling cycles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkforcePattern {
    pub pattern_id: String,
    pub description: String,
    pub pattern_type: PatternType,
    /// Confidence in [0.0, 1.0] — increases as more cycles confirm the pattern.
    pub confidence: f64,
    /// Number of cycles that contributed to this pattern.
    pub supporting_cycles: usize,
    /// Maturity level — advances as confidence and supporting_cycles grow.
    pub maturity: PatternMaturity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    /// Recurring constraint violation pattern.
    ConstraintViolation,
    /// Recurring fairness imbalance across workers.
    FairnessImbalance,
    /// Recurring fatigue accumulation pattern.
    FatigueAccumulation,
    /// Recurring disruption pattern (same disruption type recurs).
    DisruptionRecurrence,
    /// Positive pattern — consistently high fitness across cycles.
    HighPerformance,
}

/// Pattern maturity — advances as evidence accumulates.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PatternMaturity {
    /// Observed in 1–2 cycles; not yet reliable.
    Candidate,
    /// Observed in 3–5 cycles; emerging pattern.
    Observed,
    /// Observed in 6–10 cycles; reliable pattern.
    Repeated,
    /// Observed in 10+ cycles with high confidence; actionable.
    Validated,
}

impl PatternMaturity {
    fn from_cycles(count: usize, confidence: f64) -> Self {
        if count >= 10 && confidence >= 0.8 {
            PatternMaturity::Validated
        } else if count >= 6 {
            PatternMaturity::Repeated
        } else if count >= 3 {
            PatternMaturity::Observed
        } else {
            PatternMaturity::Candidate
        }
    }
}

/// A validated insight added to the Operational Knowledge Graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalInsight {
    pub insight_id: String,
    pub description: String,
    pub source_pattern_id: String,
    pub recommendation: String,
    pub confidence: f64,
    pub timestamp: u64,
}

/// A cycle review report — produced at the end of each learning loop run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleReviewReport {
    pub report_id: String,
    pub cycles_reviewed: usize,
    pub patterns_identified: usize,
    pub patterns_validated: usize,
    pub insights_added: usize,
    pub mean_fitness: f64,
    pub mean_hard_violations: f64,
    pub mean_disruption_resolution_rate: f64,
    pub summary: Vec<String>,
    pub timestamp: u64,
}

/// Full end-to-end Operational Learning Loop for UltraCrew.
///
/// Workflow:
///   1. `record_cycle` — record a completed scheduling cycle outcome
///   2. `identify_patterns` — extract patterns from accumulated outcomes
///   3. `add_insight` — add validated insights to the Operational Knowledge Graph
///   4. `generate_report` — produce a structured cycle review report
pub struct OperationalLearningLoop {
    pub outcomes: Vec<SchedulingCycleOutcome>,
    pub patterns: Vec<WorkforcePattern>,
    pub insights: Vec<OperationalInsight>,
    pattern_counter: u32,
    insight_counter: u32,
}

impl OperationalLearningLoop {
    pub fn new() -> Self {
        Self {
            outcomes: Vec::new(),
            patterns: Vec::new(),
            insights: Vec::new(),
            pattern_counter: 0,
            insight_counter: 0,
        }
    }

    /// Step 1 — Record a completed scheduling cycle outcome.
    pub fn record_cycle(&mut self, outcome: SchedulingCycleOutcome) {
        self.outcomes.push(outcome);
    }

    /// Step 2 — Identify patterns from accumulated cycle outcomes.
    ///
    /// Runs pattern detection across all recorded outcomes and updates
    /// the pattern registry. Existing patterns are updated; new patterns
    /// are added.
    pub fn identify_patterns(&mut self) {
        if self.outcomes.is_empty() {
            return;
        }

        let n = self.outcomes.len() as f64;

        // Pattern 1: Recurring hard constraint violations.
        let violation_cycles = self
            .outcomes
            .iter()
            .filter(|o| o.solution.hard_violations > 0)
            .count();
        let violation_rate = violation_cycles as f64 / n;
        if violation_rate >= 0.3 {
            self.upsert_pattern(
                "constraint-violation-recurrence",
                format!(
                    "Hard constraint violations occur in {:.0}% of scheduling cycles",
                    violation_rate * 100.0
                ),
                PatternType::ConstraintViolation,
                violation_rate.min(1.0),
                violation_cycles,
            );
        }

        // Pattern 2: Recurring fairness imbalance.
        let fairness_cycles = self
            .outcomes
            .iter()
            .filter(|o| o.solution.fairness_penalty > 10.0)
            .count();
        let fairness_rate = fairness_cycles as f64 / n;
        if fairness_rate >= 0.3 {
            self.upsert_pattern(
                "fairness-imbalance-recurrence",
                format!(
                    "Fairness imbalance (penalty > 10) occurs in {:.0}% of cycles",
                    fairness_rate * 100.0
                ),
                PatternType::FairnessImbalance,
                fairness_rate.min(1.0),
                fairness_cycles,
            );
        }

        // Pattern 3: Recurring fatigue accumulation.
        let fatigue_cycles = self
            .outcomes
            .iter()
            .filter(|o| o.solution.fatigue_penalty > 5.0)
            .count();
        let fatigue_rate = fatigue_cycles as f64 / n;
        if fatigue_rate >= 0.3 {
            self.upsert_pattern(
                "fatigue-accumulation-recurrence",
                format!(
                    "Fatigue accumulation (penalty > 5) occurs in {:.0}% of cycles",
                    fatigue_rate * 100.0
                ),
                PatternType::FatigueAccumulation,
                fatigue_rate.min(1.0),
                fatigue_cycles,
            );
        }

        // Pattern 4: Recurring disruptions.
        let disruption_cycles = self
            .outcomes
            .iter()
            .filter(|o| o.disruption_count > 0)
            .count();
        let disruption_rate = disruption_cycles as f64 / n;
        if disruption_rate >= 0.3 {
            self.upsert_pattern(
                "disruption-recurrence",
                format!(
                    "Operational disruptions occur in {:.0}% of scheduling cycles",
                    disruption_rate * 100.0
                ),
                PatternType::DisruptionRecurrence,
                disruption_rate.min(1.0),
                disruption_cycles,
            );
        }

        // Pattern 5: High performance (positive pattern).
        let high_perf_cycles = self
            .outcomes
            .iter()
            .filter(|o| o.solution.hard_violations == 0 && o.solution.fitness > 5000.0)
            .count();
        let high_perf_rate = high_perf_cycles as f64 / n;
        if high_perf_rate >= 0.5 {
            self.upsert_pattern(
                "high-performance",
                format!(
                    "High-performance schedules (0 violations, fitness > 5000) achieved in {:.0}% of cycles",
                    high_perf_rate * 100.0
                ),
                PatternType::HighPerformance,
                high_perf_rate.min(1.0),
                high_perf_cycles,
            );
        }
    }

    fn upsert_pattern(
        &mut self,
        key: &str,
        description: String,
        pattern_type: PatternType,
        confidence: f64,
        supporting_cycles: usize,
    ) {
        if let Some(existing) = self.patterns.iter_mut().find(|p| p.pattern_id == key) {
            existing.confidence = confidence;
            existing.supporting_cycles = supporting_cycles;
            existing.maturity = PatternMaturity::from_cycles(supporting_cycles, confidence);
            existing.description = description;
        } else {
            self.pattern_counter += 1;
            self.patterns.push(WorkforcePattern {
                pattern_id: key.to_string(),
                description,
                pattern_type,
                confidence,
                supporting_cycles,
                maturity: PatternMaturity::from_cycles(supporting_cycles, confidence),
            });
        }
    }

    /// Step 3 — Add a validated insight to the Operational Knowledge Graph.
    ///
    /// Only patterns at `Validated` or `Repeated` maturity should produce insights.
    pub fn add_insight(
        &mut self,
        source_pattern_id: &str,
        recommendation: impl Into<String>,
        timestamp: u64,
    ) -> Option<String> {
        let pattern = self
            .patterns
            .iter()
            .find(|p| p.pattern_id == source_pattern_id)?;
        if pattern.maturity < PatternMaturity::Repeated {
            return None; // not mature enough
        }
        self.insight_counter += 1;
        let insight_id = format!("insight-{:04}", self.insight_counter);
        let insight = OperationalInsight {
            insight_id: insight_id.clone(),
            description: pattern.description.clone(),
            source_pattern_id: source_pattern_id.to_string(),
            recommendation: recommendation.into(),
            confidence: pattern.confidence,
            timestamp,
        };
        self.insights.push(insight);
        Some(insight_id)
    }

    /// Automatically promote all mature patterns to insights.
    ///
    /// Generates a default recommendation for each `Validated` or `Repeated`
    /// pattern that does not yet have a corresponding insight.
    pub fn auto_promote_insights(&mut self, timestamp: u64) {
        let promotable: Vec<(String, String, String)> = self.patterns.iter()
            .filter(|p| p.maturity >= PatternMaturity::Repeated)
            .filter(|p| !self.insights.iter().any(|i| i.source_pattern_id == p.pattern_id))
            .map(|p| {
                let rec = match p.pattern_type {
                    PatternType::ConstraintViolation =>
                        "Review constraint configuration; consider relaxing soft constraints or increasing workforce capacity.".to_string(),
                    PatternType::FairnessImbalance =>
                        "Adjust fairness weight in optimisation profile; consider minimum-hours guarantees.".to_string(),
                    PatternType::FatigueAccumulation =>
                        "Increase rest buffer between scheduling cycles; review historical hours tracking.".to_string(),
                    PatternType::DisruptionRecurrence =>
                        "Increase reserve capacity; review disruption recovery workflow configuration.".to_string(),
                    PatternType::HighPerformance =>
                        "Document current optimisation profile as a reference configuration for future cycles.".to_string(),
                };
                (p.pattern_id.clone(), rec, p.description.clone())
            })
            .collect();

        for (pattern_id, recommendation, _) in promotable {
            self.add_insight(&pattern_id, recommendation, timestamp);
        }
    }

    /// Step 4 — Generate a structured cycle review report.
    pub fn generate_report(&self, timestamp: u64) -> CycleReviewReport {
        let n = self.outcomes.len();
        let mean_fitness = if n > 0 {
            self.outcomes
                .iter()
                .map(|o| o.solution.fitness)
                .sum::<f64>()
                / n as f64
        } else {
            0.0
        };
        let mean_hard_violations = if n > 0 {
            self.outcomes
                .iter()
                .map(|o| o.solution.hard_violations as f64)
                .sum::<f64>()
                / n as f64
        } else {
            0.0
        };
        let mean_disruption_resolution_rate = if n > 0 {
            let total_disruptions: usize = self.outcomes.iter().map(|o| o.disruption_count).sum();
            let total_resolved: usize = self.outcomes.iter().map(|o| o.disruptions_resolved).sum();
            if total_disruptions > 0 {
                total_resolved as f64 / total_disruptions as f64
            } else {
                1.0
            }
        } else {
            0.0
        };

        let validated_patterns = self
            .patterns
            .iter()
            .filter(|p| p.maturity == PatternMaturity::Validated)
            .count();

        let mut summary = Vec::new();
        summary.push(format!("Cycles reviewed: {}", n));
        summary.push(format!("Mean fitness: {:.2}", mean_fitness));
        summary.push(format!("Mean hard violations: {:.2}", mean_hard_violations));
        summary.push(format!(
            "Disruption resolution rate: {:.0}%",
            mean_disruption_resolution_rate * 100.0
        ));
        summary.push(format!("Patterns identified: {}", self.patterns.len()));
        summary.push(format!("Patterns validated: {}", validated_patterns));
        summary.push(format!(
            "Insights added to Knowledge Graph: {}",
            self.insights.len()
        ));

        for pattern in &self.patterns {
            if pattern.maturity >= PatternMaturity::Repeated {
                summary.push(format!(
                    "[{}] {} (confidence: {:.0}%, cycles: {})",
                    format!("{:?}", pattern.maturity),
                    pattern.description,
                    pattern.confidence * 100.0,
                    pattern.supporting_cycles,
                ));
            }
        }

        CycleReviewReport {
            report_id: format!("report-{}", timestamp),
            cycles_reviewed: n,
            patterns_identified: self.patterns.len(),
            patterns_validated: validated_patterns,
            insights_added: self.insights.len(),
            mean_fitness,
            mean_hard_violations,
            mean_disruption_resolution_rate,
            summary,
            timestamp,
        }
    }

    pub fn validated_patterns(&self) -> Vec<&WorkforcePattern> {
        self.patterns
            .iter()
            .filter(|p| p.maturity == PatternMaturity::Validated)
            .collect()
    }

    pub fn insights(&self) -> &[OperationalInsight] {
        &self.insights
    }
}

impl Default for OperationalLearningLoop {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_solution(
        fitness: f64,
        hard_violations: usize,
        fairness: f64,
        fatigue: f64,
    ) -> ScheduleSolution {
        ScheduleSolution {
            assignments: HashMap::new(),
            fitness,
            hard_violations,
            fairness_penalty: fairness,
            fatigue_penalty: fatigue,
            rest_violations: 0,
            recommendations: None,
            telemetry: None,
        }
    }

    fn make_outcome(
        cycle_id: &str,
        solution: ScheduleSolution,
        disruptions: usize,
        resolved: usize,
    ) -> SchedulingCycleOutcome {
        SchedulingCycleOutcome::new(cycle_id, "ws-001", solution, disruptions, resolved, 1000)
    }

    #[test]
    fn analyze_solution_returns_all_metrics() {
        let sol = make_solution(9247.3, 0, 5.0, 2.0);
        let metrics = analyze_solution(&sol);
        assert_eq!(metrics["fitness"], 9247.3);
        assert_eq!(metrics["hard_violations"], 0.0);
        assert_eq!(metrics["fairness_penalty"], 5.0);
    }

    #[test]
    fn generate_insights_no_violations() {
        let sol = make_solution(9247.3, 0, 0.0, 0.0);
        let insights = generate_insights(&sol);
        assert!(insights.iter().any(|i| i.contains("No hard")));
    }

    #[test]
    fn learning_loop_identifies_violation_pattern() {
        let mut loop_ = OperationalLearningLoop::new();
        // Record 5 cycles with hard violations.
        for i in 0..5 {
            let sol = make_solution(5000.0, 2, 0.0, 0.0);
            loop_.record_cycle(make_outcome(&format!("c-{}", i), sol, 0, 0));
        }
        loop_.identify_patterns();
        assert!(loop_
            .patterns
            .iter()
            .any(|p| p.pattern_type == PatternType::ConstraintViolation));
    }

    #[test]
    fn learning_loop_generates_report() {
        let mut loop_ = OperationalLearningLoop::new();
        for i in 0..3 {
            let sol = make_solution(8000.0, 0, 0.0, 0.0);
            loop_.record_cycle(make_outcome(&format!("c-{}", i), sol, 1, 1));
        }
        loop_.identify_patterns();
        let report = loop_.generate_report(9999);
        assert_eq!(report.cycles_reviewed, 3);
        assert_eq!(report.mean_disruption_resolution_rate, 1.0);
    }

    #[test]
    fn auto_promote_insights_from_repeated_patterns() {
        let mut loop_ = OperationalLearningLoop::new();
        // 6 cycles with violations → Repeated maturity.
        for i in 0..6 {
            let sol = make_solution(5000.0, 1, 0.0, 0.0);
            loop_.record_cycle(make_outcome(&format!("c-{}", i), sol, 0, 0));
        }
        loop_.identify_patterns();
        loop_.auto_promote_insights(1000);
        assert!(!loop_.insights().is_empty());
    }
}
