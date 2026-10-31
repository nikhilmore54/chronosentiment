use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

use crate::models::Shift;
use crate::public_contracts::ScheduleRequest;
use crate::constraint_engine::{DomainConstraintEvaluator, InrcConstraintEvaluator};
use crate::optimization::ScheduleGenome;

/// An individual worker-to-position assignment from `hospital_baseline.csv`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaselineAssignment {
    pub worker_id: String,
    pub shift_id: u64,
}

/// Request payload for `POST /api/audit/baseline`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineAuditRequest {
    pub request: ScheduleRequest,
    #[serde(default)]
    pub baseline_assignments: Option<Vec<BaselineAssignment>>,
    #[serde(default)]
    pub baseline_csv: Option<String>,
    #[serde(default)]
    pub policy_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaselineFeasibility {
    pub is_feasible: bool,
    pub hard_violations: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaselineCoverage {
    pub required_positions: usize,
    pub filled_positions: usize,
    pub unfilled_positions: usize,
    pub coverage_pct: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignmentIntegrityReport {
    pub duplicate_assignments: usize,
    pub unknown_worker_errors: usize,
    pub unknown_workers: Vec<String>,
    pub unmatched_excess_assignments: usize,
    pub unmatched_shift_ids: Vec<u64>,
    pub total_baseline_rows: usize,
    pub valid_assignments_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditFinding {
    pub code: String,
    pub severity: String,
    pub description: String,
    pub worker_id: Option<String>,
    pub shift_id: Option<u64>,
    pub day_idx: Option<usize>,
}

/// Comprehensive, independent audit report for a hospital's historical baseline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaselineAuditReport {
    pub audit_id: String,
    pub status: String,
    pub policy_version: String,
    pub policy_retrospective_note: String,
    pub feasibility: BaselineFeasibility,
    pub coverage: BaselineCoverage,
    pub rest_violations: usize,
    pub weekly_contract_violations: usize,
    pub skill_coverage_deficits: usize,
    pub senior_coverage_deficits: usize,
    pub scheduled_overtime_hours: f64,
    pub workload_variance: f64,
    pub assignment_integrity: AssignmentIntegrityReport,
    pub violated_constraints: Vec<String>,
    pub satisfied_constraints: Vec<String>,
    pub detailed_findings: Vec<AuditFinding>,
}

/// Parse `hospital_baseline.csv` content (columns: `worker_id,shift_id`).
/// Retains all lines and detects duplicate rows or parsing errors.
pub fn parse_baseline_csv(csv_text: &str) -> Result<Vec<BaselineAssignment>, Vec<String>> {
    let mut assignments = Vec::new();
    let mut errors = Vec::new();

    let lines: Vec<&str> = csv_text.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    if lines.is_empty() {
        return Err(vec!["Baseline CSV is empty.".to_string()]);
    }

    let header_line = lines[0].to_lowercase();
    let headers: Vec<&str> = header_line.split(',').map(|s| s.trim().trim_matches('"')).collect();

    let worker_col = headers.iter().position(|&h| h == "worker_id" || h == "worker" || h == "nurse_id" || h == "staff_id");
    let shift_col = headers.iter().position(|&h| h == "shift_id" || h == "shift");

    if worker_col.is_none() || shift_col.is_none() {
        return Err(vec![format!(
            "Missing required CSV columns. Required: worker_id, shift_id. Found headers: {:?}",
            headers
        )]);
    }

    let w_idx = worker_col.unwrap();
    let s_idx = shift_col.unwrap();

    for (row_idx, line) in lines.iter().enumerate().skip(1) {
        let cols: Vec<&str> = line.split(',').map(|s| s.trim().trim_matches('"')).collect();
        if cols.len() <= w_idx || cols.len() <= s_idx {
            errors.push(format!("Row {}: insufficient columns in line: '{}'", row_idx + 1, line));
            continue;
        }

        let worker_id = cols[w_idx].to_string();
        let shift_id_str = cols[s_idx];

        if worker_id.is_empty() {
            errors.push(format!("Row {}: empty worker_id", row_idx + 1));
            continue;
        }

        match shift_id_str.parse::<u64>() {
            Ok(shift_id) => {
                assignments.push(BaselineAssignment { worker_id, shift_id });
            }
            Err(_) => {
                errors.push(format!("Row {}: invalid numeric shift_id '{}'", row_idx + 1, shift_id_str));
            }
        }
    }

    if !errors.is_empty() && assignments.is_empty() {
        Err(errors)
    } else {
        Ok(assignments)
    }
}

/// Independently audit a historical hospital baseline roster against the approved planning request.
/// Re-computes all hard constraints, coverage, scheduled overtime, and assignment integrity
/// without invoking optimization or trusting cached scores.
pub fn audit_hospital_baseline(req: &BaselineAuditRequest) -> Result<BaselineAuditReport, String> {
    let raw_assignments = if let Some(ref list) = req.baseline_assignments {
        list.clone()
    } else if let Some(ref csv) = req.baseline_csv {
        parse_baseline_csv(csv).map_err(|errs| errs.join("; "))?
    } else {
        return Err("No baseline assignments or baseline CSV provided in audit request.".to_string());
    };

    let total_baseline_rows = raw_assignments.len();
    let policy_version = req.policy_version.clone().unwrap_or_else(|| "retrospective-assessment-policy-v1.0".to_string());
    let policy_retrospective_note = format!(
        "Retrospectively assessed against assessment policy version '{}'. Historical allocations are evaluated under the selected hospital rules to establish an objective baseline.",
        policy_version
    );

    // Build worker ID lookup: string worker_id -> numeric u64 id
    let mut worker_id_map: HashMap<String, u64> = HashMap::new();
    let mut numeric_to_string_worker: HashMap<u64, String> = HashMap::new();
    for w in &req.request.workers {
        let str_id = w.id.to_string();
        worker_id_map.insert(str_id.clone(), w.id);
        numeric_to_string_worker.insert(w.id, str_id);
    }

    // Build shift lookup
    let shift_map: HashMap<u64, &Shift> = req.request.shifts.iter().map(|s| (s.id, s)).collect();
    let required_positions = req.request.shifts.len();

    // Track integrity
    let mut duplicate_assignments = 0;
    let mut unknown_workers_set = HashSet::new();
    let mut unmatched_shift_ids = Vec::new();
    let mut detailed_findings = Vec::new();

    let mut seen_worker_shifts = HashSet::new();
    let mut assigned_shifts: HashMap<u64, u64> = HashMap::new(); // shift_id -> worker_numeric_id
    let mut valid_assignments_count = 0;

    for a in &raw_assignments {
        let row_key = format!("{}:{}", a.worker_id, a.shift_id);
        if seen_worker_shifts.contains(&row_key) {
            duplicate_assignments += 1;
            detailed_findings.push(AuditFinding {
                code: "INTEG-DUP-ROW".to_string(),
                severity: "Hard".to_string(),
                description: format!("Duplicate assignment row in baseline: worker {} assigned to shift {} multiple times", a.worker_id, a.shift_id),
                worker_id: Some(a.worker_id.clone()),
                shift_id: Some(a.shift_id),
                day_idx: shift_map.get(&a.shift_id).map(|s| (s.start_hour / 24) as usize),
            });
            continue;
        }
        seen_worker_shifts.insert(row_key);

        let worker_num = match worker_id_map.get(&a.worker_id) {
            Some(&id) => id,
            None => {
                unknown_workers_set.insert(a.worker_id.clone());
                detailed_findings.push(AuditFinding {
                    code: "INTEG-UNKNOWN-WORKER".to_string(),
                    severity: "Hard".to_string(),
                    description: format!("Worker '{}' in baseline roster does not exist in staff contract roster", a.worker_id),
                    worker_id: Some(a.worker_id.clone()),
                    shift_id: Some(a.shift_id),
                    day_idx: None,
                });
                continue;
            }
        };

        if !shift_map.contains_key(&a.shift_id) {
            unmatched_shift_ids.push(a.shift_id);
            detailed_findings.push(AuditFinding {
                code: "INTEG-EXCESS-SHIFT".to_string(),
                severity: "Warning".to_string(),
                description: format!("Shift ID {} in baseline does not match any required position in canonical demand (excess historical duty)", a.shift_id),
                worker_id: Some(a.worker_id.clone()),
                shift_id: Some(a.shift_id),
                day_idx: None,
            });
            continue;
        }

        if let Some(existing_worker) = assigned_shifts.get(&a.shift_id) {
            duplicate_assignments += 1;
            detailed_findings.push(AuditFinding {
                code: "INTEG-DOUBLE-COVERAGE".to_string(),
                severity: "Hard".to_string(),
                description: format!("Shift {} double-assigned to both worker {} and worker {}", a.shift_id, existing_worker, worker_num),
                worker_id: Some(a.worker_id.clone()),
                shift_id: Some(a.shift_id),
                day_idx: shift_map.get(&a.shift_id).map(|s| (s.start_hour / 24) as usize),
            });
            continue;
        }

        assigned_shifts.insert(a.shift_id, worker_num);
        valid_assignments_count += 1;
    }

    let unknown_worker_errors = unknown_workers_set.len();
    let unmatched_excess_assignments = unmatched_shift_ids.len();
    let unknown_workers: Vec<String> = unknown_workers_set.into_iter().collect();

    // Coverage calculation
    let filled_positions = assigned_shifts.len();
    let unfilled_positions = required_positions.saturating_sub(filled_positions);
    let coverage_pct = if required_positions > 0 {
        (filled_positions as f64 / required_positions as f64 * 1000.0).round() / 10.0
    } else {
        100.0
    };

    for (&shift_id, shift) in &shift_map {
        if !assigned_shifts.contains_key(&shift_id) {
            detailed_findings.push(AuditFinding {
                code: "COVERAGE-GAP".to_string(),
                severity: "Hard".to_string(),
                description: format!("Shift {} (day {}, skill {}) unstaffed in baseline roster", shift_id, shift.start_hour / 24, shift.required_skill.0),
                worker_id: None,
                shift_id: Some(shift_id),
                day_idx: Some((shift.start_hour / 24) as usize),
            });
        }
    }

    // Evaluate constraints using independent InrcConstraintEvaluator
    let context = req.request.to_context();
    let constraint_evaluator = InrcConstraintEvaluator::new(context);
    let genome = ScheduleGenome {
        assignments: assigned_shifts.clone(),
    };
    let report = constraint_evaluator.evaluate(&genome);

    // Identify senior coverage deficits
    let mut senior_coverage_deficits = 0;
    for (shift_id, worker_id) in &assigned_shifts {
        if let Some(shift) = shift_map.get(shift_id) {
            if shift.required_skill.0.to_lowercase().contains("head") || shift.required_skill.0.to_lowercase().contains("senior") {
                if let Some(w) = req.request.workers.iter().find(|w| w.id == *worker_id) {
                    if !w.skills.contains(&shift.required_skill) {
                        senior_coverage_deficits += 1;
                    }
                }
            }
        }
    }

    // Scheduled Overtime Calculation
    // Sum of hours scheduled above weekly contract cap per 168-hour contractual week
    let hc3_limit = req.request.scenario.as_ref()
        .and_then(|s| s.max_hours_per_worker)
        .unwrap_or(40.0) as u64;

    let mut worker_weekly_hours: HashMap<(u64, usize), u64> = HashMap::new();
    let mut total_worker_hours: HashMap<u64, u64> = HashMap::new();

    for (&shift_id, &worker_id) in &assigned_shifts {
        if let Some(shift) = shift_map.get(&shift_id) {
            let week_idx = (shift.start_hour / 168) as usize;
            *worker_weekly_hours.entry((worker_id, week_idx)).or_insert(0) += shift.duration_hours;
            *total_worker_hours.entry(worker_id).or_insert(0) += shift.duration_hours;
        }
    }

    let mut scheduled_overtime_hours = 0.0;
    for ((worker_id, week_idx), hours) in &worker_weekly_hours {
        if *hours > hc3_limit {
            let ot = (*hours - hc3_limit) as f64;
            scheduled_overtime_hours += ot;
            detailed_findings.push(AuditFinding {
                code: "HC3-OVERTIME".to_string(),
                severity: "Hard".to_string(),
                description: format!("Worker {} scheduled for {}h in contractual week {} (exceeds {}h cap by {}h)", worker_id, hours, week_idx + 1, hc3_limit, ot),
                worker_id: numeric_to_string_worker.get(worker_id).cloned(),
                shift_id: None,
                day_idx: Some(week_idx * 7),
            });
        }
    }

    // Workload variance
    let all_hours: Vec<f64> = req.request.workers.iter().map(|w| *total_worker_hours.get(&w.id).unwrap_or(&0) as f64).collect();
    let workload_variance = if !all_hours.is_empty() {
        let mean = all_hours.iter().sum::<f64>() / all_hours.len() as f64;
        let var = all_hours.iter().map(|h| (h - mean).powi(2)).sum::<f64>() / all_hours.len() as f64;
        (var * 10.0).round() / 10.0
    } else {
        0.0
    };

    // Rest violations details
    let min_rest = req.request.scenario.as_ref().and_then(|s| s.minimum_rest_hours).unwrap_or(10);
    for w in &req.request.workers {
        let mut shifts: Vec<&Shift> = assigned_shifts.iter()
            .filter(|(_, &w_id)| w_id == w.id)
            .filter_map(|(&s_id, _)| shift_map.get(&s_id).copied())
            .collect();
        shifts.sort_by_key(|s| s.start_hour);

        for i in 0..shifts.len().saturating_sub(1) {
            let s_cur = shifts[i];
            let s_next = shifts[i + 1];
            let gap = if s_next.start_hour >= s_cur.end_hour() {
                s_next.start_hour - s_cur.end_hour()
            } else {
                0
            };
            if gap < min_rest {
                detailed_findings.push(AuditFinding {
                    code: "REST-DEFICIT".to_string(),
                    severity: "Hard".to_string(),
                    description: format!("Rest deficit for worker {}: {}h rest between shift {} (end {}h) and shift {} (start {}h) < mandatory {}h", w.id, gap, s_cur.id, s_cur.end_hour(), s_next.id, s_next.start_hour, min_rest),
                    worker_id: Some(w.id.to_string()),
                    shift_id: Some(s_next.id),
                    day_idx: Some((s_next.start_hour / 24) as usize),
                });
            }
        }
    }

    // Skill coverage deficits details
    for (&shift_id, &worker_id) in &assigned_shifts {
        if let Some(shift) = shift_map.get(&shift_id) {
            if let Some(w) = req.request.workers.iter().find(|w| w.id == worker_id) {
                if !w.skills.contains(&shift.required_skill) {
                    detailed_findings.push(AuditFinding {
                        code: "HC1-SKILL-DEFICIT".to_string(),
                        severity: "Hard".to_string(),
                        description: format!("Worker {} lacks required skill '{}' for shift {}", worker_id, shift.required_skill.0, shift_id),
                        worker_id: Some(worker_id.to_string()),
                        shift_id: Some(shift_id),
                        day_idx: Some((shift.start_hour / 24) as usize),
                    });
                }
            }
        }
    }

    let mut violated_constraints = Vec::new();
    let mut satisfied_constraints = Vec::new();

    if report.rest_violations > 0 { violated_constraints.push("REST_PERIOD".to_string()); } else { satisfied_constraints.push("REST_PERIOD".to_string()); }
    if report.hc1_violations > 0 { violated_constraints.push("HC1_SKILL_COVERAGE".to_string()); } else { satisfied_constraints.push("HC1_SKILL_COVERAGE".to_string()); }
    if report.hc2_violations > 0 || duplicate_assignments > 0 { violated_constraints.push("HC2_OVERLAP_DOUBLE_BOOKING".to_string()); } else { satisfied_constraints.push("HC2_OVERLAP_DOUBLE_BOOKING".to_string()); }
    if report.hc3_violations > 0 { violated_constraints.push("HC3_WEEKLY_CONTRACT_MAX".to_string()); } else { satisfied_constraints.push("HC3_WEEKLY_CONTRACT_MAX".to_string()); }
    if unfilled_positions > 0 { violated_constraints.push("FULL_DEMAND_COVERAGE".to_string()); } else { satisfied_constraints.push("FULL_DEMAND_COVERAGE".to_string()); }
    if unknown_worker_errors > 0 { violated_constraints.push("VALID_WORKFORCE_IDENTIFIERS".to_string()); } else { satisfied_constraints.push("VALID_WORKFORCE_IDENTIFIERS".to_string()); }

    let assignment_integrity_errors = duplicate_assignments + unknown_worker_errors;
    let total_hard_violations = report.hard_violations + unfilled_positions + assignment_integrity_errors;
    let is_feasible = total_hard_violations == 0;

    let audit_id = format!("baseline-audit-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0));

    Ok(BaselineAuditReport {
        audit_id,
        status: "completed".to_string(),
        policy_version,
        policy_retrospective_note,
        feasibility: BaselineFeasibility {
            is_feasible,
            hard_violations: total_hard_violations,
        },
        coverage: BaselineCoverage {
            required_positions,
            filled_positions,
            unfilled_positions,
            coverage_pct,
        },
        rest_violations: report.rest_violations,
        weekly_contract_violations: report.hc3_violations,
        skill_coverage_deficits: report.hc1_violations,
        senior_coverage_deficits,
        scheduled_overtime_hours,
        workload_variance,
        assignment_integrity: AssignmentIntegrityReport {
            duplicate_assignments,
            unknown_worker_errors,
            unknown_workers,
            unmatched_excess_assignments,
            unmatched_shift_ids,
            total_baseline_rows,
            valid_assignments_count,
        },
        violated_constraints,
        satisfied_constraints,
        detailed_findings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Skill, Worker};
    use crate::public_contracts::InrcScenario;

    fn build_test_setup() -> (ScheduleRequest, String) {
        let workers = vec![
            Worker { id: 1, skills: vec![Skill::new("HeadNurse"), Skill::new("Nurse")] },
            Worker { id: 2, skills: vec![Skill::new("Nurse")] },
            Worker { id: 3, skills: vec![Skill::new("Nurse")] },
        ];
        let shifts = vec![
            Shift { id: 101, start_hour: 8, duration_hours: 8, required_skill: Skill::new("HeadNurse") },
            Shift { id: 102, start_hour: 8, duration_hours: 8, required_skill: Skill::new("Nurse") },
            Shift { id: 103, start_hour: 16, duration_hours: 8, required_skill: Skill::new("Nurse") },
        ];
        let scenario = InrcScenario {
            planning_horizon_hours: Some(168.0),
            validation_envelope_hours: Some(168.0),
            max_hours_per_worker: Some(40.0),
            minimum_rest_hours: Some(10),
            leave_requests: None,
        };
        let request = ScheduleRequest {
            workers,
            shifts,
            scenario: Some(scenario),
            ..Default::default()
        };
        let baseline_csv = "worker_id,shift_id\n1,101\n2,102\n3,103\n".to_string();
        (request, baseline_csv)
    }

    #[test]
    fn test_at21_valid_baseline_imports_correctly() {
        let (request, baseline_csv) = build_test_setup();
        let audit_req = BaselineAuditRequest {
            request,
            baseline_csv: Some(baseline_csv),
            baseline_assignments: None,
            policy_version: Some("policy-v1.0".to_string()),
        };
        let report = audit_hospital_baseline(&audit_req).expect("Audit should succeed");
        assert!(report.feasibility.is_feasible);
        assert_eq!(report.feasibility.hard_violations, 0);
        assert_eq!(report.coverage.filled_positions, 3);
        assert_eq!(report.coverage.unfilled_positions, 0);
        assert_eq!(report.assignment_integrity.duplicate_assignments, 0);
        assert_eq!(report.assignment_integrity.unknown_worker_errors, 0);
    }

    #[test]
    fn test_at22_rest_violations_are_detected() {
        let (mut request, _) = build_test_setup();
        // Add a shift at hour 20 (gap of 4h after shift ending at hour 16 < min 10h)
        request.shifts.push(Shift { id: 104, start_hour: 20, duration_hours: 8, required_skill: Skill::new("Nurse") });
        // Assign worker 3 to both shift 103 (16-24) and shift 104 (20-28) -> rest deficit & overlap
        let baseline_csv = "worker_id,shift_id\n1,101\n2,102\n3,103\n3,104\n".to_string();
        let audit_req = BaselineAuditRequest {
            request,
            baseline_csv: Some(baseline_csv),
            baseline_assignments: None,
            policy_version: None,
        };
        let report = audit_hospital_baseline(&audit_req).expect("Audit should succeed");
        assert!(!report.feasibility.is_feasible);
        assert!(report.rest_violations > 0 || report.detailed_findings.iter().any(|f| f.code.contains("REST")));
    }

    #[test]
    fn test_at24_missing_skill_detected() {
        let (request, _) = build_test_setup();
        // Worker 2 only has "Nurse", but assigned to shift 101 which requires "HeadNurse"
        let baseline_csv = "worker_id,shift_id\n2,101\n1,102\n3,103\n".to_string();
        let audit_req = BaselineAuditRequest {
            request,
            baseline_csv: Some(baseline_csv),
            baseline_assignments: None,
            policy_version: None,
        };
        let report = audit_hospital_baseline(&audit_req).expect("Audit should succeed");
        assert!(!report.feasibility.is_feasible);
        assert_eq!(report.skill_coverage_deficits, 1);
        assert!(report.violated_constraints.contains(&"HC1_SKILL_COVERAGE".to_string()));
    }

    #[test]
    fn test_at25_uncovered_demand_positions_identified() {
        let (request, _) = build_test_setup();
        // Only 2 of 3 shifts assigned in baseline
        let baseline_csv = "worker_id,shift_id\n1,101\n2,102\n".to_string();
        let audit_req = BaselineAuditRequest {
            request,
            baseline_csv: Some(baseline_csv),
            baseline_assignments: None,
            policy_version: None,
        };
        let report = audit_hospital_baseline(&audit_req).expect("Audit should succeed");
        assert!(!report.feasibility.is_feasible);
        assert_eq!(report.coverage.filled_positions, 2);
        assert_eq!(report.coverage.unfilled_positions, 1);
        assert!(report.violated_constraints.contains(&"FULL_DEMAND_COVERAGE".to_string()));
    }

    #[test]
    fn test_at26_duplicate_assignments_detected() {
        let (request, _) = build_test_setup();
        // Shift 101 assigned twice (to worker 1 and worker 2)
        let baseline_csv = "worker_id,shift_id\n1,101\n2,101\n3,103\n".to_string();
        let audit_req = BaselineAuditRequest {
            request,
            baseline_csv: Some(baseline_csv),
            baseline_assignments: None,
            policy_version: None,
        };
        let report = audit_hospital_baseline(&audit_req).expect("Audit should succeed");
        assert!(!report.feasibility.is_feasible);
        assert!(report.assignment_integrity.duplicate_assignments > 0);
    }

    #[test]
    fn test_at27_unknown_worker_and_shift_ids_produce_structured_errors() {
        let (request, _) = build_test_setup();
        // Worker 999 does not exist, shift 888 does not exist
        let baseline_csv = "worker_id,shift_id\n999,101\n2,888\n3,103\n".to_string();
        let audit_req = BaselineAuditRequest {
            request,
            baseline_csv: Some(baseline_csv),
            baseline_assignments: None,
            policy_version: None,
        };
        let report = audit_hospital_baseline(&audit_req).expect("Audit should succeed");
        assert!(!report.feasibility.is_feasible);
        assert_eq!(report.assignment_integrity.unknown_worker_errors, 1);
        assert!(report.assignment_integrity.unknown_workers.contains(&"999".to_string()));
        assert_eq!(report.assignment_integrity.unmatched_excess_assignments, 1);
        assert!(report.assignment_integrity.unmatched_shift_ids.contains(&888));
    }

    #[test]
    fn test_at31_excess_baseline_assignments_retained_and_reported() {
        let (request, _) = build_test_setup();
        // Shift 999 is an extra historical shift outside the canonical demand
        let baseline_csv = "worker_id,shift_id\n1,101\n2,102\n3,103\n1,999\n".to_string();
        let audit_req = BaselineAuditRequest {
            request,
            baseline_csv: Some(baseline_csv),
            baseline_assignments: None,
            policy_version: None,
        };
        let report = audit_hospital_baseline(&audit_req).expect("Audit should succeed");
        assert_eq!(report.assignment_integrity.unmatched_excess_assignments, 1);
        assert_eq!(report.assignment_integrity.unmatched_shift_ids, vec![999]);
        assert_eq!(report.assignment_integrity.total_baseline_rows, 4);
    }

    #[test]
    fn test_at32_policy_version_and_retrospective_note_identified() {
        let (request, baseline_csv) = build_test_setup();
        let audit_req = BaselineAuditRequest {
            request,
            baseline_csv: Some(baseline_csv),
            baseline_assignments: None,
            policy_version: Some("hospital-approved-policy-v2.1".to_string()),
        };
        let report = audit_hospital_baseline(&audit_req).expect("Audit should succeed");
        assert_eq!(report.policy_version, "hospital-approved-policy-v2.1");
        assert!(report.policy_retrospective_note.contains("hospital-approved-policy-v2.1"));
    }
}
