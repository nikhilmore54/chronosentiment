//! CS-P-006-P.1 — Decision Observatory policy registry.
//!
//! Binds the sealed Search #2 artifact to a paper-only product label.
//! Does not generate decisions, evolve, or promote a strategy.

use serde::Serialize;

use super::c3_rule_ecology::{SEARCH_THREE_AUTHORIZED, SEARCH_TWO_PROMOTION_STATUS};
use super::csp006_protocol::{
    REGIME_PERSISTENCE_EXPERIMENT_AUTHORIZED, RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH,
};

pub const OBSERVATORY_REGISTRY_CONTRACT_ID: &str = "csp006p.policy_registry.1";
pub const CANDIDATE_C3_002: &str = "C3-002";
pub const CANDIDATE_C3_002_LABEL: &str =
    "ChronoSentiment Research Policy — Candidate C3-002";
pub const OBSERVATORY_P2_STARTED: bool = false;
pub const OBSERVATORY_VERTICAL_SLICE_STARTED: bool = true;

#[derive(Debug, Clone, Serialize)]
pub struct RegisteredPaperPolicy {
    pub contract_id: String,
    pub registry_id: String,
    pub label: String,
    pub artifact_hash: String,
    pub promotion_status: String,
    pub paper_only: bool,
    pub customer_facing_system: String,
    pub search_three_authorized: bool,
    pub regime_persistence_experiment_authorized: bool,
    pub real_capital_authorized: bool,
}

pub fn candidate_c3_002() -> RegisteredPaperPolicy {
    RegisteredPaperPolicy {
        contract_id: OBSERVATORY_REGISTRY_CONTRACT_ID.to_string(),
        registry_id: CANDIDATE_C3_002.to_string(),
        label: CANDIDATE_C3_002_LABEL.to_string(),
        artifact_hash: RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH.to_string(),
        promotion_status: SEARCH_TWO_PROMOTION_STATUS.to_string(),
        paper_only: true,
        customer_facing_system: "ChronoSentiment".to_string(),
        search_three_authorized: SEARCH_THREE_AUTHORIZED,
        regime_persistence_experiment_authorized: REGIME_PERSISTENCE_EXPERIMENT_AUTHORIZED,
        real_capital_authorized: false,
    }
}

pub fn register_paper_policy(artifact_hash: &str) -> Result<RegisteredPaperPolicy, String> {
    if artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("observatory P.1 identity-gates Candidate C3-002 to Search #2".into());
    }
    Ok(candidate_c3_002())
}
