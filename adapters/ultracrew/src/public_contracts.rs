use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use crate::models::{Worker, Shift};
use crate::optimization::ScheduleContext;
use crate::ecology::WorkforceEcology;

/// Domain-independent optimization context supplied by the adapter.
/// Contains no domain-specific concepts (flights, nurses, trains).
/// Sits between Coralys (Optimization Engine) and the Solution Engine.
/// All fields are optional so existing callers remain fully compatible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    /// Total planning horizon in hours (e.g. 744.0 for a 31-day month).
    /// Used for reporting and future horizon-aware constraints.
    pub planning_horizon_hours: Option<f64>,
    /// Maximum credited hours per worker over the planning horizon.
    /// None means no per-worker upper bound is specified by the dataset;
    /// the engine falls back to DEFAULT_WEEKLY_MAX_HOURS.
    pub max_hours_per_worker: Option<f64>,
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
    pub scenario: Option<Scenario>,
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
            observatory: Arc::new(std::sync::Mutex::new(crate::optimization::Observatory::new())),
            locked_assignments: None,
            scenario: self.scenario.clone(),
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
            observatory: Arc::new(std::sync::Mutex::new(crate::optimization::Observatory::new())),
            locked_assignments: Some(locked_assignments),
            scenario: self.request.scenario.clone(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateRequest {
    pub request: ScheduleRequest,
    pub assignments: HashMap<u64, u64>,
}
