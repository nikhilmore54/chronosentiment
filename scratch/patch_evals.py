import re

with open("core/src/ga.rs", "r") as f:
    text = f.read()

# 1. edge_decay.rs 117 missing fields
with open("core/src/edge_decay.rs", "r") as f:
    edge_text = f.read()
edge_text = re.sub(r'strategy_id\)\.unwrap_or\(Strategy \{([^}]*particip[^\n]*)\n\s*\}\)', r'strategy_id).unwrap_or(Strategy {\1\n            exec_aggression: 50,\n            latency_bias: 10,\n            fill_threshold: 50,\n        })', edge_text)
with open("core/src/edge_decay.rs", "w") as f:
    f.write(edge_text)

# 2. ga.rs missing fields
# We need to find Strategy { ... } where it doesn't have exec_aggression
def add_missing(match):
    s = match.group(0)
    if "exec_aggression:" not in s:
        # insert before the closing brace
        return s.replace("}", "    exec_aggression: 50,\n            latency_bias: 10,\n            fill_threshold: 50,\n        }")
    return s

text = re.sub(r'Strategy \{.*?\}', add_missing, text, flags=re.DOTALL)

# 3. StrategyEvaluation missing evaluation_flag at 1651
# 4. StrategyEvaluation multiple pnl and evaluation_flag inside new_legacy_with_flag
# Let's completely remove all new_legacy_with_flag implementations and inject one clean one.
text = re.sub(r'impl StrategyEvaluation \{\n    pub fn new_legacy_with_flag.*?pnl_from_sl:\s*0\.0,\n        \}\n    \}', '', text, flags=re.DOTALL)

# Inject standard new_legacy_with_flag into impl StrategyEvaluation {
clean_legacy = """impl StrategyEvaluation {
    pub fn new_legacy_with_flag(flag: &str) -> Self {
        let s = Strategy::from_seed(0);
        Self {
            winner_idx: 0,
            strategy_id: "FLAGGED".to_string(),
            strategy: s,
            capability: crate::ga::ScenarioCapability::Executable,
            real_dom: 0.0,
            had_organic_signals: false,
            std_dev: 0.0,
            downside_std_dev: 0.0,
            worst: 0.0,
            robustness: 0.0,
            max_signature_credibility: 0.0,
            forced_win_ratio: 0.0,
            fitness: -0.03,
            trade_count: 0,
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
                fill_efficiency: 0.0,
                capture_efficiency: 0.0,
                fill_rate: 0.0,
                avg_slippage: 0.0,
                latency_impact: 0.0,
                queue_blocked_count: 0,
                liquidity_starved_count: 0,
                total_attempts: 0,
            },
            scenario_signature: ScenarioExecutionSignature {
                avg_queue_ahead: 0.0,
                avg_latency: 0.0,
                fill_ratio: 0.0,
                participation: 0.0,
                execution_variance: 0.0,
            },
            pnl_fingerprint: Vec::new(),
            behavioral_signature: BehavioralSignature { fingerprint: 0, axes: (0,0,0,0) },
            evaluation_flag: Some(flag.to_string()),
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
text = re.sub(r'impl StrategyEvaluation \{', clean_legacy, text, count=1)

# Fix duplicate pnl in SignalAlpha struct definition
text = text.replace("    pub pnl: f64,\n    pub pnl: f64,\n}", "    pub pnl: f64,\n}")

with open("core/src/ga.rs", "w") as f:
    f.write(text)
