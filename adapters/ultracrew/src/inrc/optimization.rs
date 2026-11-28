use coralys_moga::traits::{CrossoverOperator, Evaluated, Genome, GenomeFactory, MutationOperator};
use std::sync::Arc;

use super::inverted::InvertedContext;
use super::models::{InrcHistory, InrcScenario, InrcWeekData};
use crate::ecology::WorkforceEcology;

#[derive(Clone, Debug, Default)]
pub struct FastShiftDemand {
    pub total_min: usize,
    pub min_slots: Vec<u8>,
    pub opt_slots: Vec<u8>,
    pub opt_demands_by_skill: Vec<usize>,
}

pub struct InrcContext {
    pub scenario: Arc<InrcScenario>,
    pub week_data: Arc<InrcWeekData>,
    pub history: Arc<InrcHistory>,
    pub ecology: WorkforceEcology,
    pub inverted: Arc<InvertedContext>,
    pub rotation_pool: Arc<super::rotation_pool::RotationPool>,
    pub candidate_rotations: Vec<Vec<(super::rotation_pool::Rotation7, i32)>>,
    pub num_nurses: usize,
    pub num_days: usize,          // usually 7 for a week
    pub shift_types: Vec<String>, // ordered list of shift types
    pub skill_names: Vec<String>, // ordered list of skill names
    pub nurse_skill_masks: Vec<u8>, // skill bitmask per nurse
    pub fast_shift_demands: Vec<Vec<FastShiftDemand>>, // [d][s]
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
        let inverted = Arc::new(InvertedContext::new(&scenario, &week_data, &history));

        let skill_names: Vec<String> = scenario.skills.clone();
        let nurse_skill_masks: Vec<u8> = scenario
            .nurses
            .iter()
            .map(|n| {
                let mut mask = 0u8;
                for sk in &n.skills {
                    if let Some(pos) = skill_names.iter().position(|s| s == sk) {
                        mask |= 1 << pos;
                    }
                }
                mask
            })
            .collect();

        let num_shifts = shift_types.len();
        let mut fast_shift_demands: Vec<Vec<FastShiftDemand>> =
            vec![vec![FastShiftDemand::default(); num_shifts]; num_days];

        for d in 0..num_days {
            for s in 0..num_shifts {
                let s_name = &shift_types[s];
                let mut min_slots = Vec::new();
                let mut opt_slots = Vec::new();
                let mut opt_demands_by_skill = vec![0usize; skill_names.len()];

                for req in &week_data.requirements {
                    if req.shift_type == *s_name {
                        if let Some(skill_idx) = skill_names.iter().position(|sk| sk == &req.skill) {
                            let req_level = match d {
                                0 => &req.monday,
                                1 => &req.tuesday,
                                2 => &req.wednesday,
                                3 => &req.thursday,
                                4 => &req.friday,
                                5 => &req.saturday,
                                6 => &req.sunday,
                                _ => unreachable!(),
                            };

                            for _ in 0..req_level.minimum {
                                min_slots.push(skill_idx as u8);
                            }
                            let opt_extra = req_level.optimal.saturating_sub(req_level.minimum);
                            for _ in 0..opt_extra {
                                opt_slots.push(skill_idx as u8);
                            }
                            opt_demands_by_skill[skill_idx] = req_level.optimal;
                        }
                    }
                }

                fast_shift_demands[d][s] = FastShiftDemand {
                    total_min: min_slots.len(),
                    min_slots,
                    opt_slots,
                    opt_demands_by_skill,
                };
            }
        }

        let rotation_pool = Arc::new(super::rotation_pool::RotationPool::build(&scenario));
        let candidate_rotations = vec![Vec::new(); num_nurses];

        Self {
            scenario: Arc::new(scenario),
            week_data: Arc::new(week_data),
            history: Arc::new(history),
            ecology,
            inverted,
            rotation_pool,
            candidate_rotations,
            num_nurses,
            num_days,
            shift_types,
            skill_names,
            nurse_skill_masks,
            fast_shift_demands,
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
use rand::Rng;

impl GenomeFactory<InrcGenome> for InrcOptimizer {
    fn create(&self, rng: &mut StdRng) -> InrcGenome {
        let mut genome = self.context.inverted.construct_feasible_genome(rng);
        self.repair_hard_constraints(&mut genome);
        if rng.gen_bool(0.20) {
            self.optimize_rotations(&mut genome);
        }
        genome
    }
}

impl MutationOperator<InrcGenome> for InrcOptimizer {
    fn mutate(&self, genome: &mut InrcGenome, rng: &mut StdRng) {
        self.context.inverted.inverted_mutate(genome, rng);
        self.repair_hard_constraints(genome);
    }
}

impl CrossoverOperator<InrcGenome> for InrcOptimizer {
    fn crossover(
        &self,
        parent_a: &InrcGenome,
        parent_b: &InrcGenome,
        rng: &mut StdRng,
    ) -> (InrcGenome, InrcGenome) {
        let (mut child_a, mut child_b) =
            self.context.inverted.day_preserving_crossover(parent_a, parent_b, rng);
        self.repair_hard_constraints(&mut child_a);
        self.repair_hard_constraints(&mut child_b);
        (child_a, child_b)
    }
}
