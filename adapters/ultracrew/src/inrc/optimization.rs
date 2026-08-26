use coralys_moga::traits::{CrossoverOperator, Evaluated, Genome, GenomeFactory, MutationOperator};
use rand::Rng;
use std::sync::Arc;

use super::models::{InrcHistory, InrcScenario, InrcWeekData};
use crate::ecology::WorkforceEcology;

pub struct InrcContext {
    pub scenario: Arc<InrcScenario>,
    pub week_data: Arc<InrcWeekData>,
    pub history: Arc<InrcHistory>,
    pub ecology: WorkforceEcology,
    pub num_nurses: usize,
    pub num_days: usize,          // usually 7 for a week
    pub shift_types: Vec<String>, // ordered list of shift types
    pub weights: super::models::ObjectiveWeights,
}

impl InrcContext {
    pub fn new(
        scenario: InrcScenario,
        week_data: InrcWeekData,
        history: InrcHistory,
        ecology: WorkforceEcology,
    ) -> Self {
        let num_nurses = scenario.nurses.len();
        let num_days = 7;
        let shift_types: Vec<String> = scenario.shift_types.iter().map(|s| s.id.clone()).collect();
        let weights = super::models::ObjectiveWeights::default();

        Self {
            scenario: Arc::new(scenario),
            week_data: Arc::new(week_data),
            history: Arc::new(history),
            ecology,
            num_nurses,
            num_days,
            shift_types,
            weights,
        }
    }
}

// Genome representation:
// For each nurse, for each day, which shift type are they assigned to?
// 0 = Off, 1 = shift_types[0], 2 = shift_types[1], etc.
// We use our own InrcGenome

#[derive(Clone, Debug)]
pub struct InrcGenome {
    pub bits: Vec<bool>,
}

impl Genome for InrcGenome {}

#[derive(Clone, Debug, Default)]
pub struct SoftConstraintReport {
    pub assignment_penalty: i32,
    pub work_streak_penalty: i32,
    pub day_off_penalty: i32,
    pub weekend_penalty: i32,
    pub preferences_penalty: i32,
    pub optimal_coverage_penalty: i32,
    pub total_penalty: i32,
}

#[derive(Clone, Debug)]
pub struct InrcEvaluation {
    pub genome: InrcGenome,
    pub fitness: f64,
    pub hc_coverage: usize,
    pub hc_skills: usize,
    pub hc_one_shift_per_day: usize,
    pub hc_forbidden_successions: usize,
    pub soft_report: SoftConstraintReport,
    pub platform_result: coralys_core::EvaluationResult,
}

impl Evaluated for InrcEvaluation {
    type Genome = InrcGenome;

    fn fitness(&self) -> f64 {
        self.fitness
    }

    fn is_valid(&self) -> bool {
        // Return true so the GA evaluates it (we use penalty for hard constraints).
        true
    }

    fn genome(&self) -> &Self::Genome {
        &self.genome
    }
}

impl InrcEvaluation {
    pub fn is_feasible(&self) -> bool {
        self.hc_coverage == 0
            && self.hc_skills == 0
            && self.hc_one_shift_per_day == 0
            && self.hc_forbidden_successions == 0
    }
}

#[derive(Clone)]
pub struct InrcOptimizer {
    pub context: Arc<InrcContext>,
}

use rand::rngs::StdRng;

impl GenomeFactory<InrcGenome> for InrcOptimizer {
    fn create(&self, rng: &mut StdRng) -> InrcGenome {
        let size = self.context.num_nurses * self.context.num_days * self.context.shift_types.len();
        let mut bits = vec![false; size];

        // Initialize somewhat sparsely, around the expected density (150-200 shifts / 840 total)
        for i in 0..size {
            if rng.gen_bool(0.22) {
                bits[i] = true;
            }
        }

        InrcGenome { bits }
    }
}

impl MutationOperator<InrcGenome> for InrcOptimizer {
    fn mutate(&self, genome: &mut InrcGenome, rng: &mut StdRng) {
        let rate = 1.0 / (genome.bits.len() as f64).max(1.0);
        for i in 0..genome.bits.len() {
            if rng.gen_bool(rate) {
                genome.bits[i] = !genome.bits[i];
            }
        }
    }
}

impl CrossoverOperator<InrcGenome> for InrcOptimizer {
    fn crossover(
        &self,
        parent_a: &InrcGenome,
        parent_b: &InrcGenome,
        rng: &mut StdRng,
    ) -> (InrcGenome, InrcGenome) {
        let mut child_a_bits = Vec::with_capacity(parent_a.bits.len());
        let mut child_b_bits = Vec::with_capacity(parent_a.bits.len());

        for i in 0..parent_a.bits.len() {
            if rng.gen_bool(0.5) {
                child_a_bits.push(parent_a.bits[i]);
                child_b_bits.push(parent_b.bits[i]);
            } else {
                child_a_bits.push(parent_b.bits[i]);
                child_b_bits.push(parent_a.bits[i]);
            }
        }
        (
            InrcGenome { bits: child_a_bits },
            InrcGenome { bits: child_b_bits },
        )
    }
}
