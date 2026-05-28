//! API compatibility parsing for strategy IDs.

use chronosentiment_strategies::{parse_strategy_id_with_compatibility, AdmissibilityResult, Strategy};

/// Returns `(genome, optional scenario key)` for inspect/compare.
pub fn parse_strategy_id_full(id: &str) -> Result<(Strategy, Option<String>), String> {
    let observation = parse_strategy_id_with_compatibility(id);

    if observation.admissibility == AdmissibilityResult::Accepted {
        observation
            .parsed_strategy
            .map(|strategy| (strategy, None))
            .ok_or_else(|| format!("Could not parse strategy parameters from strategy_id: {}", id))
    } else {
        Err(observation
            .error
            .unwrap_or_else(|| format!("Could not parse strategy parameters from strategy_id: {}", id)))
    }
}
