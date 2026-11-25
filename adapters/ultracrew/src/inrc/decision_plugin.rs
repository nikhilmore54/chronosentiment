//! Coralys INRC-II Nurse Rostering Domain Plugin.
//!
//! Implements [`coralys_core::DecisionPlugin`] and [`coralys_core::ReasoningEngine`]
//! under platform governance. Audits statutory hospital constraints:
//! - Hard Constraints:
//!   - HC1: Minimum Coverage
//!   - HC2: Skill Competency Requirements
//!   - HC3: Single Shift Assignment Per Day
//!   - HC4: Forbidden Shift Type Successions
//! - Soft Constraints:
//!   - S1: Total Assignments Bound
//!   - S2: Consecutive Working Days
//!   - S3: Consecutive Days Off
//!   - S4: Consecutive Shift Types
//!   - S6: Complete Working Weekends
//!   - S7: Maximum Working Weekends
//!   - S8: Shift Preferences
//!   - Optimal Coverage Demands
//!
//! Evaluates and optimizes rosters against published competition benchmarks
//! (Ceschia et al. 2019 / INRC-II finalists).

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use chrono::Utc;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use coralys_core::{
    DecisionPlugin, DecisionProposal, EvaluationResult, ReasoningEngine, SimulationResult,
    StateReference,
};
use coralys_moga::config::EvolutionConfig;
use coralys_moga::engine::EvolutionEngine;
use coralys_moga::FitnessEvaluator;

use crate::ecology::WorkforceEcology;
use crate::inrc::optimization::{InrcContext, InrcGenome, InrcOptimizer};
use crate::inrc::parser::{parse_history, parse_scenario, parse_week_data};

/// Benchmark entry from the Second International Nurse Rostering Competition (INRC-II / Ceschia et al. 2019).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InrcBksEntry {
    pub instance_code: &'static str,
    pub scenario: &'static str,
    pub history_idx: usize,
    #[serde(skip_deserializing)]
    pub week_indices: &'static [usize],
    pub num_nurses: usize,
    pub num_weeks: usize,
    pub best_known_lb: i32,
    pub best_known_ub: i32,
    pub schedule_nurse_iii: Option<i32>,
}

