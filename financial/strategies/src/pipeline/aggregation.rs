use std::collections::HashMap;
use chronosentiment_core::{MarketEvent, SimEvent};
use chronosentiment_core::market_adapter::{Candle, convert_series_to_events};

use crate::pipeline::reporting::{MetricAggregation, SignalsSnapshot, SignalMeta, EdgeLossBreakdown};
use crate::pipeline::old::{TradeSignal, DEFAULT_CONFIDENCE_FLOOR, DEFAULT_SCORE_FLOOR};

/// Groups, slices, and batches raw market data into sliceable chronology windows.
/// It performs zero causal decision-making, semantic evaluation, or signal routing.
pub fn scenarios_from_candles(asset: &str, candles: &[Candle]) -> HashMap<String, Vec<MarketEvent>> {
    let mut scenarios: HashMap<String, Vec<MarketEvent>> = HashMap::new();
    if candles.len() < 60 {
        return scenarios;
    }

    let window = 120usize.min(candles.len());
    let stride = (window / 2).max(20);
    let mut start = 0usize;
    let mut scenario_id = 0usize;

    while start + window <= candles.len() && scenario_id < 20 {
        let slice = &candles[start..start + window];
        let sim_events = convert_series_to_events(slice, 1);
        let mut market_events: Vec<MarketEvent> = Vec::new();

        for ev in sim_events {
            if let SimEvent::MarketEvent {
                subtype,
                price,
                quantity,
                side,
                timestamp,
                ..
            } = ev {
                market_events.push(MarketEvent {
                    subtype,
                    price,
                    quantity: quantity.max(1),
                    side,
                    exchange_ts: timestamp,
                });
            }
        }

        if !market_events.is_empty() {
            scenarios.insert(format!("{}_csv_window_{}", asset, scenario_id), market_events);
        }

        start += stride;
        scenario_id += 1;
    }

    scenarios
}

pub fn generate_latest_signals(
    assets: Vec<String>,
    global_lambda: f64,
) -> SignalsSnapshot<TradeSignal> {
    generate_latest_signals_with_thresholds(
        assets,
        global_lambda,
        DEFAULT_CONFIDENCE_FLOOR,
        DEFAULT_SCORE_FLOOR,
    )
}

pub fn generate_latest_signals_with_thresholds(
    _assets: Vec<String>,
    _global_lambda: f64,
    _confidence_floor: f64,
    _score_floor: f64,
) -> SignalsSnapshot<TradeSignal> {
    // Dummy implementation for deterministic testing.
    SignalsSnapshot {
        timestamp: 0,
        signals: Vec::new(),
        meta: SignalMeta {
            total_assets: 0,
            total_scenarios: 0,
            trades: 0,
            holds: 0,
            participation: 0.0,
            edge_loss_breakdown: EdgeLossBreakdown {
                total_scenarios: 0,
                total_eval_edge: 0.0,
                total_signal_edge: 0.0,
                edge_retention_ratio: 0.0,
                true_edge_retention: 0.0,
                top_loss_reason: None,
                loss_distribution: Vec::new(),
                loss_by_reason: std::collections::HashMap::new(),
                count_by_reason: std::collections::HashMap::new(),
                executed_strong_edge: 0.0,
                executed_weak_edge: 0.0,
                weak_rejected_low_conf: 0,
                weak_rejected_low_vol: 0,
                weak_executed_count: 0,
                transfer_traces: Vec::new(),
            },
        },
        asset_name: String::new(),
    }
}

pub fn evaluate_on_real_data(
    _assets: Vec<(String, String)>,
    _global_lambda: f64,
) -> Vec<MetricAggregation> {
    unimplemented!()
}
