// Constraint engine for UltraCrew scheduling
// Provides validation of input data and constraints checking.

use crate::models::{Worker, Shift};
use crate::optimization::{ScheduleContext, ScheduleGenome};
use std::error::Error;
use std::collections::{HashSet, HashMap};
use std::sync::Arc;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConstraintReport {
    pub fitness: f64,
    pub is_valid: bool,
    pub hard_violations: usize,
    pub soft_violations: usize,
    pub warnings: Vec<String>,
    pub constraint_scores: HashMap<String, f64>,
    pub violated_constraints: Vec<String>,
    pub satisfied_constraints: Vec<String>,

    // Backward compatibility for existing public APIs
    pub hc1_violations: usize,
    pub hc2_violations: usize,
    pub hc3_violations: usize,
    pub hc4_violations: usize,
    pub rest_violations: usize,
    pub fairness_penalty: f64,
    pub fatigue_penalty: f64,
}

pub struct ConstraintEngine {
    pub context: Arc<ScheduleContext>,
}

impl ConstraintEngine {
    pub fn new(context: Arc<ScheduleContext>) -> Self {
        Self { context }
    }

    pub fn evaluate(&self, genome: &ScheduleGenome) -> ConstraintReport {
        let mut fitness = 0.0;
        let mut hc1_violations = 0;
        let mut hc2_violations = 0;
        let mut hc3_violations = 0;
        let mut hc4_violations = 0;
        let mut rest_violations = 0;
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

            // HC4: Leave Requests
            if let Some(ref scenario) = self.context.scenario {
                if let Some(ref leave_requests) = scenario.leave_requests {
                    for leave in leave_requests {
                        if leave.crew_id == *worker_id {
                            // Check overlap
                            if shift.start_hour < leave.end_hour && shift.end_hour() > leave.start_hour {
                                fitness -= 5000.0; // Severe penalty for working during leave
                                hc4_violations += 1;
                            }
                        }
                    }
                }
            }

            *worker_hours.entry(*worker_id).or_insert(0) += shift.duration_hours;
            worker_shifts.entry(*worker_id).or_default().push(shift);
        }

        // Pass 2: Evaluate HC2 (double booking), HC3 (max hours), Rest periods, SC1 (fairness), SC2 (fatigue)
        let mut hours_list = Vec::new();
        for worker in self.context.workers.iter() {
            let hours = *worker_hours.get(&worker.id).unwrap_or(&0);
            hours_list.push(hours as f64);

            // HC3: Max Hours — threshold from scenario.max_hours_per_worker if supplied,
            // otherwise falls back to DEFAULT_WEEKLY_MAX_HOURS (40h) for backward compatibility.
            const DEFAULT_WEEKLY_MAX_HOURS: u64 = 40;
            let hc3_limit = self.context.scenario
                .as_ref()
                .and_then(|s| s.max_hours_per_worker)
                .map(|h| h as u64)
                .unwrap_or(DEFAULT_WEEKLY_MAX_HOURS);
            if hours > hc3_limit {
                fitness -= 500.0;
                hc3_violations += 1;
            }

            // HC2 and Rest period checks
            if let Some(shifts) = worker_shifts.get(&worker.id) {
                let mut sorted_shifts = shifts.clone();
                sorted_shifts.sort_by_key(|s| s.start_hour);
                // HC2: check ALL pairs for overlap (double-booking)
                for i in 0..sorted_shifts.len() {
                    for j in (i + 1)..sorted_shifts.len() {
                        let s_i = sorted_shifts[i];
                        let s_j = sorted_shifts[j];
                        if s_i.overlaps_with(s_j) {
                            fitness -= 1000.0;
                            hc2_violations += 1;
                        }
                    }
                }
                let min_rest = self.context.scenario
                    .as_ref()
                    .and_then(|s| s.minimum_rest_hours)
                    .unwrap_or(10); // DGCA/EASA default

                // Rest: check only CONSECUTIVE shifts (adjacent in time order)
                // A rest violation means the gap between the end of shift[i] and
                // the start of shift[i+1] is less than the scenario minimum.
                for i in 0..sorted_shifts.len().saturating_sub(1) {
                    let s_i = sorted_shifts[i];
                    let s_next = sorted_shifts[i + 1];
                    let gap = if s_next.start_hour >= s_i.end_hour() {
                        s_next.start_hour - s_i.end_hour()
                    } else { 0 };
                    if gap < min_rest {
                        // Penalty scales with severity: short gaps cost more
                        let severity = if gap < (min_rest / 2) { 3.0 } else if gap < (min_rest - 2) { 2.0 } else { 1.0 };
                        fitness -= 800.0 * severity;
                        rest_violations += 1;
                    }
                }
            }

            // SC2: Fatigue (Ecology integration)
            let historical_fatigue = self.context.ecology.get_historical_fatigue(worker.id);
            let fatigue_cost = historical_fatigue * (hours as f64) * 2.0;
            fitness -= fatigue_cost;
            fatigue_penalty += fatigue_cost;
        }

