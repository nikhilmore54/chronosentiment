use chronosentiment_core::ga::Strategy;
use chronosentiment_core::pnl_overlay::run_pnl_overlay;
use chronosentiment_core::replay_evaluator::run_replay_with_evaluator;
use chronosentiment_core::strategy_ranking::{
    LiveEvaluator, LiveMarketState, LiveRegime, RankingWeights, StrategyProfile, StrategyRegistry,
};
use chronosentiment_core::tick_replay::{ReplayConfig, ReplayMode, TickReplayEngine};

fn main() {
    let jsonl_path = std::env::var("BINANCE_JSONL")
        .unwrap_or_else(|_| "test_assets/binance_ticks.jsonl".to_string());
    let top_k = std::env::var("TOP_K")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(5);

    let mut replay = match TickReplayEngine::from_binance_jsonl(
        &jsonl_path,
        ReplayConfig {
            mode: ReplayMode::Fast,
            ..ReplayConfig::default()
        },
        1,
    ) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to load replay file {}: {}", jsonl_path, e);
            return;
        }
    };

    let registry = StrategyRegistry::new(vec![
        StrategyProfile {
            strategy_id: "strat_live_momentum".to_string(),
            strategy: Strategy {
                queue_threshold: 80,
                base_edge: 2,
                take_profit: 12,
                stop_loss: 6,
                holding_period: 20,
                w_conviction: 100,
                w_momentum: 80,
                w_volatility: 10,
                exp_conviction: 150,
                exp_momentum: 150,
                exp_volatility: 150,
                selectivity: 75,
                archetype: 0,
            },
            preferred_regimes: vec![LiveRegime::TrendingUp, LiveRegime::TrendingDown],
            confidence_weight: 0.8,
            execution_weight: 0.9,
        },
        StrategyProfile {
            strategy_id: "strat_live_conservative".to_string(),
            strategy: Strategy {
                queue_threshold: 120,
                base_edge: 1,
                take_profit: 8,
                stop_loss: 8,
                holding_period: 20,
                w_conviction: 90,
                w_momentum: 70,
                w_volatility: 15,
                exp_conviction: 120,
                exp_momentum: 120,
                exp_volatility: 120,
                selectivity: 85,
                archetype: 1,
            },
            preferred_regimes: vec![LiveRegime::Sideways, LiveRegime::Mixed],
            confidence_weight: 0.9,
            execution_weight: 0.8,
        },
    ]);

    let mut evaluator = LiveEvaluator::new(
        LiveMarketState::new("BTCUSDT".to_string()),
        registry,
        RankingWeights::default(),
    );
    let out = run_replay_with_evaluator(&mut replay, &mut evaluator, top_k);

    println!("replay_file={}", jsonl_path);
    println!("total_events={}", out.metrics.total_events);
    println!("events_with_signals={}", out.metrics.events_with_signals);
    println!("participation_pct={:.2}", out.metrics.participation_pct);
    println!(
        "avg_suggestions_per_event={:.4}",
        out.metrics.avg_suggestions_per_event
    );
    println!(
        "avg_suggestions_per_signal_event={:.4}",
        out.metrics.avg_suggestions_per_signal_event
    );
    println!("strategy_flips={}", out.metrics.strategy_flips);
    println!(
        "debug: low_edge={} low_exec={} hold={} stability={}",
        out.metrics.final_debug.rejected_low_edge,
        out.metrics.final_debug.rejected_low_exec,
        out.metrics.final_debug.rejected_hold,
        out.metrics.final_debug.suppressed_stability
    );
    println!("--- top strategy persistence (ticks) ---");
    for (k, v) in out.metrics.top_strategy_persistence_ticks {
        println!("{}={}", k, v);
    }

    // PnL overlay run (fresh replay/evaluator instance for deterministic consistency).
    let mut replay_for_pnl = match TickReplayEngine::from_binance_jsonl(
        &jsonl_path,
        ReplayConfig {
            mode: ReplayMode::Fast,
            ..ReplayConfig::default()
        },
        1,
    ) {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "Failed to reload replay file {} for pnl overlay: {}",
                jsonl_path, e
            );
            return;
        }
    };
    let registry_for_pnl = StrategyRegistry::new(vec![
        StrategyProfile {
            strategy_id: "strat_live_momentum".to_string(),
            strategy: Strategy {
                queue_threshold: 80,
                base_edge: 2,
                take_profit: 12,
                stop_loss: 6,
                holding_period: 20,
                w_conviction: 100,
                w_momentum: 80,
                w_volatility: 10,
                exp_conviction: 150,
                exp_momentum: 150,
                exp_volatility: 150,
                selectivity: 75,
                archetype: 0,
            },
            preferred_regimes: vec![LiveRegime::TrendingUp, LiveRegime::TrendingDown],
            confidence_weight: 0.8,
            execution_weight: 0.9,
        },
        StrategyProfile {
            strategy_id: "strat_live_conservative".to_string(),
            strategy: Strategy {
                queue_threshold: 120,
                base_edge: 1,
                take_profit: 8,
                stop_loss: 8,
                holding_period: 20,
                w_conviction: 90,
                w_momentum: 70,
                w_volatility: 15,
                exp_conviction: 120,
                exp_momentum: 120,
                exp_volatility: 120,
                selectivity: 85,
                archetype: 1,
            },
            preferred_regimes: vec![LiveRegime::Sideways, LiveRegime::Mixed],
            confidence_weight: 0.9,
            execution_weight: 0.8,
        },
    ]);
    let mut evaluator_for_pnl = LiveEvaluator::new(
        LiveMarketState::new("BTCUSDT".to_string()),
        registry_for_pnl,
        RankingWeights::default(),
    );
    let horizon_ticks = std::env::var("PNL_HORIZON_TICKS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(20);
    let (_trades, pnl) =
        run_pnl_overlay(&mut replay_for_pnl, &mut evaluator_for_pnl, horizon_ticks);
    println!(
        "PNL_OVERLAY: trades={} win_rate={:.2}% avg_pnl={:.6} total_pnl={:.6} edge_retention={:.6}",
        pnl.total_trades,
        pnl.win_rate * 100.0,
        pnl.avg_pnl,
        pnl.total_pnl,
        pnl.edge_retention
    );
}
