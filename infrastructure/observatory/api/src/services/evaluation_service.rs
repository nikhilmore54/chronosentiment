use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chronosentiment_core::{SimulationResult, SimEvent};
use crate::dto::*;
use crate::errors::ApiError;
use chronosentiment_strategies::compatibility::{SignalsSnapshot, TradeSignal};

#[derive(Clone)]
pub struct EvaluationService {
    pub last_simulation: Arc<Mutex<Option<SimulationResult>>>,
    pub last_global_ranking: Arc<Mutex<Vec<CandidateEvaluationDto>>>,
}

impl EvaluationService {
    pub fn new() -> Self {
        Self {
            last_simulation: Arc::new(Mutex::new(None)),
            last_global_ranking: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn evaluate_strategy(&self, _strategy: Strategy, _scenario_names: Vec<String>, _seed: u64) -> Result<EvaluateStrategyResponse, ApiError> {
        unimplemented!()
    }

    pub fn compare_strategies(&self, _strategies: Vec<Strategy>, _scenario_names: Vec<String>, _seed: u64) -> Result<CompareStrategiesResponse, ApiError> {
        unimplemented!()
    }

    pub fn inspect_strategy(&self, _strategy_config: Strategy, _scenario: String, _seed: u64) -> Result<InspectStrategyResponse, ApiError> {
        unimplemented!()
    }

    pub fn test_determinism(&self, _strategy: Strategy, _scenario_names: Vec<String>, _seed: u64) -> Result<bool, ApiError> {
        unimplemented!()
    }

    pub fn run_ga(&self) -> Result<RunGaResponse, ApiError> {
        unimplemented!()
    }

    pub fn get_timeline(&self) -> Result<Vec<EventWrapper>, ApiError> {
        unimplemented!()
    }

    pub fn get_global_ranking(&self) -> Result<Vec<CandidateEvaluationDto>, ApiError> {
        unimplemented!()
    }

    pub fn get_latest_signals(&self) -> Result<SignalsSnapshot<TradeSignal>, ApiError> {
        unimplemented!()
    }

    pub fn get_order_inspection(&self, _order_id: String, _include_chain: bool) -> Result<TradeInspectorResponse, ApiError> {
        unimplemented!()
    }

    pub fn get_replay(&self, _seq_id: u64) -> Result<SystemState, ApiError> {
        unimplemented!()
    }

    pub fn get_trade_suggestions(&self) -> Result<TradeSuggestionsResponse, ApiError> {
        unimplemented!()
    }

    pub fn get_replay_suggestions(&self, _mode: String, _limit: usize, _sample_rate: usize, _include_full: bool) -> Result<ReplaySuggestionsResponse, ApiError> {
        unimplemented!()
    }

    pub fn load_all_real_scenarios(&self) -> Result<HashMap<String, Vec<SimEvent>>, ApiError> {
        unimplemented!()
    }
}