        // SC1: Fairness (Variance of hours)
        if !hours_list.is_empty() {
            let mean = hours_list.iter().sum::<f64>() / hours_list.len() as f64;
            let variance = hours_list.iter().map(|h| (h - mean).powi(2)).sum::<f64>() / hours_list.len() as f64;
            let fairness_cost = variance * 10.0;
            fitness -= fairness_cost;
            fairness_penalty += fairness_cost;
        }

        // Pass 3: Pairing completion — evaluated inside the GA loop as part of the fitness
        // function so the GA actively evolves toward complete, legal pairings.
        //
        // A pairing is the set of shifts sharing the same flight_id. Every flight must have:
        //   - A qualified Captain (skill ends with "-CPT" or crew_role == "Captain")
        //   - A qualified First Officer (skill ends with "-FO" or crew_role == "First Officer")
        //   - All other required crew roles filled with qualified workers
        //
        // Incomplete pairing penalty: -5000 per missing cockpit role, -2000 per unqualified role.
        // Complete pairing reward: +500 per crew member in a fully legal pairing.
        // This makes pairing completion the dominant fitness signal, overriding coverage alone.
        {
            let mut flight_shifts: HashMap<String, Vec<&Shift>> = HashMap::new();
            for shift in self.context.shifts.iter() {
                if let Some(ref fid) = shift.flight_id {
                    flight_shifts.entry(fid.clone()).or_default().push(shift);
                }
            }
            for (_fid, flt_shifts) in &flight_shifts {
                let has_captain = flt_shifts.iter().any(|s| {
                    s.crew_role.as_deref() == Some("Captain") || s.required_skill.0.ends_with("-CPT")
                });
                let has_fo = flt_shifts.iter().any(|s| {
                    s.crew_role.as_deref() == Some("First Officer") || s.required_skill.0.ends_with("-FO")
                });
                // Check all assigned workers are qualified for their shift
                let all_qualified = flt_shifts.iter().all(|shift| {
                    let worker_id = genome.assignments.get(&shift.id).unwrap();
                    let worker = self.context.workers.iter().find(|w| w.id == *worker_id).unwrap();
                    worker.skills.contains(&shift.required_skill)
                });
                if !has_captain {
                    fitness -= 5000.0;
                }
                if !has_fo {
                    fitness -= 5000.0;
                }
                if !all_qualified {
                    fitness -= 2000.0;
                }
                if has_captain && has_fo && all_qualified {
                    // Reward complete, legal pairings — incentivises the GA to fill all roles
                    fitness += 500.0 * (flt_shifts.len() as f64);
                }
            }
        }

        // Base reward for completing the schedule
        fitness += 10000.0;

        let mut violated_constraints = Vec::new();
        let mut satisfied_constraints = Vec::new();
        let mut warnings = Vec::new();
        let mut constraint_scores = HashMap::new();

        // HC1: Skill Match
        constraint_scores.insert("HC1".to_string(), (hc1_violations as f64) * 1000.0);
        if hc1_violations > 0 {
            violated_constraints.push("HC1".to_string());
        } else {
            satisfied_constraints.push("HC1".to_string());
        }

        // HC2: Double Booking
        constraint_scores.insert("HC2".to_string(), (hc2_violations as f64) * 1000.0);
        if hc2_violations > 0 {
            violated_constraints.push("HC2".to_string());
        } else {
            satisfied_constraints.push("HC2".to_string());
        }

