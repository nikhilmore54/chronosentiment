// UltraCrew core optimizer improvements
use serde::{Serialize, Deserialize};
use crate::ecology::WorkforceEcology;
use crate::models::{Shift, Worker};
use coralys_moga::traits::{CrossoverOperator, Evaluated, FitnessEvaluator, Genome, GenomeFactory, MutationOperator};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
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
    pub rest_violations: usize,
    pub fairness_penalty: f64,
    pub fatigue_penalty: f64,
}

impl Evaluated for ScheduleEvaluation {
    type Genome = ScheduleGenome;
    fn fitness(&self) -> f64 { self.fitness }
    fn is_valid(&self) -> bool { self.is_valid }
    fn genome(&self) -> &Self::Genome { &self.schedule }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationTelemetry {
    pub generation: usize,
    pub best_fitness: f64,
    pub average_fitness: f64,
    pub hard_violations: usize,
    pub soft_violations: usize,
    pub fairness_penalty: f64,
    pub workload_penalty: f64,
    pub elapsed_time_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationReport {
    pub generations: Vec<GenerationTelemetry>,
}

#[derive(Debug, Clone)]
pub struct Observatory {
    pub start_time: std::time::Instant,
    pub current_generation_evals: Vec<ScheduleEvaluation>,
    pub reports: Vec<GenerationTelemetry>,
    pub current_generation_index: usize,
    pub population_size: usize,
}

impl Observatory {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            current_generation_evals: Vec::new(),
            reports: Vec::new(),
            current_generation_index: 0,
            population_size: 50,
        }
    }

    pub fn start_run(&mut self, population_size: usize) {
        self.start_time = std::time::Instant::now();
        self.current_generation_evals.clear();
        self.reports.clear();
        self.current_generation_index = 0;
        self.population_size = population_size;
    }

    pub fn record_evaluation(&mut self, eval: &ScheduleEvaluation) {
        self.current_generation_evals.push(eval.clone());
        if self.current_generation_evals.len() >= self.population_size {
            let gen = self.current_generation_index;
            self.current_generation_index += 1;

            let mut best_eval = &self.current_generation_evals[0];
            let mut sum_fitness = 0.0;
            for e in &self.current_generation_evals {
                sum_fitness += e.fitness;
                if e.fitness > best_eval.fitness {
                    best_eval = e;
                }
            }
            let avg_fitness = sum_fitness / self.current_generation_evals.len() as f64;
            let elapsed = self.start_time.elapsed().as_millis();

            self.reports.push(GenerationTelemetry {
                generation: gen,
                best_fitness: best_eval.fitness,
                average_fitness: avg_fitness,
                hard_violations: best_eval.hc1_violations + best_eval.hc2_violations + best_eval.hc3_violations + best_eval.rest_violations,
                soft_violations: if best_eval.fairness_penalty > 0.0 { 1 } else { 0 } + if best_eval.fatigue_penalty > 0.0 { 1 } else { 0 },
                fairness_penalty: best_eval.fairness_penalty,
                workload_penalty: best_eval.fatigue_penalty,
                elapsed_time_ms: elapsed,
            });

            self.current_generation_evals.clear();
        }
    }
}

pub struct ScheduleContext {
    pub workers: Arc<Vec<Worker>>,
    pub shifts: Arc<Vec<Shift>>,
    pub ecology: WorkforceEcology,
    // Deterministic seed for reproducibility (same seed yields same schedule)
    pub rng_seed: u64,
    pub observatory: Arc<std::sync::Mutex<Observatory>>,
    pub locked_assignments: Option<HashMap<u64, u64>>,
}

#[derive(Clone)]
pub struct ScheduleOptimizer {
    pub context: Arc<ScheduleContext>,
    // Internal deterministic RNG used for mutation/crossover
    deterministic_rng: StdRng,
}

impl ScheduleOptimizer {
    pub fn new(context: Arc<ScheduleContext>) -> Self {
        let deterministic_rng = StdRng::seed_from_u64(context.rng_seed);
        Self { context, deterministic_rng }
    }

    pub fn mutate_random_reassignment(&self, genome: &mut ScheduleGenome, mutable_shifts: &[&Shift], rng: &mut StdRng) {
        let shift = mutable_shifts[rng.gen_range(0..mutable_shifts.len())];
        // Use skill-aware pick so mutations don't reintroduce HC1 violations
        let worker_id = self.skill_aware_pick(shift, rng);
        genome.assignments.insert(shift.id, worker_id);
    }

    pub fn mutate_swap(&self, genome: &mut ScheduleGenome, mutable_shifts: &[&Shift], rng: &mut StdRng) {
        if mutable_shifts.len() < 2 {
            return;
        }
        let idx1 = rng.gen_range(0..mutable_shifts.len());
        let mut idx2 = rng.gen_range(0..mutable_shifts.len());
        while idx2 == idx1 {
            idx2 = rng.gen_range(0..mutable_shifts.len());
        }
        let shift1 = mutable_shifts[idx1];
        let shift2 = mutable_shifts[idx2];

        let w1 = genome.assignments.get(&shift1.id).copied().unwrap_or(self.context.workers[0].id);
        let w2 = genome.assignments.get(&shift2.id).copied().unwrap_or(self.context.workers[0].id);
        genome.assignments.insert(shift1.id, w2);
        genome.assignments.insert(shift2.id, w1);
    }
}