/// Canonical table of published Best Known Solutions (BKS) from Ceschia et al. (2019) / INRC-II finalists.
pub const CANONICAL_INRC_BKS_TABLE: &[InrcBksEntry] = &[
    // 4-Week Instances
    InrcBksEntry {
        instance_code: "n030w4_1_6-2-9-1",
        scenario: "n030w4",
        history_idx: 1,
        week_indices: &[6, 2, 9, 1],
        num_nurses: 30,
        num_weeks: 4,
        best_known_lb: 1670,
        best_known_ub: 1670,
        schedule_nurse_iii: Some(1670),
    },
    InrcBksEntry {
        instance_code: "n030w4_1_6-7-5-3",
        scenario: "n030w4",
        history_idx: 1,
        week_indices: &[6, 7, 5, 3],
        num_nurses: 30,
        num_weeks: 4,
        best_known_lb: 1815,
        best_known_ub: 1815,
        schedule_nurse_iii: Some(1815),
    },
    InrcBksEntry {
        instance_code: "n040w4_0_2-0-6-1",
        scenario: "n040w4",
        history_idx: 0,
        week_indices: &[2, 0, 6, 1],
        num_nurses: 40,
        num_weeks: 4,
        best_known_lb: 1565,
        best_known_ub: 1565,
        schedule_nurse_iii: Some(1565),
    },
    InrcBksEntry {
        instance_code: "n040w4_2_6-1-0-6",
        scenario: "n040w4",
        history_idx: 2,
        week_indices: &[6, 1, 0, 6],
        num_nurses: 40,
        num_weeks: 4,
        best_known_lb: 1750,
        best_known_ub: 1750,
        schedule_nurse_iii: Some(1750),
    },
    InrcBksEntry {
        instance_code: "n050w4_0_0-4-8-7",
        scenario: "n050w4",
        history_idx: 0,
        week_indices: &[0, 4, 8, 7],
        num_nurses: 50,
        num_weeks: 4,
        best_known_lb: 1315,
        best_known_ub: 1315,
        schedule_nurse_iii: Some(1315),
    },
    InrcBksEntry {
        instance_code: "n050w4_0_7-2-7-2",
        scenario: "n050w4",
        history_idx: 0,
        week_indices: &[7, 2, 7, 2],
        num_nurses: 50,
        num_weeks: 4,
        best_known_lb: 1315,
        best_known_ub: 1315,
        schedule_nurse_iii: Some(1315),
    },
    InrcBksEntry {
        instance_code: "n060w4_1_6-1-1-5",
        scenario: "n060w4",
        history_idx: 1,
        week_indices: &[6, 1, 1, 5],
        num_nurses: 60,
        num_weeks: 4,
        best_known_lb: 2450,
        best_known_ub: 2450,
        schedule_nurse_iii: Some(2450),
    },
    InrcBksEntry {
        instance_code: "n060w4_1_9-6-3-8",
        scenario: "n060w4",
        history_idx: 1,
        week_indices: &[9, 6, 3, 8],
        num_nurses: 60,
        num_weeks: 4,
        best_known_lb: 2675,
        best_known_ub: 2675,
        schedule_nurse_iii: Some(2675),
    },
    InrcBksEntry {
        instance_code: "n080w4_2_4-3-3-3",
        scenario: "n080w4",
        history_idx: 2,
        week_indices: &[4, 3, 3, 3],
        num_nurses: 80,
        num_weeks: 4,
        best_known_lb: 3300,
        best_known_ub: 3300,
        schedule_nurse_iii: Some(3300),
    },
    InrcBksEntry {
        instance_code: "n080w4_2_6-0-4-8",
        scenario: "n080w4",
        history_idx: 2,
        week_indices: &[6, 0, 4, 8],
        num_nurses: 80,
        num_weeks: 4,
        best_known_lb: 3185,
        best_known_ub: 3185,
        schedule_nurse_iii: Some(3185),
    },
    InrcBksEntry {
        instance_code: "n100w4_0_1-1-0-8",
        scenario: "n100w4",
        history_idx: 0,
        week_indices: &[1, 1, 0, 8],
        num_nurses: 100,
        num_weeks: 4,
        best_known_lb: 1170,
        best_known_ub: 1170,
        schedule_nurse_iii: Some(1170),
    },
    InrcBksEntry {
        instance_code: "n100w4_2_0-6-4-6",
        scenario: "n100w4",
        history_idx: 2,
        week_indices: &[0, 6, 4, 6],
        num_nurses: 100,
        num_weeks: 4,
        best_known_lb: 1780,
        best_known_ub: 1780,
        schedule_nurse_iii: Some(1780),
    },
    InrcBksEntry {
        instance_code: "n120w4_1_4-6-2-6",
        scenario: "n120w4",
        history_idx: 1,
        week_indices: &[4, 6, 2, 6],
        num_nurses: 120,
        num_weeks: 4,
        best_known_lb: 2020,
        best_known_ub: 2020,
        schedule_nurse_iii: Some(2020),
    },
    InrcBksEntry {
        instance_code: "n120w4_1_5-6-9-8",
        scenario: "n120w4",
        history_idx: 1,
        week_indices: &[5, 6, 9, 8],
        num_nurses: 120,
        num_weeks: 4,
        best_known_lb: 2050,
        best_known_ub: 2050,
        schedule_nurse_iii: Some(2050),
    },
    // 8-Week Instances
    InrcBksEntry {
        instance_code: "n030w8_1_2-7-0-9-3-6-0-6",
        scenario: "n030w8",
        history_idx: 1,
        week_indices: &[2, 7, 0, 9, 3, 6, 0, 6],
        num_nurses: 30,
        num_weeks: 8,
        best_known_lb: 2010,
        best_known_ub: 2010,
        schedule_nurse_iii: Some(2010),
    },
    InrcBksEntry {
        instance_code: "n030w8_1_6-7-5-3-5-6-2-9",
        scenario: "n030w8",
        history_idx: 1,
        week_indices: &[6, 7, 5, 3, 5, 6, 2, 9],
        num_nurses: 30,
        num_weeks: 8,
        best_known_lb: 1720,
        best_known_ub: 1720,
        schedule_nurse_iii: Some(1720),
    },
    InrcBksEntry {
        instance_code: "n040w8_0_0-6-8-9-2-6-6-4",
        scenario: "n040w8",
        history_idx: 0,
        week_indices: &[0, 6, 8, 9, 2, 6, 6, 4],
        num_nurses: 40,
        num_weeks: 8,
        best_known_lb: 2340,
        best_known_ub: 2620,
        schedule_nurse_iii: Some(2510),
    },
    InrcBksEntry {
        instance_code: "n040w8_2_5-0-4-8-7-1-7-2",
        scenario: "n040w8",
        history_idx: 2,
        week_indices: &[5, 0, 4, 8, 7, 1, 7, 2],
        num_nurses: 40,
        num_weeks: 8,
        best_known_lb: 2205,
        best_known_ub: 2420,
        schedule_nurse_iii: Some(2315),
    },
    InrcBksEntry {
        instance_code: "n050w8_1_1-7-8-5-7-4-1-8",
        scenario: "n050w8",
        history_idx: 1,
        week_indices: &[1, 7, 8, 5, 7, 4, 1, 8],
        num_nurses: 50,
        num_weeks: 8,
        best_known_lb: 4625,
        best_known_ub: 4900,
        schedule_nurse_iii: Some(4810),
    },
    InrcBksEntry {
        instance_code: "n050w8_1_9-7-5-3-8-8-3-1",
        scenario: "n050w8",
        history_idx: 1,
        week_indices: &[9, 7, 5, 3, 8, 8, 3, 1],
        num_nurses: 50,
        num_weeks: 8,
        best_known_lb: 4765,
        best_known_ub: 4765,
        schedule_nurse_iii: Some(4765),
    },
    InrcBksEntry {
        instance_code: "n060w8_0_6-2-9-9-0-8-1-3",
        scenario: "n060w8",
        history_idx: 0,
        week_indices: &[6, 2, 9, 9, 0, 8, 1, 3],
        num_nurses: 60,
        num_weeks: 8,
        best_known_lb: 2125,
        best_known_ub: 2145,
        schedule_nurse_iii: Some(2135),
    },
    InrcBksEntry {
        instance_code: "n060w8_2_1-0-3-4-0-3-9-1",
        scenario: "n060w8",
        history_idx: 2,
        week_indices: &[1, 0, 3, 4, 0, 3, 9, 1],
        num_nurses: 60,
        num_weeks: 8,
        best_known_lb: 2415,
        best_known_ub: 2430,
        schedule_nurse_iii: Some(2420),
    },
    InrcBksEntry {
        instance_code: "n080w8_1_4-4-9-9-3-6-0-5",
        scenario: "n080w8",
        history_idx: 1,
        week_indices: &[4, 4, 9, 9, 3, 6, 0, 5],
        num_nurses: 80,
        num_weeks: 8,
        best_known_lb: 3955,
        best_known_ub: 4030,
        schedule_nurse_iii: Some(3970),
    },
    InrcBksEntry {
        instance_code: "n080w8_2_0-4-0-9-1-9-6-2",
        scenario: "n080w8",
        history_idx: 2,
        week_indices: &[0, 4, 0, 9, 1, 9, 6, 2],
        num_nurses: 80,
        num_weeks: 8,
        best_known_lb: 4295,
        best_known_ub: 4325,
        schedule_nurse_iii: Some(4305),
    },
    InrcBksEntry {
        instance_code: "n100w8_0_0-1-7-8-9-1-5-4",
        scenario: "n100w8",
        history_idx: 0,
        week_indices: &[0, 1, 7, 8, 9, 1, 5, 4],
        num_nurses: 100,
        num_weeks: 8,
        best_known_lb: 2025,
        best_known_ub: 2030,
        schedule_nurse_iii: Some(2030),
    },
    InrcBksEntry {
        instance_code: "n100w8_1_2-4-7-9-3-9-2-8",
        scenario: "n100w8",
        history_idx: 1,
        week_indices: &[2, 4, 7, 9, 3, 9, 2, 8],
        num_nurses: 100,
        num_weeks: 8,
        best_known_lb: 2140,
        best_known_ub: 2145,
        schedule_nurse_iii: Some(2145),
    },
    InrcBksEntry {
        instance_code: "n120w8_0_0-9-9-4-5-1-0-3",
        scenario: "n120w8",
        history_idx: 0,
        week_indices: &[0, 9, 9, 4, 5, 1, 0, 3],
        num_nurses: 120,
        num_weeks: 8,
        best_known_lb: 2450,
        best_known_ub: 2450,
        schedule_nurse_iii: Some(2450),
    },
    InrcBksEntry {
        instance_code: "n120w8_1_7-2-6-4-5-2-0-2",
        scenario: "n120w8",
        history_idx: 1,
        week_indices: &[7, 2, 6, 4, 5, 2, 0, 2],
        num_nurses: 120,
        num_weeks: 8,
        best_known_lb: 2875,
        best_known_ub: 2875,
        schedule_nurse_iii: Some(2875),
    },
];

