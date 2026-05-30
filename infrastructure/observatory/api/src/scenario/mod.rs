//! Scenario domain authority layer (Phase C).
//! See docs/contracts/SCENARIO_DOMAIN_CONTRACT_v1.md

mod aggregator;
mod attestation;
mod domain;
mod evaluate;
mod registry;
mod result;

pub use aggregator::{AggregatedEvaluation, ScenarioAggregator};
pub use domain::{DomainClass, ScenarioDomain, SubstrateKind, SubstrateSource};
pub use evaluate::evaluate_strategy_across_domains;
pub use registry::ScenarioRegistry;
pub use result::{AttestationStatus, ReplayStatus, ScenarioResult};
