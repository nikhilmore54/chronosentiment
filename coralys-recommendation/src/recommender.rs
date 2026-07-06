use coralys_ecology::diagnostics::EcologyState;
use serde::{Serialize, Deserialize};

/// An intervention recommendation proposed by search governance logic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InterventionRecommendation {
    pub action: String,
    pub rationale: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
}

/// Wrapped report containing a list of recommendations at a specific generation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecommendationReport {
    pub generated_at_generation: usize,
    pub recommendations: Vec<InterventionRecommendation>,
}

/// Recommendation engine that evaluates EcologyState history and suggests interventions.
pub struct EcologyRecommender {
    pub confidence_threshold: f64,
    pub evaluation_window: usize,
}

impl EcologyRecommender {
    pub fn new(confidence_threshold: f64, evaluation_window: usize) -> Self {
        Self {
            confidence_threshold,
            evaluation_window,
        }
    }

    pub fn recommend(&self, state: &EcologyState) -> RecommendationReport {
        let mut recommendations = Vec::new();
        let current_gen = state.history.last().map(|e| e.generation).unwrap_or(0);

        if state.history.is_empty() {
            return RecommendationReport {
                generated_at_generation: current_gen,
                recommendations,
            };
        }

        // 1. Accumulation Failure
        let name_accum = "AccumulationFailure";
        let mean_conf_accum = state.mean_confidence(name_accum, self.evaluation_window);
        let mean_sev_accum = state.mean_severity(name_accum, self.evaluation_window);
        if mean_conf_accum >= self.confidence_threshold {
            let mut evidence = vec![
                format!("accumulation_failure mean confidence = {:.4}", mean_conf_accum),
                format!("accumulation_failure mean severity = {:.4}", mean_sev_accum),
            ];
            if let Some(entry) = state.history.last() {
                if let Some(res) = entry.results.get(name_accum) {
                    for m in &res.supporting_metrics {
                        evidence.push(format!("latest {} = {:.4}", m.name, m.value));
                    }
                }
            }
            recommendations.push(InterventionRecommendation {
                action: "Reserve archive slots for target-improving solutions".to_string(),
                rationale: "Target-improving candidate solutions are being rejected at a high rate. Reserving dedicated archive capacity prevents proxy objectives from dominating selection.".to_string(),
                confidence: mean_conf_accum,
                evidence: evidence.clone(),
            });
            recommendations.push(InterventionRecommendation {
                action: "Reduce archive admission strictness".to_string(),
                rationale: "Allowing solutions with minor proxy degradation to be admitted temporarily gives target-improving solutions time to accumulate and mutate.".to_string(),
                confidence: mean_conf_accum,
                evidence,
            });
        }

        // 2. Attractor Objective
        let name_attr = "Attractor";
        let mean_conf_attr = state.mean_confidence(name_attr, self.evaluation_window);
        if mean_conf_attr >= self.confidence_threshold {
            let mut attractor_idx = 0;
            let mut evidence = vec![
                format!("attractor mean confidence = {:.4}", mean_conf_attr),
            ];
            if let Some(entry) = state.history.last() {
                if let Some(res) = entry.results.get(name_attr) {
                    for m in &res.supporting_metrics {
                        evidence.push(format!("latest {} = {:.4}", m.name, m.value));
                        if m.name == "attractor_index" {
                            attractor_idx = m.value as usize;
                        }
                    }
                }
            }
            recommendations.push(InterventionRecommendation {
                action: format!("Reduce selection pressure / preference weight for attractor objective (Index {})", attractor_idx),
                rationale: format!("Objective Index {} is dominating the Pareto archive geometry and pulling search towards it. Restricting its dominance allows search to progress along other axes.", attractor_idx),
                confidence: mean_conf_attr,
                evidence: evidence.clone(),
            });
            recommendations.push(InterventionRecommendation {
                action: "Increase mutation scale / reweighting".to_string(),
                rationale: "Introduce random search perturbations to break out of the attractor basin.".to_string(),
                confidence: mean_conf_attr,
                evidence,
            });
        }

        // 3. Ecology Lock-In
        let name_lockin = "EcologyLockIn";
        let mean_conf_lockin = state.mean_confidence(name_lockin, self.evaluation_window);
        if mean_conf_lockin >= self.confidence_threshold {
            let mut evidence = vec![
                format!("lock_in mean confidence = {:.4}", mean_conf_lockin),
            ];
            if let Some(entry) = state.history.last() {
                if let Some(res) = entry.results.get(name_lockin) {
                    for m in &res.supporting_metrics {
                        evidence.push(format!("latest {} = {:.4}", m.name, m.value));
                    }
                }
            }
            recommendations.push(InterventionRecommendation {
                action: "Increase mutation entropy / scaling".to_string(),
                rationale: "Archive diversity has collapsed and target score progress is stagnant. Increasing mutation entropy triggers high-entropy search diversification.".to_string(),
                confidence: mean_conf_lockin,
                evidence: evidence.clone(),
            });
            recommendations.push(InterventionRecommendation {
                action: "Inject random solutions into archive".to_string(),
                rationale: "Force search diversification by seeding new un-evolved candidates directly to break lock-in.".to_string(),
                confidence: mean_conf_lockin,
                evidence,
            });
        }

        // 4. Operator Expressiveness Failure
        let name_express = "OperatorExpressivenessFailure";
        let mean_conf_express = state.mean_confidence(name_express, self.evaluation_window);
        if mean_conf_express >= self.confidence_threshold {
            let mut evidence = vec![
                format!("expressiveness_failure mean confidence = {:.4}", mean_conf_express),
            ];
            if let Some(entry) = state.history.last() {
                if let Some(res) = entry.results.get(name_express) {
                    for m in &res.supporting_metrics {
                        evidence.push(format!("latest {} = {:.4}", m.name, m.value));
                    }
                }
            }
            recommendations.push(InterventionRecommendation {
                action: "Increase operator expressiveness".to_string(),
                rationale: "Current operators are producing diverse solutions but no improving candidates over a long period. Introduce larger neighborhood operators or reconstruction moves.".to_string(),
                confidence: mean_conf_express,
                evidence,
            });
        }

        RecommendationReport {
            generated_at_generation: current_gen,
            recommendations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use coralys_ecology::{DiagnosticResult, Metric};

    #[test]
    fn test_recommender_empty_state() {
        let state = EcologyState::new(10);
        let recommender = EcologyRecommender::new(0.5, 5);
        let report = recommender.recommend(&state);
        assert_eq!(report.generated_at_generation, 0);
        assert_eq!(report.recommendations.len(), 0);
    }

    #[test]
    fn test_recommender_accumulation_failure() {
        let mut state = EcologyState::new(10);
        let mut results = std::collections::HashMap::new();
        results.insert("AccumulationFailure".to_string(), DiagnosticResult {
            confidence: 0.8,
            severity: 0.6,
            evidence_count: 50,
            supporting_metrics: vec![Metric::new("rejection_rate", 0.8)],
        });
        state.record(100, results);

        let recommender = EcologyRecommender::new(0.5, 5);
        let report = recommender.recommend(&state);
        assert_eq!(report.generated_at_generation, 100);
        assert_eq!(report.recommendations.len(), 2);
        assert_eq!(report.recommendations[0].action, "Reserve archive slots for target-improving solutions");
        assert!(report.recommendations[0].evidence.contains(&"accumulation_failure mean confidence = 0.8000".to_string()));
        assert!(report.recommendations[0].evidence.contains(&"latest rejection_rate = 0.8000".to_string()));
    }

    #[test]
    fn test_recommender_attractor() {
        let mut state = EcologyState::new(10);
        let mut results = std::collections::HashMap::new();
        results.insert("Attractor".to_string(), DiagnosticResult {
            confidence: 0.9,
            severity: 0.4,
            evidence_count: 30,
            supporting_metrics: vec![Metric::new("attractor_index", 3.0)],
        });
        state.record(200, results);

        let recommender = EcologyRecommender::new(0.5, 5);
        let report = recommender.recommend(&state);
        assert_eq!(report.generated_at_generation, 200);
        assert_eq!(report.recommendations.len(), 2);
        assert!(report.recommendations[0].action.contains("attractor objective (Index 3)"));
    }

    #[test]
    fn test_recommender_lock_in() {
        let mut state = EcologyState::new(10);
        let mut results = std::collections::HashMap::new();
        results.insert("EcologyLockIn".to_string(), DiagnosticResult {
            confidence: 0.7,
            severity: 0.7,
            evidence_count: 10,
            supporting_metrics: vec![Metric::new("gini_coefficient", 0.05)],
        });
        state.record(300, results);

        let recommender = EcologyRecommender::new(0.5, 5);
        let report = recommender.recommend(&state);
        assert_eq!(report.generated_at_generation, 300);
        assert_eq!(report.recommendations.len(), 2);
        assert_eq!(report.recommendations[0].action, "Increase mutation entropy / scaling");
    }
}
