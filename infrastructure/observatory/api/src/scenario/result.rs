use serde::{Deserialize, Serialize};

use super::domain::DomainClass;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReplayStatus {
    Valid,
    Degraded,
    Invalid,
    Skipped,
}

/// Phase D attestation status for a domain materialization.
///
/// `ResultAttested` means event hash and result hash are stable on consecutive runs.
/// It does **not** imply economic validity, robustness in production, or strategy quality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AttestationStatus {
    Unattested,
    ResultAttested,
    Diverged,
}

/// Observational substrate for multi-domain evaluation (Phase C + Phase D attestation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScenarioResult {
    pub scenario_id: String,
    pub fitness: f64,
    pub execution_fitness: f64,
    pub avg_pnl: f64,
    pub std_dev: f64,
    pub max_drawdown: f64,
    pub trade_count: usize,
    pub replay_status: ReplayStatus,
    pub domain_class: DomainClass,
    /// Substrate provenance — diagnostic; not an input to hash verification.
    pub substrate_reference: String,
    pub event_count: usize,
    /// Event-grounded digest per `REPLAY_ATTESTATION_CONTRACT_v1.md` §6.1.
    pub expected_event_hash: String,
    /// Scored-field digest per §6.2.
    pub result_hash: String,
    pub attestation_status: AttestationStatus,
    /// RFC 3339 UTC — diagnostic only.
    pub attestation_timestamp: String,
    pub engine_mode: String,
}

impl ScenarioResult {
    pub fn unattested_stub(
        scenario_id: impl Into<String>,
        domain_class: DomainClass,
        substrate_reference: impl Into<String>,
    ) -> Self {
        Self {
            scenario_id: scenario_id.into(),
            fitness: 0.0,
            execution_fitness: 0.0,
            avg_pnl: 0.0,
            std_dev: 0.0,
            max_drawdown: 0.0,
            trade_count: 0,
            replay_status: ReplayStatus::Skipped,
            domain_class,
            substrate_reference: substrate_reference.into(),
            event_count: 0,
            expected_event_hash: String::new(),
            result_hash: String::new(),
            attestation_status: AttestationStatus::Unattested,
            attestation_timestamp: String::new(),
            engine_mode: "REAL".to_string(),
        }
    }
}
