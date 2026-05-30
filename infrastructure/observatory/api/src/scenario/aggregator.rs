//! Isolated aggregation over `ScenarioResult[]`.
//! Future contract candidate — keep policy out of registry and handlers.

use crate::signatures::compute_aggregate_hash;

use super::result::{ReplayStatus, ScenarioResult};

/// Reducer identity for `aggregate_hash` (see `fixtures/contracts/scenario_registry.json`).
pub const ROBUST_MIN_REDUCER_ID: &str = "robust_min_execution_fitness";

#[derive(Debug, Clone, PartialEq)]
pub struct AggregatedEvaluation {
    pub aggregated_execution_fitness: f64,
    pub aggregated_fitness: f64,
    pub worst_case_execution_fitness: f64,
    pub domain_consistency: f64,
    pub domains_evaluated: usize,
    pub scenario_results: Vec<ScenarioResult>,
    /// Policy reducer used — input to `aggregate_hash`.
    pub reducer_id: String,
    /// Deterministic digest over sorted `(scenario_id, result_hash)` pairs.
    pub aggregate_hash: String,
}

pub struct ScenarioAggregator;

impl ScenarioAggregator {
    fn aggregate_hash_for(results: &[ScenarioResult]) -> String {
        let mut pairs: Vec<(&str, &str)> = results
            .iter()
            .map(|result| (result.scenario_id.as_str(), result.result_hash.as_str()))
            .collect();
        pairs.sort_by_key(|(scenario_id, _)| *scenario_id);
        compute_aggregate_hash(ROBUST_MIN_REDUCER_ID, &pairs)
    }

    /// Robustness reducer: worst-case execution fitness across valid domains.
    /// Prevents a single substrate from determining standing alone.
    pub fn robust_min(results: &[ScenarioResult]) -> AggregatedEvaluation {
        let aggregate_hash = Self::aggregate_hash_for(results);
        let reducer_id = ROBUST_MIN_REDUCER_ID.to_string();

        let valid: Vec<&ScenarioResult> = results
            .iter()
            .filter(|r| r.replay_status == ReplayStatus::Valid)
            .collect();

        let domains_evaluated = valid.len();
        let scenario_results = results.to_vec();

        if valid.is_empty() {
            return AggregatedEvaluation {
                aggregated_execution_fitness: 0.0,
                aggregated_fitness: 0.0,
                worst_case_execution_fitness: 0.0,
                domain_consistency: 0.0,
                domains_evaluated: 0,
                scenario_results,
                reducer_id,
                aggregate_hash,
            };
        }

        let exec_values: Vec<f64> = valid.iter().map(|r| r.execution_fitness).collect();
        let fit_values: Vec<f64> = valid.iter().map(|r| r.fitness).collect();

        let worst_case_execution_fitness = exec_values
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);

        let mean_exec = exec_values.iter().sum::<f64>() / exec_values.len() as f64;
        let mean_fit = fit_values.iter().sum::<f64>() / fit_values.len() as f64;
        let _ = mean_fit; // reserved for future mean-based reducers

        let domain_consistency = if exec_values.len() > 1 {
            let var = exec_values
                .iter()
                .map(|v| (v - mean_exec).powi(2))
                .sum::<f64>()
                / exec_values.len() as f64;
            var.sqrt()
        } else {
            0.0
        };

        AggregatedEvaluation {
            aggregated_execution_fitness: worst_case_execution_fitness,
            aggregated_fitness: fit_values.iter().copied().fold(f64::INFINITY, f64::min),
            worst_case_execution_fitness,
            domain_consistency,
            domains_evaluated,
            scenario_results,
            reducer_id,
            aggregate_hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::domain::DomainClass;

    fn sample_result(id: &str, exec: f64, result_hash: &str) -> ScenarioResult {
        ScenarioResult {
            scenario_id: id.to_string(),
            fitness: exec * 100.0,
            execution_fitness: exec,
            avg_pnl: 0.01,
            std_dev: 0.01,
            max_drawdown: 0.01,
            trade_count: 1,
            replay_status: ReplayStatus::Valid,
            domain_class: DomainClass::CertifiedFixture,
            substrate_reference: format!("{id}_substrate"),
            event_count: 1,
            expected_event_hash: "0".repeat(64),
            result_hash: result_hash.to_string(),
            attestation_status: crate::scenario::AttestationStatus::ResultAttested,
            attestation_timestamp: "2026-05-29T00:00:00Z".to_string(),
            engine_mode: "REAL".to_string(),
        }
    }

    #[test]
    fn robust_min_penalizes_single_domain_peak() {
        let results = vec![
            sample_result("deterministic_demo", 0.9, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            sample_result(
                "deterministic_demo_execution",
                0.4,
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            ),
        ];
        let agg = ScenarioAggregator::robust_min(&results);
        assert!((agg.aggregated_execution_fitness - 0.4).abs() < f64::EPSILON);
        assert_eq!(agg.domains_evaluated, 2);
        assert_eq!(agg.scenario_results.len(), 2);
        assert_eq!(agg.reducer_id, ROBUST_MIN_REDUCER_ID);
        assert_eq!(agg.aggregate_hash.len(), 64);
    }

    #[test]
    fn aggregate_hash_traces_standing_to_domain_result_hashes() {
        let results = vec![
            sample_result("deterministic_demo", 0.9, "hash_a"),
            sample_result("deterministic_demo_execution", 0.4, "hash_b"),
        ];
        let first = ScenarioAggregator::robust_min(&results).aggregate_hash;

        let mut mutated = results.clone();
        mutated[1].result_hash = "hash_c".to_string();
        let second = ScenarioAggregator::robust_min(&mutated).aggregate_hash;

        assert_ne!(first, second);
    }
}