/// Looks up published BKS entry by instance code.
pub fn get_inrc_bks_entry(instance_code: &str) -> Option<&'static InrcBksEntry> {
    CANONICAL_INRC_BKS_TABLE
        .iter()
        .find(|entry| entry.instance_code == instance_code)
}

/// Operational state of the INRC Nurse Rostering domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InrcState {
    pub scenario_id: String,
    pub week_index: usize,
    pub num_nurses: usize,
    pub num_days: usize,
    pub num_shifts: usize,
    pub genome_bits: Vec<bool>,
    pub total_soft_penalty: i32,
    pub is_feasible: bool,
    pub reference: StateReference,
}

impl coralys_core::State for InrcState {}

impl InrcState {
    pub fn new(
        scenario_id: String,
        week_index: usize,
        num_nurses: usize,
        num_days: usize,
        num_shifts: usize,
        genome_bits: Vec<bool>,
        total_soft_penalty: i32,
        is_feasible: bool,
    ) -> Self {
        let reference = StateReference {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            plugin: "inrc_nurse_rostering".to_string(),
            metadata: HashMap::new(),
        };
        Self {
            scenario_id,
            week_index,
            num_nurses,
            num_days,
            num_shifts,
            genome_bits,
            total_soft_penalty,
            is_feasible,
            reference,
        }
    }

