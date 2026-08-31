use crate::inrc::models::{InrcScenario, InrcWeekData};
use crate::inrc::validator::validate_schedule;
use coralys_moga::engine_proof::{Evaluator, FitnessVector, Genome, MutationPolicy};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct AssignmentSlot {
    pub slot_id: usize,
    pub day: usize,
    pub shift_type: String,
    pub required_skill: String,
    pub assigned_nurse: String,
}

#[derive(Clone, Hash, PartialEq, Eq)]
/// **Compatibility Implementation / Legacy Operational Model**
///
/// Architecturally, `ScheduleGenome` is no longer the primary structural
/// representation of the domain. It serves as a compatibility implementation
/// of the Coralys `OperationalModel` during the migration to the Native
/// Operational Model (OEN).
pub struct ScheduleGenome {
    pub slots: Vec<AssignmentSlot>,
    pub num_days: usize,
    pub nurses: Vec<String>,
}

impl ScheduleGenome {
    pub fn to_flat_schedule(&self) -> HashMap<String, Vec<String>> {
        let mut flat = HashMap::new();
        for nurse in &self.nurses {
            flat.insert(nurse.clone(), vec!["".to_string(); self.num_days]);
        }
        for slot in &self.slots {
            if let Some(timeline) = flat.get_mut(&slot.assigned_nurse) {
                timeline[slot.day] = slot.shift_type.clone();
            }
        }
        flat
    }

    pub fn signatures(&self) -> Vec<u64> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut sigs = Vec::with_capacity(self.slots.len());
        for slot in &self.slots {
            let mut hasher = DefaultHasher::new();
            slot.slot_id.hash(&mut hasher);
            slot.assigned_nurse.hash(&mut hasher);
            sigs.push(hasher.finish());
        }
        sigs
    }
}

impl coralys_core::Solution for ScheduleGenome {}
impl Genome for ScheduleGenome {}
impl coralys_moga::Genome for ScheduleGenome {}

pub struct UltraCrewEvaluator {
    pub scenario: InrcScenario,
    /// Week data is required to evaluate HC1 minimum coverage.
    /// Each InrcRequirementLevel.minimum represents a hard staffing requirement.
    pub week_data: InrcWeekData,
}

