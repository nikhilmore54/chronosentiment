use crate::traits::{Genome, Evaluated, FitnessEvaluator, LocalSearchOperator, ObservedTransitionMetric, RegionIdentifier};

#[derive(Debug, Clone)]
pub struct ReachabilityObservation<R> {
    pub raw_magnitude: f64,
    pub residual_magnitude: f64,
    pub repair_delta: f64,
    pub fitness_delta: f64,
    pub retained_elite: bool,
    pub discovered_new_region: bool,
    pub returned_to_same_region: bool,
    pub s1_returned_to_same_region: bool,
    pub target_region: R,
    pub s1_region: R,
    pub s1_fitness: f64,
    pub s2_fitness: f64,
}

pub struct ReachabilityProbe<'a, G, E, F, TM, RI>
where
    G: Genome,
    E: Evaluated<Genome = G>,
    F: Fn(&mut G),
    TM: ObservedTransitionMetric<G>,
    RI: RegionIdentifier<G>,
{
    pub evaluator: &'a dyn FitnessEvaluator<G, Evaluation = E>,
    pub local_search: F,
    pub metric: &'a TM,
    pub region_identifier: &'a RI,
    pub elite_threshold: f64,
}

impl<'a, G, E, F, TM, RI> ReachabilityProbe<'a, G, E, F, TM, RI>
where
    G: Genome,
    E: Evaluated<Genome = G>,
    F: Fn(&mut G),
    TM: ObservedTransitionMetric<G>,
    RI: RegionIdentifier<G>,
{
    pub fn new(
        evaluator: &'a dyn FitnessEvaluator<G, Evaluation = E>,
        local_search: F,
        metric: &'a TM,
        region_identifier: &'a RI,
        elite_threshold: f64,
    ) -> Self {
        Self {
            evaluator,
            local_search,
            metric,
            region_identifier,
            elite_threshold,
        }
    }

    /// Evaluates a transition from `source` to `mutated_child` (before local search).
    /// The probe will apply local search to the child to simulate the repair cascade,
    /// and then compute the observed transition magnitude and region novelty.
    pub fn evaluate_transition(
        &self,
        source: &G,
        mutated_child: &mut G,
        source_fitness: f64,
        source_region: &RI::RegionId,
    ) -> ReachabilityObservation<RI::RegionId> {
        // S1: The raw mutated child
        let s1 = mutated_child.clone();
        
        // Measure S1
        let raw_magnitude = self.metric.magnitude(source, &s1);
        let empty_metrics = crate::runtime::optimization::metric::MetricReport::default();
        let s1_fitness = self.evaluator.evaluate(&s1, &empty_metrics).fitness();
        
        // 1. Apply local search repair cascade to mutate it to S2
        (self.local_search)(mutated_child);
        
        // Measure S2
        let s2_fitness = self.evaluator.evaluate(mutated_child, &empty_metrics).fitness();
        let residual_magnitude = self.metric.magnitude(source, mutated_child);
        let repair_delta = self.metric.magnitude(&s1, mutated_child);
        
        // Identify S2 region
        let target_region = self.region_identifier.region_of(mutated_child);
        let s1_region = self.region_identifier.region_of(&s1);
        let returned_to_same_region = &target_region == source_region;
        let s1_returned_to_same_region = &s1_region == source_region;
        let discovered_new_region = !returned_to_same_region;
        
        // Compute retention (based on S2)
        let retained_elite = s2_fitness >= self.elite_threshold;
        let fitness_delta = s2_fitness - source_fitness;
        
        ReachabilityObservation {
            raw_magnitude,
            residual_magnitude,
            repair_delta,
            fitness_delta,
            retained_elite,
            discovered_new_region,
            returned_to_same_region,
            s1_returned_to_same_region,
            target_region,
            s1_region,
            s1_fitness,
            s2_fitness,
        }
    }
}

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ProcessingEvent<G: Genome> {
    pub processor_index: usize,
    pub duration: Duration,
    pub generation: usize,
    _marker: std::marker::PhantomData<G>,
}

impl<G: Genome> ProcessingEvent<G> {
    pub fn new(processor_index: usize, duration: Duration, generation: usize) -> Self {
        Self {
            processor_index,
            duration,
            generation,
            _marker: std::marker::PhantomData,
        }
    }
}

pub trait PipelineObserver<G: Genome>: Send + Sync {
    fn on_event(&self, event: &ProcessingEvent<G>);
    fn on_repair_event(&self, _event: &RepairEvent) {}
    fn on_feasibility_report(&self, _report: &FeasibilityReport) {}
}

/// Read-only generation hook. Must not consume RNG or alter search order.
pub trait GenerationObserver<G: Genome, E: Evaluated<Genome = G>>: Send + Sync {
    fn on_evaluated_generation(&self, generation: usize, evaluations: &[E]);
    fn on_offspring(&self, generation: usize, parent_a: &G, parent_b: &G, child: &G);
}

#[derive(Debug, Clone)]
pub struct FeasibilityReport {
    pub hard_violations_remaining: usize,
    pub soft_violations_remaining: usize,
    pub repair_attempts: usize,
    pub constraint_coverage: f64,
}

#[derive(Debug, Clone)]
pub struct RepairEvent {
    pub generation: usize,
    pub violation_id: String,
    pub action_description: Option<String>,
    pub action_payload: Option<serde_json::Value>,
    pub action_priority: Option<f64>,
    pub attempts: usize,
    pub successful: bool,
}

#[derive(Default)]
pub struct ProcessingMetricsCollector {
    pub execution_counts: Mutex<HashMap<usize, usize>>,
    pub cumulative_times: Mutex<HashMap<usize, Duration>>,
    pub processed_count: Mutex<usize>,
    
    // Repair metrics
    pub repair_attempts: Mutex<usize>,
    pub successful_repairs: Mutex<usize>,
    pub failed_repairs: Mutex<usize>,
}

impl ProcessingMetricsCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn average_time(&self, processor_index: usize) -> Duration {
        let counts = self.execution_counts.lock().unwrap();
        let times = self.cumulative_times.lock().unwrap();
        let count = *counts.get(&processor_index).unwrap_or(&0);
        if count == 0 {
            Duration::ZERO
        } else {
            *times.get(&processor_index).unwrap_or(&Duration::ZERO) / count as u32
        }
    }
}

impl<G: Genome> PipelineObserver<G> for ProcessingMetricsCollector {
    fn on_event(&self, event: &ProcessingEvent<G>) {
        let mut counts = self.execution_counts.lock().unwrap();
        let mut times = self.cumulative_times.lock().unwrap();
        let mut total = self.processed_count.lock().unwrap();

        *counts.entry(event.processor_index).or_insert(0) += 1;
        *times.entry(event.processor_index).or_insert(Duration::ZERO) += event.duration;
        *total += 1;
    }

    fn on_repair_event(&self, event: &RepairEvent) {
        let mut attempts = self.repair_attempts.lock().unwrap();
        *attempts += event.attempts;
        
        if event.successful {
            let mut success = self.successful_repairs.lock().unwrap();
            *success += 1;
        } else {
            let mut fail = self.failed_repairs.lock().unwrap();
            *fail += 1;
        }
    }
}