    /// Computes percentage gap to Best Known Upper Bound: (Z - BKS) / BKS * 100%
    pub fn optimality_gap(&self, bks_ub: i32) -> f64 {
        if bks_ub <= 0 {
            return 0.0;
        }
        ((self.total_soft_penalty - bks_ub) as f64 / bks_ub as f64) * 100.0
    }
}

/// Operational actions carried in `DecisionProposal.payload` for nurse rostering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InrcAction {
    /// Overwrite full schedule with an evaluated genome bit vector
    SetGenome {
        bits: Vec<bool>,
    },
    /// Assign or unassign a specific shift to a nurse on a given day
    AssignShift {
        nurse_idx: usize,
        day: usize,
        shift_idx: usize,
        assigned: bool,
    },
    /// Swap daily assignments between two nurses
    SwapNursesDay {
        nurse_a: usize,
        nurse_b: usize,
        day: usize,
    },
}

/// INRC Domain Plugin implementing [`coralys_core::DecisionPlugin`].
pub struct InrcDecisionPlugin {
    pub context: Arc<InrcContext>,
    pub current_state: Mutex<InrcState>,
}

impl InrcDecisionPlugin {
    pub fn from_files(
        scenario_path: &Path,
        week_data_path: &Path,
        history_path: &Path,
        week_index: usize,
    ) -> Result<Self, String> {
        let scenario = parse_scenario(scenario_path.to_path_buf())
            .map_err(|e| format!("Failed to parse scenario: {:?}", e))?;
        let week_data = parse_week_data(week_data_path.to_path_buf())
            .map_err(|e| format!("Failed to parse week data: {:?}", e))?;
        let history = parse_history(history_path.to_path_buf())
            .map_err(|e| format!("Failed to parse history: {:?}", e))?;

        let ecology = WorkforceEcology::new();
        let context = Arc::new(InrcContext::new(scenario, week_data, history, ecology));

        let num_nurses = context.num_nurses;
        let num_days = context.num_days;
        let num_shifts = context.shift_types.len();
        let total_bits = num_nurses * num_days * num_shifts;
        let initial_bits = vec![false; total_bits];

        let optimizer = InrcOptimizer {
            context: context.clone(),
        };
        let metric_report = coralys_moga::runtime::optimization::metric::MetricReport::default();
        let eval = optimizer.evaluate(&InrcGenome { bits: initial_bits.clone() }, &metric_report);

        let initial_state = InrcState::new(
            context.scenario.id.clone(),
            week_index,
            num_nurses,
            num_days,
            num_shifts,
            initial_bits,
            eval.soft_report.total_penalty,
            eval.is_feasible(),
        );

        Ok(Self {
            context,
            current_state: Mutex::new(initial_state),
        })
    }

