use crate::{dto::{EvaluateStrategyResponse, CompareStrategiesResponse, ComparisonSummary, InspectStrategyResponse, RunGaResponse, EventWrapper, TradeInspectorResponse},
    errors::ApiError,
};
use chronosentiment_core::{self, GaConfig, Strategy, SimEvent, SimulationResult};
use std::collections::HashMap;
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};
use serde_json::json;

// Placeholder for internal engine functions (these would ideally be in a core library)
// For now, we'll assume they are accessible or mocked.

const ORDER_PRICE: u64 = 100 * chronosentiment_core::PRICE_SCALE;
const ORDER_QUANTITY: u64 = 100;
const ORDER_TIMESTAMP: u64 = 12; // A reasonable timestamp for event-driven simulation

fn parse_strategy_from_id(strategy_id: &str) -> Option<chronosentiment_core::ga::Strategy> {
    let mut nums: Vec<u64> = Vec::new();
    for part in strategy_id.split('_').rev() {
        if let Ok(v) = part.parse::<u64>() {
            nums.push(v);
            if nums.len() == 4 {
                break;
            }
        }
    }
    if nums.len() < 4 {
        return None;
    }
    Some(chronosentiment_core::ga::Strategy {
        stop_loss: nums[0],
        take_profit: nums[1],
        base_edge: nums[2],
        queue_threshold: nums[3],
    })
}

fn map_regime(regime: &str) -> chronosentiment_core::strategy_ranking::LiveRegime {
    match regime {
        "trending_up" => chronosentiment_core::strategy_ranking::LiveRegime::TrendingUp,
        "trending_down" => chronosentiment_core::strategy_ranking::LiveRegime::TrendingDown,
        "sideways" => chronosentiment_core::strategy_ranking::LiveRegime::Sideways,
        "volatile" => chronosentiment_core::strategy_ranking::LiveRegime::Volatile,
        _ => chronosentiment_core::strategy_ranking::LiveRegime::Mixed,
    }
}

#[derive(Clone)]
pub struct EvaluationService {
    pub last_simulation: Arc<Mutex<Option<SimulationResult>>>,
    pub strategy_store: Arc<Mutex<Option<chronosentiment_core::pipeline::PersistedStrategyStore>>>,
}

impl EvaluationService {
    pub fn new() -> Self {
        let strategy_store_path = "/Users/nikhil/ChronoSentiment_MEGA_FINAL/test_assets/strategy_store.json";
        let loaded_store = chronosentiment_core::pipeline::load_strategy_store(strategy_store_path).ok();
        if loaded_store.is_some() {
            println!("API_INFO: Loaded pre-trained strategy store from {}", strategy_store_path);
        } else {
            println!("API_INFO: No strategy store found at {}; recommendations will trigger GA evolution", strategy_store_path);
        }

        Self {
            last_simulation: Arc::new(Mutex::new(None)),
            strategy_store: Arc::new(Mutex::new(loaded_store)),
        }
    }

    fn wrap_event(&self, event: &SimEvent) -> EventWrapper {
        let payload = match event {
            SimEvent::MarketEvent { subtype, price, quantity, side, .. } => json!({
                "subtype": format!("{:?}", subtype).to_uppercase(),
                "price": price,
                "quantity": quantity,
                "side": side.map(|s| format!("{:?}", s).to_uppercase()),
            }),
            SimEvent::OrderIntent { order_id, side, price, quantity, .. } => json!({
                "order_id": order_id,
                "side": format!("{:?}", side).to_uppercase(),
                "price": price,
                "quantity": quantity,
            }),
            SimEvent::OrderEnteredQueue { order_id, price, queue_ahead, .. } => json!({
                "order_id": order_id,
                "price": price,
                "queue_ahead": queue_ahead,
            }),
            SimEvent::PartialFill { order_id, filled_qty, price, .. } => json!({
                "order_id": order_id,
                "filled_qty": filled_qty,
                "price": price,
            }),
            SimEvent::QueueProgression { order_id, queue_ahead, .. } => json!({
                "order_id": order_id,
                "queue_ahead": queue_ahead,
            }),
            SimEvent::OrderFilled { order_id, .. } => json!({
                "order_id": order_id,
            }),
        };

        EventWrapper {
            sequence_id: event.sequence_id(),
            timestamp: event.timestamp(),
            event_type: match event {
                SimEvent::MarketEvent { .. } => "MarketEvent".to_string(),
                SimEvent::OrderIntent { .. } => "OrderIntent".to_string(),
                SimEvent::OrderEnteredQueue { .. } => "OrderEnteredQueue".to_string(),
                SimEvent::PartialFill { .. } => "PartialFill".to_string(),
                SimEvent::QueueProgression { .. } => "QueueProgression".to_string(),
                SimEvent::OrderFilled { .. } => "OrderFilled".to_string(),
            },
            parent_sequence_id: event.parent_sequence_id(),
            payload,
        }
    }

