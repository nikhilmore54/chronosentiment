use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Clone)]
pub struct SearchSnapshot {
    pub generation: usize,
    
    // Core telemetry
    pub best_fitness: f64,
    pub best_fitness_age: usize,
    pub improvement_magnitude: f64,
    
    // Ecology metrics
    pub diversity: f64,
    pub memory_novelty_proxy: f64,
    pub history_novelty: f64,
    pub revisit_rate: f64,
    pub distance_to_incumbent_best: f64,
    pub operator_success_rate: f64,
    
    // Operator dominance
    pub dominant_operator: String,
    pub dominant_operator_share: f64,
    pub unique_successful_operators: usize,
    
    // Explicit Operator Competition
    pub tier1_attempts: usize,
    pub tier1_acceptances: usize,
    pub tier1_improvements: usize,
    pub tier2_attempts: usize,
    pub tier2_acceptances: usize,
    pub tier2_improvements: usize,
    
    // SA Exploration Pressure
    pub temperature: f64,
    pub accepted_worse_moves: usize,
    pub accepted_better_moves: usize,
    pub acceptance_rate: f64,
    pub worse_acceptance_rate: f64,
    pub better_acceptance_rate: f64,
}

pub struct SearchStateObservatory {
    last_best_fitness: f64,
    last_improvement_generation: usize,
    last_improvement_magnitude: f64,
    snapshots: Vec<SearchSnapshot>,
}

impl SearchStateObservatory {
    pub fn new() -> Self {
        Self {
            last_best_fitness: f64::NAN,
            last_improvement_generation: 0,
            last_improvement_magnitude: 0.0,
            snapshots: Vec::new(),
        }
    }

    /// Observes the state of the search for a minimization objective.
    pub fn observe_minimization(
        &mut self,
        generation: usize,
        best_fitness: f64,
        diversity: f64,
        memory_novelty_proxy: f64,
        history_novelty: f64,
        revisit_rate: f64,
        distance_to_incumbent_best: f64,
        operator_success_rate: f64,
        operator_counts: &HashMap<String, usize>,
        temperature: f64,
        accepted_worse_moves: usize,
        accepted_better_moves: usize,
        acceptance_rate: f64,
        worse_acceptance_rate: f64,
        better_acceptance_rate: f64,
        tier1_attempts: usize,
        tier1_acceptances: usize,
        tier1_improvements: usize,
        tier2_attempts: usize,
        tier2_acceptances: usize,
        tier2_improvements: usize,
    ) -> SearchSnapshot {
        self.observe(
            generation,
            best_fitness,
            diversity,
            memory_novelty_proxy,
            history_novelty,
            revisit_rate,
            distance_to_incumbent_best,
            operator_success_rate,
            operator_counts,
            temperature,
            accepted_worse_moves,
            accepted_better_moves,
            acceptance_rate,
            worse_acceptance_rate,
            better_acceptance_rate,
            tier1_attempts,
            tier1_acceptances,
            tier1_improvements,
            tier2_attempts,
            tier2_acceptances,
            tier2_improvements,
            true,
        )
    }

    /// Observes the state of the search for a maximization objective.
    pub fn observe_maximization(
        &mut self,
        generation: usize,
        best_fitness: f64,
        diversity: f64,
        memory_novelty_proxy: f64,
        history_novelty: f64,
        revisit_rate: f64,
        distance_to_incumbent_best: f64,
        operator_success_rate: f64,
        operator_counts: &HashMap<String, usize>,
        temperature: f64,
        accepted_worse_moves: usize,
        accepted_better_moves: usize,
        acceptance_rate: f64,
        worse_acceptance_rate: f64,
        better_acceptance_rate: f64,
        tier1_attempts: usize,
        tier1_acceptances: usize,
        tier1_improvements: usize,
        tier2_attempts: usize,
        tier2_acceptances: usize,
        tier2_improvements: usize,
    ) -> SearchSnapshot {
        self.observe(
            generation,
            best_fitness,
            diversity,
            memory_novelty_proxy,
            history_novelty,
            revisit_rate,
            distance_to_incumbent_best,
            operator_success_rate,
            operator_counts,
            temperature,
            accepted_worse_moves,
            accepted_better_moves,
            acceptance_rate,
            worse_acceptance_rate,
            better_acceptance_rate,
            tier1_attempts,
            tier1_acceptances,
            tier1_improvements,
            tier2_attempts,
            tier2_acceptances,
            tier2_improvements,
            false,
        )
    }

    fn observe(
        &mut self,
        generation: usize,
        best_fitness: f64,
        diversity: f64,
        memory_novelty_proxy: f64,
        history_novelty: f64,
        revisit_rate: f64,
        distance_to_incumbent_best: f64,
        operator_success_rate: f64,
        operator_counts: &HashMap<String, usize>,
        temperature: f64,
        accepted_worse_moves: usize,
        accepted_better_moves: usize,
        acceptance_rate: f64,
        worse_acceptance_rate: f64,
        better_acceptance_rate: f64,
        tier1_attempts: usize,
        tier1_acceptances: usize,
        tier1_improvements: usize,
        tier2_attempts: usize,
        tier2_acceptances: usize,
        tier2_improvements: usize,
        is_minimization: bool,
    ) -> SearchSnapshot {
        let mut is_improvement = false;
        let mut magnitude = 0.0;

        if self.last_best_fitness.is_nan() {
            is_improvement = false;
            magnitude = 0.0; // First observation is a baseline, not an improvement
            self.last_best_fitness = best_fitness;
        } else if is_minimization && best_fitness < self.last_best_fitness {
            is_improvement = true;
            magnitude = self.last_best_fitness - best_fitness;
        } else if !is_minimization && best_fitness > self.last_best_fitness {
            is_improvement = true;
            magnitude = best_fitness - self.last_best_fitness;
        }

        if is_improvement {
            self.last_best_fitness = best_fitness;
            self.last_improvement_generation = generation;
            self.last_improvement_magnitude = magnitude;
        }

        let best_fitness_age = generation.saturating_sub(self.last_improvement_generation);

        let mut dominant_operator = String::from("None");
        let mut dominant_operator_share = 0.0;
        let mut unique_successful_operators = 0;

        let total_successes: usize = operator_counts.values().sum();
        if total_successes > 0 {
            for (op, &count) in operator_counts {
                if count > 0 {
                    unique_successful_operators += 1;
                }
                let share = count as f64 / total_successes as f64;
                if share > dominant_operator_share {
                    dominant_operator_share = share;
                    dominant_operator = op.clone();
                }
            }
        }

        let snapshot = SearchSnapshot {
            generation,
            best_fitness,
            best_fitness_age,
            improvement_magnitude: if is_improvement { magnitude } else { 0.0 },
            diversity,
            memory_novelty_proxy,
            history_novelty,
            revisit_rate,
            distance_to_incumbent_best,
            operator_success_rate,
            dominant_operator,
            dominant_operator_share,
            unique_successful_operators,
            temperature,
            accepted_worse_moves,
            accepted_better_moves,
            acceptance_rate,
            worse_acceptance_rate,
            better_acceptance_rate,
            tier1_attempts,
            tier1_acceptances,
            tier1_improvements,
            tier2_attempts,
            tier2_acceptances,
            tier2_improvements,
        };

        self.snapshots.push(snapshot.clone());
        snapshot
    }

    pub fn get_history(&self) -> &[SearchSnapshot] {
        &self.snapshots
    }
}
