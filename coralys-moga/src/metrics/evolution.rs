use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProcessorMetrics {
    pub processor_name: String,
    pub invocation_count: usize,
    pub total_runtime: Duration,
    pub average_runtime: Duration,
    pub maximum_runtime: Duration,
    pub minimum_runtime: Duration,
    pub candidates_processed: usize,
}

impl Default for ProcessorMetrics {
    fn default() -> Self {
        Self {
            processor_name: String::new(),
            invocation_count: 0,
            total_runtime: Duration::ZERO,
            average_runtime: Duration::ZERO,
            maximum_runtime: Duration::ZERO,
            minimum_runtime: Duration::MAX, // Initialize min with max duration
            candidates_processed: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct EvolutionMetrics {
    pub generation: usize,
    pub best_fitness: f64,
    pub average_fitness: f64,
    pub worst_fitness: f64,
    pub fitness_stddev: f64,
    pub convergence_rate: f64,
    pub stagnation_generations: usize,
    pub evaluation_count: usize,
    pub elapsed_time: Duration,

    pub processors: HashMap<usize, ProcessorMetrics>,

    pub best_history: Vec<f64>,
    pub average_history: Vec<f64>,
}