    pub fn load_all_real_scenarios(&self) -> HashMap<String, Vec<chronosentiment_core::MarketEvent>> {
        let source = chronosentiment_core::FolderCandleSource {
            folder_path: "/Users/nikhil/ChronoSentiment_MEGA_FINAL/test_assets".to_string(),
        };
        let assets_with_candles = source.load_all();
        let mut all_scenarios = HashMap::new();
        for (asset, candles) in assets_with_candles {
            let asset_scenarios = chronosentiment_core::pipeline::scenarios_from_candles(&asset, &candles);
            all_scenarios.extend(asset_scenarios);
        }
        all_scenarios
    }

    pub fn evaluate_strategy(
        &self,
        strategy_config: Strategy,
        scenario_names: Vec<String>,
        seed: u64,
    ) -> Result<EvaluateStrategyResponse, ApiError> {
        let scenarios_map = self.load_all_real_scenarios();

        // Default to all benchmark scenarios if none provided
        let scenario_names = if scenario_names.is_empty() {
            let mut names: Vec<String> = scenarios_map.keys().cloned().collect();
            names.sort();
            names
        } else {
            scenario_names
        };

        let mut selected_scenarios: HashMap<String, Vec<chronosentiment_core::MarketEvent>> = HashMap::new();
        for scenario_name in &scenario_names {
            let market_events = scenarios_map.get(scenario_name).ok_or_else(|| {
                ApiError::EngineError(format!("Scenario '{}' not found", scenario_name))
            })?;
            selected_scenarios.insert(scenario_name.clone(), market_events.clone());
        }

        // Store first scenario's trace for UI visualization
        if let Some(first_scenario_name) = scenario_names.first() {
             if let Some(market_events) = selected_scenarios.get(first_scenario_name) {
                let (_event_log, simulation_result, _) = chronosentiment_core::harness::run_simulation_harness(
                    chronosentiment_core::ExecutionMode::Real,
                    market_events.clone(),
                    vec![chronosentiment_core::CreateOrder {
                        order_id: format!("strat_{}_{}", first_scenario_name, strategy_config.queue_threshold),
                        side: chronosentiment_core::Side::Buy,
                        price: ORDER_PRICE,
                        quantity: ORDER_QUANTITY,
                        timestamp: ORDER_TIMESTAMP,
                        fill_probability: 0.5,
                    }],
                );
                let mut last_sim = self.last_simulation.lock().unwrap_or_else(|e| e.into_inner());
                *last_sim = Some(simulation_result);
             }
        }

        let unified_eval = chronosentiment_core::pipeline::run_evaluation_orchestration(
            strategy_config,
            &selected_scenarios,
            seed,
        ).map_err(|e| ApiError::InternalError(e))?;
        
        Ok(EvaluateStrategyResponse {
            strategy_evaluation: unified_eval,
        })
    }