impl Evaluator<ScheduleGenome> for UltraCrewEvaluator {
    fn evaluate(&self, genome: &ScheduleGenome) -> FitnessVector {
        let schedule = genome.to_flat_schedule();

        let validation_report = validate_schedule(&schedule, &self.scenario);

        let mut s6_assignment_penalty = 0.0;
        let mut s7_weekend_penalty = 0.0;
        let mut recovery_penalty = 0.0;

        // Sum penalties from validation report exactly using INRC weights
        for det in &validation_report.details {
            match det.constraint.as_str() {
                "min_assignments" | "max_assignments" => {
                    let diff = if det.actual > det.required {
                        det.actual - det.required
                    } else {
                        det.required - det.actual
                    };
                    s6_assignment_penalty += (diff * 20) as f64;
                }
                "max_working_weekends" => {
                    let diff = if det.actual > det.required {
                        det.actual - det.required
                    } else {
                        det.required - det.actual
                    };
                    s7_weekend_penalty += (diff * 30) as f64;
                }
                "min_consecutive_days_off"
                | "max_consecutive_working_days"
                | "min_consecutive_working_days"
                | "complete_weekends" => {
                    let diff = if det.actual > det.required {
                        det.actual - det.required
                    } else {
                        det.required - det.actual
                    };
                    recovery_penalty += (diff * 30) as f64;
                }
                _ => {}
            }
        }

        let mut assigned_counts = Vec::new();
        let mut weekend_counts = Vec::new();

        for nurse in &self.scenario.nurses {
            let shifts = &schedule[&nurse.id];
            let mut total_assigned = 0;
            let mut total_weekends = 0;

            for d in 0..genome.num_days {
                let is_work = !shifts[d].is_empty();
                if is_work {
                    total_assigned += 1;
                }

                let weekday = d % 7;
                if weekday == 5 || weekday == 6 {
                    if is_work {
                        total_weekends += 1;
                    }
                }
            }

            assigned_counts.push(total_assigned as f64);
            weekend_counts.push(total_weekends as f64);
        }

        let mean_assign = assigned_counts.iter().sum::<f64>() / assigned_counts.len() as f64;
        let workload_balance = assigned_counts
            .iter()
            .map(|v| (v - mean_assign).powi(2))
            .sum::<f64>()
            / assigned_counts.len() as f64;

        let mean_wknd = weekend_counts.iter().sum::<f64>() / weekend_counts.len() as f64;
        let temporal_load_balance = weekend_counts
            .iter()
            .map(|v| (v - mean_wknd).powi(2))
            .sum::<f64>()
            / weekend_counts.len() as f64;

        // HC1: Minimum Coverage — hard constraint.
        // InrcConstraintId::Hc1MinimumCoverage is classified as a hard constraint
        // in the domain model (models.rs). The scalar InrcOptimizer path applies
        // hard_constraint_violation = 1000 per uncovered minimum position.
        // We encode the same semantics here as a large-penalty objective so that
        // any genome with coverage_deficit > 0 is dominated by any genome with
        // coverage_deficit = 0 on this objective, regardless of other objectives.
        // This is the standard MOGA penalty-based feasibility technique.
        let days_map = [
            "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday",
        ];
        let num_days = genome.num_days;
        let flat = genome.to_flat_schedule();
        let mut coverage_deficit: f64 = 0.0;

        for d in 0..num_days {
            let day_name = days_map[d % 7];
            for req in &self.week_data.requirements {
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
                if req_level.minimum == 0 {
                    continue;
                }
                // Count nurses assigned to this shift on this day with the required skill
                let mut filled = 0usize;
                for nurse in &self.scenario.nurses {
                    if nurse.skills.contains(&req.skill) {
                        if let Some(shifts) = flat.get(&nurse.id) {
                            if d < shifts.len() && shifts[d] == req.shift_type {
                                filled += 1;
                            }
                        }
                    }
                }
                let minimum = req_level.minimum;
                if filled < minimum {
                    // Weight matches hard_constraint_violation = 1000 from ObjectiveWeights
                    coverage_deficit += ((minimum - filled) * 1000) as f64;
                }
            }
        }

        vec![
            s6_assignment_penalty,
            s7_weekend_penalty,
            recovery_penalty,
            workload_balance,
            temporal_load_balance,
            coverage_deficit, // objective[5]: HC1 minimum coverage penalty (minimize, hard)
        ]
    }
}

pub struct UltraCrewMutator {
    pub scenario: InrcScenario,
    pub equivalence_classes: HashMap<String, Vec<String>>,
}

impl UltraCrewMutator {
    pub fn new(scenario: InrcScenario) -> Self {
        let mut equivalence_classes = HashMap::new();
        for n in &scenario.nurses {
            let mut skills_sorted = n.skills.clone();
            skills_sorted.sort();
            let key = format!("{:?}-{}", skills_sorted, n.contract);
            equivalence_classes
                .entry(key)
                .or_insert_with(Vec::new)
                .push(n.id.clone());
        }
        Self {
            scenario,
            equivalence_classes,
        }
    }

    pub fn mutate_with_tier(&self, genome: &ScheduleGenome, use_tier1: bool) -> ScheduleGenome {
        self.mutate_with_tier_logged(genome, use_tier1).0
    }

