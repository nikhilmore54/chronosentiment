use crate::pnl_overlay::{ExecutionConfig, ExecutionModel, run_pnl_overlay_with_config};
use crate::replay_evaluator::{run_replay_with_evaluator, ReplayMetrics, TradabilityBand};
use crate::strategy_ranking::{LiveEvaluator, LiveMarketState, RankingWeights, StrategyProfile, StrategyRegistry, LiveRegime};
use crate::tick_replay::{ReplayConfig, ReplayMode, TickReplayEngine};
use crate::ga::Strategy;
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

// Local helper to parse strategy from ID (simplified for reporting)
fn parse_strategy_from_id_local(id: &str) -> Option<Strategy> {
    let parts: Vec<&str> = id.split('v').collect();
    if parts.len() < 13 { return None; }
    
    // The parts are our parameters (queue_threshold to archetype)
    // Format: STRAT_QvEvTPvSLvHPvW_CONVvW_MOMvW_VOLvEXP_CONVvEXP_MOMvEXP_VOLvSELvARCH
    let q = parts[0].trim_start_matches("STRAT_").parse().ok()?;
    let e = parts[1].parse().ok()?;
    let tp = parts[2].parse().ok()?;
    let sl = parts[3].parse().ok()?;
    let holding = parts[4].parse().ok()?;
    let w_conv = parts[5].parse().ok()?;
    let w_mom = parts[6].parse().ok()?;
    let w_vol = parts[7].parse().ok()?;
    let exp_conv = parts[8].parse().ok()?;
    let exp_mom = parts[9].parse().ok()?;
    let exp_vol = parts[10].parse().ok()?;
    let selectivity = parts[11].parse().ok()?;
    let archetype = parts[12].parse().ok()?;
    
    Some(Strategy {
        queue_threshold: q,
        base_edge: e,
        take_profit: tp,
        stop_loss: sl,
        holding_period: holding,
        w_conviction: w_conv,
        w_momentum: w_mom,
        w_volatility: w_vol,
        exp_conviction: exp_conv,
        exp_momentum: exp_mom,
        exp_volatility: exp_vol,
        selectivity,
        archetype,
    })
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
    })
}

fn build_evaluator_from_snapshot(snapshot_signals: &[crate::pipeline::TradeSignal], asset_name: String) -> LiveEvaluator {
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
    let models = [ExecutionModel::Ideal, ExecutionModel::Spread, ExecutionModel::SpreadSlippage];
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

        let mut evaluator = build_evaluator_from_snapshot(initial_signal_snapshot.signals.as_slice(), asset_name.clone());
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
                eprintln!("Failed to load replay ticks for PnL overlay on {:?} model: {}", model, e);
                continue;
            }
        };
        let mut evaluator_for_pnl = build_evaluator_from_snapshot(initial_signal_snapshot.signals.as_slice(), asset_name.clone());

        let exec_config = ExecutionConfig { model, slippage_bps };
        let (_trades, pnl_metrics) =
            run_pnl_overlay_with_config(&mut replay_for_pnl, &mut evaluator_for_pnl, horizon_ticks, top_k, &exec_config);

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
