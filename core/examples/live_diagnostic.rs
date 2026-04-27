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
        folder_path: "data/nse/5m".to_string(),
    };
    let data = source.load_all_flexible();
    
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

    println!("\n▶️ Replaying {} candles for {}...", candles.len(), symbol);

    for i in lookback..candles.len() {
        let current_window = &candles[..i];
        let latest_candle = &candles[i];
        
        update_paper_registry(&mut registry, latest_candle, symbol);

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
                    registry.pending_intents.push(TradeIntent {
                        rec_id: 0,
                        symbol: symbol.clone(),
                        signal: reco.signal,
                        reference_price: reco.entry_price,
                        recommendation: reco,
                        strategy_id: 0,
                        rec_score: 0.0,
                        rec_feas: 0.0,
                        rec_conf: 0.0,
                        rec_voters: 0,
                        consensus: None,
                        age: 0,
                        max_age: 10,
                    });
                }
            }
        }
    }

    registry.summary();
}