    pub fn mutate_with_tier_logged(
        &self,
        genome: &ScheduleGenome,
        use_tier1: bool,
    ) -> (ScheduleGenome, String) {
        let mut child = genome.clone();
        if child.slots.is_empty() {
            return (child, "NoOp".to_string());
        }

        let mut rng = rand::thread_rng();
        let slot_idx = rng.gen_range(0..child.slots.len());

        let req_skill = child.slots[slot_idx].required_skill.clone();
        let day = child.slots[slot_idx].day;
        let current_nurse = child.slots[slot_idx].assigned_nurse.clone();

        let current_nurse_obj = self
            .scenario
            .nurses
            .iter()
            .find(|n| n.id == current_nurse)
            .unwrap();
        let mut skills_sorted = current_nurse_obj.skills.clone();
        skills_sorted.sort();
        let eq_key = format!("{:?}-{}", skills_sorted, current_nurse_obj.contract);

        let mut working_nurses = std::collections::HashSet::new();
        for s in &child.slots {
            if s.day == day {
                working_nurses.insert(s.assigned_nurse.clone());
            }
        }

        let mut eligible_targets = Vec::new();

        if use_tier1 {
            if let Some(class_members) = self.equivalence_classes.get(&eq_key) {
                for target_id in class_members {
                    if target_id != &current_nurse && !working_nurses.contains(target_id) {
                        eligible_targets.push(target_id.clone());
                    }
                }
            }
        }

        // Fallback to Tier 2 if Tier 1 yields no targets, or if Tier 2 was requested
        if eligible_targets.is_empty() {
            for target in &self.scenario.nurses {
                if target.id != current_nurse
                    && !working_nurses.contains(&target.id)
                    && target.skills.contains(&req_skill)
                {
                    eligible_targets.push(target.id.clone());
                }
            }
        }

        if !eligible_targets.is_empty() {
            let target_nurse = &eligible_targets[rng.gen_range(0..eligible_targets.len())];
            child.slots[slot_idx].assigned_nurse = target_nurse.clone();
        }

        let operator_name = if use_tier1 && !eligible_targets.is_empty() {
            "Tier1_EqClass_Swap"
        } else {
            "Tier2_Random_Swap"
        };

        (child, operator_name.to_string())
    }
}
#[cfg(test)]
mod hc1_coverage_tests {
    use super::*;
    use crate::inrc::models::{
        InrcContract, InrcForbiddenSuccession, InrcNurse, InrcRequirement, InrcRequirementLevel,
        InrcScenario, InrcShiftOffRequest, InrcShiftType, InrcWeekData,
    };
    use coralys_moga::engine_proof::Evaluator;

    /// Build a minimal 1-nurse, 1-day, 1-shift scenario with a single requirement:
    /// minimum = 1 nurse of skill "Nurse" on shift "Early" on Monday.
    fn minimal_scenario() -> InrcScenario {
        InrcScenario {
            id: "test".to_string(),
            number_of_weeks: 1,
            skills: vec!["Nurse".to_string()],
            shift_types: vec![InrcShiftType {
                id: "Early".to_string(),
                min_consecutive: 1,
                max_consecutive: 7,
            }],
            forbidden_shift_type_successions: vec![],
            contracts: vec![InrcContract {
                id: "FullTime".to_string(),
                min_assignments: 1,
                max_assignments: 7,
                min_consecutive_working_days: 1,
                max_consecutive_working_days: 7,
                min_consecutive_days_off: 1,
                max_consecutive_days_off: 7,
                max_working_weekends: 4,
                complete_weekends: 0,
            }],
            nurses: vec![InrcNurse {
                id: "N1".to_string(),
                contract: "FullTime".to_string(),
                skills: vec!["Nurse".to_string()],
            }],
        }
    }

    fn minimal_week_data_with_minimum(minimum: usize) -> InrcWeekData {
        let zero = InrcRequirementLevel { minimum: 0, optimal: 0 };
        let req = InrcRequirementLevel { minimum, optimal: minimum };
        InrcWeekData {
            scenario: "test".to_string(),
            requirements: vec![InrcRequirement {
                shift_type: "Early".to_string(),
                skill: "Nurse".to_string(),
                monday: req,
                tuesday: zero.clone(),
                wednesday: zero.clone(),
                thursday: zero.clone(),
                friday: zero.clone(),
                saturday: zero.clone(),
                sunday: zero,
            }],
            shift_off_requests: vec![],
        }
    }

    fn genome_with_nurse_on_monday(nurse_id: &str, num_days: usize) -> ScheduleGenome {
        ScheduleGenome {
            slots: vec![AssignmentSlot {
                slot_id: 0,
                day: 0, // Monday
                shift_type: "Early".to_string(),
                required_skill: "Nurse".to_string(),
                assigned_nurse: nurse_id.to_string(),
            }],
            num_days,
            nurses: vec!["N1".to_string()],
        }
    }