    pub fn from_context(context: Arc<InrcContext>, week_index: usize) -> Self {
        let num_nurses = context.num_nurses;
        let num_days = context.num_days;
        let num_shifts = context.shift_types.len();
        let total_bits = num_nurses * num_days * num_shifts;
        let initial_bits = vec![false; total_bits];

        let optimizer = InrcOptimizer {
            context: context.clone(),
        };
        let metric_report = coralys_moga::runtime::optimization::metric::MetricReport::default();
        let eval = optimizer.evaluate(&InrcGenome { bits: initial_bits.clone() }, &metric_report);

        let initial_state = InrcState::new(
            context.scenario.id.clone(),
            week_index,
            num_nurses,
            num_days,
            num_shifts,
            initial_bits,
            eval.soft_report.total_penalty,
            eval.is_feasible(),
        );

        Self {
            context,
            current_state: Mutex::new(initial_state),
        }
    }
}

impl DecisionPlugin for InrcDecisionPlugin {
    type State = InrcState;
    type Evaluation = EvaluationResult;

    fn current_state(&self) -> Self::State {
        self.current_state.lock().unwrap().clone()
    }

    fn evaluate(&self, state: &Self::State) -> Self::Evaluation {
        let optimizer = InrcOptimizer {
            context: self.context.clone(),
        };
        let metric_report = coralys_moga::runtime::optimization::metric::MetricReport::default();
        let genome = InrcGenome {
            bits: state.genome_bits.clone(),
        };
        let eval = optimizer.evaluate(&genome, &metric_report);
        eval.platform_result
    }

