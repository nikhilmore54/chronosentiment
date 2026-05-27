pub mod edge_decay;
pub mod edge_half_life_estimator;
pub mod evolution_engine;

pub use evolution_engine::*;

use std::sync::OnceLock;

pub struct DomainDelegates {
    pub detect_scenario: fn(f64, f64, f64, f64) -> ScenarioContext,
    pub scenario_multiplier: fn(ScenarioContext, BehavioralArchetype) -> f64,
    pub classify_behavior: fn(u8) -> BehavioralArchetype,
    pub calculate_efficiency: fn(f64, f64) -> f64,
    pub classify_efficiency: fn(f64) -> &'static str,
}

pub static DOMAIN_DELEGATES: OnceLock<DomainDelegates> = OnceLock::new();

pub fn init_domain_delegates(delegates: DomainDelegates) {
    let _ = DOMAIN_DELEGATES.set(delegates);
}

#[cfg(test)]
pub fn init_test_delegates() {
    let _ = DOMAIN_DELEGATES.set(DomainDelegates {
        detect_scenario: |_, _, _, _| ScenarioContext::MeanReversion,
        scenario_multiplier: |_, _| 1.0,
        classify_behavior: |_| BehavioralArchetype::DualCore,
        calculate_efficiency: |_, _| 1.0,
        classify_efficiency: |_| "GOOD",
    });
}
