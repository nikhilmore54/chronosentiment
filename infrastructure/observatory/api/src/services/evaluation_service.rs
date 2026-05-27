use api::replay::reduce_replay_state;
use crate::{dto::{EvaluateStrategyResponse, CompareStrategiesResponse, ComparisonSummary, InspectStrategyResponse, RunGaResponse, EventWrapper, TradeInspectorResponse, StrategyEvaluationDto},
    errors::ApiError,
};
use chronosentiment_core::{self, GaConfig, Strategy, ga::StrategyEvaluation, SimEvent, SimulationResult, Candle, MarketEvent, convert_series_to_events, RecommendationStatus, AlphaPorosity};
use std::collections::HashMap;
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};
use std::env;
use serde_json::json;

// Placeholder for internal engine functions (these would ideally be in a core library)
// For now, we'll assume they are accessible or mocked.

const ORDER_PRICE: u64 = 100;
const ORDER_QUANTITY: u64 = 100;
const ORDER_TIMESTAMP: u64 = 12; // A reasonable timestamp for event-driven simulation

#[derive(Clone)]
pub struct EvaluationService {
    pub last_simulation: Arc<Mutex<Option<SimulationResult>>>,
    pub last_global_ranking: Arc<Mutex<Vec<crate::dto::StrategyEvaluationDto>>>,
}

impl EvaluationService {
    fn deterministic_strategy_id(strategy: &Strategy, scenario_names: &[String], seed: u64) -> String {
        let mut names = scenario_names.to_vec();
        names.sort();
        format!(
            "strat_{}_{}_{}_{}_{}",
            strategy.queue_threshold,
            strategy.base_edge,
            strategy.take_profit,
            strategy.stop_loss,
            seed ^ (names.len() as u64)
        )
    }

    fn to_dual_fitness_dto(
        strategy_id: String,
        ga_eval: &StrategyEvaluation,
        execution_eval: &StrategyEvaluation,
    ) -> crate::dto::StrategyEvaluationDto {
        // Log out-of-bounds fitness (can occur on synthetic/sparse data) but do not panic.
        // Clamp to [0.0, 1.0] at the DTO boundary so the API never emits invalid values.
        if !execution_eval.fitness.is_finite() || execution_eval.fitness < 0.0 || execution_eval.fitness > 1.0 {
            eprintln!(
                "WARN: execution fitness out of canonical bounds ({:.6}); clamping to [0,1] for DTO",
                execution_eval.fitness
            );
        }
        let clamped_fitness = execution_eval.fitness.clamp(0.0, 1.0);
        crate::dto::StrategyEvaluationDto {
            strategy_id,
            avg: execution_eval.avg_pnl,
            std: execution_eval.std_dev,
            fitness: clamped_fitness,
            ga_fitness: Some(ga_eval.fitness),
            execution_fitness: clamped_fitness,
            total_trades: execution_eval.trade_count,
            classification: chronosentiment_core::ga::get_strategy_classification(execution_eval),
        }
    }

