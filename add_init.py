import re

with open("financial/strategies/src/market_regime.rs", "r") as f:
    content = f.read()

new_content = content + """

pub fn initialize_ga_delegates() {
    chronosentiment_optimization::init_domain_delegates(chronosentiment_optimization::DomainDelegates {
        detect_scenario: detect_market_regime,
        scenario_multiplier: regime_multiplier,
        classify_behavior: classify_direction_bias,
        calculate_efficiency: calculate_capture_efficiency,
        classify_efficiency: classify_efficiency,
    });
}
"""

with open("financial/strategies/src/market_regime.rs", "w") as f:
    f.write(new_content)

print("Added initialization.")
