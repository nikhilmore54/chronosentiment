use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    substrate_file: String,

    #[arg(long)]
    strategy: String, // e.g., "twap", "breakout", "momentum"

    #[arg(long, default_value_t = 0)]
    latency_ms: u64,

    #[arg(long, default_value_t = 0.0)]
    missed_fill_prob: f64,
}

#[derive(Deserialize, Debug)]
struct SubstrateTick {
    symbol: String,
    timestamp: u64,
    price: f64,
    volume: f64,
    is_buyer_maker: bool,
}

#[derive(Serialize)]
struct TradeEvent {
    signal_tick: usize,
    execution_tick: usize,
    expected_price: f64,
    actual_price: f64,
    position_after: i32,
}

#[derive(Serialize)]
struct TimelineReferences {
    market_tick_index: usize,
    signal_index: usize,
    execution_index: usize,
    portfolio_index: usize,
}

#[derive(Serialize)]
struct TradeLeg {
    signal_time: usize,
    fill_time: usize,
    fill_price: f64,
}

#[derive(Serialize)]
struct ExecutionDeltaLayer {
    delay_ms: u64,
    delay_ticks: usize,
    slippage_bps: f64,
    missed_fill: bool,
    diverged: bool,
}

#[derive(Serialize)]
struct DecisionLayer {
    signal_timestamp: usize,
    signal_type: String,
}

#[derive(Serialize)]
struct TradeDelta {
    trade_id: String,
    strategy: String,
    signal: DecisionLayer,
    baseline: Option<TradeLeg>,
    perturbed: Option<TradeLeg>,
    delta: ExecutionDeltaLayer,
    explanations: Vec<String>, // We will store rule IDs here to map to rules array
    timeline_refs: TimelineReferences,
}

#[derive(Serialize)]
struct SimulationLayer {
    state_stream: Vec<i32>,
    trades: Vec<TradeEvent>,
    missed_fills: usize,
    total_latency_ticks: usize,
}

#[derive(Serialize)]
struct RawTimelineEvent {
    tick: usize,
    market: MarketEvent,
    strategy: RawStrategyEvent,
    execution: RawExecutionEvent,
    portfolio: RawPortfolioEvent,
}

#[derive(Serialize)]
struct MarketEvent {
    price: f64,
}

#[derive(Serialize)]
struct RawStrategyEvent {
    signal: Option<String>,
}

#[derive(Serialize)]
struct RawExecutionEvent {
    order_id: Option<String>,
    status: Option<String>,
}

#[derive(Serialize)]
struct RawPortfolioEvent {
    position: i32,
}

#[derive(Serialize)]
struct DivergenceCascadeEvent {
    tick: usize,
    event: String,
}

#[derive(Serialize)]
struct StrategyEventMerged {
    intent: Option<String>,
}

#[derive(Serialize)]
struct ExecutionEventMerged {
    baseline_fill: bool,
    perturbed_fill: bool,
    missed_fill: bool,
}

#[derive(Serialize)]
struct PortfolioEventMerged {
    baseline_position: i32,
    perturbed_position: i32,
}

#[derive(Serialize)]
struct TimelineLaneEvent {
    tick: usize,
    market: MarketEvent,
    signal: StrategyEventMerged,
    execution: ExecutionEventMerged,
    portfolio: PortfolioEventMerged,
}

#[derive(Serialize)]
struct TimelineData {
    divergence_anchor_tick: Option<usize>,
    divergence_anchor_trade_id: Option<String>,
    divergence_reason: Option<String>,
    cascade: Vec<DivergenceCascadeEvent>,
    lanes: Vec<TimelineLaneEvent>,
}

#[derive(Serialize)]
struct Rule {
    id: String,
    #[serde(rename = "type")]
    rule_type: String,
    severity: String,
    message: String,
}

#[derive(Serialize)]
struct ArtifactSummary {
    what_happened: String,
    where_divergence_started: Option<usize>,
    primary_cause: String,
    severity: String,
}

#[derive(Serialize)]
struct Analytics {
    trades: usize,
    fill_rate: f64,
    missed_fills: usize,
    average_delay_ticks: f64,
    average_slippage_bps: f64,
    execution_efficiency: f64,
    simple_pnl: f64,
}