    pub fn new() -> Self {
        let sample_candles = vec![
            Candle { timestamp: 10, open: 100, high: 105, low: 99, close: 103, volume: 1000 },
            Candle { timestamp: 20, open: 103, high: 108, low: 101, close: 107, volume: 1500 },
            Candle { timestamp: 30, open: 107, high: 110, low: 105, close: 106, volume: 1200 },
        ];

        let mut events = convert_series_to_events(&sample_candles, 1);
        let next_seq_id = events.last().map(|e| e.sequence_id()).unwrap_or(0) + 1;

        let mut order_outcomes = HashMap::new();
        order_outcomes.insert("O1".to_string(), chronosentiment_core::OrderOutcome {
            order_id: "O1".to_string(),
            filled_quantity: 100,
            remaining_quantity: 0,
            arrival_time: 15,
            queue_ahead: 500,
        });

        // Add mock order events interleaved or appended for demonstration
        events.push(SimEvent::OrderIntent {
            sequence_id: next_seq_id,
            parent_sequence_id: None,
            order_id: "O1".to_string(),
            side: chronosentiment_core::Side::Buy,
            price: 100,
            quantity: 100,
            timestamp: 12,
        });
        events.push(SimEvent::OrderEnteredQueue {
            sequence_id: next_seq_id + 1,
            parent_sequence_id: Some(next_seq_id),
            order_id: "O1".to_string(),
            timestamp: 15,
            price: 100,
            queue_ahead: 500,
        });
        events.push(SimEvent::PartialFill {
            sequence_id: next_seq_id + 2,
            parent_sequence_id: Some(next_seq_id + 1),
            order_id: "O1".to_string(),
            timestamp: 18,
            filled_qty: 100,
            price: 100,
        });
        events.push(SimEvent::OrderFilled {
            sequence_id: next_seq_id + 3,
            parent_sequence_id: Some(next_seq_id + 2),
            order_id: "O1".to_string(),
            timestamp: 20,
        });

        // Ensure events are sorted by timestamp
        events.sort_by_key(|e| e.timestamp());

        let mock_simulation = SimulationResult {
            pnl: 0,
            trades: 1,
            order_outcomes,
            events,
        };

        let initial_ranking = Vec::new();

        Self {
            last_simulation: Arc::new(Mutex::new(Some(mock_simulation))),
            last_global_ranking: Arc::new(Mutex::new(initial_ranking)),
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
            source_layer: crate::dto::SourceLayer::Sequencer,
            kernel_signature: String::new(),
        }
    }

    pub fn evaluate_strategy(
        &self,
        strategy_config: Strategy,
        scenario_names: Vec<String>,
        seed: u64,
    ) -> Result<EvaluateStrategyResponse, ApiError> {
        let mut ga_config = GaConfig::default();
        ga_config.population_size = 1;
        ga_config.generations = 1;
        ga_config.mutation_rate = 0.0;
        ga_config.seed = seed;
        ga_config.order_id_prefix = "API_EVAL".to_string();
        ga_config.order_price = ORDER_PRICE;
        ga_config.order_quantity_for_strategy = ORDER_QUANTITY;
        ga_config.order_timestamp = ORDER_TIMESTAMP;
        ga_config.lambda = 0.5;
        ga_config.initial_queue_threshold = 200;

        let scenarios_map = chronosentiment_core::synthetic::generate_deterministic_scenarios("BTC", seed, ORDER_PRICE);

        // Default to all benchmark scenarios if none provided
        let scenario_names = if scenario_names.is_empty() {
            scenarios_map.keys().cloned().collect::<Vec<String>>()
        } else {
            scenario_names
        };

        // Sort scenario_names for deterministic strategy_id generation
        let mut sorted_scenario_names = scenario_names.clone();
        sorted_scenario_names.sort();

        let strategy_id = Self::deterministic_strategy_id(&strategy_config, &sorted_scenario_names, seed);

        for (i, scenario_name) in scenario_names.iter().enumerate() {
            let market_events = scenarios_map.get(scenario_name).ok_or_else(|| {
                ApiError::EngineError(format!("Scenario '{}' not found", scenario_name))
            })?;

            let (_event_log, simulation_result, _) = chronosentiment_core::harness::run_simulation_harness(
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

            // Store result for the first scenario
            if i == 0 {
                let mut last_sim = self.last_simulation.lock().unwrap_or_else(|e| e.into_inner());
                *last_sim = Some(simulation_result.clone());
            }

        }

        let mut selected_scenarios: HashMap<String, Vec<chronosentiment_core::MarketEvent>> = HashMap::new();
        for scenario_name in &scenario_names {
            let market_events = scenarios_map.get(scenario_name).ok_or_else(|| {
                ApiError::EngineError(format!("Scenario '{}' not found", scenario_name))
            })?;
            selected_scenarios.insert(scenario_name.clone(), market_events.clone());
        }

        let selected_scenarios_vec: Vec<chronosentiment_core::ga::ScenarioPair<'_>> = scenario_names.iter().map(|name| {
            let events = scenarios_map.get(name).unwrap();
            chronosentiment_core::ga::ScenarioPair {
                name,
                signal_symbol: "BTC",
                execution_symbol: "BTC",
                signal: events.as_slice(),
                execution: events.as_slice(),
            }
        }).collect();

