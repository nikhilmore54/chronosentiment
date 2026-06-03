use crate::ecology::WorkforceEcology;
use crate::models::{Shift, Worker};
use coralys_moga::traits::{CrossoverOperator, Evaluated, FitnessEvaluator, Genome, GenomeFactory, MutationOperator};
use rand::rngs::StdRng;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ScheduleGenome {
    // Maps shift_id to worker_id
    pub assignments: HashMap<u64, u64>,
}

impl Genome for ScheduleGenome {}

#[derive(Debug, Clone)]
pub struct ScheduleEvaluation {
    pub schedule: ScheduleGenome,
    pub fitness: f64,
    pub is_valid: bool,
    // Observability metrics for proof
    pub hc1_violations: usize,
    pub hc2_violations: usize,
    pub hc3_violations: usize,
    pub fairness_penalty: f64,
    pub fatigue_penalty: f64,
}

impl Evaluated for ScheduleEvaluation {
    type Genome = ScheduleGenome;

    fn fitness(&self) -> f64 {
        self.fitness
    }

    fn is_valid(&self) -> bool {
        self.is_valid
    }

    fn genome(&self) -> &Self::Genome {
        &self.schedule
    }
}

pub struct ScheduleContext {
    pub workers: Arc<Vec<Worker>>,
    pub shifts: Arc<Vec<Shift>>,
    pub ecology: WorkforceEcology,
}

pub struct ScheduleOptimizer {
    pub context: Arc<ScheduleContext>,
}

impl GenomeFactory<ScheduleGenome> for ScheduleOptimizer {
    fn create(&self, rng: &mut StdRng) -> ScheduleGenome {
        let mut assignments = HashMap::new();
        for shift in self.context.shifts.iter() {
            let worker = &self.context.workers[rng.gen_range(0..self.context.workers.len())];
            assignments.insert(shift.id, worker.id);
        }
        ScheduleGenome { assignments }
    }
}

impl MutationOperator<ScheduleGenome> for ScheduleOptimizer {
    fn mutate(&self, genome: &mut ScheduleGenome, rng: &mut StdRng) {
        // Simple mutator: pick a random shift and reassign it to a random worker
        if self.context.shifts.is_empty() {
            return;
        }
        let shift = &self.context.shifts[rng.gen_range(0..self.context.shifts.len())];
        let worker = &self.context.workers[rng.gen_range(0..self.context.workers.len())];
        genome.assignments.insert(shift.id, worker.id);
    }
}

impl CrossoverOperator<ScheduleGenome> for ScheduleOptimizer {
    fn crossover(
        &self,
        parent_a: &ScheduleGenome,
        parent_b: &ScheduleGenome,
        rng: &mut StdRng,
    ) -> (ScheduleGenome, ScheduleGenome) {
        let mut child1_assignments = HashMap::new();
        let mut child2_assignments = HashMap::new();

        for shift in self.context.shifts.iter() {
            if rng.gen_bool(0.5) {
                child1_assignments.insert(shift.id, *parent_a.assignments.get(&shift.id).unwrap());
                child2_assignments.insert(shift.id, *parent_b.assignments.get(&shift.id).unwrap());
            } else {
                child1_assignments.insert(shift.id, *parent_b.assignments.get(&shift.id).unwrap());
                child2_assignments.insert(shift.id, *parent_a.assignments.get(&shift.id).unwrap());
            }
        }

        (
            ScheduleGenome { assignments: child1_assignments },
            ScheduleGenome { assignments: child2_assignments },
        )
    }
}

impl FitnessEvaluator<ScheduleGenome> for ScheduleOptimizer {
    type Evaluation = ScheduleEvaluation;

    fn evaluate(&self, genome: &ScheduleGenome) -> Self::Evaluation {
        let mut fitness = 0.0;
        let mut hc1_violations = 0;
        let mut hc2_violations = 0;
        let mut hc3_violations = 0;
        let mut fairness_penalty = 0.0;
        let mut fatigue_penalty = 0.0;

        let mut worker_hours: HashMap<u64, u64> = HashMap::new();
        let mut worker_shifts: HashMap<u64, Vec<&Shift>> = HashMap::new();

        // Pass 1: Aggregate data and evaluate HC1 (Skills)
        for shift in self.context.shifts.iter() {
            let worker_id = genome.assignments.get(&shift.id).unwrap();
            let worker = self.context.workers.iter().find(|w| w.id == *worker_id).unwrap();

            // HC1: Skill match
            if !worker.skills.contains(&shift.required_skill) {
                fitness -= 1000.0;
                hc1_violations += 1;
            }

            *worker_hours.entry(*worker_id).or_insert(0) += shift.duration_hours;
            worker_shifts.entry(*worker_id).or_default().push(shift);
        }

        // Pass 2: Evaluate HC2, HC3, SC1, SC2
        let mut hours_list = Vec::new();

        for worker in self.context.workers.iter() {
            let hours = *worker_hours.get(&worker.id).unwrap_or(&0);
            hours_list.push(hours as f64);

            // HC3: Max Hours (40)
            if hours > 40 {
                fitness -= 500.0;
                hc3_violations += 1;
            }

            // HC2: Double Booking
            if let Some(shifts) = worker_shifts.get(&worker.id) {
                let mut sorted_shifts = shifts.clone();
                sorted_shifts.sort_by_key(|s| s.start_hour);
                for i in 0..sorted_shifts.len() {
                    for j in (i + 1)..sorted_shifts.len() {
                        if sorted_shifts[i].overlaps_with(sorted_shifts[j]) {
                            fitness -= 1000.0;
                            hc2_violations += 1;
                        }
                    }
                }
            }

            // SC2: Fatigue (Ecology integration)
            let historical_fatigue = self.context.ecology.get_historical_fatigue(worker.id);
            // Penalty scales exponentially with current hours if historical fatigue is high
            let fatigue_cost = historical_fatigue * (hours as f64) * 2.0;
            fitness -= fatigue_cost;
            fatigue_penalty += fatigue_cost;
        }

        // SC1: Fairness (Variance of hours)
        if !hours_list.is_empty() {
            let mean = hours_list.iter().sum::<f64>() / hours_list.len() as f64;
            let variance = hours_list.iter().map(|h| (h - mean).powi(2)).sum::<f64>() / hours_list.len() as f64;
            
            // Penalty proportional to variance
            let fairness_cost = variance * 10.0;
            fitness -= fairness_cost;
            fairness_penalty += fairness_cost;
        }

        // Base reward for completing the schedule
        fitness += 10000.0;

        ScheduleEvaluation {
            schedule: genome.clone(),
            fitness,
            is_valid: true, // We allow invalid schedules to have poor fitness rather than dropping them
            hc1_violations,
            hc2_violations,
            hc3_violations,
            fairness_penalty,
            fatigue_penalty,
        }
    }
}