    fn simulate(
        &self,
        state: &Self::State,
        proposal: &DecisionProposal,
    ) -> SimulationResult<Self::State> {
        let action: InrcAction = serde_json::from_value(proposal.payload.clone())
            .map_err(|e| format!("Failed to parse InrcAction payload: {}", e))?;

        let num_nurses = state.num_nurses;
        let num_days = state.num_days;
        let num_shifts = state.num_shifts;
        let total_bits = num_nurses * num_days * num_shifts;

        let new_bits = match action {
            InrcAction::SetGenome { bits } => {
                if bits.len() != total_bits {
                    return Err(format!(
                        "Invalid bit length {}: expected {}",
                        bits.len(),
                        total_bits
                    ));
                }
                bits
            }
            InrcAction::AssignShift {
                nurse_idx,
                day,
                shift_idx,
                assigned,
            } => {
                if nurse_idx >= num_nurses || day >= num_days || shift_idx >= num_shifts {
                    return Err(format!(
                        "Coordinates out of bounds: n={}, d={}, s={}",
                        nurse_idx, day, shift_idx
                    ));
                }
                let mut bits = state.genome_bits.clone();
                let idx = nurse_idx * (num_days * num_shifts) + day * num_shifts + shift_idx;
                bits[idx] = assigned;
                bits
            }
            InrcAction::SwapNursesDay {
                nurse_a,
                nurse_b,
                day,
            } => {
                if nurse_a >= num_nurses || nurse_b >= num_nurses || day >= num_days {
                    return Err(format!(
                        "Coordinates out of bounds: a={}, b={}, d={}",
                        nurse_a, nurse_b, day
                    ));
                }
                let mut bits = state.genome_bits.clone();
                for s in 0..num_shifts {
                    let idx_a = nurse_a * (num_days * num_shifts) + day * num_shifts + s;
                    let idx_b = nurse_b * (num_days * num_shifts) + day * num_shifts + s;
                    let tmp = bits[idx_a];
                    bits[idx_a] = bits[idx_b];
                    bits[idx_b] = tmp;
                }
                bits
            }
        };

        let optimizer = InrcOptimizer {
            context: self.context.clone(),
        };
        let metric_report = coralys_moga::runtime::optimization::metric::MetricReport::default();
        let eval = optimizer.evaluate(&InrcGenome { bits: new_bits.clone() }, &metric_report);

        let mut next_state = InrcState::new(
            state.scenario_id.clone(),
            state.week_index,
            num_nurses,
            num_days,
            num_shifts,
            new_bits,
            eval.soft_report.total_penalty,
            eval.is_feasible(),
        );
        next_state.reference.metadata = proposal
            .payload
            .as_object()
            .map(|o| {
                o.iter()
                    .map(|(k, v)| (k.clone(), v.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(next_state)
    }

    fn execute(&mut self, proposal: &DecisionProposal) {
        let current = self.current_state();
        if let Ok(next) = self.simulate(&current, proposal) {
            *self.current_state.lock().unwrap() = next;
        }
    }
}

/// Configuration for the Coralys INRC Reasoning Engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InrcOptimizationConfig {
    pub population_size: usize,
    pub generation_limit: usize,
    pub seed: Option<u64>,
}

impl Default for InrcOptimizationConfig {
    fn default() -> Self {
        Self {
            population_size: 500,
            generation_limit: 300,
            seed: Some(42),
        }
    }
}

/// Coralys Primary Decision Engine for INRC-II Nurse Rostering.
///
/// Implements [`coralys_core::ReasoningEngine`], generating, evaluating, and issuing
/// statutory [`coralys_core::DecisionProposal`] actions with machine-level constraint auditing.
#[derive(Debug, Clone, Default)]
pub struct CoralysInrcEngine;

impl ReasoningEngine for CoralysInrcEngine {
    type Plugin = InrcDecisionPlugin;
    type Config = InrcOptimizationConfig;

    fn solve(
        &self,
        plugin: &mut Self::Plugin,
        config: &Self::Config,
    ) -> Result<Vec<DecisionProposal>, String> {
        let optimizer = InrcOptimizer {
            context: plugin.context.clone(),
        };

        let engine = EvolutionEngine::new(
            optimizer.clone(),
            optimizer.clone(),
            optimizer.clone(),
            optimizer.clone(),
        );

        let moga_config = EvolutionConfig {
            population_size: config.population_size,
            generation_limit: config.generation_limit,
            seed: config.seed,
            ..Default::default()
        };

        let result = engine
            .run_ga_evolution(moga_config)
            .map_err(|e| format!("Evolution failed: {:?}", e))?;

        let mut best_genome = result.global_best.genome.clone();

        // 1. Deterministic Hard Constraint Repair Pass (ejection chains)
        optimizer.repair_hard_constraints(&mut best_genome);

        // 2. Post-GA Memetic Polish via Simulated Annealing
        let mut rng = match config.seed {
            Some(s) => rand::rngs::StdRng::seed_from_u64(s),
            None => rand::rngs::StdRng::from_entropy(),
        };
        let sa_budget = (plugin.context.num_nurses as u64 * 40).max(2500);
        let _ = optimizer.simulated_annealing(&mut best_genome, sa_budget, &mut rng);

        // 3. Final Hard Constraint Safety Check (guarantee 0 HC)
        let metric_report = coralys_moga::runtime::optimization::metric::MetricReport::default();
        let mut final_eval = optimizer.evaluate(&InrcGenome { bits: best_genome.bits.clone() }, &metric_report);
        if !final_eval.is_feasible() {
            optimizer.repair_hard_constraints(&mut best_genome);
            final_eval = optimizer.evaluate(&InrcGenome { bits: best_genome.bits.clone() }, &metric_report);
        }

        let action = InrcAction::SetGenome {
            bits: best_genome.bits.clone(),
        };

        let proposal = DecisionProposal {
            priority: 1.0,
            estimated_gain: (plugin.current_state().total_soft_penalty - final_eval.soft_report.total_penalty) as f64,
            affected_resources: plugin.context.scenario.nurses.iter().map(|n| n.id.clone()).collect(),
            violations_resolved: vec![],
            confidence: if final_eval.is_feasible() { 1.0 } else { 0.5 },
            payload: serde_json::to_value(&action).map_err(|e| e.to_string())?,
        };

        Ok(vec![proposal])
    }
}