#[derive(Serialize)]
struct TraceArtifact {
    metadata: Metadata,
    environment: Environment,
    strategy: StrategyConfig,
    baseline: SimulationLayer,
    perturbed: SimulationLayer,
    trade_deltas: Vec<TradeDelta>,
    timeline: TimelineData,
    rules: Vec<Rule>,
    divergence: Divergence,
    analytics: Analytics,
    artifact_summary: ArtifactSummary,
}

#[derive(Serialize)]
struct Metadata {
    generated_at: u64,
    compiler_version: String,
}

#[derive(Serialize)]
struct Environment {
    substrate_file: String,
    total_ticks: usize,
    latency_injected_ms: u64,
    missed_fill_prob: f64,
}

#[derive(Serialize)]
struct StrategyConfig {
    archetype: String,
}

#[derive(Serialize)]
struct Divergence {
    sequence_fidelity: f64,
    structural_divergence: f64,
}

fn deterministic_rand(seed: u64, counter: u64) -> f64 {
    let mut state = seed.wrapping_add(counter).wrapping_add(0x9E3779B97F4A7C15);
    state ^= state >> 30;
    state = state.wrapping_mul(0xBF58476D1CE4E5B9);
    state ^= state >> 27;
    state = state.wrapping_mul(0x94D049BB133111EB);
    state ^= state >> 31;
    (state as f64) / (u64::MAX as f64)
}

struct ArchetypeState {
    pub layer: SimulationLayer,
    pub pending_signal: Option<(usize, f64)>, // (tick, price)
    pub current_pos: i32,
    pub exit_tick: Option<usize>,
    pub entry_price: f64,
}

impl ArchetypeState {
    fn new() -> Self {
        Self {
            layer: SimulationLayer {
                state_stream: vec![],
                trades: vec![],
                missed_fills: 0,
                total_latency_ticks: 0,
            },
            pending_signal: None,
            current_pos: 0,
            exit_tick: None,
            entry_price: 0.0,
        }
    }

    fn process_queue(&mut self, idx: usize, price: f64, timestamps: &[u64], latency_ms: u64, miss_prob: f64) {
        if let Some(exit_t) = self.exit_tick {
            if idx >= exit_t {
                self.current_pos = 0;
                self.exit_tick = None;
                // exit trade tracking could go here, but we focus on entry for now
            }
        }

        let ticks_to_slip = (latency_ms / 10) as usize;

        if let Some((signal_idx, expected_price)) = self.pending_signal {
            if idx >= signal_idx + ticks_to_slip {
                let rand_val = deterministic_rand(timestamps[idx], latency_ms + idx as u64);
                if rand_val >= miss_prob {
                    if self.current_pos == 0 {
                        self.current_pos = 1;
                        self.exit_tick = Some(idx + 100);
                        self.entry_price = price;
                        
                        self.layer.trades.push(TradeEvent {
                            signal_tick: signal_idx,
                            execution_tick: idx,
                            expected_price,
                            actual_price: price,
                            position_after: 1,
                        });
                        self.layer.total_latency_ticks += idx - signal_idx;
                    }
                } else {
                    self.layer.missed_fills += 1;
                }
                self.pending_signal = None;
            }
        }

        self.layer.state_stream.push(self.current_pos);
    }
}

