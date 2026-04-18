use crate::ga::Strategy;
use crate::NormalizedMarketEvent;
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
    pub best_bid: Option<f64>,
    pub best_ask: Option<f64>,
    pub spread: f64,
    pub momentum: f64,
    pub volatility: f64,
    pub confidence: f64,
    pub expected_edge: f64,
    pub execution_score: f64,
    pub regime: LiveRegime,
}

impl LiveMarketState {
    pub fn new(asset: String) -> Self {
        Self {
            asset,
            price: 0.0,
            best_bid: None,
            best_ask: None,
            spread: 0.0,
            momentum: 0.0,
            volatility: 0.0,
            confidence: 0.0,
            expected_edge: 0.0,
            execution_score: 0.0,
            regime: LiveRegime::Mixed,
        }
    }

    /// Deterministic state update from normalized event feed.
    pub fn update_from_event(&mut self, event: &NormalizedMarketEvent) {
        let prev_price = self.price;
        if event.price.is_finite() && event.price > 0.0 {
            self.price = event.price;
        }
        self.best_bid = event.best_bid.or(self.best_bid);
        self.best_ask = event.best_ask.or(self.best_ask);
        if let (Some(bid), Some(ask)) = (self.best_bid, self.best_ask) {
            self.spread = (ask - bid).max(0.0);
        }
        if prev_price > 0.0 && self.price > 0.0 {
            let ret = (self.price / prev_price) - 1.0;
            // EWMA-like updates keep constant memory and deterministic behavior.
            self.momentum = 0.8 * self.momentum + 0.2 * ret;
            self.volatility = 0.9 * self.volatility + 0.1 * ret.abs();
            self.confidence = confidence_from_features(self.momentum, self.volatility);
            self.expected_edge = edge_from_features(self.momentum, self.spread, self.price);
            self.execution_score =
                execution_score_from_features(self.spread, self.volatility, self.price);
            self.regime = regime_from_features(self.momentum, self.volatility);
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SuggestionDebug {
    pub rejected_low_edge: usize,
    pub rejected_low_exec: usize,
    pub rejected_hold: usize,
    pub suppressed_stability: usize,
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
                let execution_component = state.execution_score.clamp(0.0, 1.0)
                    * profile.execution_weight.clamp(0.0, 1.0);
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
                .then_with(|| {
                    b.expected_edge
                        .partial_cmp(&a.expected_edge)
                        .unwrap_or(Ordering::Equal)
                })
                .then_with(|| a.strategy_id.cmp(&b.strategy_id))
        });
        rows.truncate(top_k.min(rows.len()));
        rows
    }
}

#[derive(Debug, Clone)]
pub struct LiveEvaluator {
    pub state: LiveMarketState,
    pub registry: StrategyRegistry,
    pub weights: RankingWeights,
    pub min_edge: f64,
    pub min_execution_score: f64,
    pub stability_delta: f64,
    last_top_score: Option<f64>,
    debug: SuggestionDebug,
}

impl LiveEvaluator {
    pub fn new(
        state: LiveMarketState,
        registry: StrategyRegistry,
        weights: RankingWeights,
    ) -> Self {
        Self {
            state,
            registry,
            weights,
            min_edge: 0.001,
            min_execution_score: 0.5,
            stability_delta: 0.01,
            last_top_score: None,
            debug: SuggestionDebug::default(),
        }
    }

    pub fn on_event(&mut self, event: &NormalizedMarketEvent, top_k: usize) -> Vec<RankedStrategy> {
        self.state.update_from_event(event);
        self.rank_current(top_k)
    }

