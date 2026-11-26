use chronosentiment_optimization::{Candidate, FitnessEvaluator};
use chronosentiment_strategies::evaluation::evaluator::FinancialEvaluator;

use crate::inspect_projection::run_inspect_simulation;

use super::attestation::{attest_domain_result, DomainMaterialization};
use super::registry::ScenarioRegistry;
use super::result::{ReplayStatus, ScenarioResult};

fn default_evaluator() -> FinancialEvaluator {
    FinancialEvaluator::new("BTC".to_string(), "default".to_string())
}

fn from_financial_eval(
    domain: &super::domain::ScenarioDomain,
    candidate: &Candidate,
) -> ScenarioResult {
    attest_domain_result(domain, || {
        let evaluator = default_evaluator();
        let eval = evaluator.evaluate(candidate);
        let execution_fitness = (eval.fitness / 100.0).clamp(0.0, 1.0);
        DomainMaterialization {
            scenario_id: domain.id.clone(),
            fitness: eval.fitness,
            execution_fitness,
            avg_pnl: eval.avg_pnl,
            std_dev: eval.std_dev,
            max_drawdown: eval.max_drawdown,
            trade_count: eval.trade_count,
            replay_status: if eval.evaluation_valid {
                ReplayStatus::Valid
            } else {
                ReplayStatus::Invalid
            },
            domain_class: domain.domain_class,
            events: Vec::new(),
        }
    })
}

fn from_execution_simulation(
    domain: &super::domain::ScenarioDomain,
    candidate: &Candidate,
    seed: u64,
) -> ScenarioResult {
    attest_domain_result(domain, || {
        let (sim, _) = run_inspect_simulation(candidate, seed);
        let filled = sim
            .order_outcomes
            .values()
            .map(|o| o.filled_quantity)
            .sum::<u64>();
        let total_qty = sim
            .order_outcomes
            .values()
            .map(|o| o.filled_quantity + o.remaining_quantity)
            .sum::<u64>()
            .max(1);

        let fill_rate = filled as f64 / total_qty as f64;
        let pnl_component = (sim.pnl.max(0) as f64 / 10_000.0).clamp(0.0, 1.0);
        let execution_fitness = (0.6 * fill_rate + 0.4 * pnl_component).clamp(0.0, 1.0);

        DomainMaterialization {
            scenario_id: domain.id.clone(),
            fitness: execution_fitness * 100.0,
            execution_fitness,
            avg_pnl: sim.pnl as f64 / 10_000.0,
            std_dev: 0.0,
            max_drawdown: 0.0,
            trade_count: sim.trades as usize,
            replay_status: if sim.events.is_empty() {
                ReplayStatus::Invalid
            } else {
                ReplayStatus::Valid
            },
            domain_class: domain.domain_class,
            events: sim.events,
        }
    })
}

/// Materialize per-domain results across all eligible registry entries.
pub fn evaluate_strategy_across_domains(
    registry: &ScenarioRegistry,
    candidate: &Candidate,
    seed: u64,
) -> Vec<ScenarioResult> {
    registry
        .list_eligible()
        .iter()
        .map(|domain| match domain.id.as_str() {
            "deterministic_demo" => from_financial_eval(domain, candidate),
            "deterministic_demo_execution" => from_execution_simulation(domain, candidate, seed),
            _ => ScenarioResult::unattested_stub(
                domain.id.clone(),
                domain.domain_class,
                domain.substrate_source.reference.clone(),
            ),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use chronosentiment_optimization::Candidate;

    use crate::scenario::{AttestationStatus, ReplayStatus, ScenarioRegistry};

    use super::evaluate_strategy_across_domains;

    #[test]
    fn materializes_results_for_all_eligible_domains() {
        let registry = ScenarioRegistry::v1_default();
        let results = evaluate_strategy_across_domains(&registry, &Candidate::default(), 42);
        assert_eq!(results.len(), 2);
        assert!(results
            .iter()
            .all(|r| r.replay_status == ReplayStatus::Valid));
    }

    #[test]
    fn execution_domain_is_result_attested_on_re_run() {
        let registry = ScenarioRegistry::v1_default();
        let results = evaluate_strategy_across_domains(&registry, &Candidate::default(), 42);
        let execution = results
            .iter()
            .find(|r| r.scenario_id == "deterministic_demo_execution")
            .expect("execution domain present");

        assert_eq!(
            execution.attestation_status,
            AttestationStatus::ResultAttested
        );
        assert!(execution.event_count > 0);
        assert_eq!(execution.expected_event_hash.len(), 64);
        assert_eq!(execution.result_hash.len(), 64);
        assert_eq!(
            execution.substrate_reference,
            "deterministic_demo_v1_execution_path"
        );
    }

    #[test]
    fn financial_domain_without_events_stays_unattested() {
        let registry = ScenarioRegistry::v1_default();
        let results = evaluate_strategy_across_domains(&registry, &Candidate::default(), 42);
        let financial = results
            .iter()
            .find(|r| r.scenario_id == "deterministic_demo")
            .expect("financial domain present");

        assert_eq!(financial.attestation_status, AttestationStatus::Unattested);
        assert_eq!(financial.event_count, 0);
    }

    #[test]
    fn consecutive_evaluations_produce_identical_execution_digests() {
        let registry = ScenarioRegistry::v1_default();
        let candidate = Candidate::default();
        let first = evaluate_strategy_across_domains(&registry, &candidate, 42);
        let second = evaluate_strategy_across_domains(&registry, &candidate, 42);

        let a = first
            .iter()
            .find(|r| r.scenario_id == "deterministic_demo_execution")
            .unwrap();
        let b = second
            .iter()
            .find(|r| r.scenario_id == "deterministic_demo_execution")
            .unwrap();

        assert_eq!(a.expected_event_hash, b.expected_event_hash);
        assert_eq!(a.result_hash, b.result_hash);
    }
}
