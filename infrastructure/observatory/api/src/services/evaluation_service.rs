use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chronosentiment_core::SimulationResult;
use chronosentiment_optimization::{
    run_ga_evolution, Candidate, CandidateEvaluation, FitnessEvaluator, GaConfig,
};
use chronosentiment_strategies::compatibility::{
    generate_latest_signals, SignalsSnapshot, TradeSignal,
};
use chronosentiment_strategies::evaluation::evaluator::FinancialEvaluator;

use crate::dto::*;
use crate::errors::ApiError;
use api::inspect_projection::{run_inspect_simulation, sim_event_to_wrapper};
use api::scenario::{
    evaluate_strategy_across_domains, AggregatedEvaluation, ScenarioAggregator, ScenarioRegistry,
    ScenarioResult,
};

#[derive(Clone)]
pub struct EvaluationService {
    pub last_simulation: Arc<Mutex<Option<SimulationResult>>>,
    pub last_global_ranking: Arc<Mutex<Vec<CandidateEvaluationDto>>>,
    pub scenario_registry: ScenarioRegistry,
    /// Observability substrate: strategy_id → materialized ScenarioResult[].
    pub last_scenario_results: Arc<Mutex<HashMap<String, Vec<ScenarioResult>>>>,
}

impl EvaluationService {
    pub fn new() -> Self {
        Self {
            last_simulation: Arc::new(Mutex::new(None)),
            last_global_ranking: Arc::new(Mutex::new(Vec::new())),
            scenario_registry: ScenarioRegistry::v1_default(),
            last_scenario_results: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn strategy_id_from_candidate(candidate: &Candidate) -> String {
        format!(
            "strat_{}_{}_{}_{}",
            candidate.queue_threshold,
            candidate.base_edge,
            candidate.take_profit,
            candidate.stop_loss
        )
    }

    fn to_evaluation_dto(eval: &CandidateEvaluation) -> CandidateEvaluationDto {
        let strategy_id = if eval.strategy_id.is_empty() {
            Self::strategy_id_from_candidate(&eval.candidate)
        } else {
            eval.strategy_id.clone()
        };

        // GA fitness may exceed 1.0; execution fitness is clamped for UI contract compliance.
        let ga_fitness = eval.fitness;
        let execution_fitness = (eval.fitness / 100.0).clamp(0.0, 1.0);

        CandidateEvaluationDto {
            strategy_id,
            avg: eval.avg_pnl,
            std: eval.std_dev,
            fitness: execution_fitness,
            classification: classify_strategy(eval),
            ga_fitness: Some(ga_fitness),
            execution_fitness,
            total_trades: eval.trade_count,
        }
    }

    fn upsert_ranking(store: &mut Vec<CandidateEvaluationDto>, dto: CandidateEvaluationDto) {
        if let Some(existing) = store
            .iter_mut()
            .find(|row| row.strategy_id == dto.strategy_id)
        {
            if dto.execution_fitness > existing.execution_fitness {
                *existing = dto;
            }
            return;
        }
        store.push(dto);
    }

    fn refresh_ranking_store(&self, candidates: impl IntoIterator<Item = CandidateEvaluationDto>) {
        let mut store = self
            .last_global_ranking
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        for dto in candidates {
            Self::upsert_ranking(&mut store, dto);
        }
        store.sort_by(|a, b| {
            b.execution_fitness
                .partial_cmp(&a.execution_fitness)
                .unwrap_or(Ordering::Equal)
        });
    }

    fn default_evaluator() -> FinancialEvaluator {
        FinancialEvaluator::new("BTC".to_string(), "default".to_string())
    }

    fn evaluate_candidate(
        evaluator: &FinancialEvaluator,
        candidate: &Candidate,
    ) -> CandidateEvaluationDto {
        let mut eval = evaluator.evaluate(candidate);
        if eval.strategy_id.is_empty() {
            eval.strategy_id = Self::strategy_id_from_candidate(candidate);
        }
        Self::to_evaluation_dto(&eval)
    }

    fn store_scenario_results(&self, strategy_id: &str, results: Vec<ScenarioResult>) {
        let mut store = self
            .last_scenario_results
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        store.insert(strategy_id.to_string(), results);
    }

    fn aggregated_to_dto(strategy_id: &str, agg: &AggregatedEvaluation) -> CandidateEvaluationDto {
        CandidateEvaluationDto {
            strategy_id: strategy_id.to_string(),
            avg: agg
                .scenario_results
                .iter()
                .map(|r| r.avg_pnl)
                .sum::<f64>()
                / agg.domains_evaluated.max(1) as f64,
            std: agg.domain_consistency,
            fitness: agg.aggregated_execution_fitness,
            classification: classify_execution_fitness(agg.aggregated_execution_fitness),
            ga_fitness: Some(agg.aggregated_fitness),
            execution_fitness: agg.aggregated_execution_fitness,
            total_trades: agg
                .scenario_results
                .iter()
                .map(|r| r.trade_count)
                .sum(),
        }
    }

    fn evaluate_and_aggregate(
        &self,
        strategy: &Strategy,
        seed: u64,
    ) -> (AggregatedEvaluation, CandidateEvaluationDto) {
        let strategy_id = Self::strategy_id_from_candidate(strategy);
        let scenario_results =
            evaluate_strategy_across_domains(&self.scenario_registry, strategy, seed);
        self.store_scenario_results(&strategy_id, scenario_results.clone());
        let aggregated = ScenarioAggregator::robust_min(&scenario_results);
        let dto = Self::aggregated_to_dto(&strategy_id, &aggregated);
        (aggregated, dto)
    }

    pub fn evaluate_strategy(
        &self,
        strategy: Strategy,
        _scenario_names: Vec<String>,
        seed: u64,
    ) -> Result<EvaluateStrategyResponse, ApiError> {
        let (_aggregated, dto) = self.evaluate_and_aggregate(&strategy, seed);
        self.refresh_ranking_store([dto.clone()]);
        Ok(EvaluateStrategyResponse {
            strategy_evaluation: dto,
        })
    }

    pub fn compare_strategies(
        &self,
        strategies: Vec<Strategy>,
        _scenario_names: Vec<String>,
        seed: u64,
    ) -> Result<CompareStrategiesResponse, ApiError> {
        if strategies.len() < 2 {
            return Err(ApiError::ValidationError(
                "compare_strategies requires at least two strategies".to_string(),
            ));
        }

        let mut ranking: Vec<CandidateEvaluationDto> = Vec::new();
        let mut trace: Vec<(String, AggregatedEvaluation)> = Vec::new();

        for strategy in &strategies {
            let (aggregated, dto) = self.evaluate_and_aggregate(strategy, seed);
            trace.push((dto.strategy_id.clone(), aggregated));
            ranking.push(dto);
        }

        ranking.sort_by(|a, b| {
            b.execution_fitness
                .partial_cmp(&a.execution_fitness)
                .unwrap_or(Ordering::Equal)
        });

        self.refresh_ranking_store(ranking.iter().cloned());

        let best = ranking
            .first()
            .ok_or_else(|| ApiError::InternalError("empty ranking".to_string()))?
            .clone();

        let best_trace = trace
            .iter()
            .find(|(id, _)| id == &best.strategy_id)
            .map(|(_, agg)| agg);

        let domain_note = best_trace.map_or_else(
            || String::new(),
            |agg| {
                format!(
                    " Evaluated across {} scenario domains (robust-min aggregation).",
                    agg.domains_evaluated
                )
            },
        );

        Ok(CompareStrategiesResponse {
            ranking,
            comparison_summary: ComparisonSummary {
                best_strategy: best.strategy_id.clone(),
                reason: format!(
                    "Strategy '{}' achieves superior robust-min execution fitness ({:.6}) across declared scenario domains.{}",
                    best.strategy_id, best.execution_fitness, domain_note
                ),
            },
        })
    }

    pub fn inspect_strategy(
        &self,
        strategy_config: Strategy,
        _scenario: String,
        seed: u64,
    ) -> Result<InspectStrategyResponse, ApiError> {
        let strategy_id = Self::strategy_id_from_candidate(&strategy_config);

        let (simulation, primary_order) = run_inspect_simulation(&strategy_config, seed);

        {
            let mut last = self
                .last_simulation
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            *last = Some(simulation.clone());
        }

        let evaluator = Self::default_evaluator();
        let metrics = Self::evaluate_candidate(&evaluator, &strategy_config);

        let execution_trace: Vec<EventWrapper> = simulation
            .events
            .iter()
            .map(sim_event_to_wrapper)
            .collect();

        let decision_trace: Vec<EventWrapper> = simulation
            .events
            .iter()
            .filter(|event| event.order_id() == Some(&primary_order))
            .map(sim_event_to_wrapper)
            .collect();

        Ok(InspectStrategyResponse {
            strategy_id,
            decision_trace,
            execution_trace,
            metrics,
            event_sequence: simulation.events.iter().map(sim_event_to_wrapper).collect(),
        })
    }

    pub fn test_determinism(
        &self,
        strategy: Strategy,
        _scenario_names: Vec<String>,
        _seed: u64,
    ) -> Result<bool, ApiError> {
        let evaluator = Self::default_evaluator();
        let first = Self::evaluate_candidate(&evaluator, &strategy);
        let second = Self::evaluate_candidate(&evaluator, &strategy);
        Ok((first.execution_fitness - second.execution_fitness).abs() < f64::EPSILON)
    }

    pub fn run_ga(&self) -> Result<RunGaResponse, ApiError> {
        let config = GaConfig {
            population_size: 20,
            generations: 10,
            mutation_rate: 0.1,
            crossover_rate: 0.5,
            seed: 42,
        };
        let seed = config.seed;

        let evaluator = Self::default_evaluator();
        let ga_result = run_ga_evolution(config, &evaluator);

        let generation_history: Vec<CandidateEvaluationDto> = ga_result
            .generation_history
            .iter()
            .map(Self::to_evaluation_dto)
            .collect();

        let global_best = Self::to_evaluation_dto(&ga_result.global_best);
        let final_generation_best = generation_history
            .last()
            .cloned()
            .unwrap_or_else(|| global_best.clone());

        let global_best_generation = generation_history
            .iter()
            .position(|row| row.strategy_id == global_best.strategy_id)
            .unwrap_or(0);

        // Ranking store uses ScenarioResult[] projection (GA response DTOs unchanged for UI contract).
        let (_agg, global_best_ranked) =
            self.evaluate_and_aggregate(&ga_result.global_best.candidate, seed);
        self.refresh_ranking_store(std::iter::once(global_best_ranked));

        Ok(RunGaResponse {
            results: vec![global_best.clone()],
            generation_history: generation_history.clone(),
            best_per_regime: HashMap::new(),
            global_best: global_best.clone(),
            global_best_generation,
            generation_found: global_best_generation,
            final_generation_best: final_generation_best.clone(),
            final_gen_best: final_generation_best,
        })
    }

    pub fn get_timeline(&self) -> Result<Vec<EventWrapper>, ApiError> {
        let last = self
            .last_simulation
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(sim) = last.as_ref() {
            Ok(sim.events.iter().map(sim_event_to_wrapper).collect())
        } else {
            Ok(Vec::new())
        }
    }

    pub fn get_global_ranking(&self) -> Result<GlobalRankingResponse, ApiError> {
        let store = self
            .last_global_ranking
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        let rankings: Vec<GlobalRankingRow> = store
            .iter()
            .enumerate()
            .map(|(idx, row)| GlobalRankingRow {
                strategy_id: row.strategy_id.clone(),
                execution_fitness: row.execution_fitness,
                ga_fitness: row.ga_fitness,
                avg: row.avg,
                std: row.std,
                classification: row.classification.clone(),
                rank: idx + 1,
            })
            .collect();

        let total = rankings.len();
        Ok(GlobalRankingResponse { rankings, total })
    }

    pub fn get_strategy_store(&self) -> StrategyStoreResponse {
        let store = self
            .last_global_ranking
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        StrategyStoreResponse {
            strategies: store.clone(),
            store_version: chrono::Utc::now().to_rfc3339(),
            total: store.len(),
        }
    }

    pub fn get_latest_signals(&self) -> Result<SignalsSnapshot<TradeSignal>, ApiError> {
        Ok(generate_latest_signals(vec!["BTC".to_string()], 0.5))
    }

    pub fn get_order_inspection(
        &self,
        order_id: String,
        include_chain: bool,
    ) -> Result<TradeInspectorResponse, ApiError> {
        let last = self
            .last_simulation
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let sim = last.as_ref().ok_or_else(|| {
            ApiError::InternalError(
                "No simulation available — inspect a strategy first".to_string(),
            )
        })?;

        let inspection =
            api::inspector::build_trade_inspector(&sim.events, &order_id, include_chain)?;

        Ok(TradeInspectorResponse {
            order_id: inspection.order_id,
            decision: TradeInspectorDecision {
                order_id: inspection.decision.order_id,
                side: inspection.decision.side,
                price: inspection.decision.price,
                quantity: inspection.decision.quantity,
                timestamp: inspection.decision.timestamp,
            },
            execution: inspection
                .execution
                .iter()
                .map(|event| {
                    serde_json::to_value(event).unwrap_or_else(|_| serde_json::json!({}))
                })
                .collect(),
            outcome: TradeInspectorOutcome {
                filled_qty: inspection.outcome.filled_qty,
                remaining_qty: inspection.outcome.remaining_qty,
                avg_price: inspection.outcome.avg_price,
                status: inspection.outcome.status,
            },
            causal_chain: None,
        })
    }

    pub fn get_replay(&self, _seq_id: u64) -> Result<SystemState, ApiError> {
        Err(ApiError::InternalError(
            "replay requires simulation state — not yet wired".to_string(),
        ))
    }

    pub fn get_trade_suggestions(&self) -> Result<TradeSuggestionsResponse, ApiError> {
        Err(ApiError::InternalError(
            "trade suggestions not yet wired".to_string(),
        ))
    }

    pub fn get_replay_suggestions(
        &self,
        _mode: String,
        _limit: usize,
        _sample_rate: usize,
        _include_full: bool,
    ) -> Result<ReplaySuggestionsResponse, ApiError> {
        Err(ApiError::InternalError(
            "replay suggestions not yet wired".to_string(),
        ))
    }

    pub fn load_all_real_scenarios(&self) -> Result<HashMap<String, Vec<chronosentiment_core::SimEvent>>, ApiError> {
        Ok(HashMap::new())
    }
}

fn classify_strategy(eval: &CandidateEvaluation) -> String {
    classify_execution_fitness((eval.fitness / 100.0).clamp(0.0, 1.0))
}

fn classify_execution_fitness(execution_fitness: f64) -> String {
    if execution_fitness >= 0.75 {
        "ALPHA".to_string()
    } else if execution_fitness >= 0.40 {
        "BETA".to_string()
    } else {
        "GAMMA".to_string()
    }
}
