use std::collections::HashMap;
use crate::inrc::models::InrcScenario;
use crate::inrc::types::{ValidationReport, ViolationDetail};

/// Validate a schedule against INRC constraints and return a full report.
pub fn validate_schedule(
    schedule: &HashMap<String, Vec<String>>,
    scenario: &InrcScenario,
) -> ValidationReport {
    let mut max_consecutive_work_violations = 0;
    let mut min_consecutive_work_violations = 0;
    let mut min_days_off_violations = 0;
    let mut max_days_off_violations = 0;
    let mut forbidden_successions = 0;

    let num_days = scenario.number_of_weeks * 7;
    let mut total_shifts = 0;
    let target_shifts = 16 * num_days; // 16 nurses per day * num_days

    let mut details = Vec::new();

    for nurse in &scenario.nurses {
        if let Some(shifts) = schedule.get(&nurse.id) {
            let contract = scenario
                .contracts
                .iter()
                .find(|c| c.id == nurse.contract)
                .unwrap();
            let mut current_work_streak = 0;
            let mut current_off_streak = 0;

            for d in 0..=num_days {
                // Go to num_days to evaluate the final streak
                let shift = if d < num_days {
                    shifts[d].clone()
                } else {
                    "".to_string()
                };
                let is_work = d < num_days && !shift.is_empty() && !shift.contains("SICK-");

                if is_work {
                    total_shifts += 1;

                    // Streak changed from Off to Work
                    if current_off_streak > 0 {
                        if current_off_streak < contract.min_consecutive_days_off {
                            min_days_off_violations += 1;
                            details.push(ViolationDetail {
                                nurse_id: nurse.id.clone(),
                                day: d,
                                constraint: "min_consecutive_days_off".to_string(),
                                actual: current_off_streak,
                                required: contract.min_consecutive_days_off,
                            });
                        }
                        if current_off_streak > contract.max_consecutive_days_off {
                            max_days_off_violations += 1;
                            details.push(ViolationDetail {
                                nurse_id: nurse.id.clone(),
                                day: d,
                                constraint: "max_consecutive_days_off".to_string(),
                                actual: current_off_streak,
                                required: contract.max_consecutive_days_off,
                            });
                        }
                        current_off_streak = 0;
                    }

                    current_work_streak += 1;
                    if current_work_streak > contract.max_consecutive_working_days {
                        max_consecutive_work_violations += 1;
                        details.push(ViolationDetail {
                            nurse_id: nurse.id.clone(),
                            day: d,
                            constraint: "max_consecutive_working_days".to_string(),
                            actual: current_work_streak,
                            required: contract.max_consecutive_working_days,
                        });
                        // Reset to avoid duplicate reporting on the same long streak
                        current_work_streak = 1;
                    }
                } else {
                    // Streak changed from Work to Off
                    if current_work_streak > 0 {
                        if current_work_streak < contract.min_consecutive_working_days {
                            min_consecutive_work_violations += 1;
                            details.push(ViolationDetail {
                                nurse_id: nurse.id.clone(),
                                day: d,
                                constraint: "min_consecutive_working_days".to_string(),
                                actual: current_work_streak,
                                required: contract.min_consecutive_working_days,
                            });
                        }
                        current_work_streak = 0;
                    }

                    if d < num_days {
                        current_off_streak += 1;
                    }
                }

                // Succession checks
                if d > 0 && d < num_days {
                    let prev = &shifts[d - 1];
                    let curr = &shifts[d];
                    if !prev.is_empty()
                        && !curr.is_empty()
                        && !prev.contains("SICK-")
                        && !curr.contains("SICK-")
                    {
                        let prev_clean = prev
                            .replace("NEW-", "")
                            .replace("RECOVERED-", "")
                            .replace("RETURNED-", "");
                        let curr_clean = curr
                            .replace("NEW-", "")
                            .replace("RECOVERED-", "")
                            .replace("RETURNED-", "");
                        if let Some(rule) = scenario
                            .forbidden_shift_type_successions
                            .iter()
                            .find(|r| r.preceding == prev_clean)
                        {
                            if rule.succeeding.contains(&curr_clean) {
                                forbidden_successions += 1;
                                details.push(ViolationDetail {
                                    nurse_id: nurse.id.clone(),
                                    day: d,
                                    constraint: "forbidden_shift_type_successions".to_string(),
                                    actual: 1,
                                    required: 0,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    let coverage_achieved = (total_shifts as f64 / target_shifts as f64) * 100.0;
    let is_legal = max_consecutive_work_violations == 0
        && min_consecutive_work_violations == 0
        && forbidden_successions == 0
        && min_days_off_violations == 0
        && max_days_off_violations == 0;

    ValidationReport {
        max_consecutive_work_violations,
        min_consecutive_work_violations,
        forbidden_successions,
        min_days_off_violations,
        max_days_off_violations,
        coverage_achieved,
        is_legal,
        details,
    }
}