impl GenomeFactory<ScheduleGenome> for ScheduleOptimizer {
    fn create(&self, rng: &mut StdRng) -> ScheduleGenome {
        let mut assignments = HashMap::new();
        for shift in self.context.shifts.iter() {
            let worker_id = if let Some(ref locked) = self.context.locked_assignments {
                if let Some(&w_id) = locked.get(&shift.id) {
                    // Locked assignment — honour it regardless of skill
                    w_id
                } else {
                    self.skill_aware_pick(shift, rng)
                }
            } else {
                self.skill_aware_pick(shift, rng)
            };
            assignments.insert(shift.id, worker_id);
        }
        ScheduleGenome { assignments }
    }
}

impl ScheduleOptimizer {
    /// Pick a worker who possesses the required skill for this shift.
    /// Falls back to a random worker only if no qualified worker exists (prevents panic).
    fn skill_aware_pick(&self, shift: &crate::models::Shift, rng: &mut StdRng) -> u64 {
        let qualified: Vec<u64> = self.context.workers.iter()
            .filter(|w| w.skills.contains(&shift.required_skill))
            .map(|w| w.id)
            .collect();
        if qualified.is_empty() {
            // No qualified worker — fall back to random (HC1 violation will be penalised)
            self.context.workers[rng.gen_range(0..self.context.workers.len())].id
        } else {
            qualified[rng.gen_range(0..qualified.len())]
        }
    }
}

impl MutationOperator<ScheduleGenome> for ScheduleOptimizer {
    fn mutate(&self, genome: &mut ScheduleGenome, rng: &mut StdRng) {
        if self.context.shifts.is_empty() { return; }
        
        let mutable_shifts: Vec<&Shift> = self.context.shifts.iter()
            .filter(|s| {
                if let Some(ref locked) = self.context.locked_assignments {
                    !locked.contains_key(&s.id)
                } else {
                    true
                }
            })
            .collect();
        if mutable_shifts.is_empty() { return; }

        let strategies: &[fn(&ScheduleOptimizer, &mut ScheduleGenome, &[&Shift], &mut StdRng)] = &[
            ScheduleOptimizer::mutate_random_reassignment,
            ScheduleOptimizer::mutate_swap,
        ];

        let strategy_fn = strategies[rng.gen_range(0..strategies.len())];
        strategy_fn(self, genome, &mutable_shifts, rng);
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
        (ScheduleGenome { assignments: child1_assignments }, ScheduleGenome { assignments: child2_assignments })
    }
}

impl FitnessEvaluator<ScheduleGenome> for ScheduleOptimizer {
    type Evaluation = ScheduleEvaluation;

    fn evaluate(&self, genome: &ScheduleGenome) -> Self::Evaluation {
        let engine = crate::constraint_engine::ConstraintEngine::new(self.context.clone());
        let report = engine.evaluate(genome);

        let eval = ScheduleEvaluation {
            schedule: genome.clone(),
            fitness: report.fitness,
            is_valid: report.is_valid,
            hc1_violations: report.hc1_violations,
            hc2_violations: report.hc2_violations,
            hc3_violations: report.hc3_violations,
            rest_violations: report.rest_violations,
            fairness_penalty: report.fairness_penalty,
            fatigue_penalty: report.fatigue_penalty,
        };

        self.context.observatory.lock().unwrap().record_evaluation(&eval);

        eval
    }
}

/// Generates a human‑readable explanation for a schedule evaluation.
/// Returns a multiline string where each line describes a shift assignment and any relevant penalties.
pub fn generate_explanation(eval: &ScheduleEvaluation, context: &ScheduleContext) -> String {
    let mut lines = Vec::new();
    for shift in context.shifts.iter() {
        let worker_id = eval.schedule.assignments.get(&shift.id).unwrap();
        let worker = context.workers.iter().find(|w| w.id == *worker_id).unwrap();
        let skill_ok = if worker.skills.contains(&shift.required_skill) { "✔" } else { "✖" };
        lines.push(format!(
            "Shift {} assigned to Worker {} (Skill match: {}), Duration: {}h",
            shift.id, worker.id, skill_ok, shift.duration_hours
        ));
    }
    if eval.fatigue_penalty > 0.0 {
        lines.push(format!("Fatigue penalty: {:.2}", eval.fatigue_penalty));
    }
    if eval.fairness_penalty > 0.0 {
        lines.push(format!("Fairness penalty (variance): {:.2}", eval.fairness_penalty));
    }
    if eval.rest_violations > 0 {
        lines.push(format!("Rest violations: {}", eval.rest_violations));
    }
    lines.join("\n")
}
