use crate::strategy_ranking::LiveEvaluator;
use crate::tick_replay::TickReplayEngine;
use serde::{Deserialize, Serialize};

const DEFAULT_MAX_SLIPPAGE: f64 = 0.0005; // 0.05%

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeDirection {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub entry_ts: u64,
    pub entry_price: f64,
    pub exit_ts: u64,
    pub exit_price: f64,
    /// Simple return over entry price.
    pub pnl: f64,
    pub direction: TradeDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PnLMetrics {
    pub total_trades: usize,
    pub wins: usize,
    pub losses: usize,
    pub win_rate: f64,
    pub avg_pnl: f64,
    pub total_pnl: f64,
    /// realized edge / predicted edge, clipped to [0, +inf)
    pub edge_retention: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionModel {
    Ideal,
    Spread,
    SpreadSlippage,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub model: ExecutionModel,
    /// 0.0005 = 5 bps.
    pub slippage_bps: f64,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            model: ExecutionModel::SpreadSlippage,
            slippage_bps: DEFAULT_MAX_SLIPPAGE,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct OpenPosition {
    entry_ts: u64,
    entry_price: f64,
    direction: TradeDirection,
    ticks_left: usize,
}

fn direction_from_action(action: &str) -> Option<TradeDirection> {
    match action {
        "BUY" => Some(TradeDirection::Buy),
        "SELL" => Some(TradeDirection::Sell),
        _ => None,
    }
}

fn execution_price_from_book(
    best_bid: Option<f64>,
    best_ask: Option<f64>,
    fallback_last: f64,
    taking_buy_side: bool,
) -> Option<f64> {
    let px = if taking_buy_side {
        best_ask.or(best_bid).unwrap_or(fallback_last)
    } else {
        best_bid.or(best_ask).unwrap_or(fallback_last)
    };
    if px.is_finite() && px > 0.0 {
        Some(px)
    } else {
        None
    }
}

fn deterministic_slippage(ts: u64, max_bps: f64) -> f64 {
    if max_bps <= 0.0 {
        return 0.0;
    }
    let x = (ts % 1000) as f64 / 1000.0;
    (x * max_bps).clamp(0.0, max_bps)
}

fn apply_adverse_slippage(price: f64, slippage: f64, taking_buy_side: bool) -> f64 {
    if taking_buy_side {
        price * (1.0 + slippage)
    } else {
        price * (1.0 - slippage)
    }
}

/// Deterministic replay-driven PnL overlay.
/// - Entry: first actionable top suggestion when no position is open
/// - Exit: fixed horizon in ticks
pub fn run_pnl_overlay(
    replay: &mut TickReplayEngine,
    evaluator: &mut LiveEvaluator,
    horizon_ticks: usize,
) -> (Vec<Trade>, PnLMetrics) {
    let max_slippage = std::env::var("PNL_MAX_SLIPPAGE_BPS")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(DEFAULT_MAX_SLIPPAGE)
        .max(0.0);
    let config = ExecutionConfig {
        model: ExecutionModel::SpreadSlippage,
        slippage_bps: max_slippage,
    };
    run_pnl_overlay_with_config(replay, evaluator, horizon_ticks, 5, &config)
}

/// Deterministic replay-driven PnL overlay with explicit execution model.
pub fn run_pnl_overlay_with_config(
    replay: &mut TickReplayEngine,
    evaluator: &mut LiveEvaluator,
    horizon_ticks: usize,
    top_k: usize,
    config: &ExecutionConfig,
) -> (Vec<Trade>, PnLMetrics) {
    let mut trades: Vec<Trade> = Vec::new();
    let mut open: Option<OpenPosition> = None;
    let mut predicted_edge_sum = 0.0_f64;

    while let Some(replayed) = replay.next_event() {
        let suggestions = evaluator.on_event(&replayed.event, top_k.max(1));
        let fallback_last = replayed.event.price;
        if !fallback_last.is_finite() || fallback_last <= 0.0 {
            continue;
        }

        if open.is_none() {
            if let Some(top) = suggestions.first() {
                if let Some(direction) = direction_from_action(top.action.as_str()) {
                    let entry_price = match config.model {
                        ExecutionModel::Ideal => fallback_last,
                        ExecutionModel::Spread | ExecutionModel::SpreadSlippage => {
                            let Some(mut px) = execution_price_from_book(
                                replayed.event.best_bid,
                                replayed.event.best_ask,
                                fallback_last,
                                matches!(direction, TradeDirection::Buy),
                            ) else {
                                continue;
                            };
                            if matches!(config.model, ExecutionModel::SpreadSlippage) {
                                let entry_slip = deterministic_slippage(
                                    replayed.execution_ts,
                                    config.slippage_bps.max(0.0),
                                );
                                px = apply_adverse_slippage(
                                    px,
                                    entry_slip,
                                    matches!(direction, TradeDirection::Buy),
                                );
                            }
                            px
                        }
                    };
                    open = Some(OpenPosition {
                        entry_ts: replayed.execution_ts,
                        entry_price,
                        direction,
                        ticks_left: horizon_ticks.max(1),
                    });
                    predicted_edge_sum += top.expected_edge.max(0.0);
                }
            }
        }

        if let Some(mut pos) = open {
            if pos.ticks_left > 0 {
                pos.ticks_left -= 1;
                open = Some(pos);
            } else {
                let exit_price = match config.model {
                    ExecutionModel::Ideal => fallback_last,
                    ExecutionModel::Spread | ExecutionModel::SpreadSlippage => {
                        let Some(mut px) = execution_price_from_book(
                            replayed.event.best_bid,
                            replayed.event.best_ask,
                            fallback_last,
                            matches!(pos.direction, TradeDirection::Sell),
                        ) else {
                            continue;
                        };
                        if matches!(config.model, ExecutionModel::SpreadSlippage) {
                            let exit_slip = deterministic_slippage(
                                replayed.execution_ts,
                                config.slippage_bps.max(0.0),
                            );
                            px = apply_adverse_slippage(
                                px,
                                exit_slip,
                                matches!(pos.direction, TradeDirection::Sell),
                            );
                        }
                        px
                    }
                };
                let pnl = match pos.direction {
                    TradeDirection::Buy => (exit_price - pos.entry_price) / pos.entry_price,
                    TradeDirection::Sell => (pos.entry_price - exit_price) / pos.entry_price,
                };
                trades.push(Trade {
                    entry_ts: pos.entry_ts,
                    entry_price: pos.entry_price,
                    exit_ts: replayed.execution_ts,
                    exit_price,
                    pnl,
                    direction: pos.direction,
                });
                open = None;
            }
        }
    }

    let total_trades = trades.len();
    let wins = trades.iter().filter(|t| t.pnl > 0.0).count();
    let losses = total_trades.saturating_sub(wins);
    let total_pnl: f64 = trades.iter().map(|t| t.pnl).sum();
    let avg_pnl = if total_trades == 0 {
        0.0
    } else {
        total_pnl / total_trades as f64
    };
    let win_rate = if total_trades == 0 {
        0.0
    } else {
        wins as f64 / total_trades as f64
    };
    let edge_retention = if predicted_edge_sum <= 1e-12 {
        0.0
    } else {
        (total_pnl / predicted_edge_sum).max(0.0)
    };

    (
        trades,
        PnLMetrics {
            total_trades,
            wins,
            losses,
            win_rate,
            avg_pnl,
            total_pnl,
            edge_retention,
        },
    )
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
    fn pnl_overlay_produces_trades() {
        let events = vec![
            NormalizedMarketEvent {
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
                exchange_ts: 2,
                price: 101.0,
                volume: 1.0,
                side: None,
                best_bid: Some(100.9),
                best_ask: Some(101.1),
                bids: None,
                asks: None,
            },
            NormalizedMarketEvent {
                exchange_ts: 3,
                price: 102.0,
                volume: 1.0,
                side: None,
                best_bid: Some(101.9),
                best_ask: Some(102.1),
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
        let mut evaluator = LiveEvaluator::new(
            LiveMarketState::new("BTCUSDT".to_string()),
            registry,
            RankingWeights::default(),
        );
        evaluator.min_edge = 0.0;
        evaluator.min_execution_score = 0.0;
        evaluator.stability_delta = 0.0;

        let (trades, metrics) = run_pnl_overlay(&mut replay, &mut evaluator, 1);
        assert!(!trades.is_empty());
        assert!(metrics.total_trades >= 1);
    }
}