    pub fn compare_strategies(
        &self,
        strategies: Vec<Strategy>,
        scenario_names: Vec<String>,
        seed: u64,
    ) -> Result<CompareStrategiesResponse, ApiError> {
        if strategies.len() < 2 {
            return Err(ApiError::ValidationError(
                "At least two strategies are required for comparison".to_string(),
            ));
        }

        let scenarios_map = self.load_all_real_scenarios();

        // Default to all benchmark scenarios if none provided
        let scenario_names = if scenario_names.is_empty() {
            let mut names: Vec<String> = scenarios_map.keys().cloned().collect();
            names.sort();
            names
        } else {
            scenario_names
        };

        let mut selected_scenarios: HashMap<String, Vec<chronosentiment_core::MarketEvent>> = HashMap::new();
        for scenario_name in &scenario_names {
            let market_events = scenarios_map.get(scenario_name).ok_or_else(|| {
                ApiError::EngineError(format!("Scenario '{}' not found", scenario_name))
            })?;
            selected_scenarios.insert(scenario_name.clone(), market_events.clone());
        }

        let rankings = chronosentiment_core::pipeline::run_comparison_orchestration(
            strategies,
            &selected_scenarios,
            seed,
        ).map_err(|e| ApiError::InternalError(e))?;

        let mut comparison_summary = ComparisonSummary {
            best_strategy: "".to_string(),
            reason: "No clear reason".to_string(),
        };

        if let Some(best) = rankings.first() {
            comparison_summary.best_strategy = best.strategy_id.clone();
            if rankings.len() >= 2 {
                comparison_summary.reason = format!("The best strategy {} had a higher fitness of {:.2}.", best.strategy_id, best.execution_fitness);
            }
        }

        Ok(CompareStrategiesResponse {
            ranking: rankings,
            comparison_summary,
        })
    }

    pub fn inspect_strategy(
        &self,
        strategy_config: Strategy,
        scenario_name: String,
        seed: u64,
    ) -> Result<InspectStrategyResponse, ApiError> {
        let scenarios_map = self.load_all_real_scenarios();
        let market_events = scenarios_map.get(&scenario_name).ok_or_else(|| {
            ApiError::EngineError(format!("Scenario '{}' not found", scenario_name))
        })?;

        let (event_log, simulation_result, _) = chronosentiment_core::harness::run_simulation_harness(
            chronosentiment_core::ExecutionMode::Real,
            market_events.clone(),
            vec![chronosentiment_core::CreateOrder {
                order_id: format!("strat_{}_{}", scenario_name, strategy_config.queue_threshold),
                side: chronosentiment_core::Side::Buy,
                price: ORDER_PRICE,
                quantity: ORDER_QUANTITY,
                timestamp: ORDER_TIMESTAMP,
                fill_probability: 0.5,
            }],
        );

        // Store events for the visualization layer
        {
            let mut last_sim = self.last_simulation.lock().unwrap_or_else(|e| e.into_inner());
            *last_sim = Some(simulation_result.clone());
        }

        let mut one_scenario: HashMap<String, Vec<chronosentiment_core::MarketEvent>> = HashMap::new();
        one_scenario.insert(scenario_name.clone(), market_events.clone());
        
        let unified_eval = chronosentiment_core::pipeline::run_evaluation_orchestration(
            strategy_config,
            &one_scenario,
            seed,
        ).map_err(|e| ApiError::InternalError(e))?;

        Ok(InspectStrategyResponse {
            strategy_id: unified_eval.strategy_id.clone(),
            decision_trace: event_log.iter().map(|e| self.wrap_event(e)).collect(), 
            execution_trace: event_log.iter().map(|e| self.wrap_event(e)).collect(), 
            metrics: unified_eval,
            event_sequence: event_log.iter().map(|e| self.wrap_event(e)).collect(),
        })
    }

    pub fn test_determinism(
        &self,
        strategy_config: Strategy,
        scenarios: Vec<String>,
        seed: u64,
    ) -> Result<bool, ApiError> {
        let first_run_response = self.evaluate_strategy(strategy_config.clone(), scenarios.clone(), seed)?;
        let second_run_response = self.evaluate_strategy(strategy_config, scenarios, seed)?;

        Ok(
            first_run_response == second_run_response
        )
    }

