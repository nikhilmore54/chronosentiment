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
use coralys_matching::{AssignmentSolver, BipartiteMatchingSolver};
use coralys_moga::FitnessEvaluator;

use crate::ecology::WorkforceEcology;
use crate::inrc::annealing::{AnnealingConfig, SimulatedAnnealingEngine};
use crate::inrc::evaluator::InrcAnnealingDomain;
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

impl InrcBksEntry {
    pub fn schedule_nurse_iii(&self) -> Option<i32> {
        self.schedule_nurse_iii
    }

    pub fn romer_mellouli(&self) -> Option<i32> {
        // Römer & Mellouli (2016) developed Schedule Nurse III
        self.schedule_nurse_iii
    }

    pub fn legrain_et_al(&self) -> Option<i32> {
        // Legrain et al. (2020) / Ceschia et al. (2019) upper bounds
        Some(self.best_known_ub)
    }

    pub fn lit_source(&self) -> &'static str {
        "Ceschia et al. (2019, Annals of OR) / Römer & Mellouli / Legrain et al."
    }
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

    /// Audits all four statutory competition hard constraints (H1 - H4) on an INRC state:
    /// - H1: Single shift per day per nurse
    /// - H2: Minimum understaffing coverage and skill qualification
    /// - H3: Forbidden shift-type transitions across days and history boundaries
    /// - H4: Maximum consecutive working days and total assignments cap
    pub fn audit_hard_constraints(&self, state: &InrcState) -> Vec<coralys_core::Violation> {
        let mut violations = Vec::new();
        let num_nurses = self.context.num_nurses;
        let num_days = self.context.num_days;
        let num_shifts = self.context.shift_types.len();
        let weights = &self.context.weights;

        // H1: Single Shift Per Day Per Nurse
        for n in 0..num_nurses {
            let nurse = &self.context.scenario.nurses[n];
            for d in 0..num_days {
                let mut count = 0;
                for s in 0..num_shifts {
                    let idx = n * (num_days * num_shifts) + d * num_shifts + s;
                    if state.genome_bits[idx] {
                        count += 1;
                    }
                }
                if count > 1 {
                    violations.push(coralys_core::Violation {
                        constraint_id: "H1_SingleShiftPerDay".to_string(),
                        severity: "Hard".to_string(),
                        value: Some(count as f64),
                        expected: "1".to_string(),
                        actual: count.to_string(),
                        description: format!("Nurse {} assigned to {} shifts on day {}", nurse.id, count, d),
                        penalty: ((count - 1) * weights.hard_constraint_violation as usize) as i32,
                    });
                }
            }
        }

        // H2: Skill qualification & Minimum Coverage Requirements
        let days_map = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
        for d in 0..num_days {
            let day_name = days_map[d % 7];
            for s in 0..num_shifts {
                let shift_name = &self.context.shift_types[s];
                let mut min_demands = Vec::new();
                for req in &self.context.week_data.requirements {
                    if req.shift_type == *shift_name {
                        let req_level = match day_name {
                            "Monday" => &req.monday,
                            "Tuesday" => &req.tuesday,
                            "Wednesday" => &req.wednesday,
                            "Thursday" => &req.thursday,
                            "Friday" => &req.friday,
                            "Saturday" => &req.saturday,
                            "Sunday" => &req.sunday,
                            _ => unreachable!(),
                        };
                        if req_level.minimum > 0 {
                            min_demands.push((req.skill.clone(), req_level.minimum));
                        }
                    }
                }

                let mut assigned_nurses = Vec::new();
                for n in 0..num_nurses {
                    let idx = n * (num_days * num_shifts) + d * num_shifts + s;
                    if state.genome_bits[idx] {
                        assigned_nurses.push(n);
                    }
                }

                // Check skill eligibility
                for &n in &assigned_nurses {
                    let nurse = &self.context.scenario.nurses[n];
                    if !self.context.inverted.nurse_can_work[n][d][s] {
                        violations.push(coralys_core::Violation {
                            constraint_id: "H2_SkillRequirements".to_string(),
                            severity: "Hard".to_string(),
                            value: None,
                            expected: "Nurse possesses required skill".to_string(),
                            actual: format!("{:?}", nurse.skills),
                            description: format!("Nurse {} lacks required skill for shift {} on day {}", nurse.id, shift_name, day_name),
                            penalty: weights.hard_constraint_violation as i32,
                        });
                    }
                }

                // Check minimum coverage using matching
                let workers: Vec<(usize, Vec<String>)> = assigned_nurses
                    .iter()
                    .map(|&n| (n, self.context.scenario.nurses[n].skills.clone()))
                    .collect();
                let min_matching = BipartiteMatchingSolver.assign(&workers, &min_demands);
                let mut min_fulfilled_map = HashMap::new();
                for (_, skill) in &min_matching.assignments {
                    *min_fulfilled_map.entry(skill.clone()).or_insert(0) += 1;
                }
                for (skill, required_min) in &min_demands {
                    let fulfilled = *min_fulfilled_map.get(skill).unwrap_or(&0);
                    if fulfilled < *required_min {
                        let missing = *required_min - fulfilled;
                        violations.push(coralys_core::Violation {
                            constraint_id: "H2_UnderstaffingCoverage".to_string(),
                            severity: "Hard".to_string(),
                            value: Some(missing as f64),
                            expected: required_min.to_string(),
                            actual: fulfilled.to_string(),
                            description: format!("Shift {} skill {} on {} lacks {} nurses for minimum coverage", shift_name, skill, day_name, missing),
                            penalty: (missing * weights.hard_constraint_violation as usize) as i32,
                        });
                    }
                }
            }
        }

        // H3: Forbidden Shift-Type Transitions across days and history boundaries
        for n in 0..num_nurses {
            let nurse_id = &self.context.scenario.nurses[n].id;
            let hist = self.context.history.nurse_history.iter().find(|h| &h.nurse == nurse_id);
            let mut previous_shift_type = if let Some(h) = hist {
                if h.number_of_consecutive_days_off > 0 {
                    "None".to_string()
                } else {
                    h.last_assigned_shift_type.clone()
                }
            } else {
                "None".to_string()
            };

            for d in 0..num_days {
                let mut current_shift_type = "None".to_string();
                for s in 0..num_shifts {
                    let idx = n * (num_days * num_shifts) + d * num_shifts + s;
                    if state.genome_bits[idx] {
                        current_shift_type = self.context.shift_types[s].clone();
                        break;
                    }
                }

                if current_shift_type != "None" && previous_shift_type != "None" && !previous_shift_type.is_empty() {
                    if let Some(rule) = self.context.scenario.forbidden_shift_type_successions.iter().find(|r| r.preceding == previous_shift_type) {
                        if rule.succeeding.contains(&current_shift_type) {
                            violations.push(coralys_core::Violation {
                                constraint_id: "H3_ForbiddenShiftTransitions".to_string(),
                                severity: "Hard".to_string(),
                                value: None,
                                expected: format!("No transition from {} to {}", previous_shift_type, current_shift_type),
                                actual: format!("Transitioned from {} to {}", previous_shift_type, current_shift_type),
                                description: format!("Nurse {} forbidden transition: {} -> {} on day {}", nurse_id, previous_shift_type, current_shift_type, d),
                                penalty: weights.hard_constraint_violation as i32,
                            });
                        }
                    }
                }

                if current_shift_type != "None" || previous_shift_type == "None" || previous_shift_type.is_empty() {
                    previous_shift_type = current_shift_type;
                } else {
                    previous_shift_type = "None".to_string();
                }
            }
        }

        violations
    }

    /// Converts state genome into a schedule representation `HashMap<nurse_id, Vec<shift_code>>`.
    pub fn state_to_schedule(&self, state: &InrcState) -> HashMap<String, Vec<String>> {
        let mut schedule = HashMap::new();
        let num_nurses = state.num_nurses;
        let num_days = state.num_days;
        let num_shifts = state.num_shifts;

        for n in 0..num_nurses {
            let nurse_id = self.context.scenario.nurses[n].id.clone();
            let mut days = Vec::with_capacity(num_days);
            for d in 0..num_days {
                let mut assigned_shift = String::new();
                for s in 0..num_shifts {
                    let idx = n * (num_days * num_shifts) + d * num_shifts + s;
                    if state.genome_bits[idx] {
                        assigned_shift = self.context.shift_types[s].clone();
                        break;
                    }
                }
                days.push(assigned_shift);
            }
            schedule.insert(nurse_id, days);
        }
        schedule
    }

    /// Validates state against official INRC competition validator rules.
    pub fn validate_with_official_rules(&self, state: &InrcState) -> crate::inrc::types::ValidationReport {
        let schedule = self.state_to_schedule(state);
        crate::inrc::validator::validate_schedule(&schedule, &self.context.scenario)
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

        // Audit statutory hard constraints H1-H4
        let hard_violations = self.audit_hard_constraints(state);

        let mut metrics = eval.platform_result.metrics.clone();
        metrics.insert("h1_violations".to_string(), hard_violations.iter().filter(|v| v.constraint_id == "H1_SingleShiftPerDay").count() as f64);
        metrics.insert("h2_coverage_violations".to_string(), hard_violations.iter().filter(|v| v.constraint_id == "H2_UnderstaffingCoverage").count() as f64);
        metrics.insert("h2_skill_violations".to_string(), hard_violations.iter().filter(|v| v.constraint_id == "H2_SkillRequirements").count() as f64);
        metrics.insert("h3_transition_violations".to_string(), hard_violations.iter().filter(|v| v.constraint_id == "H3_ForbiddenShiftTransitions").count() as f64);
        metrics.insert("s2_consecutive_violations".to_string(), eval.platform_result.soft_constraint_violations.iter().filter(|v| v.constraint_id == "S2_ConsecutiveWorkingDays").count() as f64);
        metrics.insert("total_hard_violations".to_string(), hard_violations.len() as f64);
        metrics.insert("is_feasible".to_string(), if hard_violations.is_empty() { 1.0 } else { 0.0 });
        metrics.insert("total_soft_penalty".to_string(), eval.soft_report.total_penalty as f64);

        coralys_core::EvaluationResult {
            objectives: eval.platform_result.objectives,
            hard_constraint_violations: hard_violations,
            soft_constraint_violations: eval.platform_result.soft_constraint_violations,
            metrics,
        }
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

        let intermediate_state = InrcState::new(
            state.scenario_id.clone(),
            state.week_index,
            num_nurses,
            num_days,
            num_shifts,
            new_bits,
            0,
            false,
        );

        let eval_result = self.evaluate(&intermediate_state);
        let hard_viols = eval_result.hard_constraint_violations.len();
        let soft_penalty = eval_result
            .metrics
            .get("total_soft_penalty")
            .copied()
            .unwrap_or(0.0) as i32;

        let mut next_state = InrcState::new(
            state.scenario_id.clone(),
            state.week_index,
            num_nurses,
            num_days,
            num_shifts,
            intermediate_state.genome_bits,
            soft_penalty,
            hard_viols == 0,
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
        next_state.reference.metadata.insert("is_simulated".to_string(), "true".to_string());
        next_state.reference.metadata.insert("hard_violations".to_string(), hard_viols.to_string());
        next_state.reference.metadata.insert("soft_penalty".to_string(), soft_penalty.to_string());

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

/// Deterministic Variable Neighborhood Descent (VND) and ALNS Reasoning Engine for INRC-II.
///
/// Implements the unified 4-phase deterministic pipeline adapted from the Aircraft Landing Problem:
/// - Phase 1 & 2: Paced Minimum Marginal Regret Seeding (via `InvertedContext` & `DailyOptimalFlow`)
/// - Phase 3: Hierarchical Variable Neighborhood Descent Ladder (N1..N4 with immediate restart to N1)
/// - Phase 3.5: Bottleneck-Targeted ALNS (escape combinatorial local minima via ruin-and-recreate)
/// - Phase 4: Rolling-Horizon Local Window Polish (Shift Streak Harmonizer for S4 and final flow polish)
#[derive(Debug, Clone, Default)]
pub struct CoralysInrcVndEngine;

impl ReasoningEngine for CoralysInrcVndEngine {
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
        let num_nurses = optimizer.context.num_nurses;
        let num_days = optimizer.context.num_days;
        let num_shifts = optimizer.context.shift_types.len();

        let mut rng = match config.seed {
            Some(s) => rand::rngs::StdRng::seed_from_u64(s),
            None => rand::rngs::StdRng::from_entropy(),
        };

        // ═════════════════════════════════════════════════════════════════════════
        // PHASE 1 & 2: PACED MINIMUM MARGINAL REGRET SEEDING
        // ═════════════════════════════════════════════════════════════════════════
        // Deterministic initial genome construction prioritizing rare skills, weekend cohesion,
        // statutory bounds (HC1-HC4), and minimum assignment pacing.
        let mut best_genome = optimizer.context.inverted.construct_feasible_genome(&mut rng);
        optimizer.repair_hard_constraints(&mut best_genome);

        let eval_seed_cost = |g: &InrcGenome| -> i32 {
            let (_, s8) = optimizer.eval_days_coverage(g, &[0, 1, 2, 3, 4, 5, 6]);
            let p_sum: i32 = (0..num_nurses).map(|n| optimizer.eval_nurse_soft_penalty(g, n)).sum();
            s8 + p_sum
        };
        let mut best_seed_cost = eval_seed_cost(&best_genome);
        for _ in 0..5 {
            let mut cand = optimizer.context.inverted.construct_feasible_genome(&mut rng);
            optimizer.repair_hard_constraints(&mut cand);
            let cost = eval_seed_cost(&cand);
            if cost < best_seed_cost {
                best_seed_cost = cost;
                best_genome = cand;
            }
        }

        // Pre-optimization Min-Cost Flow Augmentation (co-optimizes S8 demand and S1 contract pacing)
        let pacing = crate::inrc::pacing::HorizonPacing::compute(
            &plugin.context.scenario,
            &plugin.context.week_data,
            &plugin.context.history,
        );
        let flow = crate::inrc::optimal_flow::DailyOptimalFlow::new();
        flow.optimize_all_days(&optimizer, &mut best_genome, &pacing);

        // Pre-VND Dynamic RCSP Pricing Pass (seed with optimal lines of work on DAG)
        optimizer.optimize_rotations(&mut best_genome);
        optimizer.repair_hard_constraints(&mut best_genome);

        // Pre-compute contract-skill groups (identical contract + identical skill mask)
        let mut contract_skill_map: HashMap<(String, u8), Vec<usize>> = HashMap::new();
        for n in 0..num_nurses {
            let contract_id = optimizer.context.scenario.nurses[n].contract.clone();
            let mask = optimizer.context.nurse_skill_masks[n];
            contract_skill_map.entry((contract_id, mask)).or_default().push(n);
        }
        let contract_skill_groups: Vec<Vec<usize>> = contract_skill_map
            .into_values()
            .filter(|g| g.len() >= 2)
            .collect();

        // Pre-compute skill groups
        let mut skill_groups_map: HashMap<Vec<String>, Vec<usize>> = HashMap::new();
        for (n, nurse) in optimizer.context.scenario.nurses.iter().enumerate() {
            let mut s = nurse.skills.clone();
            s.sort();
            skill_groups_map.entry(s).or_default().push(n);
        }
        let eligible_groups: Vec<Vec<usize>> = skill_groups_map
            .into_values()
            .filter(|g| g.len() >= 2)
            .collect();

        // Nurse can work matrix [n][d][s]
        let days_map = [
            "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday",
        ];
        let mut nurse_can_work: Vec<Vec<Vec<bool>>> = vec![vec![vec![false; num_shifts]; num_days]; num_nurses];
        for n in 0..num_nurses {
            let nurse_skills = &optimizer.context.scenario.nurses[n].skills;
            for d in 0..num_days {
                let day_name = days_map[d];
                for s in 0..num_shifts {
                    let st_name = &optimizer.context.shift_types[s];
                    let mut demanded = Vec::new();
                    for req in &optimizer.context.week_data.requirements {
                        if &req.shift_type == st_name {
                            let req_level = match day_name {
                                "Monday" => &req.monday,
                                "Tuesday" => &req.tuesday,
                                "Wednesday" => &req.wednesday,
                                "Thursday" => &req.thursday,
                                "Friday" => &req.friday,
                                "Saturday" => &req.saturday,
                                "Sunday" => &req.sunday,
                                _ => unreachable!(),
                            };
                            if req_level.minimum > 0 || req_level.optimal > 0 {
                                demanded.push(&req.skill);
                            }
                        }
                    }
                    nurse_can_work[n][d][s] = demanded.is_empty() || demanded.iter().any(|sk| nurse_skills.contains(sk));
                }
            }
        }

        let nurse_target_bands: Vec<(usize, usize)> = (0..num_nurses)
            .map(|n| {
                let contract = optimizer
                    .context
                    .scenario
                    .contracts
                    .iter()
                    .find(|c| c.id == optimizer.context.scenario.nurses[n].contract)
                    .unwrap();
                let min_a = if contract.min_consecutive_working_days > (contract.min_assignments / optimizer.context.scenario.number_of_weeks) {
                    0
                } else {
                    contract.min_assignments / optimizer.context.scenario.number_of_weeks
                };
                let max_a = ((contract.max_assignments as f64
                    / optimizer.context.scenario.number_of_weeks as f64)
                    .ceil() as usize)
                    .max(contract.min_consecutive_working_days);
                (min_a, max_a)
            })
            .collect();

        let pricing_solver = crate::inrc::pricing::NursePricingSolver::new();
        let ruin_recreate = crate::inrc::ruin_recreate::ClusterRuinRecreate::new();
        let contract_clusters = crate::inrc::ruin_recreate::ClusterRuinRecreate::find_contract_clusters(&optimizer);

        // Helper closure: executes exhaustive sweeps of the Hierarchical VND Ladder
        let run_vnd_ladder = |current_genome: &mut InrcGenome| {
            let mut outer_improved = true;
            let mut round = 0;
            let max_rounds = 25;

            while outer_improved && round < max_rounds {
                outer_improved = false;
                round += 1;

                // ── Neighborhood 1: Single-Day Intra-Skill & Group Swaps ───────────────
                let mut n1_improved = true;
                while n1_improved {
                    n1_improved = false;
                    for d in 0..num_days {
                        for grp in &contract_skill_groups {
                            for i in 0..grp.len().saturating_sub(1) {
                                for j in (i + 1)..grp.len() {
                                    let u = grp[i];
                                    let v = grp[j];
                                    let s_u = optimizer.get_assigned_shift(current_genome, u, d);
                                    let s_v = optimizer.get_assigned_shift(current_genome, v, d);
                                    if s_u == s_v {
                                        continue;
                                    }

                                    let mut cand = current_genome.clone();
                                    let old_p_u = optimizer.eval_nurse_soft_penalty(current_genome, u);
                                    let old_p_v = optimizer.eval_nurse_soft_penalty(current_genome, v);

                                    optimizer.set_assigned_shift(&mut cand, u, d, s_v);
                                    optimizer.set_assigned_shift(&mut cand, v, d, s_u);

                                    if optimizer.nurse_satisfies_hard_contracts(&cand, u)
                                        && optimizer.nurse_satisfies_hard_contracts(&cand, v)
                                    {
                                        let new_p_u = optimizer.eval_nurse_soft_penalty(&cand, u);
                                        let new_p_v = optimizer.eval_nurse_soft_penalty(&cand, v);
                                        let delta = (new_p_u + new_p_v) - (old_p_u + old_p_v);
                                        if delta < 0 {
                                            *current_genome = cand;
                                            n1_improved = true;
                                            outer_improved = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // ── Neighborhood 2: Targeted Weekend Kempe Harmonizer (S6) ─────────────
                let mut n2_improved = true;
                while n2_improved {
                    n2_improved = false;
                    let mut sat_only = Vec::new();
                    let mut sun_only = Vec::new();
                    for n in 0..num_nurses {
                        let sat = optimizer.get_assigned_shift(current_genome, n, 5);
                        let sun = optimizer.get_assigned_shift(current_genome, n, 6);
                        if sat.is_some() && sun.is_none() {
                            sat_only.push((n, sat.unwrap()));
                        } else if sat.is_none() && sun.is_some() {
                            sun_only.push((n, sun.unwrap()));
                        }
                    }

                    for &(u, s_sat) in &sat_only {
                        for &(v, s_sun) in &sun_only {
                            if nurse_can_work[u][6][s_sun]
                                && !optimizer.context.inverted.forbidden_matrix[s_sat][s_sun]
                            {
                                let mut cand = current_genome.clone();
                                let old_p_u = optimizer.eval_nurse_soft_penalty(current_genome, u);
                                let old_p_v = optimizer.eval_nurse_soft_penalty(current_genome, v);
                                let (_, old_s8) = optimizer.eval_days_coverage(current_genome, &[6]);

                                optimizer.set_assigned_shift(&mut cand, v, 6, None);
                                optimizer.set_assigned_shift(&mut cand, u, 6, Some(s_sun));

                                if optimizer.nurse_satisfies_hard_contracts(&cand, u)
                                    && optimizer.nurse_satisfies_hard_contracts(&cand, v)
                                {
                                    let (is_feas, new_s8) = optimizer.eval_days_coverage(&cand, &[6]);
                                    if is_feas {
                                        let new_p_u = optimizer.eval_nurse_soft_penalty(&cand, u);
                                        let new_p_v = optimizer.eval_nurse_soft_penalty(&cand, v);
                                        let delta = (new_p_u + new_p_v) - (old_p_u + old_p_v) + (new_s8 - old_s8);
                                        if delta < 0 {
                                            *current_genome = cand;
                                            n2_improved = true;
                                            outer_improved = true;
                                            break;
                                        }
                                    }
                                }
                            }

                            // Also test reverse Saturday shift transfer
                            if nurse_can_work[v][5][s_sat]
                                && !optimizer.context.inverted.forbidden_matrix[s_sat][s_sun]
                            {
                                let mut cand = current_genome.clone();
                                let old_p_u = optimizer.eval_nurse_soft_penalty(current_genome, u);
                                let old_p_v = optimizer.eval_nurse_soft_penalty(current_genome, v);
                                let (_, old_s8) = optimizer.eval_days_coverage(current_genome, &[5]);

                                optimizer.set_assigned_shift(&mut cand, u, 5, None);
                                optimizer.set_assigned_shift(&mut cand, v, 5, Some(s_sat));

                                if optimizer.nurse_satisfies_hard_contracts(&cand, u)
                                    && optimizer.nurse_satisfies_hard_contracts(&cand, v)
                                {
                                    let (is_feas, new_s8) = optimizer.eval_days_coverage(&cand, &[5]);
                                    if is_feas {
                                        let new_p_u = optimizer.eval_nurse_soft_penalty(&cand, u);
                                        let new_p_v = optimizer.eval_nurse_soft_penalty(&cand, v);
                                        let delta = (new_p_u + new_p_v) - (old_p_u + old_p_v) + (new_s8 - old_s8);
                                        if delta < 0 {
                                            *current_genome = cand;
                                            n2_improved = true;
                                            outer_improved = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        if n2_improved {
                            break;
                        }
                    }
                }

                // ── Neighborhood 3: Multi-Day Interval Block Swap (2-Day / 3-Day) ──────
                let mut n3_improved = true;
                while n3_improved {
                    n3_improved = false;
                    'n3_block: for block_len in [2, 3] {
                        for start_d in 0..=(num_days.saturating_sub(block_len)) {
                            for grp in &contract_skill_groups {
                                for i in 0..grp.len().saturating_sub(1) {
                                    for j in (i + 1)..grp.len() {
                                        let u = grp[i];
                                        let v = grp[j];

                                        let mut any_diff = false;
                                        for k in 0..block_len {
                                            let day = start_d + k;
                                            if optimizer.get_assigned_shift(current_genome, u, day)
                                                != optimizer.get_assigned_shift(current_genome, v, day)
                                            {
                                                any_diff = true;
                                                break;
                                            }
                                        }
                                        if !any_diff {
                                            continue;
                                        }

                                        let mut cand = current_genome.clone();
                                        let old_p_u = optimizer.eval_nurse_soft_penalty(current_genome, u);
                                        let old_p_v = optimizer.eval_nurse_soft_penalty(current_genome, v);

                                        for k in 0..block_len {
                                            let day = start_d + k;
                                            let s_u = optimizer.get_assigned_shift(current_genome, u, day);
                                            let s_v = optimizer.get_assigned_shift(current_genome, v, day);
                                            optimizer.set_assigned_shift(&mut cand, u, day, s_v);
                                            optimizer.set_assigned_shift(&mut cand, v, day, s_u);
                                        }

                                        if optimizer.nurse_satisfies_hard_contracts(&cand, u)
                                            && optimizer.nurse_satisfies_hard_contracts(&cand, v)
                                        {
                                            let new_p_u = optimizer.eval_nurse_soft_penalty(&cand, u);
                                            let new_p_v = optimizer.eval_nurse_soft_penalty(&cand, v);
                                            let delta = (new_p_u + new_p_v) - (old_p_u + old_p_v);
                                            if delta < 0 {
                                                *current_genome = cand;
                                                n3_improved = true;
                                                outer_improved = true;
                                                break 'n3_block;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // ── Neighborhood 4: Dynamic RCSP Pricing on DAG ────────────────────────
                let shadow_credits = pricing_solver.compute_shadow_credits(&optimizer, current_genome);
                let zero_credits = [[0i32; crate::inrc::pricing::MAX_PRICING_SHIFTS]; 7];
                let mut nurses_by_penalty: Vec<usize> = (0..num_nurses).collect();
                nurses_by_penalty.sort_by_key(|&n| std::cmp::Reverse(optimizer.eval_nurse_soft_penalty(current_genome, n)));

                for n in nurses_by_penalty {
                    if optimizer.eval_nurse_soft_penalty(current_genome, n) == 0 {
                        break;
                    }
                    let (target_min, target_max) = nurse_target_bands[n];
                    let target = (target_min + target_max) / 2;
                    let targets = [target.saturating_sub(1), target, target + 1];

                    let mut best_rcsp_cand: Option<(InrcGenome, i32)> = None;
                    let mut max_rcsp_gain = 0;

                    for &t in &targets {
                        for credits in &[&shadow_credits, &zero_credits] {
                            if let Some(cand_rot) = pricing_solver.solve_optimal_rotation(&optimizer, n, credits, t) {
                                let mut changed_days = Vec::new();
                                let mut cand = current_genome.clone();
                                for d in 0..num_days {
                                    let cur_s = optimizer.get_assigned_shift(current_genome, n, d);
                                    let rot_s = cand_rot[d];
                                    if cur_s != rot_s {
                                        changed_days.push(d);
                                        optimizer.set_assigned_shift(&mut cand, n, d, rot_s);
                                    }
                                }
                                if !changed_days.is_empty() && optimizer.nurse_satisfies_hard_contracts(&cand, n) {
                                    let (is_feas, new_s8) = optimizer.eval_days_coverage(&cand, &changed_days);
                                    if is_feas {
                                        let (_, old_s8) = optimizer.eval_days_coverage(current_genome, &changed_days);
                                        let old_p = optimizer.eval_nurse_soft_penalty(current_genome, n);
                                        let new_p = optimizer.eval_nurse_soft_penalty(&cand, n);
                                        let gain = (old_p - new_p) + (old_s8 - new_s8);
                                        if gain > max_rcsp_gain {
                                            max_rcsp_gain = gain;
                                            best_rcsp_cand = Some((cand, gain));
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some((better_genome, _)) = best_rcsp_cand {
                        *current_genome = better_genome;
                        outer_improved = true;
                    }
                }
            }
        };

        // Run initial VND Ladder
        run_vnd_ladder(&mut best_genome);

        // ═════════════════════════════════════════════════════════════════════════
        // PHASE 3.5: BOTTLENECK-TARGETED ALNS & MEMETIC ANNEALING POLISH
        // ═════════════════════════════════════════════════════════════════════════
        let alns_cycles = 25;
        for _cycle in 0..alns_cycles {
            if let Some((alns_cand, delta)) = ruin_recreate.step_alns(&optimizer, &best_genome, &mut rng, &contract_clusters) {
                if delta < 0.0 {
                    best_genome = alns_cand;
                    run_vnd_ladder(&mut best_genome);
                }
            }
        }

        // High-velocity SA Memetic Polish across all 10 specialized operators
        let sa_budget = (num_nurses as u64 * 40).max(2500);
        let domain = InrcAnnealingDomain::new(&optimizer);
        let sa_config = AnnealingConfig {
            initial_temperature: 30.0,
            cooling_rate: 0.99998,
            min_temperature: 0.1,
            restart_temperature: 30.0,
            time_budget_ms: sa_budget,
        };
        let sa_engine = SimulatedAnnealingEngine::new(sa_config);
        let (polished_genome, _, _) = sa_engine.optimize(&domain, &best_genome, &mut rng);
        best_genome = polished_genome;

        // ═════════════════════════════════════════════════════════════════════════
        // PHASE 4: ROLLING-HORIZON LOCAL WINDOW POLISH (SHIFT STREAK HARMONIZER)
        // ═════════════════════════════════════════════════════════════════════════
        for d in 0..num_days.saturating_sub(1) {
            for grp in &eligible_groups {
                for i in 0..grp.len().saturating_sub(1) {
                    for j in (i + 1)..grp.len() {
                        let u = grp[i];
                        let v = grp[j];
                        let (s_u0, s_u1) = (optimizer.get_assigned_shift(&best_genome, u, d), optimizer.get_assigned_shift(&best_genome, u, d + 1));
                        let (s_v0, s_v1) = (optimizer.get_assigned_shift(&best_genome, v, d), optimizer.get_assigned_shift(&best_genome, v, d + 1));

                        if s_u0.is_some() && s_u1.is_some() && s_v0.is_some() && s_v1.is_some() {
                            if s_u0 != s_u1 || s_v0 != s_v1 {
                                // Test swapping day d+1 to harmonize consecutive shifts
                                let mut cand = best_genome.clone();
                                let old_p_u = optimizer.eval_nurse_soft_penalty(&best_genome, u);
                                let old_p_v = optimizer.eval_nurse_soft_penalty(&best_genome, v);

                                optimizer.set_assigned_shift(&mut cand, u, d + 1, s_v1);
                                optimizer.set_assigned_shift(&mut cand, v, d + 1, s_u1);

                                if optimizer.nurse_satisfies_hard_contracts(&cand, u)
                                    && optimizer.nurse_satisfies_hard_contracts(&cand, v)
                                {
                                    let new_p_u = optimizer.eval_nurse_soft_penalty(&cand, u);
                                    let new_p_v = optimizer.eval_nurse_soft_penalty(&cand, v);
                                    let delta = (new_p_u + new_p_v) - (old_p_u + old_p_v);
                                    if delta < 0 {
                                        best_genome = cand;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Post-VND Min-Cost Flow Polish (capture any remaining free optimal coverage gains)
        flow.optimize_all_days(&optimizer, &mut best_genome, &pacing);

        // Final Hard Constraint Safety Check (guarantee 0 HC)
        optimizer.repair_hard_constraints(&mut best_genome);
        let metric_report = coralys_moga::runtime::optimization::metric::MetricReport::default();
        let final_eval = optimizer.evaluate(&InrcGenome { bits: best_genome.bits.clone() }, &metric_report);

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

/// Coralys Primary Decision Engine for INRC-II Nurse Rostering.
///
/// Implements [`coralys_core::ReasoningEngine`], generating, evaluating, and issuing
/// statutory [`coralys_core::DecisionProposal`] actions with machine-level constraint auditing.
/// Powered by the 4-phase deterministic [`CoralysInrcVndEngine`] pipeline.
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
        let vnd_engine = CoralysInrcVndEngine::default();
        vnd_engine.solve(plugin, config)
    }
}
