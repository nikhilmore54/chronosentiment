use chronosentiment_core::ga::{
    evaluate_strategy, GaConfig, ScenarioPair, Strategy, StrategyEvaluation,
};
use chronosentiment_core::{from_real, Candle, MarketEvent, MarketEventType, Side};
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

    println!(
        "GENERATED: {} events from {} candles",
        events.len(),
        window_candles.len()
    );

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
        queue_threshold: 220,
        base_edge: base_edge_genome,
        take_profit: 420,
        stop_loss: 270,
        holding_period: 200,
        w_conviction: 50,
        w_momentum: 50,
        w_volatility: 50,
        exp_conviction: 100,
        exp_momentum: 100,
        exp_volatility: 100,
        selectivity: 12,
        archetype: 0,
        direction_bias: 50,
        vol_floor: 0,
        mom_floor: 0,
        edge_ratio: 150,
        participation_threshold: 0,
        entry_offset: 0,
    };

    let mut best_sniper_error = f64::MAX;
    let mut best_consistent_error = f64::MAX;

    let mut best_sniper: Option<Strategy> = None;
    let mut best_sniper_eval: Option<StrategyEvaluation> = None;

    let mut best_consistent: Option<Strategy> = None;
    let mut best_consistent_eval: Option<StrategyEvaluation> = None;

    let mut sniper_candidates: Vec<StrategyEvaluation> = Vec::new();
    let mut consistent_candidates: Vec<StrategyEvaluation> = Vec::new();

    // High resolution search over key sniper genes
    // ================= SNIPER SEARCH =================
    for selectivity in (5..=20).step_by(5) {
        for w_conv in (10..=200).step_by(40) {
            for w_vol in (10..=200).step_by(40) {
                for exp_vol in (50..=200).step_by(50) {
                    let mut strat = template_dna.clone();
                    strat.edge_ratio = 220;
                    strat.holding_period = 160;
                    strat.selectivity = selectivity as u8;
                    strat.w_conviction = w_conv;
                    strat.w_volatility = w_vol;
                    strat.exp_volatility = exp_vol as u64;

                    if let Some(eval) = evaluate_strategy(&strat, &scenario, &config, 0, 0.0, 1, 0.0, 1.0) {
                        // 🔴 SNIPER ONLY
                        let min_trades = if eval.avg_pnl > 0.0012 {
                            2 // allow exceptional high edge
                        } else {
                            3 // normal requirement
                        };
                        if eval.avg_pnl > 0.0009
                            && eval.trade_count >= min_trades
                            && eval.trade_count <= 4
                            && eval.avg_exec_prob > 0.55
                        {
                            let err = calculate_error(&eval);

                            if err < best_sniper_error {
                                best_sniper_error = err;
                                best_sniper = Some(strat.clone());
                                best_sniper_eval = Some(eval.clone());

                                println!(
                                    "🔴 SNIPER_BEST → n={} edge={:.6} wr={:.2} err={:.4}",
                                    eval.trade_count, eval.avg_pnl, eval.win_rate, err
                                );
                            }

                            sniper_candidates.push(eval.clone());
                        }
                    }
                }
            }
        }
    }

    // ================= CONSISTENT SEARCH =================
    for selectivity in (15..=40).step_by(5) {
        for w_conv in (10..=200).step_by(40) {
            for w_vol in (10..=200).step_by(40) {
                for exp_vol in (50..=200).step_by(50) {
                    let mut strat = template_dna.clone();
                    strat.edge_ratio = 120;
                    strat.holding_period = 240;
                    strat.selectivity = selectivity as u8;
                    strat.w_conviction = w_conv;
                    strat.w_volatility = w_vol;
                    strat.exp_volatility = exp_vol as u64;

                    if let Some(eval) = evaluate_strategy(&strat, &scenario, &config, 0, 0.0, 1, 0.0, 1.0) {
                        // 🔵 CONSISTENT ONLY
                        if eval.avg_pnl > 0.0003 && eval.trade_count >= 6 {
                            let err = calculate_error(&eval);

                            if err < best_consistent_error {
                                best_consistent_error = err;
                                best_consistent = Some(strat.clone());
                                best_consistent_eval = Some(eval.clone());

                                println!(
                                    "🔵 CONSISTENT_BEST → n={} edge={:.6} wr={:.2} err={:.4}",
                                    eval.trade_count, eval.avg_pnl, eval.win_rate, err
                                );
                            }

                            consistent_candidates.push(eval.clone());
                        }
                    }
                }
            }
        }
    }

    println!("\n================ DEBUG SUMMARY ================\n");

    // 🔴 SNIPER
    println!("🔴 SNIPER CANDIDATES:");
    sniper_candidates.sort_by(|a, b| b.avg_pnl.partial_cmp(&a.avg_pnl).unwrap());

    for eval in sniper_candidates.iter().take(5) {
        println!(
            "n={} edge={:.6} wr={:.2} exec={:.2}",
            eval.trade_count, eval.avg_pnl, eval.win_rate, eval.avg_exec_prob
        );
    }

    // 🔵 CONSISTENT
    println!("\n🔵 CONSISTENT CANDIDATES:");
    consistent_candidates.sort_by(|a, b| b.avg_pnl.partial_cmp(&a.avg_pnl).unwrap());

    for eval in consistent_candidates.iter().take(5) {
        println!(
            "n={} edge={:.6} wr={:.2} exec={:.2}",
            eval.trade_count, eval.avg_pnl, eval.win_rate, eval.avg_exec_prob
        );
    }

    if let Some(eval) = &best_sniper_eval {
        if eval.trade_count < 2 {
            panic!("❌ INVALID SNIPER: too few trades");
        }
    }

    if let Some(eval) = &best_consistent_eval {
        if eval.trade_count < 5 {
            panic!("❌ INVALID CONSISTENT: too few trades");
        }
    }

    // 4. Final Validation & PARITY_REPORT
    println!("\n✅ CALIBRATION COMPLETE. GENERATING PARITY_REPORT:");
    println!("\n================ FINAL OUTPUT ================\n");

    // 🔴 SNIPER
    println!("🔴 SNIPER RESULT:");
    if let (Some(strat), Some(eval)) = (&best_sniper, &best_sniper_eval) {
        println!("-------------------------------------------");
        println!("trade_count: {}", eval.trade_count);
        println!("avg_edge: {:.6}", eval.avg_pnl);
        println!("win_rate: {:.2}", eval.win_rate);
        println!("avg_exec_prob: {:.4}", eval.avg_exec_prob);
        println!("edge_std_dev: {:.6}", eval.edge_std_dev);
        println!("-------------------------------------------");

        println!("SNIPER GENOME:");
        println!("\n🔴 SNIPER TRADES:");

        for t in &eval.pnl_history {
            println!(
        "exit_idx={} side={:?} pnl={:.6} quality={:.3} e_score={:.3} eff={:.3} edge_q={:.3} reason={:?}",
        t.exit_event_idx,
        t.side,
        t.pnl,
        t.quality,
        t.e_score,
        t.efficiency,
        t.edge_quality,
        t.exit_reason
    );
        }
        println!("{}", serde_json::to_string_pretty(strat).unwrap());
    } else {
        println!("❌ NO SNIPER FOUND");
    }

    // 🔵 CONSISTENT
    println!("\n🔵 CONSISTENT RESULT:");
    if let (Some(strat), Some(eval)) = (&best_consistent, &best_consistent_eval) {
        println!("-------------------------------------------");
        println!("trade_count: {}", eval.trade_count);
        println!("avg_edge: {:.6}", eval.avg_pnl);
        println!("win_rate: {:.2}", eval.win_rate);
        println!("avg_exec_prob: {:.4}", eval.avg_exec_prob);
        println!("edge_std_dev: {:.6}", eval.edge_std_dev);
        println!("-------------------------------------------");

        println!("CONSISTENT GENOME:");
        println!("\n🔵 CONSISTENT TRADES:");

        for t in &eval.pnl_history {
            println!(
        "exit_idx={} side={:?} pnl={:.6} quality={:.3} e_score={:.3} eff={:.3} edge_q={:.3} reason={:?}",
        t.exit_event_idx,
        t.side,
        t.pnl,
        t.quality,
        t.e_score,
        t.efficiency,
        t.edge_quality,
        t.exit_reason
    );
        }
        println!("{}", serde_json::to_string_pretty(strat).unwrap());
    } else {
        println!("❌ NO CONSISTENT FOUND");
    }
    if let (Some(s), Some(c)) = (&best_sniper_eval, &best_consistent_eval) {
        let combined_edge = 0.3 * s.avg_pnl + 0.7 * c.avg_pnl;

        println!("\n================ PORTFOLIO ================\n");
        println!("Combined Edge: {:.6}", combined_edge);
    }
}

