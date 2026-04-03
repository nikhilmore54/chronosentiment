use chronosentiment_core::ga::{GaConfig, DecisionReport, SignalType, evaluate_consensus_status, load_elite_strategies, calculate_capture_efficiency, classify_efficiency};
use chronosentiment_core::data_source::{YahooCandleSource, CandleSource, PythonCandleSource};
use chronosentiment_core::csv_source::CsvCandleSource;
use chronosentiment_core::folder_source::FolderCandleSource;
use chronosentiment_core::PRICE_SCALE;
use std::time::{Duration, SystemTime};
use tokio::time::sleep;
use chrono::Local;
use serde::{Serialize, Deserialize};
use std::env;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDecision {
    pub trade_id: u64,
    pub symbol: String,
    pub entry_price: f64,
    pub expected_return: f64,
    pub direction: i32, // 1 = Buy, -1 = Sell
    pub entry_timestamp: u64,
    pub target_timestamp: u64,
    pub horizon_bars: usize,
    pub entry_index: usize,
    pub conviction: f64,
    pub mfe: f64,
    pub mae: f64,
    pub execution_score: f64,
    pub execution_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LiveRecommendation {
    symbol: String,
    signal: String,
    margin_ratio: f64,
    confidence: f64,
    conviction: f64,
    entropy: f64,
    aqg_health: f64,
    timestamp: String,
}

fn classify_signal(signal: SignalType, margin: f64) -> (String, f64) {
    let direction = match signal {
        SignalType::BUY => "BUY",
        SignalType::SELL => "SELL",
        SignalType::WAIT => "WAIT",
    };

    match margin {
        m if m > 3.0 => (format!("STRONG_{}", direction), 0.95),
        m if m > 2.0 => (direction.to_string(), 0.85),
        m if m > 1.5 => (format!("WEAK_{}", direction), 0.65),
        _ => ("SKIP".to_string(), 0.0),
    }
}

fn print_recommendations(recs: &Vec<LiveRecommendation>) {
    let time_str = Local::now().format("%H:%M:%S").to_string();
    println!("\n📊 LIVE RECOMMENDATIONS ({})", time_str);
    println!("{:<20} {:<15} {:<10} {:<10} {:<10}", 
             "Symbol", "Signal", "Margin", "Conv", "Confidence");
    println!("{}", "-".repeat(65));

    for r in recs {
        println!("{:<20} {:<15} {:<10.2} {:<10.2} {:<10.2}", 
                 r.symbol, r.signal, r.margin_ratio, r.conviction, r.confidence);
    }
    println!("{}", "-".repeat(65));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🚀 CHRONOSENTIMENT INSTITUTIONAL LIVE INFERENCE");
    println!("{}", "=".repeat(120));
    
    // 1. CLI Parsing
    let args: Vec<String> = env::args().collect();
    let use_live_api = args.contains(&"--live-api".to_string());
    
    let mut custom_csv_dir = None;
    if let Some(pos) = args.iter().position(|a| a == "--csv-dir") {
        if pos + 1 < args.len() {
            custom_csv_dir = Some(args[pos + 1].clone());
        }
    }

    // --- POSITIONAL ARG RESOLUTION (skipping flags) ---
    let mut positional = Vec::new();
    let mut skip_next = false;
    for (i, arg) in args.iter().enumerate() {
        if i == 0 { continue; }
        if skip_next {
            skip_next = false;
            continue;
        }
        if arg == "--live-api" { continue; }
        if arg == "--csv-dir" {
            skip_next = true;
            continue;
        }
        if arg.starts_with("--") {
            continue;
        }
        positional.push(arg.as_str());
    }

    let (cli_symbols, interval) = if positional.is_empty() {
        (Vec::new(), "1m".to_string())
    } else {
        let last = positional.last().unwrap();
        let allowed_intervals = ["1m", "5m", "15m", "1h", "1d"];
        if allowed_intervals.contains(last) {
            let interval = last.to_string();
            let syms = positional[..positional.len()-1].iter().map(|s| s.to_string()).collect();
            (syms, interval)
        } else {
            (positional.iter().map(|s| s.to_string()).collect(), "1m".to_string())
        }
    };

    let csv_dir = custom_csv_dir.unwrap_or_else(|| {
        let base_path = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut nse_path = base_path.join("data/nse");
        if !nse_path.exists() {
            nse_path = base_path.join("../data/nse");
        }
        format!("{}/{}", nse_path.display(), interval)
    });

    if use_live_api && cli_symbols.is_empty() {
        panic!("❌ ERROR: --live-api mode requires explicit symbols (e.g., RELIANCE.NS)");
    }

    let symbols = if use_live_api {
        cli_symbols
    } else {
        if cli_symbols.is_empty() {
            println!("📂 DATA SOURCE: CSV PIPELINE (Default Mode)");
            println!("📁 Directory: {}", csv_dir);
            let folder_source = FolderCandleSource { folder_path: csv_dir.clone() };
            let discovered = folder_source.list_symbols();
            println!("📊 Symbols: {:?}", discovered);
            discovered
        } else {
            println!("📂 DATA SOURCE: CSV PIPELINE (Specific Symbols)");
            println!("📁 Directory: {}", csv_dir);
            println!("📊 Symbols: {:?}", cli_symbols);
            cli_symbols
        }
    };

    if symbols.is_empty() {
        panic!("❌ ERROR: No symbols found and none provided.");
    }

    if use_live_api {
        println!("🌐 DATA SOURCE: LIVE API");
        println!("⚠️ Non-deterministic mode");
        println!("📊 Symbols: {:?}", symbols);
    }

    let window_size = 100;
    
    // Logic Resolution
    let elite_path = "core/elite/latest.json";
    let elite_strategies = load_elite_strategies(elite_path);
    if elite_strategies.is_empty() {
        panic!("❌ ERROR: No elite strategies found at {}. Run training first.", elite_path);
    }
    println!("✅ PERSISTENCE_LOADED: Found {} active elite genomes for consensus.", elite_strategies.len());

    let config = GaConfig {
        lambda: 0.0001,
        latency_ticks: 1,
        slippage_factor: 0.1,
        max_hold_bars: 20,
        ..Default::default()
    };
    
    // --- SOURCE INITIALIZATION ---
    let mut sources: HashMap<String, Box<dyn CandleSource>> = HashMap::new();
    let intl_suffixes = [".NS", ".BO", ".HK", ".T", ".L", ".DE", ".F", ".AX", ".TO"];
    
    for symbol in &symbols {
        if use_live_api {
            let use_python = intl_suffixes.iter().any(|sfx| symbol.ends_with(sfx));
            let source: Box<dyn CandleSource> = if use_python {
                Box::new(PythonCandleSource::new(symbol, &interval, window_size))
            } else {
                Box::new(YahooCandleSource::new(symbol, &interval, window_size))
            };
            sources.insert(symbol.clone(), source);
        } else {
            let path = Path::new(&csv_dir).join(format!("{}.csv", symbol));
            if !path.exists() {
                println!("⚠️ WARNING: CSV file not found for symbol {}: {:?}", symbol, path);
                continue;
            }
            sources.insert(symbol.clone(), Box::new(CsvCandleSource { path: path.to_str().unwrap().to_string() }));
        }
    }

    let poll_secs = 60;
    
    // MANDATORY STATE ISOLATION
    let mut last_processed_ts: HashMap<String, u64> = HashMap::new();
    let mut last_signals: HashMap<String, SignalType> = HashMap::new();
    let mut consistency_counts: HashMap<String, usize> = HashMap::new();
    let mut pending_map: HashMap<String, Vec<PendingDecision>> = HashMap::new();
    let mut global_indices: HashMap<String, usize> = HashMap::new();
    let mut next_trade_id = 1u64;
    
    // File Safety tracking
    let mut last_file_modified: HashMap<String, SystemTime> = HashMap::new();
    let mut last_file_size: HashMap<String, u64> = HashMap::new();

    println!("\n📡 Starting Monitoring Loop (Pause between cycles: {}s)", poll_secs);
    println!("{}", "-".repeat(120));

    loop {
        let mut recommendations = Vec::new();

        for symbol in &symbols {
            if !sources.contains_key(symbol) { continue; }
            
            let is_first_run = !last_processed_ts.contains_key(symbol);
            if is_first_run {
                println!("🔍 Warmup: Analyzing {} (latest state)...", symbol);
            } else {
                // Heartbeat for live mode
                // println!("🔍 Checking {}...", symbol); 
            }
            if !use_live_api {
                let path_str = Path::new(&csv_dir).join(format!("{}.csv", symbol));
                let meta = match fs::metadata(&path_str) {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                let size = meta.len();

                let prev_modified = last_file_modified.get(symbol).copied().unwrap_or(SystemTime::UNIX_EPOCH);
                let prev_size = last_file_size.get(symbol).copied().unwrap_or(0);

                if modified <= prev_modified && size == prev_size {
                    continue;
                }
                
                if size < prev_size {
                    println!("🔄 RE-INDEX: File for {} was truncated. Resetting tracking.", symbol);
                    last_processed_ts.insert(symbol.clone(), 0);
                }

                last_file_modified.insert(symbol.clone(), modified);
                last_file_size.insert(symbol.clone(), size);
            }

            let source = sources.get_mut(symbol).unwrap();
            let all_candles = source.get_candles_async().await;
            if all_candles.is_empty() || all_candles.len() < window_size { continue; }

            let last_ts = last_processed_ts.get(symbol).copied().unwrap_or(0);
            let latest_c = all_candles.last().unwrap();
            let is_new_candle = latest_c.timestamp > last_ts;

            // Continuous Evaluation: We always analyze, but we only progress the "clock" for new candles
            let last_signal = last_signals.get(symbol).copied().unwrap_or(SignalType::WAIT);
            let consistency_count = consistency_counts.get(symbol).copied().unwrap_or(0);
            let mut global_index = global_indices.get(symbol).copied().unwrap_or(0);
            
            if is_new_candle && last_ts != 0 {
                global_index += 1;
            }

            let report = evaluate_consensus_status(
                &elite_strategies,
                &all_candles,
                &config,
                symbol,
                last_signal,
                consistency_count,
            );

            // 🚀 RECOMMENDATION LAYER (Relaxed Gates for Visibility)
            let margin_ratio = if report.execution_threshold > 0.0 {
                (report.execution_score / report.execution_threshold).min(10.0)
            } else { 0.0 };
            
            // HEARTBEAT DEBUG: Show raw consensus temperature
            println!("   ├─ {} Consensus: {:.4} (Thr: {:.4}) | Conv: {:.2} | Result: {:?}", 
                     symbol, report.execution_score, report.execution_threshold, report.conviction_score, report.signal);

            if report.signal != SignalType::WAIT && report.execution_feasible {
                let (classified_sig, confidence) = classify_signal(report.signal, margin_ratio);
                // Gating: margin > 1.2, conviction > 0.2
                if margin_ratio > 1.2 && report.conviction_score > 0.2 && report.threshold > 0.0 && classified_sig != "SKIP" {
                    recommendations.push(LiveRecommendation {
                        symbol: symbol.clone(),
                        signal: classified_sig,
                        margin_ratio,
                        confidence,
                        conviction: report.conviction_score,
                        entropy: 0.42,
                        aqg_health: report.execution_score,
                        timestamp: Local::now().format("%H:%M:%S").to_string(),
                    });
                }
                if margin_ratio > 3.0 {
                    let sig_text = match report.signal {
                        SignalType::BUY => "BUY", SignalType::SELL => "SELL", _ => "WAIT",
                    };
                    println!("🚨 HIGH CONVICTION: {} {}", symbol, sig_text);
                }
            }

            // --- Outcome Monitoring ---
            let pending_decisions = pending_map.entry(symbol.to_string()).or_default();
            for pending in pending_decisions.iter_mut() {
                let current_price = latest_c.close as f64 / PRICE_SCALE as f64;
                let move_rel = (current_price - pending.entry_price) / pending.entry_price;
                let direction_move = pending.direction as f64 * move_rel;
                if direction_move > pending.mfe { pending.mfe = direction_move; }
                if direction_move < pending.mae { pending.mae = direction_move; }
            }

            // --- Resolution Logic ---
            let mut resolved_indices = Vec::new();
            for (idx, pending) in pending_decisions.iter().enumerate() {
                if global_index - pending.entry_index >= pending.horizon_bars {
                    let exit_price = latest_c.close as f64 / PRICE_SCALE as f64;
                    let realized_return = (pending.direction as f64) * (exit_price - pending.entry_price) / pending.entry_price;
                    let efficiency = calculate_capture_efficiency(realized_return, pending.expected_return);
                    let label = classify_efficiency(efficiency);
                    
                    println!("\n\x1b[1;36m[RESOLVED] {} TRADE_{} | Eff: {:.2} ({}) | PnL: {:.2}bps\x1b[0m",
                             symbol, pending.trade_id, efficiency, label, realized_return * 10000.0);
                    
                    // --- Resolution Logging ---
                    let mut resolved_report = report.clone();
                    resolved_report.realized_return = Some(realized_return);
                    resolved_report.capture_efficiency = Some(efficiency);
                    resolved_report.efficiency_label = label.to_string();
                    log_resolution(&resolved_report, pending);
                    
                    resolved_indices.push(idx);
                }
            }
            for idx in resolved_indices.iter().rev() { pending_decisions.remove(*idx); }

            // --- Pending Management (Only on New Candles) ---
            if is_new_candle && report.signal != SignalType::WAIT && report.execution_feasible {
                let entry_price = latest_c.close as f64 / PRICE_SCALE as f64;
                let direction = if report.signal == SignalType::BUY { 1 } else { -1 };
                let interval_secs = match interval.as_str() {
                    "1m" => 60, "5m" => 300, "15m" => 900, "1h" => 3600, "1d" => 86400, _ => 60,
                };
                let target_ts = latest_c.timestamp + (config.max_hold_bars as u64 * interval_secs);

                pending_decisions.push(PendingDecision {
                    trade_id: next_trade_id,
                    symbol: symbol.to_string(),
                    entry_price,
                    expected_return: report.expected_return,
                    direction,
                    entry_timestamp: latest_c.timestamp,
                    target_timestamp: target_ts,
                    horizon_bars: config.max_hold_bars,
                    entry_index: global_index,
                    conviction: report.confidence,
                    mfe: 0.0,
                    mae: 0.0,
                    execution_score: report.execution_score,
                    execution_threshold: report.execution_threshold,
                });
                next_trade_id += 1;
            }

            // Update state
            last_processed_ts.insert(symbol.clone(), latest_c.timestamp);
            last_signals.insert(symbol.to_string(), report.signal);
            consistency_counts.insert(symbol.to_string(), report.consistency);
            global_indices.insert(symbol.to_string(), global_index);
            
            // --- Audit Logging ---
            if is_new_candle && report.signal != SignalType::WAIT {
                log_decision(&report);
            }
        }

        println!("✅ Cycle Complete. Analyzed {} symbols. Top Recommendations: {}", symbols.len(), recommendations.len());
        
        // --- RANKING & DISPLAY ---
        if !recommendations.is_empty() {
            recommendations.sort_by(|a, b| {
                let score_a = a.margin_ratio * a.conviction;
                let score_b = b.margin_ratio * b.conviction;
                score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
            });
            let top_5: Vec<LiveRecommendation> = recommendations.into_iter().take(5).collect();
            print_recommendations(&top_5);
        }

        sleep(Duration::from_secs(poll_secs)).await;
    }
}

fn log_decision(report: &DecisionReport) {
    use std::fs::OpenOptions;
    use std::io::Write;
    let path = "inference_log.csv";
    let file_exists = std::path::Path::new(path).exists();
    let mut file = OpenOptions::new().create(true).append(true).open(path).unwrap();
    if !file_exists {
        writeln!(file, "timestamp,trade_id,symbol,decision,conviction,execution_score,execution_threshold,expected_return,realized_return,capture_efficiency,efficiency_label,horizon").unwrap();
    }
    let ts = Local::now().to_rfc3339();
    writeln!(file, "{},{},{},{:?},{:.4},{:.2},{:.2},{:.4},{},{},{},{}", 
             ts, report.trade_id, report.symbol, report.signal, report.confidence, 
             report.execution_score, report.execution_threshold, report.expected_return, 
             report.realized_return.map(|v| v.to_string()).unwrap_or_else(|| "---".to_string()),
             report.capture_efficiency.map(|v| v.to_string()).unwrap_or_else(|| "---".to_string()),
             if report.efficiency_label.is_empty() { "---" } else { &report.efficiency_label },
             report.horizon_bars).unwrap();
}

fn log_resolution(report: &DecisionReport, pending: &PendingDecision) {
    log_decision(report);
    let feedback_path = "capture_feedback.json";
    let mut log = if std::path::Path::new(feedback_path).exists() {
        let data = std::fs::read_to_string(feedback_path).unwrap_or_else(|_| "[]".to_string());
        serde_json::from_str::<Vec<serde_json::Value>>(&data).unwrap_or_default()
    } else { Vec::new() };
    let mut entry = serde_json::to_value(report).unwrap();
    if let serde_json::Value::Object(ref mut map) = entry {
        map.insert("mfe".to_string(), serde_json::to_value(pending.mfe).unwrap());
        map.insert("mae".to_string(), serde_json::to_value(pending.mae).unwrap());
    }
    log.push(entry);
    std::fs::write(feedback_path, serde_json::to_string_pretty(&log).unwrap()).unwrap();
}