fn simulate(strategy: &str, prices: &[f64], timestamps: &[u64], latency_ms: u64, miss_prob: f64) -> (SimulationLayer, Vec<RawTimelineEvent>) {
    let mut state = ArchetypeState::new();
    let twap_intervals = vec![120, 360, 600, 840, 1080, 1320];
    
    let mut timeline = Vec::with_capacity(prices.len());

    for i in 0..prices.len() {
        let prev_pending = state.pending_signal.is_some();
        let prev_pos = state.current_pos;
        let prev_trades = state.layer.trades.len();
        
        state.process_queue(i, prices[i], timestamps, latency_ms, miss_prob);

        let mut signal_generated = None;

        match strategy {
            "twap" => {
                if twap_intervals.contains(&i) && state.pending_signal.is_none() && state.current_pos == 0 {
                    state.pending_signal = Some((i, prices[i]));
                    signal_generated = Some("BUY".to_string());
                }
            }
            "breakout" => {
                if i >= 50 {
                    let local_high = prices[i-50..i].iter().cloned().fold(f64::NAN, f64::max);
                    if prices[i] > local_high && state.pending_signal.is_none() && state.current_pos == 0 {
                        state.pending_signal = Some((i, prices[i]));
                        signal_generated = Some("BUY".to_string());
                    }
                }
            }
            "momentum" => {
                if i >= 3 {
                    if prices[i] > prices[i-1] && prices[i-1] > prices[i-2] && prices[i-2] > prices[i-3] {
                        if state.pending_signal.is_none() && state.current_pos == 0 {
                            state.pending_signal = Some((i, prices[i]));
                            signal_generated = Some("BUY".to_string());
                        }
                    }
                }
            }
            "mean_reversion" => {
                if i >= 20 {
                    let sma = prices[i-20..=i].iter().sum::<f64>() / 21.0;
                    if prices[i] < sma * 0.998 {
                        if state.pending_signal.is_none() && state.current_pos == 0 {
                            state.pending_signal = Some((i, prices[i]));
                            signal_generated = Some("BUY".to_string());
                        }
                    }
                }
            }
            _ => {}
        }
        
        let mut order_id = None;
        let mut exec_status = None;
        
        if signal_generated.is_some() {
            order_id = Some(format!("O{}", state.layer.trades.len() + 1));
            exec_status = Some("SUBMITTED".to_string());
        } else if prev_pending && state.pending_signal.is_none() {
            // Either filled or missed
            order_id = Some(format!("O{}", prev_trades + 1));
            if state.layer.trades.len() > prev_trades {
                exec_status = Some("FILLED".to_string());
            } else {
                exec_status = Some("MISSED".to_string());
            }
        } else if state.pending_signal.is_some() {
            order_id = Some(format!("O{}", state.layer.trades.len() + 1));
            exec_status = Some("DELAYED".to_string());
        }

        timeline.push(RawTimelineEvent {
            tick: i,
            market: MarketEvent { price: prices[i] },
            strategy: RawStrategyEvent { signal: signal_generated },
            execution: RawExecutionEvent { order_id, status: exec_status },
            portfolio: RawPortfolioEvent { position: state.current_pos },
        });
    }

    (state.layer, timeline)
}

fn compute_lcs(seq1: &[i32], seq2: &[i32]) -> usize {
    let m = seq1.len();
    let n = seq2.len();
    let mut dp = vec![0; n + 1];
    for i in 1..=m {
        let mut prev = 0;
        for j in 1..=n {
            let temp = dp[j];
            if seq1[i-1] == seq2[j-1] {
                dp[j] = prev + 1;
            } else {
                dp[j] = std::cmp::max(dp[j], dp[j-1]);
            }
            prev = temp;
        }
    }
    dp[n]
}

