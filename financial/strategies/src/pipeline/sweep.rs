use crate::pipeline::aggregation::generate_latest_signals_with_thresholds;
use crate::pipeline::reporting::{ThresholdSweepRow, SignalsSnapshot};
use crate::pipeline::old::{SignalAction, TradeSignal};

/// Runs a parameter sweep over confidence and score thresholds.
/// Returns a sorted vector of `ThresholdSweepRow` results.
pub fn run_threshold_sweep(
    assets: Vec<String>,
    global_lambda: f64,
    confidence_floors: &[f64],
    score_floors: &[f64],
) -> Vec<ThresholdSweepRow> {
    let mut rows: Vec<ThresholdSweepRow> = Vec::new();
    for &confidence_floor in confidence_floors {
        for &score_floor in score_floors {
            let snapshot = generate_latest_signals_with_thresholds(
                assets.clone(),
                global_lambda,
                confidence_floor,
                score_floor,
            );
            // Compute global average PnL.
            let pnls: Vec<f64> = snapshot.signals.iter().map(|s| s.scenario_pnl).collect();
            let global_avg = if pnls.is_empty() { 0.0 } else { pnls.iter().sum::<f64>() / pnls.len() as f64 };
            // Compute variance for std_dev.
            let variance = if pnls.is_empty() {
                0.0
            } else {
                pnls.iter().map(|p| (p - global_avg).powi(2)).sum::<f64>() / pnls.len() as f64
            };
            // Compute traded average PnL (excluding HOLD actions).
            let traded: Vec<f64> = snapshot
                .signals
                .iter()
                .filter(|s| s.action != SignalAction::HOLD)
                .map(|s| s.scenario_pnl)
                .collect();
            let traded_avg = if traded.is_empty() {
                0.0
            } else {
                traded.iter().sum::<f64>() / traded.len() as f64
            };
            rows.push(ThresholdSweepRow {
                confidence_floor,
                score_floor,
                participation: snapshot.meta.participation,
                trades: snapshot.meta.trades,
                total_scenarios: snapshot.meta.total_scenarios,
                global_avg_pnl: global_avg,
                traded_avg_pnl: traded_avg,
                std_dev: variance.sqrt(),
            });
        }
    }

    // Sort rows: prioritize participation band then descending global_avg_pnl.
    rows.sort_by(|a, b| {
        let a_in_band = (0.15..=0.30).contains(&a.participation);
        let b_in_band = (0.15..=0.30).contains(&b.participation);
        match (a_in_band, b_in_band) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => b
                .global_avg_pnl
                .partial_cmp(&a.global_avg_pnl)
                .unwrap_or(std::cmp::Ordering::Equal),
        }
    });

    rows
}
