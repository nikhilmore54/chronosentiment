use crate::exit::ExitReason;
use chronosentiment_core::market_adapter::{convert_series_to_events, Candle};
use chronosentiment_core::{MarketEvent, SimEvent};
use serde::{Deserialize, Serialize};

const VOLATILITY_THRESHOLD: f64 = 0.01;
const TREND_THRESHOLD: f64 = 0.01;
const VOL_NORM_FACTOR: f64 = 0.02;
pub const DEFAULT_CONFIDENCE_FLOOR: f64 = 0.30;
pub const DEFAULT_SCORE_FLOOR: f64 = 0.40;

use crate::pipeline::reporting::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SignalAction {
    BUY,
    SELL,
    HOLD,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecommendationStatus {
    NEW,      // Emitted by engine
    PENDING,  // Waiting for fill
    ACTIVE,   // Position open
    CLOSED,   // Completed
    REJECTED, // Filtered out
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlphaPorosity {
    Dead,         // capture < 0
    Fragile,      // 0 <= capture < 0.25
    Transitional, // 0.25 <= capture < 0.6
    Live,         // capture >= 0.6
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntryType {
    MARKET,
    PULLBACK,
    BREAKOUT,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeSignal {
    pub asset: String,
    pub regime: String,
    pub confidence: f64,
    pub action: SignalAction,
    pub entry_type: EntryType,
    pub entry_zone: Option<(f64, f64)>,
    pub stop_loss: Option<f64>,
    pub target: Option<f64>,
    pub expected_edge: f64,
    pub scenario_pnl: f64,
    pub risk_reward: f64,
    pub position_size: f64,
    pub conviction: f64,
    pub composite_score: f64,
    pub reject_reason: Option<String>,
    pub expected_holding_time: String,
    pub strategy_id: String,
    pub reason: String,
    pub status: RecommendationStatus,
    pub porosity: AlphaPorosity,
    pub porosity_trend: f64, // Rate of change in porosity
    pub is_open: bool,
    pub current_pnl: f64,
    pub peak_pnl: f64,
    pub exit_reason: Option<ExitReason>,
    pub rank_score: f64,
    pub rank_position: Option<u32>,
    pub allocated_capital: Option<f64>,
    pub quantity: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
enum RejectReason {
    LowConfidence,
    LowScore,
    NegativeEdge,
    SidewaysRegime,
}

impl RejectReason {
    fn as_str(self) -> &'static str {
        match self {
            RejectReason::LowConfidence => "REJECT_LOW_CONF",
            RejectReason::LowScore => "REJECT_LOW_SCORE",
            RejectReason::NegativeEdge => "REJECT_NEG_EDGE",
            RejectReason::SidewaysRegime => "REJECT_SIDEWAYS",
        }
    }
}

fn edge_reason_from_gate_reject(reject_reason: Option<RejectReason>) -> EdgeLossReason {
    match reject_reason {
        None => EdgeLossReason::Accepted,
        Some(RejectReason::LowConfidence) => EdgeLossReason::LowConfidence,
        Some(RejectReason::LowScore) => EdgeLossReason::LowScore,
        Some(RejectReason::NegativeEdge) => EdgeLossReason::RiskFiltered,
        Some(RejectReason::SidewaysRegime) => EdgeLossReason::SidewaysMarket,
    }
}

#[derive(Debug, Clone, Copy)]
struct GateDecision {
    trade_allowed: bool,
    position_size: f64,
    composite_score: f64,
    reject_reason: Option<RejectReason>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Regime {
    Volatile,
    TrendingUp,
    TrendingDown,
    Sideways,
    Mixed,
}

impl Regime {
    fn as_str(self) -> &'static str {
        match self {
            Regime::Volatile => "volatile",
            Regime::TrendingUp => "trending_up",
            Regime::TrendingDown => "trending_down",
            Regime::Sideways => "sideways",
            Regime::Mixed => "mixed",
        }
    }
}

fn detect_regime_from_events(events: &[MarketEvent]) -> (Regime, f64) {
    if events.len() < 3 {
        return (Regime::Mixed, 0.0);
    }

    let prices: Vec<f64> = events.iter().map(|e| e.price as f64).collect();
    let first = prices.first().copied().unwrap_or(0.0);
    let last = prices.last().copied().unwrap_or(first);
    if first <= 0.0 {
        return (Regime::Mixed, 0.0);
    }

    let mut returns = Vec::with_capacity(prices.len().saturating_sub(1));
    for i in 1..prices.len() {
        let prev = prices[i - 1];
        let curr = prices[i];
        if prev > 0.0 {
            returns.push((curr / prev) - 1.0);
        }
    }

    let mean_ret = if returns.is_empty() {
        0.0
    } else {
        returns.iter().sum::<f64>() / returns.len() as f64
    };
    let variance = if returns.is_empty() {
        0.0
    } else {
        returns.iter().map(|r| (r - mean_ret).powi(2)).sum::<f64>() / returns.len() as f64
    };
    let volatility = variance.sqrt();
    let trend = (last - first) / first;
    let trend_strength = trend.abs().clamp(0.0, 1.0);
    let vol_strength = (volatility / VOL_NORM_FACTOR).clamp(0.0, 1.0);

    let regime = if volatility > VOLATILITY_THRESHOLD {
        Regime::Volatile
    } else if trend > TREND_THRESHOLD {
        Regime::TrendingUp
    } else if trend < -TREND_THRESHOLD {
        Regime::TrendingDown
    } else {
        Regime::Sideways
    };

    let dominance_denominator = trend_strength + vol_strength + 1e-6;
    let confidence = match regime {
        Regime::TrendingUp | Regime::TrendingDown => trend_strength / dominance_denominator,
        Regime::Volatile => vol_strength / dominance_denominator,
        Regime::Sideways => 1.0 - trend_strength.max(vol_strength),
        Regime::Mixed => 0.0,
    }
    .clamp(0.0, 1.0);

    (regime, confidence)
}

fn should_trade(execution_fitness: f64) -> bool {
    execution_fitness.is_finite() && execution_fitness > 0.0
}

fn regime_quality(regime: Regime) -> f64 {
    match regime {
        Regime::TrendingUp | Regime::TrendingDown => 1.0,
        Regime::Volatile => 0.7,
        Regime::Sideways => 0.0,
        Regime::Mixed => 0.4,
    }
}

fn edge_norm(execution_fitness: f64) -> f64 {
    // Calibrated to current bounded execution-fitness scale (~0.00-0.02 typical).
    (execution_fitness / 0.01).clamp(0.0, 1.0)
}

fn tiered_position_size(confidence: f64) -> f64 {
    if confidence > 0.75 {
        1.0
    } else if confidence > 0.65 {
        0.8
    } else if confidence > 0.55 {
        0.6
    } else if confidence > 0.45 {
        0.3
    } else {
        0.0
    }
}

fn evaluate_gate(
    regime: Regime,
    confidence: f64,
    execution_fitness: f64,
    confidence_floor: f64,
    score_floor: f64,
) -> GateDecision {
    if matches!(regime, Regime::Sideways) {
        return GateDecision {
            trade_allowed: false,
            position_size: 0.0,
            composite_score: 0.0,
            reject_reason: Some(RejectReason::SidewaysRegime),
        };
    }
    if confidence < confidence_floor {
        return GateDecision {
            trade_allowed: false,
            position_size: 0.0,
            composite_score: 0.0,
            reject_reason: Some(RejectReason::LowConfidence),
        };
    }
    if !should_trade(execution_fitness) {
        return GateDecision {
            trade_allowed: false,
            position_size: 0.0,
            composite_score: 0.0,
            reject_reason: Some(RejectReason::NegativeEdge),
        };
    }

    let score =
        0.5 * confidence + 0.3 * edge_norm(execution_fitness) + 0.2 * regime_quality(regime);
    if score < score_floor {
        return GateDecision {
            trade_allowed: false,
            position_size: 0.0,
            composite_score: score,
            reject_reason: Some(RejectReason::LowScore),
        };
    }

    GateDecision {
        trade_allowed: true,
        position_size: tiered_position_size(confidence),
        composite_score: score,
        reject_reason: None,
    }
}

fn compute_atr(events: &[MarketEvent]) -> f64 {
    if events.len() < 2 {
        return 0.0;
    }
    let mut true_ranges: Vec<f64> = Vec::with_capacity(events.len() - 1);
    for i in 1..events.len() {
        let prev = events[i - 1].price as f64;
        let curr = events[i].price as f64;
        true_ranges.push((curr - prev).abs());
    }
    true_ranges.iter().sum::<f64>() / true_ranges.len() as f64
}

fn build_trade_signal(
    asset: &str,
    regime: Regime,
    confidence: f64,
    selected_strategy_id: &str,
    report_fitness: f64,
    report_pnl: f64,
    gate: GateDecision,
    last_price: f64,
    atr: f64,
) -> TradeSignal {
    let trade_allowed = gate.trade_allowed;
    let (action, entry_type, entry_zone, stop_loss, target, expected_holding_time, reason) =
        if trade_allowed {
            match regime {
                Regime::TrendingUp => {
                    let entry_zone = (last_price * 0.996, last_price * 1.000);
                    let effective_atr = atr.max(last_price * 0.003);
                    let stop_loss = last_price - (1.5 * effective_atr);
                    let risk = (last_price - stop_loss).abs();
                    let target = last_price + (2.0 * risk);
                    assert!(
                        target > entry_zone.1,
                        "Target must exceed entry zone for BUY"
                    );
                    assert!(
                        stop_loss < entry_zone.0,
                        "Stop loss must be below entry zone for BUY"
                    );
                    (
                        SignalAction::BUY,
                        EntryType::PULLBACK,
                        Some(entry_zone),
                        Some(stop_loss),
                        Some(target),
                        "30m-2h".to_string(),
                        format!(
                            "Regime={} with confidence {:.2}, execution fitness {:.2}",
                            regime.as_str(),
                            confidence,
                            report_fitness
                        ),
                    )
                }
                Regime::TrendingDown => {
                    let entry_zone = (last_price * 1.000, last_price * 1.004);
                    let effective_atr = atr.max(last_price * 0.003);
                    let stop_loss = last_price + (1.5 * effective_atr);
                    let risk = (stop_loss - last_price).abs();
                    let target = last_price - (2.0 * risk);
                    assert!(
                        target < entry_zone.0,
                        "Target must be below entry zone for SELL"
                    );
                    assert!(
                        stop_loss > entry_zone.1,
                        "Stop loss must be above entry zone for SELL"
                    );
                    (
                        SignalAction::SELL,
                        EntryType::PULLBACK,
                        Some(entry_zone),
                        Some(stop_loss),
                        Some(target),
                        "30m-2h".to_string(),
                        format!(
                            "Regime={} with confidence {:.2}, execution fitness {:.2}",
                            regime.as_str(),
                            confidence,
                            report_fitness
                        ),
                    )
                }
                Regime::Sideways => (
                    SignalAction::HOLD,
                    EntryType::MARKET,
                    None,
                    None,
                    None,
                    "N/A".to_string(),
                    "Low directional edge (sideways regime)".to_string(),
                ),
                _ => (
                    SignalAction::HOLD,
                    EntryType::MARKET,
                    None,
                    None,
                    None,
                    "N/A".to_string(),
                    "Non-directional regime routed to HOLD for phase-1 safety".to_string(),
                ),
            }
        } else {
            let reject_code = gate
                .reject_reason
                .map(|r| r.as_str())
                .unwrap_or("REJECT_UNKNOWN");
            (
                SignalAction::HOLD,
                EntryType::MARKET,
                None,
                None,
                None,
                "N/A".to_string(),
                format!("Rejected by gate: {}", reject_code),
            )
        };

    let risk_reward = match (entry_zone, stop_loss, target, &action) {
        (Some((entry_lo, entry_hi)), Some(sl), Some(tp), SignalAction::BUY) => {
            let entry = (entry_lo + entry_hi) * 0.5;
            let risk = (entry - sl).max(1e-9);
            let reward = (tp - entry).max(0.0);
            reward / risk
        }
        (Some((entry_lo, entry_hi)), Some(sl), Some(tp), SignalAction::SELL) => {
            let entry = (entry_lo + entry_hi) * 0.5;
            let risk = (sl - entry).max(1e-9);
            let reward = (entry - tp).max(0.0);
            reward / risk
        }
        _ => 0.0,
    };
    if matches!(action, SignalAction::BUY | SignalAction::SELL) {
        assert!(
            risk_reward >= 1.5,
            "Risk-reward below minimum threshold: {}",
            risk_reward
        );
    }

    TradeSignal {
        asset: asset.to_string(),
        regime: regime.as_str().to_string(),
        confidence,
        action,
        entry_type,
        entry_zone,
        stop_loss,
        target,
        expected_edge: report_fitness,
        scenario_pnl: if trade_allowed { report_pnl } else { 0.0 },
        risk_reward,
        position_size: if trade_allowed {
            gate.position_size
        } else {
            0.0
        },
        conviction: confidence * report_fitness.max(0.0),
        composite_score: gate.composite_score,
        reject_reason: gate.reject_reason.map(|r| r.as_str().to_string()),
        expected_holding_time,
        strategy_id: selected_strategy_id.to_string(),
        reason,
        status: RecommendationStatus::NEW,
        porosity: AlphaPorosity::Live, // Placeholder, usually updated by registry
        porosity_trend: 0.0,
        is_open: trade_allowed,
        current_pnl: 0.0,
        peak_pnl: 0.0,
        exit_reason: None,
        rank_score: 0.0,
        rank_position: None,
        allocated_capital: None,
        quantity: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chronosentiment_core::MarketEventType;

    #[test]
    fn test_detect_regime_is_deterministic() {
        let events = vec![
            MarketEvent {
                subtype: MarketEventType::Trade,
                price: 100,
                quantity: 1,
                side: None,
                exchange_ts: 1,
            },
            MarketEvent {
                subtype: MarketEventType::Trade,
                price: 101,
                quantity: 1,
                side: None,
                exchange_ts: 2,
            },
            MarketEvent {
                subtype: MarketEventType::Trade,
                price: 102,
                quantity: 1,
                side: None,
                exchange_ts: 3,
            },
            MarketEvent {
                subtype: MarketEventType::Trade,
                price: 103,
                quantity: 1,
                side: None,
                exchange_ts: 4,
            },
        ];
        let r1 = detect_regime_from_events(&events);
        let r2 = detect_regime_from_events(&events);
        assert_eq!(r1.0, r2.0);
        assert!((r1.1 - r2.1).abs() < 1e-12);
    }

    #[test]
    fn test_detect_regime_flat_is_sideways_or_mixed_with_high_confidence() {
        let events = vec![
            MarketEvent {
                subtype: MarketEventType::Trade,
                price: 100,
                quantity: 1,
                side: None,
                exchange_ts: 1,
            },
            MarketEvent {
                subtype: MarketEventType::Trade,
                price: 100,
                quantity: 1,
                side: None,
                exchange_ts: 2,
            },
            MarketEvent {
                subtype: MarketEventType::Trade,
                price: 100,
                quantity: 1,
                side: None,
                exchange_ts: 3,
            },
            MarketEvent {
                subtype: MarketEventType::Trade,
                price: 100,
                quantity: 1,
                side: None,
                exchange_ts: 4,
            },
        ];
        let (regime, confidence) = detect_regime_from_events(&events);
        assert!(matches!(regime, Regime::Sideways | Regime::Mixed));
        assert!(confidence >= 0.9 || matches!(regime, Regime::Mixed));
    }
}