fn main() {
    let args = Args::parse();
    let file = File::open(&args.substrate_file).expect("Failed to open substrate file");
    let reader = BufReader::new(file);
    
    let mut prices = Vec::new();
    let mut timestamps = Vec::new();
    
    for line in reader.lines() {
        if let Ok(l) = line {
            if l.trim().is_empty() { continue; }
            if let Ok(tick) = serde_json::from_str::<SubstrateTick>(&l) {
                prices.push(tick.price);
                timestamps.push(tick.timestamp);
            }
        }
    }

    let (baseline, baseline_timeline) = simulate(&args.strategy, &prices, &timestamps, 0, 0.0);
    let (perturbed, perturbed_timeline) = simulate(&args.strategy, &prices, &timestamps, args.latency_ms, args.missed_fill_prob);

    let n = baseline.state_stream.len();
    let same_ticks = baseline.state_stream.iter().zip(perturbed.state_stream.iter()).filter(|(a, b)| a == b).count();
    let structural_divergence = 1.0 - (same_ticks as f64 / n as f64);
    
    let lcs = compute_lcs(&baseline.state_stream, &perturbed.state_stream);
    let sequence_fidelity = lcs as f64 / n as f64;

    let mut rules = vec![];
    if structural_divergence > 0.05 {
        rules.push(Rule {
            id: "R1".to_string(),
            rule_type: "LATENCY_OFFSET".to_string(),
            severity: "critical".to_string(),
            message: "Latency offset structurally warped exposure window by >5%".to_string(),
        });
    }
    if perturbed.missed_fills > baseline.missed_fills {
        rules.push(Rule {
            id: "R2".to_string(),
            rule_type: "QUEUE_EXHAUSTION".to_string(),
            severity: "critical".to_string(),
            message: "Missed fills increased due to queue exhaustion".to_string(),
        });
    }
    if perturbed.total_latency_ticks > baseline.total_latency_ticks {
        rules.push(Rule {
            id: "R3".to_string(),
            rule_type: "ENTRY_DRIFT".to_string(),
            severity: "warning".to_string(),
            message: "Entry drift detected".to_string(),
        });
    }

    let mut trade_deltas = vec![];
    let max_trades = std::cmp::max(baseline.trades.len(), perturbed.trades.len());
    
    for i in 0..max_trades {
        let b = baseline.trades.get(i);
        let p = perturbed.trades.get(i);
        
        let mut diverged = false;
        let mut delay_ticks = 0;
        let mut slippage_bps = 0.0;
        let mut explanation_refs = vec![];
        
        if let (Some(base), Some(pert)) = (b, p) {
            delay_ticks = pert.execution_tick.saturating_sub(base.execution_tick);
            if delay_ticks > 0 {
                diverged = true;
                explanation_refs.push("R3".to_string());
            }
            slippage_bps = (pert.actual_price - base.actual_price) / base.actual_price * 10000.0;
            if slippage_bps > 0.0 {
                diverged = true;
            }
        } else if b.is_some() || p.is_some() {
            diverged = true;
            if p.is_none() {
                explanation_refs.push("R2".to_string());
            }
        }
        
        if structural_divergence > 0.05 && diverged {
            explanation_refs.push("R1".to_string());
        }
        
        let delay_ms = delay_ticks as u64 * (if args.latency_ms > 0 { args.latency_ms / delay_ticks.max(1) as u64 } else { 0 });

        let signal_tick = p.map(|t| t.signal_tick).or(b.map(|t| t.signal_tick)).unwrap_or(0);

        trade_deltas.push(TradeDelta {
            trade_id: format!("T{}", i + 1),
            strategy: args.strategy.clone(),
            signal: DecisionLayer {
                signal_timestamp: signal_tick,
                signal_type: "BUY".to_string(),
            },
            baseline: b.map(|t| TradeLeg {
                signal_time: t.signal_tick,
                fill_time: t.execution_tick,
                fill_price: t.actual_price,
            }),
            perturbed: p.map(|t| TradeLeg {
                signal_time: t.signal_tick,
                fill_time: t.execution_tick,
                fill_price: t.actual_price,
            }),
            delta: ExecutionDeltaLayer {
                delay_ms,
                delay_ticks,
                slippage_bps,
                missed_fill: p.is_none(),
                diverged,
            },
            explanations: explanation_refs,
            timeline_refs: TimelineReferences {
                market_tick_index: signal_tick,
                signal_index: signal_tick,
                execution_index: p.map(|t| t.execution_tick).or(b.map(|t| t.execution_tick)).unwrap_or(signal_tick),
                portfolio_index: p.map(|t| t.execution_tick).or(b.map(|t| t.execution_tick)).unwrap_or(signal_tick),
            },
        });
    }

    let trades_count = perturbed.trades.len();
    let perturbed_missed_fills = perturbed.missed_fills;
    let mut total_slippage_bps = 0.0;
    let mut simple_pnl = 0.0;
    
    for t in &perturbed.trades {
        let slip = (t.actual_price - t.expected_price) / t.expected_price * 10000.0;
        total_slippage_bps += slip;
        
        // Simple 100-tick exit pnl approximation for MVP
        let exit_tick = std::cmp::min(t.execution_tick + 100, prices.len() - 1);
        let exit_price = prices[exit_tick];
        simple_pnl += (exit_price - t.actual_price) / t.actual_price;
    }

    let avg_delay = if trades_count > 0 { perturbed.total_latency_ticks as f64 / trades_count as f64 } else { 0.0 };
    let avg_slip = if trades_count > 0 { total_slippage_bps / trades_count as f64 } else { 0.0 };
    
    let total_signals = trades_count + perturbed_missed_fills;
    let fill_rate = if total_signals > 0 { trades_count as f64 / total_signals as f64 } else { 1.0 };
    
    let baseline_pnl = baseline.trades.iter().map(|t| {
        let exit_tick = std::cmp::min(t.execution_tick + 100, prices.len() - 1);
        (prices[exit_tick] - t.actual_price) / t.actual_price
    }).sum::<f64>();
    
    let execution_efficiency = if baseline_pnl > 0.0 { simple_pnl / baseline_pnl } else { 1.0 };

    let mut anchor_tick = None;
    let mut anchor_trade = None;
    let mut cascade = Vec::new();
    let mut lanes = Vec::with_capacity(prices.len());

    let mut currently_delayed = false;
    let mut currently_offset = false;

    for i in 0..prices.len() {
        let b_ev = &baseline_timeline[i];
        let p_ev = &perturbed_timeline[i];

        let b_fill = b_ev.execution.status.as_deref() == Some("FILLED");
        let p_fill = p_ev.execution.status.as_deref() == Some("FILLED");
        let p_miss = p_ev.execution.status.as_deref() == Some("MISSED");
        let p_delay = p_ev.execution.status.as_deref() == Some("DELAYED");
        
        let b_pos = b_ev.portfolio.position;
        let p_pos = p_ev.portfolio.position;

        if anchor_tick.is_none() && (p_delay || p_miss || (p_pos != b_pos)) {
            anchor_tick = Some(i);
            anchor_trade = p_ev.execution.order_id.clone().or(b_ev.execution.order_id.clone());
        }

        if anchor_tick.is_some() {
            if p_delay && !currently_delayed {
                cascade.push(DivergenceCascadeEvent { tick: i, event: "DELAYED_FILL".to_string() });
                currently_delayed = true;
            }
            if p_fill && currently_delayed {
                cascade.push(DivergenceCascadeEvent { tick: i, event: "LATE_FILL_EXECUTED".to_string() });
                currently_delayed = false;
            }
            if p_miss && !currently_delayed {
                cascade.push(DivergenceCascadeEvent { tick: i, event: "MISSED_FILL".to_string() });
            }
            
            if p_pos != b_pos && !currently_offset {
                cascade.push(DivergenceCascadeEvent { tick: i, event: "EXPOSURE_OFFSET_BEGIN".to_string() });
                currently_offset = true;
            } else if p_pos == b_pos && currently_offset {
                cascade.push(DivergenceCascadeEvent { tick: i, event: "EXPOSURE_OFFSET_END".to_string() });
                currently_offset = false;
            }
        }

        lanes.push(TimelineLaneEvent {
            tick: i,
            market: MarketEvent { price: prices[i] },
            signal: StrategyEventMerged { intent: p_ev.strategy.signal.clone() },
            execution: ExecutionEventMerged {
                baseline_fill: b_fill,
                perturbed_fill: p_fill,
                missed_fill: p_miss,
            },
            portfolio: PortfolioEventMerged {
                baseline_position: b_pos,
                perturbed_position: p_pos,
            },
        });
    }
    
    // Add entry drift if needed
    if let Some(t) = anchor_tick {
        if let Some(r) = rules.iter().find(|r| r.rule_type == "ENTRY_DRIFT") {
            cascade.push(DivergenceCascadeEvent { tick: t, event: "ENTRY_DRIFT".to_string() });
        }
    }
    cascade.sort_by_key(|c| c.tick);

    let timeline_data = TimelineData {
        divergence_anchor_tick: anchor_tick,
        divergence_anchor_trade_id: anchor_trade,
        divergence_reason: rules.first().map(|r| r.rule_type.clone()),
        cascade,
        lanes,
    };

    let summary = ArtifactSummary {
        what_happened: if structural_divergence > 0.05 {
            "Latency caused material structural divergence".to_string()
        } else if perturbed_missed_fills > baseline.missed_fills {
            "Execution quality degraded via missed fills".to_string()
        } else {
            "Execution was delayed but structurally intact".to_string()
        },
        where_divergence_started: anchor_tick,
        primary_cause: rules.first().map(|r| r.rule_type.clone()).unwrap_or_else(|| "NONE".to_string()),
        severity: rules.first().map(|r| r.severity.clone()).unwrap_or_else(|| "LOW".to_string()),
    };

    let artifact = TraceArtifact {
        metadata: Metadata {
            generated_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            compiler_version: "1.0.0".to_string(),
        },
        environment: Environment {
            substrate_file: args.substrate_file.clone(),
            total_ticks: n,
            latency_injected_ms: args.latency_ms,
            missed_fill_prob: args.missed_fill_prob,
        },
        strategy: StrategyConfig {
            archetype: args.strategy.clone(),
        },
        baseline,
        perturbed,
        trade_deltas,
        timeline: timeline_data,
        rules,
        divergence: Divergence {
            sequence_fidelity,
            structural_divergence,
        },
        analytics: Analytics {
            trades: trades_count,
            fill_rate,
            missed_fills: perturbed_missed_fills,
            average_delay_ticks: avg_delay,
            average_slippage_bps: avg_slip,
            execution_efficiency,
            simple_pnl,
        },
        artifact_summary: summary,
    };

    println!("{}", serde_json::to_string_pretty(&artifact).unwrap());
}
