use chronosentiment_core::ga::{
    evaluate_current_status, update_paper_registry, GaConfig, PaperRegistry, Strategy, 
    SignalType, PercentileBuffer, DistributionStats, TradeIntent,
};
use chronosentiment_core::market_adapter::Candle;
use chronosentiment_core::folder_source::FolderCandleSource;

fn main() {
    println!("🚀 CHRONOSENTIMENT — HIGH-FIDELITY LIVE REPLAY (CONSENSUS-AWARE)");
    println!("{}", "=".repeat(70));

    let config = GaConfig::default();
    let mut edge_buffer = PercentileBuffer::new(500);
    let mut current_stats = DistributionStats::default();

    let source = FolderCandleSource {
        folder_path: "test_assets".to_string(),
    };
    let data = source.load_all();
    
    if data.is_empty() {
        return;
    }

    let (symbol, candles) = data.iter().next().unwrap();

    let mut strategy = Strategy::default();
    strategy.archetype = 2; // Mean Reversion
    strategy.selectivity = 100;
    strategy.base_edge = 500;
    strategy.queue_threshold = 1000;
    strategy.take_profit = 250; 
    strategy.stop_loss = 150;
    strategy.holding_period = 50;
    strategy.vol_floor = 0;
    strategy.mom_floor = 0;
    strategy.w_conviction = 1000;
    strategy.w_momentum = 1000;
    strategy.w_volatility = 1000;

    let mut registry = PaperRegistry::default();
    let lookback = 300;

    let mut last_signal = chronosentiment_core::ga::SignalType::WAIT;
    let mut consistency = 0;
    let mut next_rec_id = 1u64;

    println!("\n▶️ Replaying {} candles for {}...", candles.len(), symbol);

    let mut symbol_linear_updates: usize = 0;
    for i in lookback..candles.len() {
        let current_window = &candles[..i];
        let latest_candle = &candles[i];
        symbol_linear_updates = symbol_linear_updates.saturating_add(1);
        let trigger_momentum_3 = if i >= 3 {
            let last = candles[i].close as f64;
            let lag3 = candles[i - 3].close as f64;
            last - lag3
        } else {
            0.0
        };
        let lo = (i as usize).saturating_sub(4);
        let mut window_vals: Vec<f64> = Vec::new();
        for c in &candles[lo..=i] {
            window_vals.push(c.close as f64);
        }
        let trigger_vol_5 = if window_vals.is_empty() {
            0.0
        } else {
            let n = window_vals.len() as f64;
            let mean = window_vals.iter().sum::<f64>() / n;
            let var = window_vals
                .iter()
                .map(|v| {
                    let d = *v - mean;
                    d * d
                })
                .sum::<f64>()
                / n;
            var.sqrt()
        };
        update_paper_registry(
            &mut registry,
            latest_candle,
            symbol,
            symbol_linear_updates,
            trigger_momentum_3,
            trigger_vol_5,
            false, // brutal_truth
        );

        let report = evaluate_current_status(
            &strategy,
            current_window,
            &config,
            symbol,
            last_signal,
            consistency,
            &current_stats,
        );

        if report.raw_edge > 0.0 {
            edge_buffer.push(report.raw_edge);
            if i % 10 == 0 {
                current_stats = edge_buffer.get_stats();
            }
        }

        last_signal = report.signal;
        consistency = report.consistency;

        if let Some(reco) = report.recommendation {
            // FIRE ELITE INTENTS ONLY
            if reco.rank >= 0.7 && reco.is_execution {
                let already_busy = registry.active_trades.iter().any(|t| t.symbol == *symbol) 
                                || registry.pending_intents.iter().any(|i| i.symbol == *symbol);
                
                if !already_busy && (registry.active_trades.len() + registry.pending_intents.len()) < registry.max_concurrent {
                    if registry.submit_intent(TradeIntent {
                        rec_id: next_rec_id,
                        symbol: symbol.to_string(),
                        signal: reco.signal,
                        reference_price: reco.entry_price,
                        birth_price: reco.entry_price,
                        recommendation: reco,
                        strategy_id: 0,
                        rec_score: 0.0,
                        rec_feas: 0.0,
                        rec_conf: 0.0,
                        rec_voters: 0,
                        momentum_3: 0.0,
                        vol_5: 0.0,
                        score_std_5: 0.0,
                        consensus: None,
                        age: 0,
                        max_age: 10,
                        intent_created_symbol_updates: symbol_linear_updates,
                        confirm_delta_symbol_updates: 0,
                        immediate_market_fill: false,
                        use_recommendation_tpsl: false,
                        sketch_risk_span: 0.0,
                        mode: "DIAG".to_string(),
                        entry_path: "DIAG".to_string(),
                        regime: "mixed".to_string(),
                        birth_timestamp: latest_candle.timestamp,
                        intensity: 0.0,
                        stability: 0.0,
                        tier: "DIAG".to_string(),
                    }) {
                        next_rec_id += 1;
                    }
                }
            }
        }
    }

    registry.summary();
}
