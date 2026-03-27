use crate::ga::Strategy;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LiveRegime {
    TrendingUp,
    TrendingDown,
    Sideways,
    Volatile,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveMarketState {
    pub asset: String,
    pub price: f64,
    pub confidence: f64,
    pub expected_edge: f64,
    pub execution_score: f64,
    pub regime: LiveRegime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyProfile {
    pub strategy_id: String,
    pub strategy: Strategy,
    pub preferred_regimes: Vec<LiveRegime>,
    /// Historical confidence multiplier in [0,1].
    pub confidence_weight: f64,
    /// Historical execution multiplier in [0,1].
    pub execution_weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedStrategy {
    pub strategy_id: String,
    pub action: String,
    pub confidence: f64,
    pub expected_edge: f64,
    pub execution_score: f64,
    pub live_score: f64,
}

#[derive(Debug, Clone)]
pub struct StrategyRegistry {
    strategies: Vec<StrategyProfile>,
}

impl StrategyRegistry {
    pub fn new(strategies: Vec<StrategyProfile>) -> Self {
        Self { strategies }
    }

    pub fn len(&self) -> usize {
        self.strategies.len()
    }

    pub fn is_empty(&self) -> bool {
        self.strategies.is_empty()
    }

    /// Deterministic live ranking over pre-trained strategy library.
    /// No GA/search here: this is selection only.
    pub fn rank_live(
        &self,
        state: &LiveMarketState,
        top_k: usize,
        weights: RankingWeights,
    ) -> Vec<RankedStrategy> {
        if top_k == 0 || self.strategies.is_empty() {
            return Vec::new();
        }

        let mut rows: Vec<RankedStrategy> = self
            .strategies
            .iter()
            .map(|profile| {
                let regime_fit = if profile.preferred_regimes.contains(&state.regime) {
                    1.0
                } else {
                    0.5
                };
                let confidence_component =
                    state.confidence.clamp(0.0, 1.0) * profile.confidence_weight.clamp(0.0, 1.0);
                let execution_component =
                    state.execution_score.clamp(0.0, 1.0) * profile.execution_weight.clamp(0.0, 1.0);
                let edge_component = edge_to_unit(state.expected_edge);
                let live_score = weights.edge * edge_component
                    + weights.execution * execution_component
                    + weights.confidence * confidence_component
                    + weights.regime_fit * regime_fit;
                RankedStrategy {
                    strategy_id: profile.strategy_id.clone(),
                    action: infer_action(state, profile),
                    confidence: state.confidence.clamp(0.0, 1.0),
                    expected_edge: state.expected_edge,
                    execution_score: state.execution_score.clamp(0.0, 1.0),
                    live_score,
                }
            })
            .collect();

        rows.sort_by(|a, b| {
            b.live_score
                .partial_cmp(&a.live_score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| b.expected_edge.partial_cmp(&a.expected_edge).unwrap_or(Ordering::Equal))
                .then_with(|| a.strategy_id.cmp(&b.strategy_id))
        });
        rows.truncate(top_k.min(rows.len()));
        rows
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RankingWeights {
    pub edge: f64,
    pub execution: f64,
    pub confidence: f64,
    pub regime_fit: f64,
}

impl Default for RankingWeights {
    fn default() -> Self {
        Self {
            edge: 0.45,
            execution: 0.25,
            confidence: 0.20,
            regime_fit: 0.10,
        }
    }
}

fn edge_to_unit(edge: f64) -> f64 {
    // Deterministic bounded transform around realistic intraday edge ranges.
    (edge / 0.01).clamp(-1.0, 1.0).max(0.0)
}

fn infer_action(state: &LiveMarketState, profile: &StrategyProfile) -> String {
    if state.expected_edge <= 0.0 || state.confidence < 0.05 {
        return "HOLD".to_string();
    }
    if profile.strategy.take_profit >= profile.strategy.stop_loss {
        "BUY".to_string()
    } else {
        "SELL".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strat(id: &str, tp: u64, sl: u64, regimes: Vec<LiveRegime>) -> StrategyProfile {
        StrategyProfile {
            strategy_id: id.to_string(),
            strategy: Strategy {
                queue_threshold: 100,
                base_edge: 2,
                take_profit: tp,
                stop_loss: sl,
            },
            preferred_regimes: regimes,
            confidence_weight: 0.8,
            execution_weight: 0.9,
        }
    }

    #[test]
    fn ranks_strategies_deterministically() {
        let reg = StrategyRegistry::new(vec![
            strat("strat_b", 8, 4, vec![LiveRegime::TrendingUp]),
            strat("strat_a", 8, 4, vec![LiveRegime::TrendingUp]),
        ]);
        let state = LiveMarketState {
            asset: "BTCUSDT".to_string(),
            price: 65000.0,
            confidence: 0.7,
            expected_edge: 0.004,
            execution_score: 0.8,
            regime: LiveRegime::TrendingUp,
        };
        let ranked = reg.rank_live(&state, 2, RankingWeights::default());
        assert_eq!(ranked.len(), 2);
        // Tie-breaker is strategy_id asc.
        assert_eq!(ranked[0].strategy_id, "strat_a");
        assert_eq!(ranked[1].strategy_id, "strat_b");
    }

    #[test]
    fn returns_hold_when_edge_non_positive() {
        let reg = StrategyRegistry::new(vec![strat("s1", 10, 5, vec![LiveRegime::Mixed])]);
        let state = LiveMarketState {
            asset: "BTCUSDT".to_string(),
            price: 65000.0,
            confidence: 0.8,
            expected_edge: -0.001,
            execution_score: 0.9,
            regime: LiveRegime::Mixed,
        };
        let ranked = reg.rank_live(&state, 1, RankingWeights::default());
        assert_eq!(ranked[0].action, "HOLD");
    }
}
