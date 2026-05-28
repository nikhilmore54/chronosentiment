pub use chronosentiment_optimization::Candidate as Strategy;
pub use chronosentiment_optimization::CandidateEvaluation as StrategyEvaluation;

pub fn strategy_to_id(strategy: &Strategy) -> String {
    format!(
        "STRAT_{}v{}v{}v{}v{}v{}v{}v{}v{}v{}v{}v{}v{}v{}v{}v{}v{}v{}v{}",
        strategy.queue_threshold,
        strategy.base_edge,
        strategy.take_profit,
        strategy.stop_loss,
        strategy.holding_period,
        strategy.w_conviction,
        strategy.w_momentum,
        strategy.w_volatility,
        strategy.exp_conviction,
        strategy.exp_momentum,
        strategy.exp_volatility,
        strategy.selectivity,
        strategy.archetype,
        strategy.entry_offset,
        strategy.direction_bias,
        strategy.vol_floor,
        strategy.mom_floor,
        strategy.edge_ratio,
        strategy.participation_threshold,
    )
}