        // HC3: Max Hours
        constraint_scores.insert("HC3".to_string(), (hc3_violations as f64) * 500.0);
        if hc3_violations > 0 {
            violated_constraints.push("HC3".to_string());
        } else {
            satisfied_constraints.push("HC3".to_string());
        }

        // HC4: Leave Violations
        constraint_scores.insert("HC4".to_string(), (hc4_violations as f64) * 5000.0);
        if hc4_violations > 0 {
            violated_constraints.push("HC4".to_string());
        } else {
            satisfied_constraints.push("HC4".to_string());
        }

        // Rest period checks
        constraint_scores.insert("Rest".to_string(), (rest_violations as f64) * 200.0);
        if rest_violations > 0 {
            violated_constraints.push("Rest".to_string());
        } else {
            satisfied_constraints.push("Rest".to_string());
        }

        // SC1: Fairness
        constraint_scores.insert("SC1".to_string(), fairness_penalty);
        if fairness_penalty > 0.0 {
            violated_constraints.push("SC1".to_string());
        } else {
            satisfied_constraints.push("SC1".to_string());
        }

        // SC2: Fatigue
        constraint_scores.insert("SC2".to_string(), fatigue_penalty);
        if fatigue_penalty > 0.0 {
            violated_constraints.push("SC2".to_string());
        } else {
            satisfied_constraints.push("SC2".to_string());
        }

        // Warnings generation
        for (w_id, &hours) in &worker_hours {
            if hours > 35 && hours <= 40 {
                warnings.push(format!("Worker {} is near weekly hours limit: {} hours", w_id, hours));
            }
        }
        if fatigue_penalty > 100.0 {
            warnings.push(format!("High cumulative fatigue penalty: {:.2}", fatigue_penalty));
        }

        let hard_violations = hc1_violations + hc2_violations + hc3_violations + hc4_violations + rest_violations;
        let soft_violations = if fairness_penalty > 0.0 { 1 } else { 0 } + if fatigue_penalty > 0.0 { 1 } else { 0 };

        ConstraintReport {
            fitness,
            is_valid: true,
            hard_violations,
            soft_violations,
            warnings,
            constraint_scores,
            violated_constraints,
            satisfied_constraints,
            hc1_violations,
            hc2_violations,
            hc3_violations,
            hc4_violations,
            rest_violations,
            fairness_penalty,
            fatigue_penalty,
        }
    }
}

/// Validate the scheduling context (dataset).
/// Returns Ok(()) if the dataset is valid, otherwise an Err with a description.
pub fn validate_context(context: &ScheduleContext) -> Result<(), Box<dyn Error>> {
    // Ensure there is at least one worker and one shift.
    if context.workers.is_empty() {
        return Err("No workers provided".into());
    }
    if context.shifts.is_empty() {
        return Err("No shifts provided".into());
    }
    // Check that each shift has at least one worker with the required skill.
    for shift in context.shifts.iter() {
        let mut has_worker = false;
        for worker in context.workers.iter() {
            if worker.skills.contains(&shift.required_skill) {
                has_worker = true;
                break;
            }
        }
        if !has_worker {
            return Err(format!("No worker possesses required skill for shift {}", shift.id).into());
        }
    }
    // Ensure worker IDs are unique.
    let mut ids = HashSet::new();
    for w in context.workers.iter() {
        if !ids.insert(w.id) {
            return Err(format!("Duplicate worker id {}", w.id).into());
        }
    }
    // Ensure shift IDs are unique and start hour sanity.
    // Horizon is derived from the dataset (max start_hour + 1); only start_hour is checked.
    let horizon: u64 = context.shifts.iter().map(|s| s.start_hour).max().unwrap_or(0) + 1;
    let mut shift_ids = HashSet::new();
    for s in context.shifts.iter() {
        if !shift_ids.insert(s.id) {
            return Err(format!("Duplicate shift id {}", s.id).into());
        }
        if s.start_hour >= horizon {
            return Err(format!("Shift {} start hour out of range", s.id).into());
        }
    }
    Ok(())
}

/// Validate a completed schedule solution against hard constraints.
/// Returns true if the schedule satisfies all hard constraints.
pub fn validate_schedule(solution: &crate::schedule_solution::ScheduleSolution) -> bool {
    solution.hard_violations == 0
}