fn calculate_error(eval: &chronosentiment_core::ga::StrategyEvaluation) -> f64 {
    let target_edge = 0.001;
    let target_count = 5.0;

    // Balanced error function
    let edge_err = ((eval.avg_pnl - target_edge).abs() / target_edge) * 5.0; // ALPHA IS PRIMARY
    let exec_err = (0.65 - eval.avg_exec_prob).max(0.0) * 3.0;
    let count_err = ((eval.trade_count as f64 - target_count).abs() / target_count) * 2.0;
    let wr_err = if eval.trade_count < 5 {
        0.0 // ✅ allow perfect WR for small samples (sniper)
    } else if eval.win_rate >= 0.95 {
        5.0 // 🚨 unrealistic for larger samples
    } else if eval.win_rate < 0.5 {
        5.0 // bad system
    } else {
        (0.7 - eval.win_rate).abs() * 2.0
    };
    edge_err + count_err + wr_err + exec_err
}

fn load_candles_from_csv(path: &str) -> Vec<Candle> {
    let file = File::open(path).expect("Could not open CSV file");
    let reader = BufReader::new(file);
    let mut candles = Vec::new();

    for line in reader.lines().skip(1) {
        let l = line.expect("Could not read line");
        let parts: Vec<&str> = l.split(',').collect();
        if parts.len() < 5 {
            continue;
        }

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
