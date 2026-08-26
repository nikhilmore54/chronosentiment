use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressObservation {
    pub improvement_rate_100: f64,
    pub improvement_rate_500: f64,
    pub average_improvement_size_100: f64,
    pub average_improvement_size_all_time: f64,
    pub largest_improvement: f64,
    pub stagnation_duration: usize,
    pub basin_residency_ratio: f64,
}

struct ImprovementEvent {
    generation: usize,
    size: f64,
}

pub struct ProgressTracker {
    last_fitness: Option<f64>,
    improvements: VecDeque<ImprovementEvent>,
    total_improvements: usize,
    sum_improvements: f64,
    largest_improvement: f64,
    last_improvement_generation: usize,
}

impl ProgressTracker {
    pub fn new() -> Self {
        Self {
            last_fitness: None,
            improvements: VecDeque::new(),
            total_improvements: 0,
            sum_improvements: 0.0,
            largest_improvement: 0.0,
            last_improvement_generation: 1, // Start at 1 to avoid division by zero
        }
    }

    pub fn observe_minimization(
        &mut self,
        generation: usize,
        current_cost: f64,
    ) -> ProgressObservation {
        if let Some(prev) = self.last_fitness {
            if current_cost < prev {
                let size = prev - current_cost;
                self.improvements
                    .push_back(ImprovementEvent { generation, size });
                self.total_improvements += 1;
                self.sum_improvements += size;
                if size > self.largest_improvement {
                    self.largest_improvement = size;
                }
                self.last_improvement_generation = generation;
            }
        }
        self.last_fitness = Some(current_cost);

        self.compute_observation(generation)
    }

    pub fn observe_maximization(
        &mut self,
        generation: usize,
        current_fitness: f64,
    ) -> ProgressObservation {
        if let Some(prev) = self.last_fitness {
            if current_fitness > prev {
                let size = current_fitness - prev;
                self.improvements
                    .push_back(ImprovementEvent { generation, size });
                self.total_improvements += 1;
                self.sum_improvements += size;
                if size > self.largest_improvement {
                    self.largest_improvement = size;
                }
                self.last_improvement_generation = generation;
            }
        }
        self.last_fitness = Some(current_fitness);

        self.compute_observation(generation)
    }

    fn compute_observation(&mut self, generation: usize) -> ProgressObservation {
        // Remove old events outside the maximum window we care about (500)
        while let Some(evt) = self.improvements.front() {
            if evt.generation + 500 < generation {
                self.improvements.pop_front();
            } else {
                break;
            }
        }

        let mut improvements_100 = 0;
        let mut sum_size_100 = 0.0;
        let mut improvements_500 = 0;

        for evt in self.improvements.iter().rev() {
            if evt.generation + 100 >= generation {
                improvements_100 += 1;
                sum_size_100 += evt.size;
            }
            if evt.generation + 500 >= generation {
                improvements_500 += 1;
            } else {
                break;
            }
        }

        let improvement_rate_100 = improvements_100 as f64 / 100.0;
        let improvement_rate_500 = improvements_500 as f64 / 500.0;
        let average_improvement_size_100 = if improvements_100 > 0 {
            sum_size_100 / improvements_100 as f64
        } else {
            0.0
        };
        let average_improvement_size_all_time = if self.total_improvements > 0 {
            self.sum_improvements / self.total_improvements as f64
        } else {
            0.0
        };
        let stagnation_duration = generation.saturating_sub(self.last_improvement_generation);

        let basin_residency_ratio = if generation > 0 {
            stagnation_duration as f64 / generation as f64
        } else {
            0.0
        };

        ProgressObservation {
            improvement_rate_100,
            improvement_rate_500,
            average_improvement_size_100,
            average_improvement_size_all_time,
            largest_improvement: self.largest_improvement,
            stagnation_duration,
            basin_residency_ratio,
        }
    }
}
