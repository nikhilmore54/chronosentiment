import re

file_path = "core/src/ga.rs"
with open(file_path, "r") as f:
    text = f.read()

# 1. Convert f64 execution genes to u8
text = text.replace("pub exec_aggression: f64,", "pub exec_aggression: u8,")
text = text.replace("pub latency_bias: f64,", "pub latency_bias: u8,")
text = text.replace("pub fill_threshold: f64,", "pub fill_threshold: u8,")

# Scale rng.gen_range
text = text.replace("exec_aggression: rng.gen_range(0.2..0.8),", "exec_aggression: rng.gen_range(20..80),")
text = text.replace("latency_bias: rng.gen_range(0.0..1.0),", "latency_bias: rng.gen_range(0..100),")
text = text.replace("fill_threshold: rng.gen_range(0.2..0.8),", "fill_threshold: rng.gen_range(20..80),")

# Scale from_seed
text = text.replace("exec_aggression: 0.5,", "exec_aggression: 50,")
text = text.replace("latency_bias: 0.1,", "latency_bias: 10,")
text = text.replace("fill_threshold: 0.5,", "fill_threshold: 50,")

# mutate logic fixing
old_mutate = """        new.exec_aggression = (new.exec_aggression + adjusted_delta).clamp(0.0, 1.0);
        new.latency_bias = (new.latency_bias + adjusted_delta).clamp(0.0, 1.0);
        new.fill_threshold = (new.fill_threshold + adjusted_delta).clamp(0.0, 1.0);"""
new_mutate = """        new.exec_aggression = (new.exec_aggression as f64 + adjusted_delta * 100.0).clamp(0.0, 100.0) as u8;
        new.latency_bias = (new.latency_bias as f64 + adjusted_delta * 100.0).clamp(0.0, 100.0) as u8;
        new.fill_threshold = (new.fill_threshold as f64 + adjusted_delta * 100.0).clamp(0.0, 100.0) as u8;"""
text = text.replace(old_mutate, new_mutate)

# 2. Fix duplicated pnl in SignalAlpha
# Let's find struct SignalAlpha and remove duplicate pnl
signal_struct = re.search(r'pub struct SignalAlpha \{.*?\}', text, re.DOTALL).group(0)
fixed_signal = re.sub(r'(\s*pub pnl: f64,)', '', signal_struct, count=1)
text = text.replace(signal_struct, fixed_signal)

# 3. Fix multiple new_legacy_with_flag implementations
# There is a block we inserted that looks like impl StrategyEvaluation { pub fn new_legacy_with_flag(flag: &str) -> Self { let s = Strategy::from_seed(0); ... }
# Then there is the existing one. We must remove all but one!
blocks = text.split("pub fn new_legacy_with_flag(flag: &str) -> Self {")
if len(blocks) > 2:
    # preserve the first split and the last block (the original one, wait we want to keep the one we injected)
    # The first one we injected is at the top. Let's find all `impl StrategyEvaluation { ... pub fn new_legacy_with_flag` 
    pass
# It's safer to just replace using regex.
text = re.sub(r'impl StrategyEvaluation \{\n    pub fn new_legacy_with_flag.*?\}\n    \}\n', 'impl StrategyEvaluation {\n', text, flags=re.DOTALL, count=1)
# That removes the first duplicated injected block. We still need to replace the original with explicit fields.
old_legacy = """    pub fn new_legacy_with_flag(flag: &str) -> Self {
        Self {
            evaluation_flag: Some(flag.to_string()),
            ..Default::default()
        }
    }"""
new_legacy = """    pub fn new_legacy_with_flag(flag: &str) -> Self {
        let s = Strategy::from_seed(0);
        Self {
            evaluation_flag: Some(flag.to_string()),
            strategy_id: "FLAGGED".to_string(),
            strategy: s,
            fitness: -0.03,
            pnl: 0.0,
            eff: 0.0,
            confidence: 0.0,
            trade_count: 0,
            execution_trace: None,
            capability: crate::ga::ScenarioCapability::Executable,
            real_dom: 0.0,
            had_organic_signals: false,
            std_dev: 0.0,
            downside_std_dev: 0.0,
            worst: 0.0,
            robustness: 0.0,
            max_signature_credibility: 0.0,
            forced_win_ratio: 0.0,
            max_drawdown: 0.0,
            participation_rate: 0.0,
            profitable_trades: 0,
            zero_pnl_trades: 0,
            quality_trades: 0.0,
            total_pnl: 0.0,
            avg_pnl: 0.0,
            pnl_history: Vec::new(),
            win_rate: 0.0,
            payoff: 0.0,
            payoff_ratio: 0.0,
            direction_ratio: 0.0,
            baseline_pnl: 0.0,
            execution_metrics: ExecutionMetrics {
                fill_rate: 0.0,
                avg_slippage: 0.0,
                latency_impact: 0.0,
                capture_efficiency: 0.0,
                fill_efficiency: 0.0,
                profit_factor: 0.0,
                volume_participation: 0.0,
            },
            scenario_signature: ScenarioExecutionSignature {
                avg_queue_ahead: 0.0,
                avg_latency: 0.0,
                participation: 0.0,
                execution_variance: 0.0,
            },
            pnl_fingerprint: Vec::new(),
            behavioral_signature: BehavioralSignature { fingerprint: 0, axes: (0,0,0,0) },
            avg_conviction: 0.0,
            avg_efficiency: 0.0,
            avg_edge_quality: 0.0,
            directional_accuracy: 0.0,
            decisiveness: 0.0,
            execution_friction: 0.0,
            emitted_signals: Vec::new(),
            short_term_capture_eff: 0.0,
            long_term_capture_eff: 0.0,
            trade_density: 0.0,
            queue_blocked_count: 0,
            liquidity_starved_count: 0,
            total_attempts: 0,
            exec_opportunity_rate: 0.0,
            failure_profile: Vec::new(),
            realized_pnl_rolling: 0.0,
            predicted_pnl_rolling: 0.0,
            trade_qualities: Vec::new(),
            outcome_consistency: 0.0,
            avg_trade_quality: 0.0,
            std_trade_quality: 0.0,
            exit_tp_count: 0,
            exit_sl_count: 0,
            exit_ts_count: 0,
            avg_hold_time: 0.0,
            consistency_score: 0.0,
            recent_performance: 0.0,
            pnl_from_tp: 0.0,
            pnl_from_sl: 0.0,
        }
    }"""
text = text.replace(old_legacy, new_legacy)

# 4. Fix missing initializations of Strategy at line 3835, 4761, 4807.
missing_genes = """            exec_aggression: 50,
            latency_bias: 10,
            fill_threshold: 50,"""
text = re.sub(r'(strategy: Strategy \{[^}]*particip[^\n]*)(\n\s*\})', r'\1\n' + missing_genes + r'\2', text)

with open(file_path, "w") as f:
    f.write(text)
