//! Phase D attestation binding for `ScenarioResult`.
//! See `docs/contracts/REPLAY_ATTESTATION_CONTRACT_v1.md`.

use chrono::Utc;
use chronosentiment_core::SimEvent;

use crate::signatures::{compute_event_stream_hash, compute_scenario_result_hash};

use super::domain::{DomainClass, ScenarioDomain};
use super::result::{AttestationStatus, ScenarioResult};

const ENGINE_MODE: &str = "REAL";

#[derive(Debug, Clone)]
pub struct DomainMaterialization {
    pub scenario_id: String,
    pub fitness: f64,
    pub execution_fitness: f64,
    pub avg_pnl: f64,
    pub std_dev: f64,
    pub max_drawdown: f64,
    pub trade_count: usize,
    pub replay_status: super::result::ReplayStatus,
    pub domain_class: DomainClass,
    pub events: Vec<SimEvent>,
}

fn domain_class_label(class: DomainClass) -> &'static str {
    match class {
        DomainClass::CertifiedFixture => "CERTIFIED_FIXTURE",
        DomainClass::HistoricalSlice => "HISTORICAL_SLICE",
        DomainClass::SyntheticRegime => "SYNTHETIC_REGIME",
        DomainClass::Holdout => "HOLDOUT",
    }
}

fn result_hash_for(materialization: &DomainMaterialization) -> String {
    compute_scenario_result_hash(
        &materialization.scenario_id,
        domain_class_label(materialization.domain_class),
        ENGINE_MODE,
        materialization.fitness,
        materialization.execution_fitness,
        materialization.avg_pnl,
        materialization.std_dev,
        materialization.max_drawdown,
        materialization.trade_count,
    )
}

/// Bind event stream + scored fields into an attested `ScenarioResult`.
///
/// Runs the domain materializer twice; stable digests → `ResultAttested`, else `Diverged`.
/// Domains without an event stream remain `Unattested` (no false event certification).
pub fn attest_domain_result(
    domain: &ScenarioDomain,
    materialize: impl Fn() -> DomainMaterialization,
) -> ScenarioResult {
    let substrate_reference = domain.substrate_source.reference.clone();
    let first = materialize();
    let second = materialize();

    let expected_event_hash = compute_event_stream_hash(
        &first.events,
        &substrate_reference,
        ENGINE_MODE,
    );
    let result_hash = result_hash_for(&first);

    let event_hash_second = compute_event_stream_hash(
        &second.events,
        &substrate_reference,
        ENGINE_MODE,
    );
    let result_hash_second = result_hash_for(&second);

    let attestation_status = if first.events.is_empty() {
        AttestationStatus::Unattested
    } else if expected_event_hash != event_hash_second {
        AttestationStatus::Diverged
    } else if result_hash != result_hash_second {
        AttestationStatus::Diverged
    } else {
        AttestationStatus::ResultAttested
    };

    ScenarioResult {
        scenario_id: first.scenario_id,
        fitness: first.fitness,
        execution_fitness: first.execution_fitness,
        avg_pnl: first.avg_pnl,
        std_dev: first.std_dev,
        max_drawdown: first.max_drawdown,
        trade_count: first.trade_count,
        replay_status: first.replay_status,
        domain_class: first.domain_class,
        substrate_reference,
        event_count: first.events.len(),
        expected_event_hash,
        result_hash,
        attestation_status,
        attestation_timestamp: Utc::now().to_rfc3339(),
        engine_mode: ENGINE_MODE.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::domain::SubstrateKind;
    use crate::scenario::result::ReplayStatus;

    #[test]
    fn empty_event_stream_is_unattested_not_diverged() {
        let domain = ScenarioDomain {
            id: "test".to_string(),
            substrate_source: super::super::domain::SubstrateSource {
                kind: SubstrateKind::Fixture,
                reference: "fixture_v1".to_string(),
                version: Some("v1".to_string()),
            },
            domain_class: DomainClass::CertifiedFixture,
            evaluation_eligible: true,
        };

        let result = attest_domain_result(&domain, || DomainMaterialization {
            scenario_id: "test".to_string(),
            fitness: 1.0,
            execution_fitness: 0.01,
            avg_pnl: 0.0,
            std_dev: 0.0,
            max_drawdown: 0.0,
            trade_count: 0,
            replay_status: ReplayStatus::Valid,
            domain_class: DomainClass::CertifiedFixture,
            events: Vec::new(),
        });

        assert_eq!(result.attestation_status, AttestationStatus::Unattested);
        assert_eq!(result.event_count, 0);
    }
}