    fn genome_empty(num_days: usize) -> ScheduleGenome {
        ScheduleGenome {
            slots: vec![],
            num_days,
            nurses: vec!["N1".to_string()],
        }
    }

    /// HC1-G1: A genome that fills the required position has coverage_deficit = 0.
    #[test]
    fn hc1_g1_filled_position_has_zero_coverage_deficit() {
        let scenario = minimal_scenario();
        let week_data = minimal_week_data_with_minimum(1);
        let evaluator = UltraCrewEvaluator { scenario, week_data };
        let genome = genome_with_nurse_on_monday("N1", 7);
        let fitness = evaluator.evaluate(&genome);
        assert_eq!(fitness.len(), 6, "FitnessVector must have 6 objectives");
        assert_eq!(
            fitness[5], 0.0,
            "coverage_deficit must be 0 when required position is filled"
        );
    }

    /// HC1-G2: A genome that leaves the required position empty has coverage_deficit = 1000.
    #[test]
    fn hc1_g2_empty_genome_has_nonzero_coverage_deficit() {
        let scenario = minimal_scenario();
        let week_data = minimal_week_data_with_minimum(1);
        let evaluator = UltraCrewEvaluator { scenario, week_data };
        let genome = genome_empty(7);
        let fitness = evaluator.evaluate(&genome);
        assert_eq!(fitness.len(), 6, "FitnessVector must have 6 objectives");
        assert_eq!(
            fitness[5], 1000.0,
            "coverage_deficit must be 1000 (1 uncovered × 1000) when required position is empty"
        );
    }

    /// HC1-G3: A genome with coverage_deficit > 0 is dominated by one with coverage_deficit = 0
    /// on objective[5], regardless of other objectives.
    /// This is the core invariant: 40/196 must not dominate 194/196.
    #[test]
    fn hc1_g3_high_deficit_dominated_by_zero_deficit_on_objective5() {
        let scenario = minimal_scenario();
        let week_data = minimal_week_data_with_minimum(1);
        let evaluator = UltraCrewEvaluator { scenario, week_data };

        let filled = genome_with_nurse_on_monday("N1", 7);
        let empty = genome_empty(7);

        let fitness_filled = evaluator.evaluate(&filled);
        let fitness_empty = evaluator.evaluate(&empty);

        // The filled genome must have lower (better) coverage_deficit
        assert!(
            fitness_filled[5] < fitness_empty[5],
            "Filled genome (coverage_deficit={}) must have lower objective[5] than empty genome (coverage_deficit={})",
            fitness_filled[5],
            fitness_empty[5]
        );
    }

    /// HC1-G4: When minimum = 0, coverage_deficit is always 0 regardless of assignments.
    #[test]
    fn hc1_g4_zero_minimum_never_penalized() {
        let scenario = minimal_scenario();
        let week_data = minimal_week_data_with_minimum(0);
        let evaluator = UltraCrewEvaluator { scenario, week_data };
        let genome = genome_empty(7);
        let fitness = evaluator.evaluate(&genome);
        assert_eq!(
            fitness[5], 0.0,
            "coverage_deficit must be 0 when minimum requirement is 0"
        );
    }

    /// HC1-G5: FitnessVector always has exactly 6 objectives.
    #[test]
    fn hc1_g5_fitness_vector_has_six_objectives() {
        let scenario = minimal_scenario();
        let week_data = minimal_week_data_with_minimum(1);
        let evaluator = UltraCrewEvaluator { scenario, week_data };
        let genome = genome_with_nurse_on_monday("N1", 7);
        let fitness = evaluator.evaluate(&genome);
        assert_eq!(
            fitness.len(),
            6,
            "FitnessVector must have exactly 6 objectives (5 original + HC1 coverage)"
        );
    }
}

impl MutationPolicy<ScheduleGenome> for UltraCrewMutator {
    fn mutate(&self, genome: &ScheduleGenome) -> ScheduleGenome {
        self.mutate_with_tier(genome, false) // default fallback
    }
}
