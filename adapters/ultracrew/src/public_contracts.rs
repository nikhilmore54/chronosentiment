use crate::config::FatigueConfig;
use crate::ecology::WorkforceEcology;
use crate::models::{Shift, Worker};
use crate::optimization::ScheduleContext;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveRequest {
    pub crew_id: u64,
    pub start_hour: u64,
    pub end_hour: u64,
}

/// Domain-independent optimization context supplied by the INRC adapter.
/// Sits between Coralys (Optimization Engine) and the Solution Engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InrcScenario {
    /// Total planning horizon in hours (e.g. 744.0 for a 31-day month).
    /// Used for reporting and future horizon-aware constraints.
    pub planning_horizon_hours: Option<f64>,
    /// Maximum credited hours per worker over the planning horizon.
    /// None means no per-worker upper bound is specified by the dataset;
    /// the engine falls back to DEFAULT_WEEKLY_MAX_HOURS.
    pub max_hours_per_worker: Option<f64>,
    /// Minimum required rest gap between consecutive shifts for a worker.
    /// None means the engine falls back to 10 hours.
    pub minimum_rest_hours: Option<u64>,
    /// Active leave requests that must be respected as hard constraints.
    pub leave_requests: Option<Vec<LeaveRequest>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleRequest {
    pub workers: Vec<Worker>,
    pub shifts: Vec<Shift>,
    pub historical_workloads: Option<HashMap<u64, Vec<f64>>>,
    pub rng_seed: Option<u64>,
    pub generation_limit: Option<usize>,
    /// Optional domain-independent scenario contract supplied by the adapter.
    /// When present, the engine uses scenario fields to contextualise constraints.
    /// When absent, engine defaults apply (backward-compatible).
    #[serde(default)]
    pub scenario: Option<InrcScenario>,
    #[serde(default)]
    pub fatigue: FatigueConfig,
}

impl Default for ScheduleRequest {
    fn default() -> Self {
        Self {
            workers: Vec::new(),
            shifts: Vec::new(),
            historical_workloads: None,
            rng_seed: None,
            generation_limit: None,
            scenario: None,
            fatigue: FatigueConfig::default(),
        }
    }
}

impl ScheduleRequest {
    pub fn to_context(&self) -> Arc<ScheduleContext> {
        let mut ecology = WorkforceEcology::new();
        if let Some(ref workloads) = self.historical_workloads {
            for (&worker_id, hours_list) in workloads {
                for &hours in hours_list {
                    ecology.record_historical_hours(worker_id, hours);
                }
            }
        }
        Arc::new(ScheduleContext {
            workers: Arc::new(self.workers.clone()),
            shifts: Arc::new(self.shifts.clone()),
            ecology,
            rng_seed: self.rng_seed.unwrap_or(0),
            observatory: Arc::new(std::sync::Mutex::new(
                crate::optimization::Observatory::new(),
            )),
            locked_assignments: None,
            scenario: self.scenario.clone(),
            enable_fatigue: self.fatigue.enable_fatigue,
            fatigue_weight: self.fatigue.fatigue_weight,
            hc3_aware_initialization: false,
            temporal_scarcity_construction: false,
            disable_global_constructor: false,
            constructor_budget_ms: None,
            precomputed_seeds: None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RescheduleRequest {
    pub request: ScheduleRequest,
    pub existing_assignments: HashMap<u64, u64>,
    pub locked_shift_ids: Option<Vec<u64>>,
    pub generation_limit: Option<usize>,
    pub tournament_size: Option<usize>,
    pub population_size: Option<usize>,
    pub mutation_rate: Option<f64>,
    pub crossover_rate: Option<f64>,
    pub elite_count: Option<usize>,
}

impl RescheduleRequest {
    pub fn to_context(&self) -> Arc<ScheduleContext> {
        let mut ecology = WorkforceEcology::new();
        if let Some(ref workloads) = self.request.historical_workloads {
            for (&worker_id, hours_list) in workloads {
                for &hours in hours_list {
                    ecology.record_historical_hours(worker_id, hours);
                }
            }
        }
        let mut locked_assignments = HashMap::new();
        if let Some(ref locked_ids) = self.locked_shift_ids {
            for &shift_id in locked_ids {
                if let Some(&worker_id) = self.existing_assignments.get(&shift_id) {
                    locked_assignments.insert(shift_id, worker_id);
                }
            }
        }
        Arc::new(ScheduleContext {
            workers: Arc::new(self.request.workers.clone()),
            shifts: Arc::new(self.request.shifts.clone()),
            ecology,
            rng_seed: self.request.rng_seed.unwrap_or(0),
            observatory: Arc::new(std::sync::Mutex::new(
                crate::optimization::Observatory::new(),
            )),
            locked_assignments: Some(locked_assignments),
            scenario: self.request.scenario.clone(),
            enable_fatigue: self.request.fatigue.enable_fatigue,
            fatigue_weight: self.request.fatigue.fatigue_weight,
            hc3_aware_initialization: false,
            temporal_scarcity_construction: false,
            disable_global_constructor: false,
            constructor_budget_ms: None,
            precomputed_seeds: None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateRequest {
    pub request: ScheduleRequest,
    pub assignments: HashMap<u64, u64>,
}

/// A single solution on the Pareto frontier produced by the INRC startup pipeline.
/// Returned by `pipeline::run_inrc_startup_pipeline` so the application layer
/// does not need to import `coralys_moga` types directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InrcParetoSolution {
    pub s6_assignment_penalty: f64,
    pub s7_weekend_penalty: f64,
    pub recovery_penalty: f64,
    pub workload_balance: f64,
    pub temporal_load_balance: f64,
    pub schedule: std::collections::HashMap<String, Vec<String>>,
}

/// Result returned by `pipeline::run_inrc_startup_pipeline`.
/// Contains the best schedule and the full Pareto frontier.
#[derive(Debug, Clone)]
pub struct InrcStartupResult {
    pub schedule: std::collections::HashMap<String, Vec<String>>,
    pub pareto_solutions: Vec<InrcParetoSolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisruptionRecoveryRequest {
    pub base_request: ScheduleRequest,
    pub existing_assignments: HashMap<u64, u64>,
    pub disrupted_worker_id: u64,
    pub disruption_start_hour: u64,
    pub disruption_end_hour: u64,
    pub target_shift_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryScenario {
    pub title: String,
    pub description: String,
    pub cost_penalty: f64,
    pub delay_hours: u64,
    pub reserve_used: usize,
    pub shifts_affected: usize,
    pub assignments: HashMap<u64, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisruptionRecoveryResult {
    pub original_penalty: f64,
    pub recovery_options: Vec<RecoveryScenario>,
}
