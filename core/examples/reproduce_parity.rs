use chronosentiment_core::ga::{
    evaluate_strategy, GaConfig, ScenarioPair, Strategy,
};
use chronosentiment_core::{Candle, MarketEvent, PRICE_SCALE, from_real, Side, MarketEventType};
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    println!("--- 🧪 LEGACY DNA RECONSTRUCTION HARNESS V5 ---");
    std::env::set_var("GA_BYPASS_AQG", "1");

    // 1. Mandatory Baseline
    let base_edge_genome: u64 = 10;
    
    // 2. Load Target Data: TATAMOTORS Window 0
    let csv_path = "/Users/nikhil/ChronoSentiment_MEGA_FINAL/test_assets/TATAMOTORS_5m_clean.csv";
    let candles = load_candles_from_csv(csv_path);
    
    // Extract a larger context for sniper discovery (400 bars)
    let window_size = 400;
    let window_candles = &candles[0..window_size.min(candles.len())];
    let events = convert_candles_to_ohlc_events(window_candles);
    
    println!("GENERATED: {} events from {} candles", events.len(), window_candles.len());

    let scenario = ScenarioPair {
        name: "TATAMOTORS_csv_window_0",
        signal_symbol: "TATAMOTORS",
        execution_symbol: "TATAMOTORS",
        signal: &events,
        execution: &events,
    };

    let mut config = GaConfig::default();
    config.latency_ticks = 1;
    config.max_hold_bars = 200;

    // 3. Calibration Loop (Phase 2A: Coarse Scan)
    println!("\n🚀 PHASE 2A: COARSE SCAN (Basin Discovery)...");
    
    let template_dna = Strategy {
        queue_threshold: 392,
        base_edge: base_edge_genome,
        take_profit: 240,
        stop_loss: 270,
        holding_period: 200,
        w_conviction: 50,
        w_momentum: 50,
        w_volatility: 50,
        exp_conviction: 100,
        exp_momentum: 100,
        exp_volatility: 100,
        selectivity: 100,
        archetype: 0,
        direction_bias: 50,
        vol_floor: 0, 
        mom_floor: 0, 
        edge_ratio: 150,
        participation_threshold: 0,
    };

    let mut best_total_error = f64::MAX;
    let mut best_dna = template_dna.clone();

    // High resolution search over key sniper genes
    for selectivity in (10..=100).step_by(10) {
        for w_conv in (10..=200).step_by(40) {
            for w_vol in (10..=200).step_by(40) {
                for exp_vol in (50..=200).step_by(50) {
                    let mut strat = template_dna.clone();
                    strat.selectivity = selectivity as u8;
                    strat.w_conviction = w_conv;
                    strat.w_volatility = w_vol;
                    strat.exp_volatility = exp_vol as u64;

                    if let Some(eval) = evaluate_strategy(&strat, &scenario, &config, 0, 0.0, 1) {
                        if eval.trade_count >= 1 {
                            let err = calculate_error(&eval);
                            if err < best_total_error {
                                best_total_error = err;
                                best_dna = strat.clone();
                                println!("NEW_BEST → n={} edge={:.6} wr={:.2} err={:.4} (sel={} w_c={} w_v={})", 
                                    eval.trade_count, eval.avg_pnl, eval.win_rate, err, selectivity, w_conv, w_vol);
                            }
                        }
                    }
                }
            }
        }
    }

    // 4. Final Validation & PARITY_REPORT
    println!("\n✅ CALIBRATION COMPLETE. GENERATING PARITY_REPORT:");
    if let Some(eval) = evaluate_strategy(&best_dna, &scenario, &config, 0, 0.0, 1) {
        println!("-------------------------------------------");
        println!("PARITY_REPORT:");
        println!("- trade_count: {}", eval.trade_count);
        println!("- avg_edge: {:.6}", eval.avg_pnl); 
        println!("- win_rate: {:.2}", eval.win_rate);
        println!("- avg_exec_prob: {:.4}", eval.avg_exec_prob);
        println!("- edge_std_dev: {:.6}", eval.edge_std_dev);
        println!("-------------------------------------------");
        
        println!("\nFINAL GENOME (18-GENE) RECONSTRUCTED:");
        println!("{}", serde_json::to_string_pretty(&best_dna).unwrap());
    }
}

fn calculate_error(eval: &chronosentiment_core::ga::StrategyEvaluation) -> f64 {
    let target_edge = 0.00894;
    let target_count = 5.0;
    
    // Balanced error function
    let edge_err = ((eval.avg_pnl - target_edge).abs() / target_edge) * 5.0; // ALPHA IS PRIMARY
    let count_err = ((eval.trade_count as f64 - target_count).abs() / target_count) * 2.0;
    let wr_err = if eval.win_rate < 0.9 { (0.9 - eval.win_rate) * 10.0 } else { 0.0 }; // STERN WR penalty
    
    edge_err + count_err + wr_err
}

fn load_candles_from_csv(path: &str) -> Vec<Candle> {
    let file = File::open(path).expect("Could not open CSV file");
    let reader = BufReader::new(file);
    let mut candles = Vec::new();
    
    for line in reader.lines().skip(1) {
        let l = line.expect("Could not read line");
        let parts: Vec<&str> = l.split(',').collect();
        if parts.len() < 5 { continue; }
        
        candles.push(Candle {
            timestamp: parts[0].parse().unwrap_or(0),
            open: from_real(parts[1].parse().unwrap_or(0.0)),
            high: from_real(parts[2].parse().unwrap_or(0.0)),
            low: from_real(parts[3].parse().unwrap_or(0.0)),
            close: from_real(parts[4].parse().unwrap_or(0.0)),
            volume: 1000,
        });
    }
    candles
}

fn convert_candles_to_ohlc_events(candles: &[Candle]) -> Vec<MarketEvent> {
    let mut events = Vec::new();
    for c in candles {
        let prices = [c.open, c.high, c.low, c.close];
        for (i, &px) in prices.iter().enumerate() {
            events.push(MarketEvent {
                subtype: MarketEventType::Trade,
                price: px,
                quantity: 100,
                side: Some(Side::Buy),
                exchange_ts: c.timestamp + i as u64,
            });
        }
    }
    events
}