    pub fn rank_current(&mut self, top_k: usize) -> Vec<RankedStrategy> {
        let ranked = self
            .registry
            .rank_live(&self.state, top_k.max(5), self.weights);
        let mut filtered: Vec<RankedStrategy> = Vec::new();
        for row in ranked {
            if row.action == "HOLD" {
                self.debug.rejected_hold += 1;
                continue;
            }
            if row.expected_edge < self.min_edge {
                self.debug.rejected_low_edge += 1;
                continue;
            }
            if row.execution_score < self.min_execution_score {
                self.debug.rejected_low_exec += 1;
                continue;
            }
            filtered.push(row);
        }
        filtered.sort_by(|a, b| {
            b.live_score
                .partial_cmp(&a.live_score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.strategy_id.cmp(&b.strategy_id))
        });
        filtered.truncate(top_k);

        let top_score = filtered.first().map(|x| x.live_score);
        if let (Some(prev), Some(cur)) = (self.last_top_score, top_score) {
            if (cur - prev).abs() < self.stability_delta {
                self.debug.suppressed_stability += 1;
                return Vec::new();
            }
        }
        self.last_top_score = top_score;
        filtered
    }

    pub fn debug_snapshot(&self) -> SuggestionDebug {
        self.debug.clone()
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

fn confidence_from_features(momentum: f64, volatility: f64) -> f64 {
    (momentum.abs() / (volatility + 1e-6)).clamp(0.0, 1.0)
}

fn edge_from_features(momentum: f64, spread: f64, price: f64) -> f64 {
    if price <= 0.0 {
        return 0.0;
    }
    let spread_penalty = (spread / price).clamp(0.0, 0.01);
    (momentum.abs() - spread_penalty).max(0.0)
}

fn execution_score_from_features(spread: f64, volatility: f64, price: f64) -> f64 {
    if price <= 0.0 {
        return 0.0;
    }
    let spread_norm = 1.0 - (spread / price * 200.0).clamp(0.0, 1.0);
    let vol_norm = 1.0 - (volatility * 25.0).clamp(0.0, 1.0);
    (0.7 * spread_norm + 0.3 * vol_norm).clamp(0.0, 1.0)
}

fn regime_from_features(momentum: f64, volatility: f64) -> LiveRegime {
    if volatility > 0.01 {
        LiveRegime::Volatile
    } else if momentum > 0.001 {
        LiveRegime::TrendingUp
    } else if momentum < -0.001 {
        LiveRegime::TrendingDown
    } else {
        LiveRegime::Sideways
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
                base_edge: 5,
                take_profit: 200,
                stop_loss: 100,
                holding_period: 20,
                w_conviction: 100,
                w_momentum: 80,
                w_volatility: 10,
                exp_conviction: 150,
                exp_momentum: 150,
                exp_volatility: 150,
                selectivity: 75,
                archetype: 0,
                direction_bias: 50,
                vol_floor: 20,
                mom_floor: 20,
                edge_ratio: 150,
                participation_threshold: 30,
            exec_aggression: 50, latency_bias: 10, fill_threshold: 50,
                entry_offset: 0, // TODO: replace with latency-derived offset
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
        let mut state = LiveMarketState::new("BTCUSDT".to_string());
        state.price = 65000.0;
        state.confidence = 0.7;
        state.expected_edge = 0.004;
        state.execution_score = 0.8;
        state.regime = LiveRegime::TrendingUp;
        let ranked = reg.rank_live(&state, 2, RankingWeights::default());
        assert_eq!(ranked.len(), 2);
        // Tie-breaker is strategy_id asc.
        assert_eq!(ranked[0].strategy_id, "strat_a");
        assert_eq!(ranked[1].strategy_id, "strat_b");
    }

    #[test]
    fn returns_hold_when_edge_non_positive() {
        let reg = StrategyRegistry::new(vec![strat("s1", 10, 5, vec![LiveRegime::Mixed])]);
        let mut state = LiveMarketState::new("BTCUSDT".to_string());
        state.price = 65000.0;
        state.confidence = 0.8;
        state.expected_edge = -0.001;
        state.execution_score = 0.9;
        state.regime = LiveRegime::Mixed;
        let ranked = reg.rank_live(&state, 1, RankingWeights::default());
        assert_eq!(ranked[0].action, "HOLD");
    }
}
