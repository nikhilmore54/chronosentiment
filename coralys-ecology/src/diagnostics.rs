use serde::{Deserialize, Serialize};

/// Universal objective values wrapper representing the multi-objective score.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObjectiveVector {
    pub values: Vec<f64>,
}

impl ObjectiveVector {
    pub fn new(values: Vec<f64>) -> Self {
        Self { values }
    }
}

/// Telemetry details for an individual offspring or candidate solution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateObservation {
    pub objectives: ObjectiveVector,
    pub admitted: bool,
    pub feasible: bool,
    pub parent_objectives: Option<ObjectiveVector>,
}

/// Generation-level trajectory telemetry container.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchObservation {
    pub generation: usize,
    pub archive_size: usize,
    pub diversity_score: f64,
    pub candidates: Vec<CandidateObservation>,
    pub archive_objectives: Vec<ObjectiveVector>,
    pub telemetry: Option<coralys_core::telemetry::SearchTelemetry>,
}

/// Scalar key-value metric for diagnostic reporting details.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Metric {
    pub name: String,
    pub value: f64,
}

impl Metric {
    pub fn new(name: impl Into<String>, value: f64) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }
}

/// Output report for a given diagnostic detector evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiagnosticResult {
    pub confidence: f64, // 0.0 to 1.0
    pub severity: f64,   // 0.0 to 1.0
    pub evidence_count: usize,
    pub supporting_metrics: Vec<Metric>,
}

/// Generic trait for evaluating search observations to produce a diagnostic result.
pub trait DiagnosticDetector {
    fn evaluate(&self, observations: &[SearchObservation]) -> DiagnosticResult;
}

/// A snapshot of diagnostic results for a single generation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiagnosticHistoryEntry {
    pub generation: usize,
    pub results: std::collections::HashMap<String, DiagnosticResult>,
}

/// Accumulates search diagnostics over time to evaluate persistence and transitions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EcologyState {
    pub history: Vec<DiagnosticHistoryEntry>,
    pub max_history_size: usize,
}

impl EcologyState {
    pub fn new(max_history_size: usize) -> Self {
        Self {
            history: Vec::new(),
            max_history_size,
        }
    }

    pub fn record(
        &mut self,
        generation: usize,
        results: std::collections::HashMap<String, DiagnosticResult>,
    ) {
        self.history.push(DiagnosticHistoryEntry {
            generation,
            results,
        });
        if self.history.len() > self.max_history_size {
            self.history.remove(0);
        }
    }

    pub fn mean_confidence(&self, detector_name: &str, window: usize) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        let start = self.history.len().saturating_sub(window);
        let slice = &self.history[start..];
        let mut sum = 0.0;
        let mut count = 0;
        for entry in slice {
            if let Some(res) = entry.results.get(detector_name) {
                sum += res.confidence;
                count += 1;
            }
        }
        if count == 0 { 0.0 } else { sum / count as f64 }
    }

    pub fn mean_severity(&self, detector_name: &str, window: usize) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        let start = self.history.len().saturating_sub(window);
        let slice = &self.history[start..];
        let mut sum = 0.0;
        let mut count = 0;
        for entry in slice {
            if let Some(res) = entry.results.get(detector_name) {
                sum += res.severity;
                count += 1;
            }
        }
        if count == 0 { 0.0 } else { sum / count as f64 }
    }

    pub fn persistence_score(&self, detector_name: &str, threshold: f64, window: usize) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        let start = self.history.len().saturating_sub(window);
        let slice = &self.history[start..];
        let mut triggered = 0;
        let mut count = 0;
        for entry in slice {
            if let Some(res) = entry.results.get(detector_name) {
                if res.confidence >= threshold {
                    triggered += 1;
                }
                count += 1;
            }
        }
        if count == 0 {
            0.0
        } else {
            triggered as f64 / count as f64
        }
    }
}

// --- Mathematical Helper Functions ---

