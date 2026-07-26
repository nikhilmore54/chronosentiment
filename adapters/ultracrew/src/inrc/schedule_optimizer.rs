use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rand::Rng;
use coralys_moga::engine_proof::{Genome, Evaluator, MutationPolicy, FitnessVector};
use crate::inrc::models::InrcScenario;
use crate::inrc::validator::validate_schedule;

#[derive(Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct AssignmentSlot {
    pub slot_id: usize,
    pub day: usize,
    pub shift_type: String,
    pub required_skill: String,
    pub assigned_nurse: String,
}

#[derive(Clone, Hash, PartialEq, Eq)]
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
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
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

        let mean_assign =
            assigned_counts.iter().sum::<f64>() / assigned_counts.len() as f64;
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

        vec![
            s6_assignment_penalty,
            s7_weekend_penalty,
            recovery_penalty,
            workload_balance,
            temporal_load_balance,
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

impl MutationPolicy<ScheduleGenome> for UltraCrewMutator {
    fn mutate(&self, genome: &ScheduleGenome) -> ScheduleGenome {
        self.mutate_with_tier(genome, false) // default fallback
    }
}