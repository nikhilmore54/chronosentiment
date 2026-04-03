use crate::strategy_ranking::{LiveEvaluator, RankedStrategy, SuggestionDebug};
use crate::tick_replay::TickReplayEngine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionEvent {
    pub exchange_ts: u64,
    pub decision_ts: u64,
    pub execution_ts: u64,
    pub suggestions: Vec<RankedStrategy>,
    pub debug: SuggestionDebug,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReplayMetrics {
    pub total_events: usize,
    pub events_with_signals: usize,
    pub participation_pct: f64,
    pub avg_suggestions_per_event: f64,
    pub avg_suggestions_per_signal_event: f64,
    pub strategy_flips: usize,
    pub flip_rate: f64,
    pub effective_signals: f64,
    pub effective_signal_rate: f64,
    pub tradability_band: TradabilityBand,
    pub top_strategy_persistence_ticks: HashMap<String, usize>,
    pub final_debug: SuggestionDebug,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum TradabilityBand {
    #[default]
    Unusable,
    Sparse,
    Tradable,
    Strong,
    Overactive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayEvaluationResult {
    pub timeline: Vec<SuggestionEvent>,
    pub metrics: ReplayMetrics,
}

pub fn run_replay_with_evaluator(
    replay: &mut TickReplayEngine,
    evaluator: &mut LiveEvaluator,
    top_k: usize,
) -> ReplayEvaluationResult {
    let mut timeline: Vec<SuggestionEvent> = Vec::new();
    let mut total_events = 0usize;
    let mut signal_events = 0usize;
    let mut total_suggestions = 0usize;
    let mut flips = 0usize;
    let mut persistence: HashMap<String, usize> = HashMap::new();
    let mut prev_top: Option<String> = None;

    while let Some(replayed) = replay.next_event() {
        total_events += 1;
        let suggestions = evaluator.on_event(&replayed.event, top_k);
        if !suggestions.is_empty() {
            signal_events += 1;
            total_suggestions += suggestions.len();
            if let Some(top) = suggestions.first() {
                *persistence.entry(top.strategy_id.clone()).or_insert(0) += 1;
                if let Some(prev) = &prev_top {
                    if prev != &top.strategy_id {
                        flips += 1;
                    }
                }
                prev_top = Some(top.strategy_id.clone());
            }
        }
        timeline.push(SuggestionEvent {
            exchange_ts: replayed.event.exchange_ts,
            decision_ts: replayed.decision_ts,
            execution_ts: replayed.execution_ts,
            suggestions,
            debug: evaluator.debug_snapshot(),
        });
    }

    let participation_pct = if total_events == 0 {
        0.0
    } else {
        (signal_events as f64 / total_events as f64) * 100.0
    };
    let avg_suggestions_per_event = if total_events == 0 {
        0.0
    } else {
        total_suggestions as f64 / total_events as f64
    };
    let avg_suggestions_per_signal_event = if signal_events == 0 {
        0.0
    } else {
        total_suggestions as f64 / signal_events as f64
    };
    let flip_rate = if signal_events == 0 {
        0.0
    } else {
        flips as f64 / signal_events as f64
    };
    let effective_signals = (signal_events as f64) * (1.0 - flip_rate).clamp(0.0, 1.0);
    let effective_signal_rate = if total_events == 0 {
        0.0
    } else {
        effective_signals / total_events as f64
    };
    let tradability_band = match effective_signal_rate {
        x if x < 0.01 => TradabilityBand::Unusable,
        x if x < 0.05 => TradabilityBand::Sparse,
        x if x < 0.15 => TradabilityBand::Tradable,
        x if x < 0.30 => TradabilityBand::Strong,
        _ => TradabilityBand::Overactive,
    };

    ReplayEvaluationResult {
        timeline,
        metrics: ReplayMetrics {
            total_events,
            events_with_signals: signal_events,
            participation_pct,
            avg_suggestions_per_event,
            avg_suggestions_per_signal_event,
            strategy_flips: flips,
            flip_rate,
            effective_signals,
            effective_signal_rate,
            tradability_band,
            top_strategy_persistence_ticks: persistence,
            final_debug: evaluator.debug_snapshot(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ga::Strategy;
    use crate::strategy_ranking::{
        LiveMarketState, LiveRegime, RankingWeights, StrategyProfile, StrategyRegistry,
    };
    use crate::tick_replay::{ReplayConfig, ReplayMode};
    use crate::NormalizedMarketEvent;

    #[test]
    fn replay_metrics_populate() {
        let events = vec![
            NormalizedMarketEvent {
                asset: "TEST".to_string(),
                exchange_ts: 1,
                price: 100.0,
                volume: 1.0,
                side: None,
                best_bid: Some(99.9),
                best_ask: Some(100.1),
                bids: None,
                asks: None,
            },
            NormalizedMarketEvent {
                asset: "TEST".to_string(),
                exchange_ts: 2,
                price: 101.0,
                volume: 1.0,
                side: None,
                best_bid: Some(100.9),
                best_ask: Some(101.1),
                bids: None,
                asks: None,
            },
        ];
        let mut replay = TickReplayEngine::from_events(
            events,
            ReplayConfig {
                mode: ReplayMode::Fast,
                ..ReplayConfig::default()
            },
        );
        let registry = StrategyRegistry::new(vec![StrategyProfile {
            strategy_id: "s1".to_string(),
            strategy: Strategy {
                queue_threshold: 10,
                base_edge: 1,
                take_profit: 10,
                stop_loss: 5,
            },
            preferred_regimes: vec![LiveRegime::TrendingUp, LiveRegime::Mixed],
            confidence_weight: 1.0,
            execution_weight: 1.0,
        }]);
        let mut evaluator = crate::strategy_ranking::LiveEvaluator::new(
            LiveMarketState::new("BTCUSDT".to_string()),
            registry,
            RankingWeights::default(),
        );
        evaluator.min_edge = 0.0;
        evaluator.min_execution_score = 0.0;
        evaluator.stability_delta = 0.0;

        let out = run_replay_with_evaluator(&mut replay, &mut evaluator, 3);
        assert_eq!(out.metrics.total_events, 2);
        assert_eq!(out.timeline.len(), 2);
    }
}