/// Pearson correlation coefficient between two floating-point slices.
pub fn pearson_correlation(x: &[f64], y: &[f64]) -> f64 {
    if x.len() < 2 || x.len() != y.len() {
        return 0.0;
    }
    let n = x.len() as f64;
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;

    let mut num = 0.0;
    let mut den_x = 0.0;
    let mut den_y = 0.0;
    for (val_x, val_y) in x.iter().zip(y.iter()) {
        let dx = val_x - mean_x;
        let dy = val_y - mean_y;
        num += dx * dy;
        den_x += dx * dx;
        den_y += dy * dy;
    }

    if den_x == 0.0 || den_y == 0.0 {
        return 0.0;
    }

    num / (den_x * den_y).sqrt()
}

/// Gini coefficient of inequality for a slice of values (handles negative values by relative shifting).
pub fn gini_coefficient(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let n = values.len();
    if n == 1 {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let min_val = sorted[0];
    let offset = if min_val < 0.0 { -min_val } else { 0.0 };

    let mut sum = 0.0;
    let mut weighted_sum = 0.0;
    for (i, &v) in sorted.iter().enumerate() {
        let val = v + offset;
        sum += val;
        weighted_sum += (i as f64 + 1.0) * val;
    }

    if sum == 0.0 {
        return 0.0;
    }

    (2.0 * weighted_sum) / (n as f64 * sum) - ((n + 1) as f64) / n as f64
}

// --- The Five Generic Detectors ---

/// 1. AttractorDetector
/// Diagnoses if a single objective dominates correlation with archive admission.
#[derive(Debug, Clone)]
pub struct AttractorDetector {
    pub min_correlation: f64,
    pub margin_threshold: f64,
}

impl AttractorDetector {
    pub fn new(min_correlation: f64, margin_threshold: f64) -> Self {
        Self {
            min_correlation,
            margin_threshold,
        }
    }
}

impl Default for AttractorDetector {
    fn default() -> Self {
        Self::new(0.3, 0.2)
    }
}

impl DiagnosticDetector for AttractorDetector {
    fn evaluate(&self, observations: &[SearchObservation]) -> DiagnosticResult {
        let mut candidates = Vec::new();
        for obs in observations {
            candidates.extend(obs.candidates.iter().cloned());
        }

        let evidence_count = candidates.len();
        if evidence_count < 5 {
            return DiagnosticResult {
                confidence: 0.0,
                severity: 0.0,
                evidence_count,
                supporting_metrics: Vec::new(),
            };
        }

        let num_objectives = candidates[0].objectives.values.len();
        let mut max_corr = -1.0;
        let mut second_corr = -1.0;
        let mut max_index = 0;
        let mut supporting_metrics = Vec::new();

        let admissions: Vec<f64> = candidates
            .iter()
            .map(|c| if c.admitted { 1.0 } else { 0.0 })
            .collect();

        for obj_idx in 0..num_objectives {
            let improvements: Vec<f64> = candidates
                .iter()
                .map(|c| {
                    if let Some(ref parent) = c.parent_objectives {
                        parent.values[obj_idx] - c.objectives.values[obj_idx]
                    } else {
                        -c.objectives.values[obj_idx] // lower is better, so negate value
                    }
                })
                .collect();

            let r = pearson_correlation(&improvements, &admissions);
            supporting_metrics.push(Metric::new(format!("correlation_obj_{}", obj_idx), r));

            if r > max_corr {
                second_corr = max_corr;
                max_corr = r;
                max_index = obj_idx;
            } else if r > second_corr {
                second_corr = r;
            }
        }

        supporting_metrics.push(Metric::new("attractor_index", max_index as f64));
        supporting_metrics.push(Metric::new("max_correlation", max_corr));

        let diff = max_corr - second_corr;
        let confidence = if max_corr > self.min_correlation && diff > 0.0 {
            (diff / self.margin_threshold).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let severity = (max_corr.clamp(0.0, 1.0) * confidence).clamp(0.0, 1.0);

        DiagnosticResult {
            confidence,
            severity,
            evidence_count,
            supporting_metrics,
        }
    }
}

/// 2. TradeoffBasinDetector
/// Diagnoses if improvement along X systematically induces degradation in Y.
#[derive(Debug, Clone)]
pub struct TradeoffBasinDetector {
    pub x_index: usize,
    pub y_index: usize,
    pub min_correlation: f64, // expected negative correlation, e.g. -0.5
}

impl TradeoffBasinDetector {
    pub fn new(x_index: usize, y_index: usize, min_correlation: f64) -> Self {
        Self {
            x_index,
            y_index,
            min_correlation,
        }
    }
}

impl DiagnosticDetector for TradeoffBasinDetector {
    fn evaluate(&self, observations: &[SearchObservation]) -> DiagnosticResult {
        let mut candidates_with_parent = Vec::new();
        for obs in observations {
            for c in &obs.candidates {
                if c.parent_objectives.is_some() {
                    candidates_with_parent.push(c.clone());
                }
            }
        }

        let evidence_count = candidates_with_parent.len();
        if evidence_count < 5 {
            return DiagnosticResult {
                confidence: 0.0,
                severity: 0.0,
                evidence_count,
                supporting_metrics: Vec::new(),
            };
        }

        let mut dx_vals = Vec::with_capacity(evidence_count);
        let mut dy_vals = Vec::with_capacity(evidence_count);

        let mut x_improved_count = 0;
        let mut y_degraded_when_x_improved = 0;

        for c in &candidates_with_parent {
            let parent = c.parent_objectives.as_ref().unwrap();
            let dx = parent.values[self.x_index] - c.objectives.values[self.x_index];
            let dy = parent.values[self.y_index] - c.objectives.values[self.y_index];

            dx_vals.push(dx);
            dy_vals.push(dy);

            if dx > 0.0 {
                x_improved_count += 1;
                if dy < 0.0 {
                    y_degraded_when_x_improved += 1;
                }
            }
        }

        let r = pearson_correlation(&dx_vals, &dy_vals);
        let probability_degrade = if x_improved_count > 0 {
            y_degraded_when_x_improved as f64 / x_improved_count as f64
        } else {
            0.0
        };

        let confidence = if r < 0.0 {
            (r.abs() / self.min_correlation.abs()).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let severity = (probability_degrade * confidence).clamp(0.0, 1.0);

        DiagnosticResult {
            confidence,
            severity,
            evidence_count,
            supporting_metrics: vec![
                Metric::new("correlation_x_y", r),
                Metric::new("tradeoff_probability_y_given_x", probability_degrade),
            ],
        }
    }
}

/// 3. EcologyLockInDetector
/// Diagnoses if search converges to a stagnant state with collapsed archive diversity.
#[derive(Debug, Clone)]
pub struct EcologyLockInDetector {
    pub target_index: usize,
    pub gini_threshold: f64,
    pub stagnation_generations: usize,
}

impl EcologyLockInDetector {
    pub fn new(target_index: usize, gini_threshold: f64, stagnation_generations: usize) -> Self {
        Self {
            target_index,
            gini_threshold,
            stagnation_generations,
        }
    }
}

impl Default for EcologyLockInDetector {
    fn default() -> Self {
        Self::new(0, 0.15, 10)
    }
}

impl DiagnosticDetector for EcologyLockInDetector {
    fn evaluate(&self, observations: &[SearchObservation]) -> DiagnosticResult {
        let evidence_count = observations.len();
        if evidence_count <= self.stagnation_generations || observations.is_empty() {
            return DiagnosticResult {
                confidence: 0.0,
                severity: 0.0,
                evidence_count,
                supporting_metrics: Vec::new(),
            };
        }

        let latest = &observations[evidence_count - 1];
        let archive = &latest.archive_objectives;
        if archive.is_empty() {
            return DiagnosticResult {
                confidence: 0.0,
                severity: 0.0,
                evidence_count,
                supporting_metrics: Vec::new(),
            };
        }

        let target_vals: Vec<f64> = archive
            .iter()
            .map(|obj| obj.values[self.target_index])
            .collect();
        let gini = gini_coefficient(&target_vals);

        // Find best target value in each generation
        let get_best_target = |obs: &SearchObservation| -> f64 {
            obs.archive_objectives
                .iter()
                .map(|obj| obj.values[self.target_index])
                .fold(f64::INFINITY, |a, b| a.min(b))
        };

        let best_now = get_best_target(latest);
        let old_obs = &observations[evidence_count - 1 - self.stagnation_generations];
        let best_old = get_best_target(old_obs);

        let best_delta = (best_old - best_now).abs();
        let is_stagnant = best_delta < 1e-6;

        let confidence = if is_stagnant {
            if archive.len() > 1 {
                if gini < self.gini_threshold {
                    1.0 - (gini / self.gini_threshold)
                } else {
                    0.0
                }
            } else {
                // For single-objective domains (archive size 1), rely on population diversity score
                if latest.diversity_score < self.gini_threshold {
                    1.0 - (latest.diversity_score / self.gini_threshold)
                } else {
                    0.0
                }
            }
        } else {
            0.0
        };

        DiagnosticResult {
            confidence,
            severity: confidence, // severity maps directly to lock-in confidence
            evidence_count,
            supporting_metrics: vec![
                Metric::new("gini_coefficient", gini),
                Metric::new("best_target_delta", best_delta),
            ],
        }
    }
}

/// 4. AccumulationFailureDetector
/// Diagnoses if target improvements fail to persist in the archive.
#[derive(Debug, Clone)]
pub struct AccumulationFailureDetector {
    pub target_index: usize,
    pub rejection_threshold: f64,
}

impl AccumulationFailureDetector {
    pub fn new(target_index: usize, rejection_threshold: f64) -> Self {
        Self {
            target_index,
            rejection_threshold,
        }
    }
}

impl Default for AccumulationFailureDetector {
    fn default() -> Self {
        Self::new(0, 0.75)
    }
}

impl DiagnosticDetector for AccumulationFailureDetector {
    fn evaluate(&self, observations: &[SearchObservation]) -> DiagnosticResult {
        let mut target_improving_count = 0;
        let mut rejected_count = 0;

        for obs in observations {
            for c in &obs.candidates {
                if let Some(ref parent) = c.parent_objectives {
                    let delta_target =
                        parent.values[self.target_index] - c.objectives.values[self.target_index];
                    if delta_target > 0.0 {
                        target_improving_count += 1;
                        if !c.admitted {
                            rejected_count += 1;
                        }
                    }
                }
            }
        }

        let evidence_count = target_improving_count;
        if evidence_count < 2 {
            // Require at least 2 improving candidates in the window
            return DiagnosticResult {
                confidence: 0.0,
                severity: 0.0,
                evidence_count,
                supporting_metrics: vec![Metric::new("rejection_rate", 0.0)],
            };
        }

        let rejection_rate = rejected_count as f64 / target_improving_count as f64;
        let confidence = (rejection_rate / self.rejection_threshold).clamp(0.0, 1.0);
        let severity = rejection_rate * confidence;

        DiagnosticResult {
            confidence,
            severity,
            evidence_count,
            supporting_metrics: vec![
                Metric::new("rejection_rate", rejection_rate),
                Metric::new("target_improving_count", target_improving_count as f64),
                Metric::new("rejected_target_improving_count", rejected_count as f64),
            ],
        }
    }
}

/// 5. ProxySuppressionDetector
/// Diagnoses if target improvements are rejected because they degrade proxy objectives.
#[derive(Debug, Clone)]
pub struct ProxySuppressionDetector {
    pub target_index: usize,
    pub proxy_indices: Vec<usize>,
    pub suppression_threshold: f64,
}

impl ProxySuppressionDetector {
    pub fn new(target_index: usize, proxy_indices: Vec<usize>, suppression_threshold: f64) -> Self {
        Self {
            target_index,
            proxy_indices,
            suppression_threshold,
        }
    }
}

impl DiagnosticDetector for ProxySuppressionDetector {
    fn evaluate(&self, observations: &[SearchObservation]) -> DiagnosticResult {
        let mut total_rejected_improving = 0;
        let mut degraded_proxy_count = 0;

        for obs in observations {
            for c in &obs.candidates {
                if let Some(ref parent) = c.parent_objectives {
                    let delta_target =
                        parent.values[self.target_index] - c.objectives.values[self.target_index];
                    if delta_target > 0.0 && !c.admitted {
                        total_rejected_improving += 1;
                        let mut degraded = false;
                        for &p_idx in &self.proxy_indices {
                            let delta_proxy = parent.values[p_idx] - c.objectives.values[p_idx];
                            if delta_proxy < 0.0 {
                                degraded = true;
                                break;
                            }
                        }
                        if degraded {
                            degraded_proxy_count += 1;
                        }
                    }
                }
            }
        }

        let evidence_count = total_rejected_improving;
        if evidence_count == 0 {
            return DiagnosticResult {
                confidence: 0.0,
                severity: 0.0,
                evidence_count,
                supporting_metrics: vec![Metric::new("suppression_ratio", 0.0)],
            };
        }

        let suppression_ratio = degraded_proxy_count as f64 / total_rejected_improving as f64;
        let confidence = (suppression_ratio / self.suppression_threshold).clamp(0.0, 1.0);
        let severity = suppression_ratio * confidence;

        DiagnosticResult {
            confidence,
            severity,
            evidence_count,
            supporting_metrics: vec![
                Metric::new("suppression_ratio", suppression_ratio),
                Metric::new(
                    "rejected_target_improving_count",
                    total_rejected_improving as f64,
                ),
                Metric::new("degraded_proxy_count", degraded_proxy_count as f64),
            ],
        }
    }
}

/// 6. OperatorExpressivenessFailureDetector
/// Diagnoses if search operators are producing diverse solutions but no improving candidates over a long period.
#[derive(Debug, Clone)]
pub struct OperatorExpressivenessFailureDetector {
    pub stagnation_generations: usize,
    pub diversity_threshold: f64,
}

impl OperatorExpressivenessFailureDetector {
    pub fn new(stagnation_generations: usize, diversity_threshold: f64) -> Self {
        Self {
            stagnation_generations,
            diversity_threshold,
        }
    }
}

impl Default for OperatorExpressivenessFailureDetector {
    fn default() -> Self {
        Self::new(100, 0.7)
    }
}

impl DiagnosticDetector for OperatorExpressivenessFailureDetector {
    fn evaluate(&self, observations: &[SearchObservation]) -> DiagnosticResult {
        let evidence_count = observations.len();
        if evidence_count < self.stagnation_generations {
            return DiagnosticResult {
                confidence: 0.0,
                severity: 0.0,
                evidence_count,
                supporting_metrics: Vec::new(),
            };
        }

        let slice = &observations[evidence_count - self.stagnation_generations..];

        let mut improving_count = 0;
        let mut sum_diversity = 0.0;

        for obs in slice {
            sum_diversity += obs.diversity_score;
            for c in &obs.candidates {
                if c.admitted {
                    improving_count += 1;
                }
            }
        }

        let avg_diversity = sum_diversity / slice.len() as f64;

        let confidence = if improving_count == 0 && avg_diversity >= self.diversity_threshold {
            1.0
        } else {
            0.0
        };

        DiagnosticResult {
            confidence,
            severity: confidence,
            evidence_count: slice.len(),
            supporting_metrics: vec![
                Metric::new("improving_candidates_in_window", improving_count as f64),
                Metric::new("average_diversity_in_window", avg_diversity),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pearson_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        assert!((pearson_correlation(&x, &y) - 1.0).abs() < 1e-6);

        let y_neg = vec![-2.0, -4.0, -6.0, -8.0, -10.0];
        assert!((pearson_correlation(&x, &y_neg) - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_gini_coefficient() {
        // Equal distribution
        let val_equal = vec![10.0, 10.0, 10.0];
        assert!(gini_coefficient(&val_equal).abs() < 1e-6);

        // Unequal distribution
        let val_unequal = vec![0.0, 0.0, 10.0];
        // Sorted: 0, 0, 10. Sum: 10. Weighted sum: 1*0 + 2*0 + 3*10 = 30.
        // Gini = 2*30 / (3 * 10) - 4 / 3 = 60/30 - 1.33333 = 2.0 - 1.33333 = 0.666667
        assert!((gini_coefficient(&val_unequal) - 0.666667).abs() < 1e-5);
    }

    #[test]
    fn test_attractor_detector() {
        let detector = AttractorDetector::new(0.3, 0.2);

        // Mock candidates where Obj 1 has high improvement, Obj 0 has low/no improvement.
        // Admitted is 1.0 when Obj 1 improves, 0.0 otherwise.
        let mut candidates = Vec::new();
        for i in 0..10 {
            let (obj1_imp, admitted) = if i % 2 == 0 {
                (10.0, true)
            } else {
                (0.0, false)
            };
            candidates.push(CandidateObservation {
                objectives: ObjectiveVector::new(vec![5.0, 10.0 - obj1_imp]),
                admitted,
                feasible: true,
                parent_objectives: Some(ObjectiveVector::new(vec![5.0, 10.0])),
            });
        }

        let obs = SearchObservation {
            generation: 1,
            archive_size: 5,
            diversity_score: 0.5,
            candidates,
            archive_objectives: vec![],
            telemetry: None,
        };

        let result = detector.evaluate(&[obs]);
        assert!(
            result.confidence > 0.8,
            "Confidence was {}",
            result.confidence
        );
        assert!(result.severity > 0.8, "Severity was {}", result.severity);

        // The attractor index should point to objective 1
        let attractor_idx = result
            .supporting_metrics
            .iter()
            .find(|m| m.name == "attractor_index")
            .unwrap()
            .value;
        assert_eq!(attractor_idx, 1.0);
    }

    #[test]
    fn test_tradeoff_basin_detector() {
        let detector = TradeoffBasinDetector::new(0, 1, -0.5);

        // When X improves (parent - child > 0), Y always degrades (parent - child < 0).
        let mut candidates = Vec::new();
        for i in 1..11 {
            let val = i as f64;
            candidates.push(CandidateObservation {
                objectives: ObjectiveVector::new(vec![10.0 - val, 10.0 + val]),
                admitted: true,
                feasible: true,
                parent_objectives: Some(ObjectiveVector::new(vec![10.0, 10.0])),
            });
        }

        let obs = SearchObservation {
            generation: 1,
            archive_size: 5,
            diversity_score: 0.5,
            candidates,
            archive_objectives: vec![],
            telemetry: None,
        };

        let result = detector.evaluate(&[obs]);
        assert!(
            result.confidence > 0.9,
            "Confidence was {}",
            result.confidence
        );
        assert!(result.severity > 0.9, "Severity was {}", result.severity);
    }

    #[test]
    fn test_ecology_lock_in_detector() {
        let detector = EcologyLockInDetector::new(0, 0.15, 3);

        // Create stagnant history (target score stays exactly 5.0)
        // and archive with low diversity (all solutions have target score 5.0, so gini = 0.0)
        let mut observations = Vec::new();
        for g in 0..5 {
            observations.push(SearchObservation {
                generation: g,
                archive_size: 3,
                diversity_score: 0.1,
                candidates: vec![],
                archive_objectives: vec![
                    ObjectiveVector::new(vec![5.0, 10.0]),
                    ObjectiveVector::new(vec![5.0, 11.0]),
                    ObjectiveVector::new(vec![5.0, 9.0]),
                ],
                telemetry: None,
            });
        }

        let result = detector.evaluate(&observations);
        assert!(
            result.confidence > 0.9,
            "Confidence was {}",
            result.confidence
        );
        assert!(result.severity > 0.9, "Severity was {}", result.severity);
    }

    #[test]
    fn test_accumulation_failure_detector() {
        let detector = AccumulationFailureDetector::new(0, 0.75);

        // Target improved candidates (5.0 -> 4.0) are rejected (admitted = false)
        let mut candidates = Vec::new();
        for i in 0..10 {
            let admitted = i == 0; // 90% rejection rate
            candidates.push(CandidateObservation {
                objectives: ObjectiveVector::new(vec![4.0, 10.0]),
                admitted,
                feasible: true,
                parent_objectives: Some(ObjectiveVector::new(vec![5.0, 10.0])),
            });
        }

        let obs = SearchObservation {
            generation: 1,
            archive_size: 5,
            diversity_score: 0.5,
            candidates,
            archive_objectives: vec![],
            telemetry: None,
        };

        let result = detector.evaluate(&[obs]);
        assert!(
            result.confidence > 0.9,
            "Confidence was {}",
            result.confidence
        );
        assert!(result.severity > 0.8, "Severity was {}", result.severity);
    }

    #[test]
    fn test_proxy_suppression_detector() {
        let detector = ProxySuppressionDetector::new(0, vec![1], 0.8);

        // Target improves (5.0 -> 4.0), rejected (admitted = false), and proxy degrades (10.0 -> 12.0)
        let mut candidates = Vec::new();
        for _ in 0..10 {
            candidates.push(CandidateObservation {
                objectives: ObjectiveVector::new(vec![4.0, 12.0]),
                admitted: false,
                feasible: true,
                parent_objectives: Some(ObjectiveVector::new(vec![5.0, 10.0])),
            });
        }

        let obs = SearchObservation {
            generation: 1,
            archive_size: 5,
            diversity_score: 0.5,
            candidates,
            archive_objectives: vec![],
            telemetry: None,
        };

        let result = detector.evaluate(&[obs]);
        assert!(
            result.confidence > 0.9,
            "Confidence was {}",
            result.confidence
        );
        assert!(result.severity > 0.9, "Severity was {}", result.severity);
    }

    #[test]
    fn test_ecology_state_history() {
        let mut state = EcologyState::new(3);
        assert_eq!(state.history.len(), 0);

        // Record some dummy entries
        let mut results1 = std::collections::HashMap::new();
        results1.insert(
            "AccumulationFailure".to_string(),
            DiagnosticResult {
                confidence: 0.8,
                severity: 0.5,
                evidence_count: 10,
                supporting_metrics: vec![],
            },
        );
        state.record(1, results1);

        let mut results2 = std::collections::HashMap::new();
        results2.insert(
            "AccumulationFailure".to_string(),
            DiagnosticResult {
                confidence: 0.9,
                severity: 0.7,
                evidence_count: 12,
                supporting_metrics: vec![],
            },
        );
        state.record(2, results2);

        let mut results3 = std::collections::HashMap::new();
        results3.insert(
            "AccumulationFailure".to_string(),
            DiagnosticResult {
                confidence: 0.4,
                severity: 0.3,
                evidence_count: 8,
                supporting_metrics: vec![],
            },
        );
        state.record(3, results3);

        assert_eq!(state.history.len(), 3);

        // Overwrite oldest due to limit of 3
        let mut results4 = std::collections::HashMap::new();
        results4.insert(
            "AccumulationFailure".to_string(),
            DiagnosticResult {
                confidence: 0.5,
                severity: 0.2,
                evidence_count: 6,
                supporting_metrics: vec![],
            },
        );
        state.record(4, results4);

        assert_eq!(state.history.len(), 3);
        assert_eq!(state.history[0].generation, 2); // gen 1 got dropped

        // Check math
        // History contains gen 2 (conf 0.9, sev 0.7), gen 3 (conf 0.4, sev 0.3), gen 4 (conf 0.5, sev 0.2)
        // Mean confidence over window 2: (0.4 + 0.5)/2 = 0.45
        assert!((state.mean_confidence("AccumulationFailure", 2) - 0.45).abs() < 1e-6);
        // Mean severity over window 3: (0.7 + 0.3 + 0.2)/3 = 0.4
        assert!((state.mean_severity("AccumulationFailure", 3) - 0.4).abs() < 1e-6);
        // Persistence score over window 3 with threshold 0.5: gen 2 (0.9 >= 0.5), gen 3 (0.4 < 0.5), gen 4 (0.5 >= 0.5) -> 2 out of 3 = 0.666667
        assert!((state.persistence_score("AccumulationFailure", 0.5, 3) - 0.666667).abs() < 1e-5);
    }
}
