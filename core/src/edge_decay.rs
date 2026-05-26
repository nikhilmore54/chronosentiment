use crate::ga::Strategy;
use crate::pnl_overlay::{run_pnl_overlay_with_config, ExecutionConfig, ExecutionModel};
use crate::replay_evaluator::{run_replay_with_evaluator, ReplayMetrics, TradabilityBand};
use crate::strategy_ranking::{
    LiveEvaluator, LiveMarketState, LiveRegime, RankingWeights, StrategyProfile, StrategyRegistry,
};
use crate::tick_replay::{ReplayConfig, ReplayMode, TickReplayEngine};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDecayResult {
    pub model: ExecutionModel,
    pub avg_pnl: f64,
    pub total_pnl: f64,
    pub win_rate: f64,
    pub edge_retention: f64,
    pub effective_signal_rate: f64,
    pub tradability_band: TradabilityBand,
    pub edge_decay_pct: f64,
    pub debug_metrics: ReplayMetrics,
}

// Local helper to map regime string to LiveRegime enum
fn map_regime_local(regime: &str) -> LiveRegime {
    match regime.to_lowercase().as_str() {
        "trending_up" => LiveRegime::TrendingUp,
        "trending_down" => LiveRegime::TrendingDown,
        "volatile" => LiveRegime::Volatile,
        "sideways" => LiveRegime::Sideways,
        _ => LiveRegime::Sideways,
    }
}

// V-001: edge-decay is the first legacy lineage routed through canonical identity parsing.
fn parse_strategy_from_id_local(id: &str) -> Option<Strategy> {
    crate::strategy_id::parse_strategy_id(id).parsed_strategy
}

fn _parse_fallback(sig: &StrategyProfile) -> Strategy {
    parse_strategy_from_id_local(&sig.strategy_id).unwrap_or(Strategy {
        queue_threshold: 100,
        base_edge: 2,
        take_profit: 20,
        stop_loss: 10,
        holding_period: 0,
        w_conviction: 50,
        w_momentum: 30,
        w_volatility: 20,
        exp_conviction: 100,
        exp_momentum: 100,
        exp_volatility: 100,
        selectivity: 75,
        archetype: 0,
        direction_bias: 50,
        vol_floor: 20,
        mom_floor: 20,
        edge_ratio: 150,
        participation_threshold: 30,
        entry_offset: 0,
        exec_aggression: 50, latency_bias: 10, fill_threshold: 50, lineage: 0,
        })
}

fn build_evaluator_from_snapshot(
    snapshot_signals: &[crate::pipeline::TradeSignal],
    asset_name: String,
) -> LiveEvaluator {
    let mut registry_rows: Vec<StrategyProfile> = Vec::new();
    for sig in snapshot_signals {
        let strategy = parse_strategy_from_id_local(&sig.strategy_id).unwrap_or(Strategy {
            queue_threshold: 100,
            base_edge: 2,
            take_profit: 20,
            stop_loss: 10,
            holding_period: 0,
            w_conviction: 50,
            w_momentum: 30,
            w_volatility: 20,
            exp_conviction: 100,
            exp_momentum: 100,
            exp_volatility: 100,
            selectivity: 75,
            archetype: 0,
            direction_bias: 50,
            vol_floor: 20,
            mom_floor: 20,
            edge_ratio: 150,
            participation_threshold: 30,
            entry_offset: 0,
            exec_aggression: 50, latency_bias: 10, fill_threshold: 50, lineage: 0,
        });
        registry_rows.push(StrategyProfile {
            strategy_id: sig.strategy_id.clone(),
            strategy,
            preferred_regimes: vec![map_regime_local(&sig.regime)],
            confidence_weight: sig.confidence.clamp(0.0, 1.0),
            execution_weight: sig.composite_score.clamp(0.0, 1.0),
        });
    }
    registry_rows.sort_by(|a, b| a.strategy_id.cmp(&b.strategy_id));
    registry_rows.dedup_by(|a, b| a.strategy_id == b.strategy_id);
    let registry = StrategyRegistry::new(registry_rows);
    LiveEvaluator::new(
        LiveMarketState::new(asset_name),
        registry,
        RankingWeights::default(),
    )
}

pub fn run_edge_decay(
    jsonl_path: &str,
    asset_name: String,
    horizon_ticks: usize,
    top_k: usize,
    slippage_bps: f64,
    initial_signal_snapshot: &crate::pipeline::SignalsSnapshot,
) -> Vec<EdgeDecayResult> {
    let models = [
        ExecutionModel::Ideal,
        ExecutionModel::Spread,
        ExecutionModel::SpreadSlippage,
    ];
    let mut results = Vec::new();

    for model in models {
        let mut replay = match TickReplayEngine::from_binance_jsonl(
            jsonl_path,
            ReplayConfig {
                mode: ReplayMode::Fast,
                ..ReplayConfig::default()
            },
            1,
        ) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to load replay ticks for {:?} model: {}", model, e);
                continue;
            }
        };

        let mut evaluator = build_evaluator_from_snapshot(
            initial_signal_snapshot.signals.as_slice(),
            asset_name.clone(),
        );
        let replay_out = run_replay_with_evaluator(&mut replay, &mut evaluator, top_k);

        let mut replay_for_pnl = match TickReplayEngine::from_binance_jsonl(
            jsonl_path,
            ReplayConfig {
                mode: ReplayMode::Fast,
                ..ReplayConfig::default()
            },
            1,
        ) {
            Ok(r) => r,
            Err(e) => {
                eprintln!(
                    "Failed to load replay ticks for PnL overlay on {:?} model: {}",
                    model, e
                );
                continue;
            }
        };
        let mut evaluator_for_pnl = build_evaluator_from_snapshot(
            initial_signal_snapshot.signals.as_slice(),
            asset_name.clone(),
        );

        let exec_config = ExecutionConfig {
            model,
            slippage_bps,
        };
        let (_trades, pnl_metrics) = run_pnl_overlay_with_config(
            &mut replay_for_pnl,
            &mut evaluator_for_pnl,
            horizon_ticks,
            top_k,
            &exec_config,
        );

        let edge_decay_pct = 1.0 - pnl_metrics.edge_retention;

        results.push(EdgeDecayResult {
            model,
            avg_pnl: pnl_metrics.avg_pnl,
            total_pnl: pnl_metrics.total_pnl,
            win_rate: pnl_metrics.win_rate,
            edge_retention: pnl_metrics.edge_retention,
            effective_signal_rate: replay_out.metrics.effective_signal_rate,
            tradability_band: replay_out.metrics.tradability_band.clone(),
            edge_decay_pct,
            debug_metrics: replay_out.metrics,
        });
    }

    results.sort_by(|a, b| {
        b.total_pnl
            .partial_cmp(&a.total_pnl)
            .unwrap_or(Ordering::Equal)
            .then_with(|| {
                b.edge_retention
                    .partial_cmp(&a.edge_retention)
                    .unwrap_or(Ordering::Equal)
            })
            .then_with(|| {
                b.effective_signal_rate
                    .partial_cmp(&a.effective_signal_rate)
                    .unwrap_or(Ordering::Equal)
            })
    });

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_regime() {
        assert_eq!(map_regime_local("trending_up"), LiveRegime::TrendingUp);
        assert_eq!(map_regime_local("volatile"), LiveRegime::Volatile);
    }
}
