import re

with open("core/src/ga.rs", "r") as f:
    ga_text = f.read()

new_legacy = """impl StrategyEvaluation {
    pub fn new_legacy(
        strategy_id: String,
        strategy: Strategy,
        fitness: f64,
        avg_pnl: f64,
        trade_count: usize,
        profitable_trades: usize,
        avg_entropy: f64,
    ) -> Self {
        let mut slf = Self::new_legacy_with_flag("EMPTY");
        slf.strategy_id = strategy_id;
        slf.strategy = strategy.clone();
        slf.behavioral_signature = strategy.get_signature();
        slf.fitness = fitness;
        slf.avg_pnl = avg_pnl;
        slf.trade_count = trade_count;
        slf.profitable_trades = profitable_trades;
        slf.win_rate = if trade_count > 0 {
            profitable_trades as f64 / trade_count as f64
        } else {
            0.0
        };
        slf.evaluation_flag = None;
        slf
    }

    pub fn new_legacy_with_flag(flag: &str) -> Self {
        Self {
            winner_idx: 0,
            strategy_id: "FLAGGED".to_string(),
            strategy: Strategy::from_seed(0),
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
ga_text = ga_text.replace("impl StrategyEvaluation {", new_legacy, 1)

with open("core/src/ga.rs", "w") as f:
    f.write(ga_text)

with open("core/src/pipeline.rs", "r") as f:
    pipe = f.read()

pipe = pipe.replace("use crate::safe_log;", "")

with open("core/src/pipeline.rs", "w") as f:
    f.write(pipe)
