use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SolutionQuality {
    pub best_fitness: f64,
    pub average_fitness: f64,
    pub worst_fitness: f64,
    pub gap_to_bks: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExecutionMetrics {
    pub runtime_ms: u128,
    pub evaluations: usize,
    pub generations: usize,
    pub population_size: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EngineMetrics {
    pub num_processors_executed: usize,
    pub processor_execution_time_ms: HashMap<usize, f64>,
    pub processor_invocation_counts: HashMap<usize, usize>,
    pub processing_overhead_ms: f64,
    pub observer_overhead_ms: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConvergenceMetrics {
    pub best_fitness_per_generation: Vec<f64>,
    pub average_fitness_per_generation: Vec<f64>,
    pub diversity: Option<f64>,
    pub stagnation_generation: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MogaBenchmarkReport {
    pub milestone: String,
    pub timestamp: String,
    pub solution_quality: SolutionQuality,
    pub execution_metrics: ExecutionMetrics,
    pub engine_metrics: EngineMetrics,
    pub convergence_metrics: ConvergenceMetrics,
}