    pub fn run_ga(
        &self,
    ) -> Result<RunGaResponse, ApiError> {
        let seed = 42;
        let ga_config = GaConfig {
            population_size: 50,
            generations: 20,
            mutation_rate: 0.1,
            seed,
            order_id_prefix: "API_GA_RUN".to_string(),
            order_price: ORDER_PRICE,
            order_quantity_for_strategy: ORDER_QUANTITY,
            order_timestamp: ORDER_TIMESTAMP,
            lambda: 0.5,
            initial_queue_threshold: 200,
        };

        println!("API_INFO: Calling Core unified GA pipeline (seed={})", seed);
        let scenarios_map = self.load_all_real_scenarios();
        
        let mut unified_result = chronosentiment_core::pipeline::run_ga_orchestration(
            ga_config,
            &scenarios_map,
            0.2, // 20% holdout
        ).map_err(|e| ApiError::InternalError(e))?;

        // Deduplicate results based on strategy_id
        let mut seen_strategies = std::collections::HashSet::new();
        unified_result.results.retain(|eval| {
            seen_strategies.insert(eval.strategy_id.clone())
        });

        Ok(RunGaResponse::from(unified_result))
    }

    pub fn get_timeline(&self) -> Result<Vec<EventWrapper>, ApiError> {
        let last_sim = self.last_simulation.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(sim) = last_sim.as_ref() {
            Ok(sim.events.iter().map(|e| self.wrap_event(e)).collect())
        } else {
            Ok(Vec::new())
        }
    }

    pub fn get_global_ranking(&self) -> Result<Vec<crate::dto::StrategyEvaluationDto>, ApiError> {
        // Temporarily disabled to enforce single source of truth (run_ga results)
        Ok(Vec::new())
    }

    pub fn get_latest_signals(&self) -> Result<chronosentiment_core::pipeline::SignalsSnapshot, ApiError> {
        // CSV-only deterministic source: load the canonical clean folder and never synthesize fallback assets.
        let source = chronosentiment_core::FolderCandleSource {
            folder_path: "/Users/nikhil/ChronoSentiment_MEGA_FINAL/test_assets".to_string(),
        };
        let mut assets: Vec<String> = source
            .load_all()
            .into_iter()
            .map(|(asset, _)| asset)
            .collect();
        assets.sort();
        assets.dedup();
        if assets.is_empty() {
            return Err(ApiError::EngineError(
                "No *_5m_clean.csv datasets found in test_assets; recommendations require CSV data.".to_string(),
            ));
        }
        println!("API_INFO: latest_signals CSV-only assets={:?}", assets);
        
        let store_guard = self.strategy_store.lock().unwrap_or_else(|e| e.into_inner());
        let snapshot = if let Some(_store) = store_guard.as_ref() {
            println!("API_INFO: Generating signals using pre-trained strategy store");
            chronosentiment_core::pipeline::generate_latest_signals_from_saved_strategies(
                assets.clone(),
                0.5,
                0.45,
                0.35,
                Some("/Users/nikhil/ChronoSentiment_MEGA_FINAL/test_assets/strategy_store.json".to_string())
            ).unwrap_or_else(|_| chronosentiment_core::pipeline::generate_latest_signals_with_thresholds(assets, 0.5, 0.45, 0.35))
        } else {
            chronosentiment_core::pipeline::generate_latest_signals_with_thresholds(assets, 0.5, 0.45, 0.35)
        };

        Ok(snapshot)
    }

