//! Optimization metrics.
//!
//! [`OptimizationMetrics`] collects counters during a search run.  It is
//! passed by mutable reference to search strategies so they can record
//! their activity without coupling to a specific logging framework.

/// Counters collected during an optimization run.
#[derive(Debug, Clone, Default)]
pub struct OptimizationMetrics {
    /// Total number of cost evaluations performed.
    evaluations: usize,
    /// Number of moves that improved the current solution.
    improvements: usize,
    /// Number of feasibility checks performed (legality oracle calls).
    feasibility_checks: usize,
    /// Number of search iterations completed.
    iterations: usize,
}

impl OptimizationMetrics {
    /// Create a new, zeroed [`OptimizationMetrics`].
    pub fn new() -> Self {
        Self::default()
    }

    // ── Recording ─────────────────────────────────────────────────────────────

    /// Record one cost evaluation.
    pub fn record_evaluation(&mut self) {
        self.evaluations += 1;
    }

    /// Record one improving move.
    pub fn record_improvement(&mut self) {
        self.improvements += 1;
    }

    /// Record one feasibility check.
    pub fn record_feasibility_check(&mut self) {
        self.feasibility_checks += 1;
    }

    /// Record one completed search iteration.
    pub fn record_iteration(&mut self) {
        self.iterations += 1;
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    /// Total cost evaluations.
    pub fn evaluations(&self) -> usize {
        self.evaluations
    }

    /// Number of improving moves accepted.
    pub fn improvements(&self) -> usize {
        self.improvements
    }

    /// Number of feasibility checks.
    pub fn feasibility_checks(&self) -> usize {
        self.feasibility_checks
    }

    /// Number of search iterations.
    pub fn iterations(&self) -> usize {
        self.iterations
    }

    /// Improvement rate: improvements / evaluations, or 0.0 if no evaluations.
    pub fn improvement_rate(&self) -> f64 {
        if self.evaluations == 0 {
            0.0
        } else {
            self.improvements as f64 / self.evaluations as f64
        }
    }
}

impl std::fmt::Display for OptimizationMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OptimizationMetrics {{ evaluations: {}, improvements: {}, \
             feasibility_checks: {}, iterations: {}, improvement_rate: {:.2}% }}",
            self.evaluations,
            self.improvements,
            self.feasibility_checks,
            self.iterations,
            self.improvement_rate() * 100.0,
        )
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_metrics_are_zero() {
        let m = OptimizationMetrics::new();
        assert_eq!(m.evaluations(), 0);
        assert_eq!(m.improvements(), 0);
        assert_eq!(m.feasibility_checks(), 0);
        assert_eq!(m.iterations(), 0);
        assert_eq!(m.improvement_rate(), 0.0);
    }

    #[test]
    fn record_evaluation_increments() {
        let mut m = OptimizationMetrics::new();
        m.record_evaluation();
        m.record_evaluation();
        assert_eq!(m.evaluations(), 2);
    }

    #[test]
    fn improvement_rate_computed_correctly() {
        let mut m = OptimizationMetrics::new();
        m.record_evaluation();
        m.record_evaluation();
        m.record_evaluation();
        m.record_evaluation();
        m.record_improvement();
        m.record_improvement();
        assert!((m.improvement_rate() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn display_does_not_panic() {
        let mut m = OptimizationMetrics::new();
        m.record_evaluation();
        m.record_improvement();
        let s = format!("{m}");
        assert!(s.contains("evaluations: 1"));
    }
}