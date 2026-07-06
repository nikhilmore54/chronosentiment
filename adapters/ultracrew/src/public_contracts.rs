use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use crate::models::{Worker, Shift};
use crate::optimization::ScheduleContext;
use crate::ecology::WorkforceEcology;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleRequest {
    pub workers: Vec<Worker>,
    pub shifts: Vec<Shift>,
    pub historical_workloads: Option<HashMap<u64, Vec<f64>>>,
    pub rng_seed: Option<u64>,
    pub generation_limit: Option<usize>,
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
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateRequest {
    pub request: ScheduleRequest,
    pub assignments: HashMap<u64, u64>,
}