    /// Real-time style suggestions from pre-trained strategy pool (no online GA).
    pub fn get_trade_suggestions(&self) -> Result<crate::dto::TradeSuggestionsResponse, ApiError> {
        use chronosentiment_core::strategy_ranking::{
            LiveEvaluator, LiveMarketState, RankingWeights, StrategyProfile,
            StrategyRegistry, SuggestionDebug,
        };

        let snapshot = self.get_latest_signals()?;
        if snapshot.signals.is_empty() {
            return Ok(crate::dto::TradeSuggestionsResponse {
                asset: "MULTI".to_string(),
                timestamp: snapshot.timestamp,
                suggestions: Vec::new(),
                count: 0,
                debug: SuggestionDebug::default(),
            });
        }

        let mut registry_rows: Vec<StrategyProfile> = Vec::new();
        for sig in &snapshot.signals {
            let strategy = parse_strategy_from_id(&sig.strategy_id).unwrap_or(chronosentiment_core::ga::Strategy {
                queue_threshold: 100,
                base_edge: 2,
                take_profit: 10,
                stop_loss: 5,
            });
            registry_rows.push(StrategyProfile {
                strategy_id: sig.strategy_id.clone(),
                strategy,
                preferred_regimes: vec![map_regime(&sig.regime)],
                confidence_weight: sig.confidence.clamp(0.0, 1.0),
                execution_weight: sig.composite_score.clamp(0.0, 1.0),
            });
        }
        registry_rows.sort_by(|a, b| a.strategy_id.cmp(&b.strategy_id));
        registry_rows.dedup_by(|a, b| a.strategy_id == b.strategy_id);
        let registry = StrategyRegistry::new(registry_rows);

        let mut states_by_asset: HashMap<String, LiveMarketState> = HashMap::new();
        for sig in &snapshot.signals {
            let state = states_by_asset
                .entry(sig.asset.clone())
                .or_insert_with(|| LiveMarketState::new(sig.asset.clone()));
            state.confidence = state.confidence.max(sig.confidence);
            state.expected_edge = state.expected_edge.max(sig.expected_edge);
            state.execution_score = state.execution_score.max(sig.composite_score.clamp(0.0, 1.0));
            state.regime = map_regime(&sig.regime);
        }

        let mut all_suggestions = Vec::new();
        let mut agg_debug = SuggestionDebug::default();
        for (_asset, state) in states_by_asset {
            let mut evaluator = LiveEvaluator::new(state, registry.clone(), RankingWeights::default());
            let mut top = evaluator.rank_current(3);
            let dbg = evaluator.debug_snapshot();
            agg_debug.rejected_hold += dbg.rejected_hold;
            agg_debug.rejected_low_edge += dbg.rejected_low_edge;
            agg_debug.rejected_low_exec += dbg.rejected_low_exec;
            agg_debug.suppressed_stability += dbg.suppressed_stability;
            all_suggestions.append(&mut top);
        }

        all_suggestions.sort_by(|a, b| {
            b.live_score
                .partial_cmp(&a.live_score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.strategy_id.cmp(&b.strategy_id))
        });
        all_suggestions.truncate(5);

        Ok(crate::dto::TradeSuggestionsResponse {
            asset: "MULTI".to_string(),
            timestamp: snapshot.timestamp,
            count: all_suggestions.len(),
            suggestions: all_suggestions,
            debug: agg_debug,
        })
    }

