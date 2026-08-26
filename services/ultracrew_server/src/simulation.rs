use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use ultracrew::inrc::models::InrcScenario;

#[derive(Serialize, Deserialize, Clone)]
pub struct BlockedRecovery {
    pub day: usize,
    pub reason: String,
    pub constraint: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BalanceChange {
    pub previous: i32,
    pub current: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum CandidateType {
    CreditorSwap,
    CoverageGap,
    OpenSlot,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RecoveryAudit {
    pub day: usize,
    pub candidate_type: CandidateType,
    pub accepted: bool,
    pub recovering_nurse: String,
    pub creditor: Option<String>,
    pub sick_nurse_legal: bool,
    pub creditor_legal: bool,
    pub imbalance_before: i32,
    pub imbalance_after: i32,
    pub blocked_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RecoveryPlan {
    pub affected_nurse: String,
    pub missed_shifts: i32,
    pub recovered_shifts: i32,
    pub recovery_eta: i32,
    pub creditors: HashMap<String, i32>,
    pub balance_changes: HashMap<String, BalanceChange>,
    pub coverage_impact: String,
    pub audit_trail: Vec<RecoveryAudit>,
    pub requested_shifts: i32,
    pub feasible_shifts: i32,
    pub blocked_recoveries: Vec<BlockedRecovery>,
}

#[derive(Serialize, Clone)]
pub struct Coverage {
    pub covered: i32,
    pub understaffed: i32,
    pub critical: i32,
}

#[derive(Serialize, Clone)]
pub struct Alert {
    pub employee: String,
    pub severity: String,
    pub message: String,
}

// ViolationDetail and ValidationReport have been moved to the UltraCrew Solution Adapter.
// Re-exported here for backward compatibility with existing server code.
pub use ultracrew::inrc::types::{ValidationReport, ViolationDetail};

#[derive(Serialize, Clone)]
pub struct WorkloadAudit {
    pub nurse_id: String,
    pub expected_assignments: i32,
    pub actual_assignments: i32,
    pub deviation: i32,
    pub max_work_streak: usize,
    pub max_off_streak: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SkillDeficit {
    pub day: usize,
    pub shift: String,
    pub skill: String,
    pub required: usize,
    pub assigned: usize,
    pub deficit: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SkillCoverageAudit {
    pub skill_coverage_percentage: f64,
    pub total_skill_deficits: usize,
    pub worst_skill: String,
    pub worst_shift: String,
    pub deficits: Vec<SkillDeficit>,
}

#[derive(Serialize, Clone)]
pub struct CoverageAudit {
    pub required_assignments: usize,
    pub actual_assignments: usize,
    pub coverage_percentage: f64,
    pub daily_assignments: Vec<usize>,
}

#[derive(Serialize, Clone)]
pub struct ConstraintAudit {
    pub nurse_id: String,
    pub min_work_streak_violations: usize,
    pub max_work_streak_violations: usize,
    pub min_off_streak_violations: usize,
    pub max_off_streak_violations: usize,
}

#[derive(Serialize, Clone)]
pub struct RosterHealth {
    pub legality_score: i32,
    pub coverage_score: i32,
    pub balance_score: i32,
    pub fragmentation_score: i32,
    pub recovery_score: i32,
}

#[derive(Serialize, Clone)]
pub struct BaselineStatus {
    pub state: String, // "Legal", "RepairFailed", "Incomplete"
    pub is_legal: bool,
    pub repair_attempts: i32,
    pub exhausted_search: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Bottleneck {
    pub description: String,
    pub severity: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FeasibilityReport {
    pub overall_feasible: bool,
    pub skill_feasible: bool,
    pub contract_feasible: bool,
    pub structural_feasible: bool,
    pub bottlenecks: Vec<Bottleneck>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ParetoFrontierSolution {
    pub s6_assignment_penalty: f64,
    pub s7_weekend_penalty: f64,
    pub recovery_penalty: f64,
    pub workload_balance: f64,
    pub temporal_load_balance: f64,
    pub schedule: HashMap<String, Vec<String>>,
}

#[derive(Serialize, Clone)]
pub struct Dashboard {
    pub feasibility_report: Option<FeasibilityReport>,
    pub skill_coverage_audit: Option<SkillCoverageAudit>,
    pub coverage: Coverage,
    pub coverage_audit: CoverageAudit,
    pub alerts: Vec<Alert>,
    pub recommendations: Vec<String>,
    pub validation_report: ValidationReport,
    pub workload_audit: Vec<WorkloadAudit>,
    pub constraint_audit: Vec<ConstraintAudit>,
    pub roster_health: RosterHealth,
    pub baseline_status: BaselineStatus,
    pub pareto_frontier: Option<Vec<ParetoFrontierSolution>>,
}

#[derive(Serialize, Clone)]
pub struct NurseBalance {
    pub nurse_id: String,
    pub balance: i32,
    pub explanation: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct VerificationReports {
    pub baseline: Option<ValidationReport>,
    pub sickness: Option<ValidationReport>,
    pub recovery: Option<ValidationReport>,
}

#[derive(Serialize, Clone)]
pub struct SimulationState {
    pub schedule: HashMap<String, Vec<String>>,
    pub dashboard: Dashboard,
    pub balances: Vec<NurseBalance>,
    pub recovery_plan: Option<RecoveryPlan>,
    pub verification_reports: VerificationReports,
}

pub fn generate_baseline_schedule(
    scenario: &ultracrew::inrc::models::InrcScenario,
    requirements: &Vec<ultracrew::inrc::models::InrcRequirement>,
) -> Result<crate::optimizer::ScheduleGenome, String> {
    use crate::optimizer::{AssignmentSlot, ScheduleGenome};

    let mut slots = Vec::new();
    let mut slot_id_counter = 0;

    // Track assigned load per nurse to pick least-loaded (Phase B)
    let mut nurse_load: std::collections::HashMap<String, i32> = std::collections::HashMap::new();

    let mut nurses_list = Vec::new();
    for nurse in &scenario.nurses {
        nurse_load.insert(nurse.id.clone(), 0);
        nurses_list.push(nurse.id.clone());
    }

    let mapped_shift_types = vec![
        ("Early", "Early"),
        ("Day", "Day"),
        ("Late", "Late"),
        ("Night", "Night"),
    ];

    let num_days = scenario.number_of_weeks * 7;
    for d in 0..num_days {
        let weekday = d % 7;

        let mut daily_slots = Vec::new();
        for req in requirements {
            let required = match weekday {
                0 => req.monday.optimal,
                1 => req.tuesday.optimal,
                2 => req.wednesday.optimal,
                3 => req.thursday.optimal,
                4 => req.friday.optimal,
                5 => req.saturday.optimal,
                6 => req.sunday.optimal,
                _ => 0,
            };
            let mapped_shift = mapped_shift_types
                .iter()
                .find(|(k, _)| *k == req.shift_type)
                .map(|(_, v)| *v)
                .unwrap_or("");
            for _ in 0..required {
                daily_slots.push((mapped_shift, req.skill.clone()));
            }
        }

        // Randomly assign least-loaded available nurse with skill
        let mut available_nurses: Vec<String> =
            scenario.nurses.iter().map(|n| n.id.clone()).collect();
        let mut rng = rand::thread_rng();

        for (shift, req_skill) in daily_slots {
            // Find a candidate
            let mut best_nurse = None;
            let mut min_load = i32::MAX;

            // Randomize tie-breaking
            let mut candidates = available_nurses.clone();
            use rand::seq::SliceRandom;
            candidates.shuffle(&mut rng);

            for candidate in candidates {
                let nurse_obj = scenario.nurses.iter().find(|n| n.id == candidate).unwrap();
                if nurse_obj.skills.contains(&req_skill) {
                    let load = *nurse_load.get(&candidate).unwrap();
                    if load < min_load {
                        min_load = load;
                        best_nurse = Some(candidate);
                    }
                }
            }

            if let Some(nurse) = best_nurse {
                available_nurses.retain(|n| n != &nurse);
                *nurse_load.get_mut(&nurse).unwrap() += 1;

                slots.push(AssignmentSlot {
                    slot_id: slot_id_counter,
                    day: d,
                    shift_type: shift.to_string(),
                    required_skill: req_skill,
                    assigned_nurse: nurse,
                });
                slot_id_counter += 1;
            } else {
                // If we can't find a nurse, we still push the slot to preserve the requirement?
                // For Tier-0 constructor, we just skip it, meaning Volume deficit
            }
        }
    }

    Ok(ScheduleGenome {
        slots,
        num_days,
        nurses: nurses_list,
    })
}

pub fn can_recover(
    nurse_id: &str,
    day_idx: usize,
    shift_type: &str,
    current_schedule: &HashMap<String, Vec<String>>,
    scenario: &InrcScenario,
) -> Result<(), String> {
    let nurse = scenario.nurses.iter().find(|n| n.id == nurse_id).unwrap();
    let contract = scenario
        .contracts
        .iter()
        .find(|c| c.id == nurse.contract)
        .unwrap();

    let shifts = current_schedule.get(nurse_id).unwrap();

    // Check if they are already working that day
    if !shifts[day_idx].is_empty() {
        return Err("Already assigned a shift on this day".to_string());
    }

    // Create hypothetical schedule
    let mut hyp = shifts.clone();
    hyp[day_idx] = shift_type.to_string();

    // Validate INRC constraints

    // 1. Max assignments
    let total_assignments = hyp
        .iter()
        .filter(|s| !s.is_empty() && !s.contains("SICK-"))
        .count();
    let num_days = (scenario.number_of_weeks * 7) as usize;
    let max_assignments = (contract.max_assignments as f64
        * (num_days as f64 / (scenario.number_of_weeks as f64 * 7.0)))
        .ceil() as usize;
    if total_assignments > max_assignments {
        return Err("Would exceed maximum total assignments".to_string());
    }

    // 2. Max consecutive working days & Min consecutive days off
    let mut current_work_streak = 0;
    let mut current_off_streak = 0;

    for d in 0..num_days {
        if !hyp[d].is_empty() && !hyp[d].contains("SICK-") {
            current_work_streak += 1;

            // Check max consecutive
            if current_work_streak > contract.max_consecutive_working_days {
                return Err("Would exceed maximum consecutive working days".to_string());
            }

            // Check min days off ended prematurely
            if current_off_streak > 0 && current_off_streak < contract.min_consecutive_days_off {
                return Err("Would violate minimum consecutive days off".to_string());
            }
            current_off_streak = 0;
        } else {
            current_off_streak += 1;

            // Min consecutive working days
            if current_work_streak > 0
                && current_work_streak < contract.min_consecutive_working_days
            {
                return Err("Would violate minimum consecutive working days".to_string());
            }
            current_work_streak = 0;
        }
    }

    // 3. Shift successions (e.g. N -> E is bad)
    for d in 1..num_days {
        let prev = &hyp[d - 1];
        let curr = &hyp[d];
        if !prev.is_empty()
            && !curr.is_empty()
            && !prev.contains("SICK-")
            && !curr.contains("SICK-")
        {
            let prev_clean = prev.replace("NEW-", "").replace("RECOVERED-", "");
            let curr_clean = curr.replace("NEW-", "").replace("RECOVERED-", "");

            if let Some(rule) = scenario
                .forbidden_shift_type_successions
                .iter()
                .find(|r| r.preceding == prev_clean)
            {
                if rule.succeeding.contains(&curr_clean) {
                    return Err(format!(
                        "Forbidden shift succession: {} -> {}",
                        prev_clean, curr_clean
                    ));
                }
            }
        }
    }

    Ok(())
}