        let aggregated_evaluation = match chronosentiment_core::evaluate_and_aggregate(
            &strategy_config,
            &ga_config,
            &selected_scenarios_vec,
            0, 0.0, 0, 1.0, 0
        ) {
            Some(mut eval) => {
                eval.strategy_id = strategy_id.clone();
                eval
            },
            None => {
                return Err(ApiError::InternalError("Failed to aggregate strategy reports".to_string()));
            }
        };
        
        Ok(crate::dto::EvaluateStrategyResponse {
            strategy_evaluation: aggregated_evaluation.into(),
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
        let mut ga_config = GaConfig::default();
        ga_config.population_size = strategies.len();
        ga_config.generations = 1;
        ga_config.mutation_rate = 0.0;
        ga_config.seed = seed;
        ga_config.order_id_prefix = "API_COMPARE".to_string();
        ga_config.order_price = ORDER_PRICE;
        ga_config.order_quantity_for_strategy = ORDER_QUANTITY;
        ga_config.order_timestamp = ORDER_TIMESTAMP;
        ga_config.lambda = 0.5;
        ga_config.initial_queue_threshold = 200;

        let scenarios_map = chronosentiment_core::synthetic::generate_deterministic_scenarios("BTC", seed, ORDER_PRICE);

        // Default to all benchmark scenarios if none provided
        let scenario_names = if scenario_names.is_empty() {
            scenarios_map.keys().cloned().collect::<Vec<String>>()
        } else {
            scenario_names
        };

        let mut rankings: Vec<StrategyEvaluation> = Vec::new();

        // Sort scenario_names for deterministic strategy_id generation in compare_strategies
        let mut sorted_scenario_names = scenario_names.clone();
        sorted_scenario_names.sort();

        for strategy_config in strategies {
            let strategy_id = Self::deterministic_strategy_id(&strategy_config, &sorted_scenario_names, seed);
            let mut selected_scenarios: HashMap<String, Vec<chronosentiment_core::MarketEvent>> = HashMap::new();
            for scenario_name in &scenario_names {
                let market_events = scenarios_map.get(scenario_name).ok_or_else(|| {
                    ApiError::EngineError(format!("Scenario '{}' not found", scenario_name))
                })?;
                selected_scenarios.insert(scenario_name.clone(), market_events.clone());
            }

            let selected_scenarios_vec: Vec<chronosentiment_core::ga::ScenarioPair<'_>> = scenario_names.iter().map(|name| {
                let events = scenarios_map.get(name).unwrap();
                chronosentiment_core::ga::ScenarioPair {
                    name,
                    signal_symbol: "BTC",
                    execution_symbol: "BTC",
                    signal: events.as_slice(),
                    execution: events.as_slice(),
                }
            }).collect();

            // Aggregate across scenarios via the canonical helper.
            let mut aggregated_report = chronosentiment_core::evaluate_and_aggregate(
                &strategy_config,
                &ga_config,
                &selected_scenarios_vec,
                0, 0.0, 0, 1.0, 0
            ).ok_or_else(|| ApiError::InternalError("No scenario reports generated.".to_string()))?;
            aggregated_report.strategy_id = strategy_id.clone();
            rankings.push(aggregated_report);
        }