    pub fn get_replay_suggestions(
        &self,
        mode: String,
        limit: usize,
        sample_rate: usize,
        include_full: bool,
    ) -> Result<crate::dto::ReplaySuggestionsResponse, ApiError> {
        use chronosentiment_core::pnl_overlay::run_pnl_overlay;
        use chronosentiment_core::replay_evaluator::run_replay_with_evaluator;
        use chronosentiment_core::strategy_ranking::{
            LiveEvaluator, LiveMarketState, RankingWeights, StrategyProfile, StrategyRegistry,
        };
        use chronosentiment_core::tick_replay::{ReplayConfig, ReplayMode, TickReplayEngine};

        let jsonl_path = std::env::var("BINANCE_JSONL")
            .unwrap_or_else(|_| "/Users/nikhil/ChronoSentiment_MEGA_FINAL/test_assets/binance_ticks.jsonl".to_string());
        let mut replay = TickReplayEngine::from_binance_jsonl(
            &jsonl_path,
            ReplayConfig {
                mode: ReplayMode::Fast,
                ..ReplayConfig::default()
            },
            1,
        )
        .map_err(|e| {
            ApiError::EngineError(format!("Failed to load replay ticks from {}: {}", jsonl_path, e))
        })?;

        let snapshot = self.get_latest_signals()?;
        let mut registry_rows: Vec<StrategyProfile> = Vec::new();
        for sig in &snapshot.signals {
            let strategy = parse_strategy_from_id(&sig.strategy_id).unwrap_or(chronosentiment_core::ga::Strategy {
                queue_threshold: 100,
                base_edge: 2,
                take_profit: 10,
                stop_loss: 5,
            });
            registry_rows.push(StrategyProfile {
                strategy_id: sig.strategy_id.clone(),
                strategy,
                preferred_regimes: vec![map_regime(&sig.regime)],
                confidence_weight: sig.confidence.clamp(0.0, 1.0),
                execution_weight: sig.composite_score.clamp(0.0, 1.0),
            });
        }
        registry_rows.sort_by(|a, b| a.strategy_id.cmp(&b.strategy_id));
        registry_rows.dedup_by(|a, b| a.strategy_id == b.strategy_id);
        if registry_rows.is_empty() {
            return Err(ApiError::EngineError(
                "No strategy profiles available for replay evaluation.".to_string(),
            ));
        }

        let registry_rows_for_pnl = registry_rows.clone();
        let registry = StrategyRegistry::new(registry_rows);
        let mut evaluator = LiveEvaluator::new(
            LiveMarketState::new("BTCUSDT".to_string()),
            registry,
            RankingWeights::default(),
        );
        let replay_out = run_replay_with_evaluator(&mut replay, &mut evaluator, 5);

        // Run PnL overlay on an isolated replay/evaluator instance so results remain deterministic
        // and independent of timeline sampling mode.
        let mut replay_for_pnl = TickReplayEngine::from_binance_jsonl(
            &jsonl_path,
            ReplayConfig {
                mode: ReplayMode::Fast,
                ..ReplayConfig::default()
            },
            1,
        )
        .map_err(|e| {
            ApiError::EngineError(format!(
                "Failed to load replay ticks for pnl overlay from {}: {}",
                jsonl_path, e
            ))
        })?;
        let mut evaluator_for_pnl = LiveEvaluator::new(
            LiveMarketState::new("BTCUSDT".to_string()),
            StrategyRegistry::new(registry_rows_for_pnl),
            RankingWeights::default(),
        );
        let horizon_ticks = std::env::var("PNL_HORIZON_TICKS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(20);
        let (_trades, pnl_metrics) =
            run_pnl_overlay(&mut replay_for_pnl, &mut evaluator_for_pnl, horizon_ticks);

        let mode_norm = mode.to_lowercase();
        let include_timeline = include_full || mode_norm == "full" || mode_norm == "sampled";
        let sample_every = if mode_norm == "sampled" {
            sample_rate.max(1)
        } else {
            1
        };
        let cap = limit.max(1);

        let mut timeline: Vec<crate::dto::ReplaySuggestionPoint> = Vec::new();
        let mut prev_strategy: Option<String> = None;
        if include_timeline {
            for (idx, point) in replay_out.timeline.iter().enumerate() {
                if idx % sample_every != 0 {
                    continue;
                }
                let top = point.suggestions.first().map(|s| crate::dto::TopStrategySnapshot {
                    strategy_id: s.strategy_id.clone(),
                    action: s.action.clone(),
                    live_score: s.live_score,
                    expected_edge: s.expected_edge,
                    execution_score: s.execution_score,
                });
                timeline.push(crate::dto::ReplaySuggestionPoint {
                    ts: point.exchange_ts,
                    decision_ts: point.decision_ts,
                    execution_ts: point.execution_ts,
                    suggestion_count: point.suggestions.len(),
                    prev_strategy: prev_strategy.clone(),
                    flip_occurred: matches!(
                        (&prev_strategy, &top),
                        (Some(prev), Some(curr)) if prev != &curr.strategy_id
                    ),
                    top_strategy: top,
                });
                prev_strategy = timeline
                    .last()
                    .and_then(|p| p.top_strategy.as_ref().map(|x| x.strategy_id.clone()));
                if timeline.len() >= cap {
                    break;
                }
            }
        }

        Ok(crate::dto::ReplaySuggestionsResponse {
            asset: "BTCUSDT".to_string(),
            metrics: replay_out.metrics,
            timeline,
            pnl: Some(pnl_metrics),
        })
    }

    pub fn get_order_inspection(&self, order_id: String, include_chain: bool) -> Result<TradeInspectorResponse, ApiError> {
        let last_sim = self.last_simulation.lock().unwrap_or_else(|e| e.into_inner());
        
        let sim = last_sim.as_ref().ok_or_else(|| ApiError::InternalError("No simulation results available".to_string()))?;
        
        // Use the core inspector logic
        let inspection = chronosentiment_core::inspector::inspect_trade(&order_id, sim);
        
        let mut execution_steps = Vec::new();
        // Construct execution steps for UI
        for event in sim.events.iter() {
            if event.order_id() == Some(&order_id) {
                match event {
                    SimEvent::OrderEnteredQueue { queue_ahead, sequence_id, timestamp, .. } => {
                        execution_steps.push(json!({
                            "type": "OrderEnteredQueue",
                            "queue_ahead": queue_ahead,
                            "sequence_id": sequence_id,
                            "timestamp": timestamp,
                        }));
                    }
                    SimEvent::QueueProgression { queue_ahead, sequence_id, timestamp, .. } => {
                        execution_steps.push(json!({
                            "type": "QueueProgression",
                            "queue_ahead": queue_ahead,
                            "sequence_id": sequence_id,
                            "timestamp": timestamp,
                        }));
                    }
                    SimEvent::PartialFill { filled_qty, price, sequence_id, timestamp, .. } => {
                        execution_steps.push(json!({
                            "type": "PartialFillExecution",
                            "filled_qty": filled_qty,
                            "price": price,
                            "sequence_id": sequence_id,
                            "timestamp": timestamp,
                        }));
                    }
                    SimEvent::OrderFilled { sequence_id, timestamp, .. } => {
                        execution_steps.push(json!({
                            "type": "OrderFilledExecution",
                            "sequence_id": sequence_id,
                            "timestamp": timestamp,
                        }));
                    }
                    _ => {}
                }
            }
        }

        Ok(TradeInspectorResponse {
            order_id: order_id.clone(),
            decision: inspection.decision,
            execution: execution_steps,
            outcome: inspection.outcome,
            causal_chain: if include_chain {
                Some(inspection.execution.causal_chain.iter().map(|e| self.wrap_event(e)).collect())
            } else {
                None
            },
        })
    }

    pub fn get_replay(&self, seq_id: u64) -> Result<crate::dto::SystemState, ApiError> {
        let last_sim = self.last_simulation.lock().unwrap_or_else(|e| e.into_inner());
        
        let sim = last_sim.as_ref().ok_or_else(|| ApiError::InternalError("No simulation results available".to_string()))?;
        let events = &sim.events;
        
        let mut orders: HashMap<String, crate::dto::OrderState> = HashMap::new();
        let mut pnl = 0.0;
        let mut position = 0i64;

        for event in events.iter() {
            if event.sequence_id() > seq_id {
                break;
            }

            match event {
                SimEvent::OrderIntent { order_id, side, price, quantity, .. } => {
                    orders.insert(order_id.clone(), crate::dto::OrderState {
                        order_id: order_id.clone(),
                        status: "NEW".to_string(),
                        quantity_total: *quantity,
                        quantity_filled: 0,
                        quantity_remaining: *quantity,
                        queue_ahead: 0,
                        price: *price as f64 / chronosentiment_core::PRICE_SCALE as f64,
                        side: *side,
                    });
                }
                SimEvent::OrderEnteredQueue { order_id, queue_ahead, .. } => {
                    if let Some(order) = orders.get_mut(order_id) {
                        order.status = "ACTIVE".to_string();
                        order.queue_ahead = *queue_ahead;
                    }
                }
                SimEvent::PartialFill { order_id, filled_qty, price, .. } => {
                    if let Some(order) = orders.get_mut(order_id) {
                        order.status = "PARTIAL".to_string();
                        order.quantity_filled += *filled_qty;
                        order.quantity_remaining = order.quantity_remaining.saturating_sub(*filled_qty);
                        
                        // Update portfolio
                        let multiplier = match order.side {
                            chronosentiment_core::Side::Buy => 1,
                            chronosentiment_core::Side::Sell => -1,
                        };
                        position += multiplier * (*filled_qty as i64);
                        let p = *price as f64 / chronosentiment_core::PRICE_SCALE as f64;
                        pnl += multiplier as f64 * (*filled_qty as f64) * p;
                    }
                }
                SimEvent::QueueProgression { order_id, queue_ahead, .. } => {
                    if let Some(order) = orders.get_mut(order_id) {
                        order.queue_ahead = *queue_ahead;
                    }
                }
                SimEvent::OrderFilled { order_id, .. } => {
                    if let Some(order) = orders.get_mut(order_id) {
                        order.status = "FILLED".to_string();
                        order.quantity_remaining = 0;
                    }
                }
                SimEvent::MarketEvent { .. } => {
                    // Market events don't change our internal state directly in this simple replay
                }
            }
        }

        Ok(crate::dto::SystemState {
            orders,
            portfolio: crate::dto::PortfolioState { pnl, position },
            last_sequence_id: seq_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{RunGaRequest, ScenarioInput};
    use chronosentiment_core::MarketEvent;

    #[test]
    fn test_run_ga_api_determinism() {
        let request = RunGaRequest {
            population_size: 10,
            generations: 5,
            mutation_rate: 0.1,
            scenarios: vec![ScenarioInput { events: vec![
                MarketEvent { subtype: chronosentiment_core::MarketEventType::NewOrder, price: 100, quantity: 2000, side: Some(chronosentiment_core::Side::Sell), exchange_ts: 10 },
                MarketEvent { subtype: chronosentiment_core::MarketEventType::Trade, price: 100, quantity: 500, side: None, exchange_ts: 15 },
            ]}],
            seed: 789,
            top_k: Some(3),
            lambda: Some(0.5),
        };

        let service = EvaluationService::new();
        let r1 = service.run_ga(request.clone()).expect("Failed to run GA in test");
        let r2 = service.run_ga(request).expect("Failed to run GA in test");

        assert_eq!(r1.results.len(), r2.results.len(), "Results length diverged");
        for i in 0..r1.results.len() {
            assert_eq!(r1.results[i].strategy_id, r2.results[i].strategy_id, "Strategy ID diverged at index {}", i);
            assert_eq!(r1.results[i].ga_fitness, r2.results[i].ga_fitness, "GA fitness diverged at index {}", i);
            assert_eq!(r1.results[i].execution_fitness, r2.results[i].execution_fitness, "Execution fitness diverged at index {}", i);
        }

        assert_eq!(r1.generation_history.len(), r2.generation_history.len(), "Generation history length diverged");
        for i in 0..r1.generation_history.len() {
            assert_eq!(r1.generation_history[i].strategy_id, r2.generation_history[i].strategy_id, "Generation history strategy ID diverged at index {}", i);
            assert_eq!(r1.generation_history[i].ga_fitness, r2.generation_history[i].ga_fitness, "Generation history GA fitness diverged at index {}", i);
            assert_eq!(r1.generation_history[i].execution_fitness, r2.generation_history[i].execution_fitness, "Generation history execution fitness diverged at index {}", i);
        }
        assert_eq!(r1.best_per_regime.len(), r2.best_per_regime.len(), "Per-regime map length diverged");
        for (k, v1) in &r1.best_per_regime {
            let v2 = r2.best_per_regime.get(k).expect("Missing regime key in deterministic run");
            assert_eq!(v1.strategy_id, v2.strategy_id, "Per-regime strategy ID diverged for key {}", k);
            assert_eq!(v1.ga_fitness, v2.ga_fitness, "Per-regime GA fitness diverged for key {}", k);
            assert_eq!(v1.execution_fitness, v2.execution_fitness, "Per-regime execution fitness diverged for key {}", k);
        }

        println!("✅ run_ga API determinism test passed.");
    }
}