        rankings.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(Ordering::Equal).then_with(|| a.strategy_id.cmp(&b.strategy_id)));

        let mut comparison_summary = ComparisonSummary {
            best_strategy: "".to_string(), // Will be updated
            reason: "No clear reason".to_string(),
        };

        if let Some(best) = rankings.first() {
            comparison_summary.best_strategy = best.strategy_id.clone();
            if rankings.len() >= 2 {
                comparison_summary.reason = format!("The best strategy {} had a higher fitness of {:.2}.", best.strategy_id, best.fitness);
            }
        }

        Ok(CompareStrategiesResponse {
            ranking: rankings.into_iter().map(|e| e.into()).collect(),
            comparison_summary,
        })
    }

    pub fn inspect_strategy(
        &self,
        strategy_config: Strategy,
        scenario_name: String,
        seed: u64,
    ) -> Result<InspectStrategyResponse, ApiError> {
        let strategy_id = Self::deterministic_strategy_id(&strategy_config, &vec![scenario_name.clone()], seed);

        let mut ga_config = GaConfig::default();
        ga_config.population_size = 1;
        ga_config.generations = 1;
        ga_config.mutation_rate = 0.0;
        ga_config.seed = seed;
        ga_config.order_id_prefix = "API_INSPECT".to_string();
        ga_config.order_price = ORDER_PRICE;
        ga_config.order_quantity_for_strategy = ORDER_QUANTITY;
        ga_config.order_timestamp = ORDER_TIMESTAMP;
        ga_config.lambda = 0.5;
        ga_config.initial_queue_threshold = 200;

        let scenarios_map = chronosentiment_core::synthetic::generate_deterministic_scenarios("BTC", seed, ORDER_PRICE);
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

        let mut one_scenario_map: HashMap<String, Vec<MarketEvent>> = HashMap::new();
        one_scenario_map.insert(scenario_name.clone(), market_events.clone());
        let one_scenario_vec = [chronosentiment_core::ga::ScenarioPair {
            name: &scenario_name,
            signal_symbol: "BTC",
            execution_symbol: "BTC",
            signal: market_events.as_slice(),
            execution: market_events.as_slice(),
        }];
        let strategy_report = chronosentiment_core::evaluate_and_aggregate(
            &strategy_config,
            &ga_config,
            &one_scenario_vec,
            0, 0.0, 0, 1.0, 0
        ).ok_or_else(|| ApiError::InternalError("Strategy produced no evaluable trades".to_string()))?;

        Ok(InspectStrategyResponse {
            strategy_id,
            decision_trace: event_log.iter().map(|e| self.wrap_event(e)).collect(), 
            execution_trace: event_log.iter().map(|e| self.wrap_event(e)).collect(), 
            metrics: strategy_report.into(),
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
        let mut ga_config = GaConfig::default();
        ga_config.population_size = 50;
        ga_config.generations = 20;
        ga_config.mutation_rate = 0.1;
        ga_config.seed = seed;
        ga_config.order_id_prefix = "API_GA_RUN".to_string();
        ga_config.order_price = ORDER_PRICE;
        ga_config.order_quantity_for_strategy = ORDER_QUANTITY;
        ga_config.order_timestamp = ORDER_TIMESTAMP;
        ga_config.lambda = 0.5;
        ga_config.initial_queue_threshold = 200;

        // Enforce single source: Only use canonical synthetic scenarios
        println!("API_INFO: Using canonical synthetic scenarios for GA (seed={})", seed);
        let scenarios_map = chronosentiment_core::synthetic::generate_deterministic_scenarios("BTC", seed, ORDER_PRICE);
        let mut scenario_names: Vec<String> = scenarios_map.keys().cloned().collect();
        scenario_names.sort();

        let (train_names, holdout_names): (Vec<String>, Vec<String>) = if scenario_names.len() <= 2 {
            (scenario_names.clone(), scenario_names.clone())
        } else {
            let holdout_count = (scenario_names.len() / 5).max(1);
            let split_at = scenario_names.len() - holdout_count;
            (scenario_names[..split_at].to_vec(), scenario_names[split_at..].to_vec())
        };

        let mut train_scenarios: HashMap<String, Vec<chronosentiment_core::MarketEvent>> = HashMap::new();
        for name in &train_names {
            if let Some(events) = scenarios_map.get(name) {
                train_scenarios.insert(name.clone(), events.clone());
            }
        }
        let mut holdout_scenarios: HashMap<String, Vec<chronosentiment_core::MarketEvent>> = HashMap::new();
        for name in &holdout_names {
            if let Some(events) = scenarios_map.get(name) {
                holdout_scenarios.insert(name.clone(), events.clone());
            }
        }
        println!(
            "API_INFO: GA split train={} holdout={} (deterministic)",
            train_scenarios.len(),
            holdout_scenarios.len()
        );

        let train_scenarios_vec: Vec<chronosentiment_core::ga::ScenarioPair<'_>> = train_names.iter().map(|name| {
            let events = scenarios_map.get(name).unwrap();
            chronosentiment_core::ga::ScenarioPair {
                name,
                signal_symbol: "BTC",
                execution_symbol: "BTC",
                signal: events.as_slice(),
                execution: events.as_slice(),
            }
        }).collect();

        let (ga_result, _) = chronosentiment_core::run_ga_evolution(ga_config.clone(), &train_scenarios_vec, &chronosentiment_core::ga::GlobalEvoState::default());
        let execution_scenarios_vec: Vec<chronosentiment_core::ga::ScenarioPair<'_>> = holdout_scenarios.iter().map(|(name, events)| {
            chronosentiment_core::ga::ScenarioPair {
                name,
                signal_symbol: "BTC",
                execution_symbol: "BTC",
                signal: events.as_slice(),
                execution: events.as_slice(),
            }
        }).collect();

        let global_exec_eval = chronosentiment_core::evaluate_and_aggregate(
            &ga_result.global_best.strategy,
            &ga_config,
            &execution_scenarios_vec,
            0, 0.0, 0, 1.0, 0
        ).ok_or_else(|| ApiError::InternalError("Failed to evaluate global best execution fitness".to_string()))?;
        let final_exec_eval = chronosentiment_core::evaluate_and_aggregate(
            &ga_result.final_generation_best.strategy,
            &ga_config,
            &execution_scenarios_vec,
            0, 0.0, 0, 1.0, 0
        ).ok_or_else(|| ApiError::InternalError("Failed to evaluate final generation execution fitness".to_string()))?;

        let global_best_dto = Self::to_dual_fitness_dto(
            ga_result.global_best.strategy_id.clone(),
            &ga_result.global_best,
            &global_exec_eval,
        );
        let final_gen_best_dto = Self::to_dual_fitness_dto(
            ga_result.final_generation_best.strategy_id.clone(),
            &ga_result.final_generation_best,
            &final_exec_eval,
        );
        let mut generation_history: Vec<StrategyEvaluationDto> = Vec::new();
        for eval in &ga_result.generation_history {
            let exec_eval = chronosentiment_core::evaluate_and_aggregate(
                &eval.strategy,
                &ga_config,
                &execution_scenarios_vec,
                0, 0.0, 0, 1.0, 0
            ).ok_or_else(|| ApiError::InternalError("Failed to evaluate generation history execution fitness".to_string()))?;
            generation_history.push(Self::to_dual_fitness_dto(
                eval.strategy_id.clone(),
                eval,
                &exec_eval,
            ));
        }
        let mut best_per_regime: HashMap<String, StrategyEvaluationDto> = HashMap::new();
        for (regime_key, eval) in &ga_result.best_per_regime {
            let exec_eval = chronosentiment_core::evaluate_and_aggregate(
                &eval.strategy,
                &ga_config,
                &execution_scenarios_vec,
                0, 0.0, 0, 1.0, 0
            ).ok_or_else(|| ApiError::InternalError("Failed to evaluate per-regime execution fitness".to_string()))?;
            best_per_regime.insert(
                regime_key.clone(),
                Self::to_dual_fitness_dto(
                    eval.strategy_id.clone(),
                    eval,
                    &exec_eval,
                ),
            );
        }

        Ok(RunGaResponse {
            // Backward-compatible fields
            results: vec![global_best_dto.clone(), final_gen_best_dto.clone()],
            generation_history,
            best_per_regime,
            // New extended fields
            global_best: global_best_dto,
            global_best_generation: ga_result.global_best_generation,
            generation_found: ga_result.global_best_generation,
            final_generation_best: final_gen_best_dto.clone(),
            final_gen_best: final_gen_best_dto,
        })
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
        // Deterministic signal asset selection; folder mode mirrors filtered clean CSV universe.
        let data_source = env::var("DATA_SOURCE")
            .unwrap_or_else(|_| "synthetic".to_string())
            .to_lowercase();
        let assets = if data_source == "folder" {
            let folder_path = chronosentiment_core::resolve_test_assets_dir()
                .map_err(|e| ApiError::EngineError(format!("test assets path resolution failed: {e}")))?;
            let source = chronosentiment_core::FolderCandleSource {
                folder_path: folder_path.to_string_lossy().into_owned(),
            };
            let mut names: Vec<String> = source
                .load_all()
                .into_iter()
                .map(|(asset, _)| asset)
                .collect();
            names.sort();
            names.dedup();
            if names.is_empty() {
                vec!["BTC".to_string()]
            } else {
                names
            }
        } else {
            vec!["BTC".to_string()]
        };
        println!("API_INFO: latest_signals assets={:?}", assets);
        let snapshot = chronosentiment_core::pipeline::generate_latest_signals(assets, 0.5);
        Ok(snapshot)
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
            decision: crate::dto::TradeInspectorDecision {
                order_id: inspection.decision.order_id,
                side: inspection.decision.side,
                price: chronosentiment_core::to_real(inspection.decision.price),
                quantity: inspection.decision.quantity,
                timestamp: inspection.decision.timestamp,
            },
            execution: execution_steps,
            outcome: crate::dto::TradeInspectorOutcome {
                filled_qty: inspection.outcome.filled_quantity,
                remaining_qty: inspection.outcome.remaining_quantity,
                avg_price: chronosentiment_core::to_real(inspection.outcome.average_price),
                status: if inspection.outcome.remaining_quantity == 0 { "FILLED".to_string() } else { "PARTIAL".to_string() },
            },
            causal_chain: if include_chain {
                Some(inspection.execution.causal_chain.iter().map(|e| self.wrap_event(e)).collect())
            } else {
                None
            },
        })
    }

    pub fn get_replay(&self, seq_id: u64) -> Result<crate::dto::SystemState, ApiError> {
        let last_sim = self.last_simulation.lock().unwrap_or_else(|e| e.into_inner());

        let sim = last_sim
            .as_ref()
            .ok_or_else(|| ApiError::InternalError("No simulation results available".to_string()))?;

        Ok(reduce_replay_state(&sim.events, seq_id))
    }
    pub fn get_trade_suggestions(&self) -> Result<crate::dto::TradeSuggestionsResponse, ApiError> {
        Ok(crate::dto::TradeSuggestionsResponse {
            asset: "BTC".to_string(),
            timestamp: 0,
            suggestions: Vec::new(),
            count: 0,
            debug: chronosentiment_core::strategy_ranking::SuggestionDebug::default(),
        })
    }

    pub fn get_replay_suggestions(&self, mode: String, limit: usize, sample_rate: usize, include_full: bool) -> Result<crate::dto::ReplaySuggestionsResponse, ApiError> {
        Ok(crate::dto::ReplaySuggestionsResponse {
            asset: "BTC".to_string(),
            metrics: chronosentiment_core::replay_evaluator::ReplayMetrics::default(),
            timeline: Vec::new(),
            pnl: None,
        })
    }

    pub fn load_all_real_scenarios(
        &self,
    ) -> Result<HashMap<String, Vec<chronosentiment_core::SimEvent>>, ApiError> {
        let folder_path = chronosentiment_core::resolve_test_assets_dir()
            .map_err(|e| ApiError::EngineError(format!("test assets path resolution failed: {e}")))?;
        let source = chronosentiment_core::FolderCandleSource {
            folder_path: folder_path.to_string_lossy().into_owned(),
        };
        let mut scenarios = HashMap::new();
        for (asset, candles) in source.load_all() {
            scenarios.insert(asset, chronosentiment_core::convert_series_to_events(&candles, 1));
        }
        Ok(scenarios)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_ga_api_determinism() {
        let service = EvaluationService::new();
        let r1 = service.run_ga().expect("Failed to run GA in test");
        let r2 = service.run_ga().expect("Failed to run GA in test");

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
