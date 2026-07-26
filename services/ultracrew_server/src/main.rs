
use axum::{
    routing::{get, post, put, delete},
    Router,
    Json,
    extract::{Path as AxPath, Path, State},
    response::{IntoResponse, Response},
    http::{Method, StatusCode},
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use serde_json::json;
use ultracrew::inrc::parser::{parse_scenario, parse_week_data};
use ultracrew_server::simulation::{SkillCoverageAudit, SkillDeficit, ValidationReport};
use ultracrew::inrc::models::{InrcScenario, InrcNurse};
use std::path::{Path as StdPath, PathBuf};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use ultracrew_server::models::{DecisionCase, ScheduleVersion, DecisionLog};
use ultracrew_server::persistence::{load_collection, save_item, delete_item};
use ultracrew::inrc::validator::validate_schedule;
use ultracrew_server::simulation::{ConstraintAudit, WorkloadAudit, FeasibilityReport, ParetoFrontierSolution, Bottleneck, can_recover, NurseBalance, BalanceChange, SimulationState, Dashboard, Coverage, CoverageAudit, Alert, RosterHealth, BaselineStatus, VerificationReports, RecoveryPlan, CandidateType, BlockedRecovery, RecoveryAudit};



#[derive(Serialize, Deserialize)]
struct StatusResponse {
    status: String,
}

#[derive(Deserialize)]
struct SickLeaveRequest {
    employee_id: String,
    sick_days: Vec<usize>,
}

struct AppState {
    scenario: InrcScenario,
    baseline_state: SimulationState,
    original_state: SimulationState,
    last_solution: Option<ultracrew::schedule_solution::ScheduleSolution>,
    last_request: Option<ultracrew::public_contracts::ScheduleRequest>,
    decisions: Vec<DecisionCase>,
    schedule_versions: Vec<ScheduleVersion>,
}

fn make_dynamic_dashboard(
    schedule: &HashMap<String, Vec<String>>,
    scenario: &InrcScenario,
    week_data: &ultracrew::inrc::models::InrcWeekData,
    validation_report: &ValidationReport,
    feasibility_report: Option<FeasibilityReport>,
    pareto_frontier: Option<Vec<ParetoFrontierSolution>>,
) -> Dashboard {
    let mut skill_deficits = Vec::new();
    let mut total_req_slots = 0;
    let mut filled_req_slots = 0;
    let num_days = (scenario.number_of_weeks * 7) as usize;
    
    for d in 0..num_days {
        let weekday = d % 7;
        for req in &week_data.requirements {
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
            
            if required > 0 {
                let mapped_shift = req.shift_type.as_str();
                
                let mut assigned = 0;
                for nurse in &scenario.nurses {
                    let sched_day = &schedule[&nurse.id][d];
                    if sched_day == mapped_shift || sched_day.ends_with(&format!("-{}", mapped_shift)) {
                        if nurse.skills.contains(&req.skill) {
                            assigned += 1;
                        }
                    }
                }
                
                total_req_slots += required;
                filled_req_slots += std::cmp::min(assigned, required);
                
                let deficit = required as i32 - assigned as i32;
                if deficit > 0 {
                    skill_deficits.push(SkillDeficit {
                        day: d,
                        shift: req.shift_type.clone(),
                        skill: req.skill.clone(),
                        required,
                        assigned,
                        deficit,
                    });
                }
            }
        }
    }
    
    let mut skill_counts = HashMap::new();
    let mut shift_counts = HashMap::new();
    for d in &skill_deficits {
        *skill_counts.entry(d.skill.clone()).or_insert(0) += d.deficit;
        *shift_counts.entry(d.shift.clone()).or_insert(0) += d.deficit;
    }
    
    let worst_skill = skill_counts.into_iter().max_by_key(|&(_, v)| v).map(|(k, _)| k).unwrap_or_else(|| "None".to_string());
    let worst_shift = shift_counts.into_iter().max_by_key(|&(_, v)| v).map(|(k, _)| k).unwrap_or_else(|| "None".to_string());
    
    let skill_coverage_percentage = if total_req_slots > 0 {
        (filled_req_slots as f64 / total_req_slots as f64) * 100.0
    } else {
        100.0
    };
    
    let skill_coverage_audit = SkillCoverageAudit {
        skill_coverage_percentage,
        total_skill_deficits: skill_deficits.len(),
        worst_skill,
        worst_shift,
        deficits: skill_deficits,
    };
    
    let mut actual_assignments = 0;
    let mut daily_assignments = vec![0; num_days];
    for shifts in schedule.values() {
        for d in 0..num_days {
            let s = &shifts[d];
            if !s.is_empty() && !s.contains("SICK-") {
                actual_assignments += 1;
                daily_assignments[d] += 1;
            }
        }
    }
    
    let required_assignments = 16 * num_days;
    let coverage_percentage = (actual_assignments as f64 / required_assignments as f64) * 100.0;
    
    let coverage_audit = CoverageAudit {
        required_assignments,
        actual_assignments,
        coverage_percentage,
        daily_assignments,
    };
    
    let mut understaffed = 0;
    let mut critical = 0;
    for def in &skill_coverage_audit.deficits {
        understaffed += def.deficit;
        let weekday = def.day % 7;
        let req = week_data.requirements.iter().find(|r| r.shift_type == def.shift && r.skill == def.skill).unwrap();
        let minimum = match weekday {
            0 => req.monday.minimum,
            1 => req.tuesday.minimum,
            2 => req.wednesday.minimum,
            3 => req.thursday.minimum,
            4 => req.friday.minimum,
            5 => req.saturday.minimum,
            6 => req.sunday.minimum,
            _ => 0,
        };
        if def.assigned < minimum {
            critical += (minimum as i32 - def.assigned as i32).max(0);
        }
    }
    
    let mut constraint_audit = Vec::new();
    for nurse in &scenario.nurses {
        let mut min_work = 0;
        let mut max_work = 0;
        let mut min_off = 0;
        let mut max_off = 0;
        
        for det in &validation_report.details {
            if det.nurse_id == nurse.id {
                if det.constraint == "min_consecutive_working_days" { min_work += 1; }
                if det.constraint == "max_consecutive_working_days" { max_work += 1; }
                if det.constraint == "min_consecutive_days_off" { min_off += 1; }
                if det.constraint == "max_consecutive_days_off" { max_off += 1; }
            }
        }
        constraint_audit.push(ConstraintAudit {
            nurse_id: nurse.id.clone(),
            min_work_streak_violations: min_work,
            max_work_streak_violations: max_work,
            min_off_streak_violations: min_off,
            max_off_streak_violations: max_off,
        });
    }

    let mut workload_audit = Vec::new();
    for nurse in &scenario.nurses {
        let contract = scenario.contracts.iter().find(|c| c.id == nurse.contract).unwrap();
        let scale = 56.0 / (scenario.number_of_weeks as f64 * 7.0);
        let min_assign = (contract.min_assignments as f64 * scale) as i32;
        let max_assign = (contract.max_assignments as f64 * scale) as i32;
        let expected = (min_assign + max_assign) / 2;
        let actual = schedule[&nurse.id].iter().filter(|s| !s.is_empty() && !s.contains("SICK-")).count() as i32;
        let mut max_work = 0;
        let mut max_off = 0;
        let mut curr_work = 0;
        let mut curr_off = 0;
        for d in 0..num_days {
            let shift = &schedule[&nurse.id][d];
            if !shift.is_empty() && !shift.contains("SICK-") {
                curr_work += 1;
                curr_off = 0;
                if curr_work > max_work { max_work = curr_work; }
            } else {
                curr_off += 1;
                curr_work = 0;
                if curr_off > max_off { max_off = curr_off; }
            }
        }
        
        workload_audit.push(WorkloadAudit {
            nurse_id: nurse.id.clone(),
            expected_assignments: expected,
            actual_assignments: actual,
            deviation: actual - expected,
            max_work_streak: max_work,
            max_off_streak: max_off,
        });
    }

    let legality_score = if validation_report.is_legal { 100 } else { 
        let v = validation_report.details.len() as i32;
        std::cmp::max(0, 100 - (v * 5))
    };
    let coverage_score = coverage_percentage as i32;
    
    let mut max_dev = 0;
    for wa in &workload_audit {
        if wa.deviation.abs() > max_dev { max_dev = wa.deviation.abs(); }
    }
    let balance_score = std::cmp::max(0, 100 - (max_dev * 10));
    let fragmentation_score = std::cmp::max(0, 100 - (validation_report.details.len() as i32 * 2));
    let recovery_score = if validation_report.is_legal { 100 } else { std::cmp::max(0, 100 - (validation_report.details.len() as i32 * 5)) };
    
    let roster_health = RosterHealth {
        legality_score,
        coverage_score,
        balance_score,
        fragmentation_score,
        recovery_score,
    };
    
    let baseline_status = BaselineStatus {
        state: if validation_report.is_legal { "Legal".to_string() } else { "RepairFailed".to_string() },
        is_legal: validation_report.is_legal,
        repair_attempts: 50,
        exhausted_search: !validation_report.is_legal,
    };

    let mut alerts = Vec::new();
    let mut seen_alerts = std::collections::HashSet::new();
    for detail in &validation_report.details {
        let message = match detail.constraint.as_str() {
            "min_consecutive_days_off" => format!("Consecutive days off of {} is below minimum {}", detail.actual, detail.required),
            "max_consecutive_days_off" => format!("Consecutive days off of {} exceeds maximum {}", detail.actual, detail.required),
            "max_consecutive_working_days" => format!("Consecutive working days of {} exceeds maximum {}", detail.actual, detail.required),
            "min_consecutive_working_days" => format!("Consecutive working days of {} is below minimum {}", detail.actual, detail.required),
            "forbidden_shift_type_successions" => "Forbidden shift succession (rest violation)".to_string(),
            _ => format!("Violation of {}", detail.constraint),
        };
        let alert_key = (detail.nurse_id.clone(), message.clone());
        if !seen_alerts.contains(&alert_key) {
            seen_alerts.insert(alert_key);
            let severity = match detail.constraint.as_str() {
                "forbidden_shift_type_successions" => "high".to_string(),
                "max_consecutive_working_days" => "high".to_string(),
                _ => "medium".to_string(),
            };
            alerts.push(Alert {
                employee: detail.nurse_id.clone(),
                severity,
                message,
            });
        }
    }
    
    let mut constraint_report = ultracrew::constraint_engine::ConstraintReport {
        fitness: 0.0,
        is_valid: validation_report.is_legal,
        hard_violations: validation_report.max_consecutive_work_violations 
                       + validation_report.min_consecutive_work_violations 
                       + validation_report.forbidden_successions 
                       + validation_report.min_days_off_violations 
                       + validation_report.max_days_off_violations,
        soft_violations: 0,
        warnings: Vec::new(),
        constraint_scores: HashMap::new(),
        violated_constraints: Vec::new(),
        satisfied_constraints: Vec::new(),
        hc1_violations: skill_coverage_audit.total_skill_deficits,
        hc2_violations: 0,
        hc3_violations: 0,
        rest_violations: validation_report.forbidden_successions + validation_report.min_days_off_violations,
        fairness_penalty: 0.0,
        fatigue_penalty: 0.0,
    };
    
    let rec_engine = ultracrew::recommendation::RecommendationEngine::new();
    let rec_report = rec_engine.generate_recommendations(&constraint_report);
    let mut recommendations = rec_report.into_iter().map(|r| {
        format!("{}: {}", r.explanation, r.recommended_action)
    }).collect::<Vec<String>>();

    if recommendations.is_empty() {
        recommendations.push("Roster is fully legal. All operational constraints and skills coverage requirements are satisfied.".to_string());
        recommendations.push("Monitor fatigue accumulation in upcoming weeks to ensure workload equity remains optimal.".to_string());
    }
    
    Dashboard {
        feasibility_report,
        skill_coverage_audit: Some(skill_coverage_audit),
        coverage: Coverage {
            covered: skill_coverage_percentage as i32,
            understaffed,
            critical,
        },
        coverage_audit,
        alerts,
        recommendations,
        validation_report: validation_report.clone(),
        workload_audit,
        constraint_audit,
        roster_health,
        baseline_status,
        pareto_frontier,
    }
}

async fn list_decision_cases(State(state): State<Arc<Mutex<AppState>>>) -> Json<Vec<DecisionCase>> {
    let app_state = state.lock().unwrap();
    Json(app_state.decisions.clone())
}

#[derive(Deserialize)]
struct NewDecisionCaseInput {
    title: String,
    description: String,
    schedule: Option<std::collections::HashMap<String, Vec<String>>>,
    metadata: Option<serde_json::Value>,
}

async fn create_decision_case(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(input): Json<NewDecisionCaseInput>,
) -> impl IntoResponse {
    let mut app_state = state.lock().unwrap();
    let case = DecisionCase::new(
        input.title,
        input.description,
        input.schedule,
        input.metadata,
    );
    let dir = StdPath::new("data/decision_cases");
    save_item(dir, &case.id, &case);
    app_state.decisions.push(case.clone());
    (StatusCode::CREATED, Json(case))
}

async fn get_decision_case(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let app_state = state.lock().unwrap();
    match app_state.decisions.iter().find(|c| c.id == id) {
        Some(case) => (StatusCode::OK, Json(case.clone())).into_response(),
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Decision case not found"}))).into_response(),
    }
}

#[derive(Deserialize)]
struct UpdateDecisionCaseInput {
    title: Option<String>,
    description: Option<String>,
    schedule: Option<std::collections::HashMap<String, Vec<String>>>,
    metadata: Option<serde_json::Value>,
}

async fn update_decision_case(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<String>,
    Json(input): Json<UpdateDecisionCaseInput>,
) -> Response {
    let mut app_state = state.lock().unwrap();
    if let Some(case) = app_state.decisions.iter_mut().find(|c| c.id == id) {
        if let Some(title) = input.title {
            case.title = title;
        }
        if let Some(description) = input.description {
            case.description = description;
        }
        if let Some(schedule) = input.schedule {
            case.schedule = Some(schedule);
        }
        if let Some(metadata) = input.metadata {
            case.metadata = Some(metadata);
        }
        // Persist updated case
        let dir = StdPath::new("data/decision_cases");
        save_item(dir, &case.id, case);
        (StatusCode::OK, Json(case.clone())).into_response()
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Decision case not found"}))).into_response()
    }
}

async fn delete_decision_case(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut app_state = state.lock().unwrap();
    if let Some(pos) = app_state.decisions.iter().position(|c| c.id == id) {
        app_state.decisions.remove(pos);
        let dir = StdPath::new("data/decision_cases");
        delete_item(dir, &id);
        (StatusCode::NO_CONTENT, Json(serde_json::json!({})))
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Decision case not found"})))
    }
}

#[derive(Deserialize)]
struct CommitScheduleInput {
    schedule: std::collections::HashMap<String, Vec<String>>,
    author: String,
    description: Option<String>,
}

async fn commit_schedule_version(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<String>,
    Json(input): Json<CommitScheduleInput>,
) -> Response {
    let mut app_state = state.lock().unwrap();
    if app_state.decisions.iter().any(|c| c.id == id) {
        let version = ScheduleVersion::new(
            id.clone(),
            input.schedule,
            input.author,
            input.description,
        );
        let dir = StdPath::new("data/schedule_versions");
        save_item(dir, &version.version_id, &version);
        app_state.schedule_versions.push(version.clone());
        (StatusCode::CREATED, Json(version)).into_response()
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Decision case not found"}))).into_response()
    }
}

async fn export_decision_case_csv(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): AxPath<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let app_state = state.lock().unwrap();
    if let Some(case) = app_state.decisions.iter().find(|c| c.id == id) {
        let csv = format!("id,title,description\n{},{},{}\n", case.id, case.title, case.description);
        let header = [(axum::http::header::CONTENT_TYPE, axum::http::HeaderValue::from_static("text/csv"))];
        Ok((StatusCode::OK, header, csv))
    } else {
        Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Decision case not found"}))))
    }
}

async fn health_check() -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "ok".to_string(),
    })
}

async fn get_scenario(State(state): State<Arc<Mutex<AppState>>>) -> Json<InrcScenario> {
    let state = state.lock().unwrap();
    Json(state.scenario.clone())
}

async fn get_state(State(state): State<Arc<Mutex<AppState>>>) -> Json<SimulationState> {
    let state = state.lock().unwrap();
    Json(state.baseline_state.clone())
}

async fn reset_simulation(State(state): State<Arc<Mutex<AppState>>>) -> Json<SimulationState> {
    let mut state = state.lock().unwrap();
    state.baseline_state = state.original_state.clone();
    println!("Reset simulation state on server successfully.");
    Json(state.baseline_state.clone())
}

async fn simulate_sick_leave(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(req): Json<SickLeaveRequest>,
) -> Json<SimulationState> {
    println!("Received SickLeaveRequest for employee {}", req.employee_id);
    let mut state = state.lock().unwrap();
    let mut current_state = state.baseline_state.clone();
    let scenario = &state.scenario;
    let today_index = 14;

    let nurse_id = &req.employee_id;
    let sick_nurse = match scenario.nurses.iter().find(|n| n.id == *nurse_id) {
        Some(n) => n,
        None => {
            println!("Nurse {} not found in scenario!", nurse_id);
            return Json(current_state);
        }
    };
    
    let mut new_schedule = current_state.schedule.clone();
    let mut missed_total = 0;
    let mut creditors: HashMap<String, i32> = HashMap::new();
    
    for &day_index in &req.sick_days {
        let shifts = new_schedule.get_mut(nurse_id).unwrap();
        let shift = shifts[day_index].clone();
        
        if !shift.is_empty() && !shift.contains("SICK-") && day_index >= today_index {
            shifts[day_index] = format!("SICK-{}", shift);
            missed_total += 1;
            
            let mut pool: Vec<&ultracrew::inrc::models::InrcNurse> = scenario.nurses.iter().filter(|n| {
                n.id != *nurse_id && new_schedule[&n.id][day_index].is_empty() && n.skills[0] == sick_nurse.skills[0]
            }).collect();
            
            if pool.is_empty() {
                pool = scenario.nurses.iter().filter(|n| {
                    n.id != *nurse_id && new_schedule[&n.id][day_index].is_empty()
                }).collect();
            }
            
            if !pool.is_empty() {
                let replacement = pool[0];
                new_schedule.get_mut(&replacement.id).unwrap()[day_index] = format!("NEW-{}", shift);
                *creditors.entry(replacement.id.clone()).or_insert(0) += 1;
            }
        }
    }
    
    let sickness_validation = validate_schedule(&new_schedule, scenario);
    
    let recovery_start = today_index + 7;
    let mut max_recovery_day = today_index;
    let mut recovered_shifts = 0;
    let mut requested_shifts = 0;
    let mut feasible_shifts = 0;
    
    let mut creditors_to_process = creditors.clone();
    
    let mut blocked_recoveries = Vec::new();
    
    let mut audit_trail = Vec::new();
    let mut remaining_to_recover = missed_total;
    requested_shifts = missed_total;
    
    let get_balance = |n_id: &String, sched: &HashMap<String, Vec<String>>| -> i32 {
        let base = current_state.balances.iter().find(|b| b.nurse_id == *n_id).map(|b| b.balance).unwrap_or(0);
        let shifts = sched.get(n_id).unwrap();
        let sick_count = shifts.iter().filter(|s| s.starts_with("SICK-")).count() as i32;
        let recovered_count = shifts.iter().filter(|s| s.starts_with("RECOVERED-")).count() as i32;
        let new_count = shifts.iter().filter(|s| s.starts_with("NEW-")).count() as i32;
        let returned_count = shifts.iter().filter(|s| s.starts_with("RETURNED-")).count() as i32;
        base - sick_count + recovered_count + new_count - returned_count
    };

    let num_days = (scenario.number_of_weeks * 7) as usize;
    for day_index in recovery_start..num_days {
        if remaining_to_recover <= 0 { break; }
        
        let sick_nurse_shifts = new_schedule.get(nurse_id).unwrap();
        if !sick_nurse_shifts[day_index].is_empty() {
            continue;
        }
        
        let shift_candidates = vec!["Early".to_string(), "Late".to_string(), "Night".to_string()];
        
        for shift_cand in shift_candidates {
            if remaining_to_recover <= 0 { break; }
            
            let mut sick_nurse_legal = true;
            let mut blocked_reason = None;
            
            match can_recover(nurse_id, day_index, &shift_cand, &new_schedule, scenario) {
                Ok(_) => {},
                Err(reason) => {
                    sick_nurse_legal = false;
                    blocked_reason = Some(reason.clone());
                    blocked_recoveries.push(BlockedRecovery {
                        day: day_index,
                        reason: reason.clone(),
                        constraint: if reason.contains("succession") { "ShiftSuccession".to_string() } else if reason.contains("consecutive") { "Streak".to_string() } else { "Other".to_string() },
                    });
                }
            }
            
            let mut creditor_opt = None;
            let mut candidate_type = CandidateType::OpenSlot;
            
            for (cred_id, &shifts_owed) in &creditors {
                if shifts_owed > 0 {
                    let cred_shift = new_schedule[cred_id][day_index].clone();
                    let clean_shift = cred_shift.replace("NEW-", "").replace("RECOVERED-", "").replace("RETURNED-", "");
                    if clean_shift == shift_cand {
                        creditor_opt = Some(cred_id.clone());
                        candidate_type = CandidateType::CreditorSwap;
                        break;
                    }
                }
            }
            
            if creditor_opt.is_none() {
                let mut currently_assigned = 0;
                for shifts in new_schedule.values() {
                    let s = &shifts[day_index];
                    if !s.is_empty() && !s.contains("SICK-") {
                        currently_assigned += 1;
                    }
                }
                if currently_assigned < 16 {
                    candidate_type = CandidateType::CoverageGap;
                }
            }
            
            let creditor_legal = true;
            let mut accepted = false;
            
            let current_imbalance_sick = get_balance(nurse_id, &new_schedule).abs();
            let mut current_imbalance_cred = 0;
            if let Some(ref c_id) = creditor_opt {
                current_imbalance_cred = get_balance(c_id, &new_schedule).abs();
            }
            let total_current_imbalance = current_imbalance_sick + current_imbalance_cred;
            
            if sick_nurse_legal {
                new_schedule.get_mut(nurse_id).unwrap()[day_index] = format!("RECOVERED-{}", shift_cand);
                if let Some(ref c_id) = creditor_opt {
                    let old_shift = new_schedule[c_id][day_index].clone();
                    new_schedule.get_mut(c_id).unwrap()[day_index] = format!("RETURNED-{}", old_shift);
                }
                
                let next_imbalance_sick = get_balance(nurse_id, &new_schedule).abs();
                let mut next_imbalance_cred = 0;
                if let Some(ref c_id) = creditor_opt {
                    next_imbalance_cred = get_balance(c_id, &new_schedule).abs();
                }
                let total_next_imbalance = next_imbalance_sick + next_imbalance_cred;
                
                if total_next_imbalance < total_current_imbalance {
                    accepted = true;
                    remaining_to_recover -= 1;
                    recovered_shifts += 1;
                    feasible_shifts += 1;
                    if day_index > max_recovery_day { max_recovery_day = day_index; }
                    
                    if let Some(ref c_id) = creditor_opt {
                        *creditors.get_mut(c_id).unwrap() -= 1;
                    }
                } else {
                    blocked_reason = Some("Recovery would unfairly overload another employee".to_string());
                    blocked_recoveries.push(BlockedRecovery {
                        day: day_index,
                        reason: blocked_reason.clone().unwrap(),
                        constraint: "Fairness / Monotonicity".to_string(),
                    });
                    
                    new_schedule.get_mut(nurse_id).unwrap()[day_index] = "".to_string();
                    if let Some(ref c_id) = creditor_opt {
                        let old_shift = new_schedule[c_id][day_index].replace("RETURNED-", "");
                        new_schedule.get_mut(c_id).unwrap()[day_index] = old_shift;
                    }
                }
            }
            
            let next_imbalance_sick = get_balance(nurse_id, &new_schedule).abs();
            let mut next_imbalance_cred = 0;
            if let Some(ref c_id) = creditor_opt {
                next_imbalance_cred = get_balance(c_id, &new_schedule).abs();
            }
            
            audit_trail.push(RecoveryAudit {
                day: day_index,
                candidate_type,
                accepted,
                recovering_nurse: nurse_id.clone(),
                creditor: creditor_opt,
                sick_nurse_legal,
                creditor_legal,
                imbalance_before: total_current_imbalance,
                imbalance_after: next_imbalance_sick + next_imbalance_cred,
                blocked_reason,
            });
            
            if accepted {
                break;
            }
        }
    }
    let recovery_eta = if max_recovery_day > today_index {
        ((max_recovery_day - (today_index + 7)) as f64 / 7.0).ceil() as i32 + 1
    } else {
        0
    };
    
    let mut balance_changes: HashMap<String, BalanceChange> = HashMap::new();
    
    let covered_sickness: i32 = creditors.values().sum();
    if covered_sickness > 0 {
        let sick_nurse_balance = current_state.balances.iter().find(|b| b.nurse_id == *nurse_id).map(|b| b.balance).unwrap_or(0);
        balance_changes.insert(nurse_id.clone(), BalanceChange {
            previous: sick_nurse_balance,
            current: sick_nurse_balance - covered_sickness,
        });
    }
    
    for (cred_id, &shifts_owed) in &creditors_to_process {
        if shifts_owed > 0 {
            let cred_balance = current_state.balances.iter().find(|b| b.nurse_id == *cred_id).map(|b| b.balance).unwrap_or(0);
            balance_changes.insert(cred_id.clone(), BalanceChange {
                previous: cred_balance,
                current: cred_balance + shifts_owed,
            });
        }
    }
    
    if recovered_shifts > 0 {
        let sick_nurse_balance = balance_changes.get(nurse_id.as_str()).map(|bc| bc.current).unwrap_or_else(|| {
            current_state.balances.iter().find(|b| b.nurse_id == *nurse_id).map(|b| b.balance).unwrap_or(0)
        });
        let prev = balance_changes.get(nurse_id.as_str()).map(|bc| bc.previous).unwrap_or_else(|| {
            current_state.balances.iter().find(|b| b.nurse_id == *nurse_id).map(|b| b.balance).unwrap_or(0)
        });
        balance_changes.insert(nurse_id.clone(), BalanceChange {
            previous: prev,
            current: sick_nurse_balance + recovered_shifts,
        });
        
        for audit in &audit_trail {
            if audit.accepted {
                if let Some(ref cred_id) = audit.creditor {
                    let cred_balance = balance_changes.get(cred_id.as_str()).map(|bc| bc.current).unwrap_or_else(|| {
                        current_state.balances.iter().find(|b| b.nurse_id == *cred_id).map(|b| b.balance).unwrap_or(0)
                    });
                    let prev = balance_changes.get(cred_id.as_str()).map(|bc| bc.previous).unwrap_or_else(|| {
                        current_state.balances.iter().find(|b| b.nurse_id == *cred_id).map(|b| b.balance).unwrap_or(0)
                    });
                    balance_changes.insert(cred_id.clone(), BalanceChange {
                        previous: prev,
                        current: cred_balance - 1,
                    });
                }
            }
        }
    }
    
    let post_recovery_validation = validate_schedule(&new_schedule, scenario);
    
    current_state.schedule = new_schedule.clone();
    current_state.verification_reports.sickness = Some(sickness_validation);
    current_state.verification_reports.recovery = Some(post_recovery_validation.clone());
    
    for b in current_state.balances.iter_mut() {
        if let Some(bc) = balance_changes.get(b.nurse_id.as_str()) {
            b.balance = bc.current;
        }
    }
    
    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../adapters/ultracrew/tests/data/n030w4");
    let week_data = parse_week_data(base_dir.join("WD-n030w4-0.json")).unwrap();
    let dynamic_dash = make_dynamic_dashboard(
        &new_schedule,
        scenario,
        &week_data,
        &post_recovery_validation,
        current_state.dashboard.feasibility_report.clone(),
        current_state.dashboard.pareto_frontier.clone(),
    );
    current_state.dashboard = dynamic_dash;

    current_state.dashboard.alerts.insert(0, Alert {
        employee: nurse_id.clone(),
        severity: "high".to_string(),
        message: format!("Behind schedule by {} shifts due to sickness", missed_total),
    });
    
    current_state.recovery_plan = Some(RecoveryPlan {
        affected_nurse: req.employee_id.clone(),
        missed_shifts: missed_total,
        recovered_shifts,
        recovery_eta,
        creditors,
        balance_changes,
        coverage_impact: "None".to_string(),
        audit_trail,
        requested_shifts,
        feasible_shifts,
        blocked_recoveries,
    });
    
    current_state.dashboard.validation_report = post_recovery_validation;
    
    state.baseline_state = current_state.clone();
    
    Json(current_state)
}

#[derive(Serialize)]
struct ValidateResponse {
    is_valid: bool,
    violations: usize,
}

#[derive(Serialize)]
struct RecommendationsResponse {
    recommendations: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct ScheduleResponse {
    schedule: HashMap<u64, u64>,
    metrics: HashMap<String, f64>,
    constraint_report: Option<ultracrew::constraint_engine::ConstraintReport>,
    recommendations: Vec<ultracrew::recommendation::SchedulingRecommendation>,
    telemetry: Option<ultracrew::optimization::OptimizationReport>,
}

async fn schedule_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(req): Json<ultracrew::public_contracts::ScheduleRequest>,
) -> Result<Json<ScheduleResponse>, (axum::http::StatusCode, String)> {
    let context = req.to_context();
    
    if let Err(e) = ultracrew::constraint_engine::validate_context(&context) {
        return Err((axum::http::StatusCode::BAD_REQUEST, format!("Dataset validation failed: {}", e)));
    }
    
    let solution = ultracrew::pipeline::run_pipeline_from_request(
        context.clone(),
        req.generation_limit,
        None, None, None, None, None,
    ).map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Optimization failed: {}", e)))?;
        
    let mut app_state = state.lock().unwrap();
    app_state.last_solution = Some(solution.clone());
    app_state.last_request = Some(req);

    let constraint_engine = ultracrew::constraint_engine::ConstraintEngine::new(context);
    let genome = ultracrew::optimization::ScheduleGenome { assignments: solution.assignments.clone() };
    let report = constraint_engine.evaluate(&genome);

    let rec_engine = ultracrew::recommendation::RecommendationEngine::new();
    let recommendations = rec_engine.generate_recommendations(&report);

    let metrics = ultracrew::decision_intelligence::analyze_solution(&solution);

    let response = ScheduleResponse {
        schedule: solution.assignments.clone(),
        metrics,
        constraint_report: Some(report),
        recommendations,
        telemetry: solution.telemetry.clone(),
    };
    
    Ok(Json(response))
}

async fn reschedule_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(req): Json<ultracrew::public_contracts::RescheduleRequest>,
) -> Result<Json<ScheduleResponse>, (axum::http::StatusCode, String)> {
    let context = req.to_context();

    let solution = ultracrew::pipeline::run_pipeline_from_request(
        context.clone(),
        req.generation_limit,
        req.tournament_size,
        req.population_size,
        req.mutation_rate,
        req.crossover_rate,
        req.elite_count,
    ).map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Rescheduling failed: {}", e)))?;
        
    let mut app_state = state.lock().unwrap();
    app_state.last_solution = Some(solution.clone());
    
    let constraint_engine = ultracrew::constraint_engine::ConstraintEngine::new(context);
    let genome = ultracrew::optimization::ScheduleGenome { assignments: solution.assignments.clone() };
    let report = constraint_engine.evaluate(&genome);

    let rec_engine = ultracrew::recommendation::RecommendationEngine::new();
    let recommendations = rec_engine.generate_recommendations(&report);

    let metrics = ultracrew::decision_intelligence::analyze_solution(&solution);

    let response = ScheduleResponse {
        schedule: solution.assignments.clone(),
        metrics,
        constraint_report: Some(report),
        recommendations,
        telemetry: solution.telemetry.clone(),
    };
    
    Ok(Json(response))
}

async fn validate_handler(
    State(_state): State<Arc<Mutex<AppState>>>,
    Json(req): Json<ultracrew::public_contracts::ValidateRequest>,
) -> Result<Json<ScheduleResponse>, (axum::http::StatusCode, String)> {
    let context = req.request.to_context();
    
    let constraint_engine = ultracrew::constraint_engine::ConstraintEngine::new(context);
    let genome = ultracrew::optimization::ScheduleGenome { assignments: req.assignments.clone() };
    let report = constraint_engine.evaluate(&genome);

    let rec_engine = ultracrew::recommendation::RecommendationEngine::new();
    let recommendations = rec_engine.generate_recommendations(&report);

    let solution = ultracrew::schedule_solution::ScheduleSolution {
        assignments: req.assignments.clone(),
        fitness: report.fitness,
        hard_violations: report.hard_violations,
        fairness_penalty: report.fairness_penalty,
        fatigue_penalty: report.fatigue_penalty,
        rest_violations: report.rest_violations,
        recommendations: None,
        telemetry: None,
    };

    let metrics = ultracrew::decision_intelligence::analyze_solution(&solution);

    let response = ScheduleResponse {
        schedule: req.assignments.clone(),
        metrics,
        constraint_report: Some(report),
        recommendations,
        telemetry: None,
    };
    
    Ok(Json(response))
}

async fn recommendations_handler(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Result<Json<RecommendationsResponse>, (axum::http::StatusCode, String)> {
    let app_state = state.lock().unwrap();
    if let Some(ref sol) = app_state.last_solution {
        let recommendations = ultracrew::decision_intelligence::generate_insights(sol);
        Ok(Json(RecommendationsResponse { recommendations }))
    } else {
        Err((axum::http::StatusCode::NOT_FOUND, "No active schedule solution found. Please call /api/schedule first.".to_string()))
    }
}

async fn metrics_handler(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Result<Json<HashMap<String, f64>>, (axum::http::StatusCode, String)> {
    let app_state = state.lock().unwrap();
    if let Some(ref sol) = app_state.last_solution {
        let metrics = ultracrew::decision_intelligence::analyze_solution(sol);
        Ok(Json(metrics))
    } else {
        Err((axum::http::StatusCode::NOT_FOUND, "No active schedule solution found. Please call /api/schedule first.".to_string()))
    }
}

async fn export_formats_handler() -> Json<Vec<ultracrew::generic_export::FormatDescriptor>> {
    Json(ultracrew::generic_export::GenericExporter::supported_formats())
}

async fn export_solution_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Path(format): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    use ultracrew::generic_export::{ExportConfig, ExportFormat, GenericExporter};

    let fmt = ExportFormat::from_str(&format)
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, e.to_string()))?;

    let app_state = state.lock().unwrap();
    let sol = app_state.last_solution.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::NOT_FOUND,
            "No active schedule solution. Call POST /api/schedule first.".to_string(),
        )
    })?;

    let config = ExportConfig {
        format: fmt.clone(),
        pretty_json: params.get("pretty").map(|v| v == "true").unwrap_or(false),
        include_telemetry: params.get("telemetry").map(|v| v != "false").unwrap_or(true),
        include_recommendations: params
            .get("recommendations")
            .map(|v| v != "false")
            .unwrap_or(true),
        ..Default::default()
    };

    let result = GenericExporter::export(sol, &config)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mime = result.mime_type.clone();
    let body = result.content;

    Ok((
        axum::http::StatusCode::OK,
        [(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_str(&mime)
                .unwrap_or(axum::http::HeaderValue::from_static("application/octet-stream")),
        )],
        body,
    ))
}

async fn get_balance_handler(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Json<Vec<NurseBalance>> {
    let state = state.lock().unwrap();
    Json(state.baseline_state.balances.clone())
}

async fn get_dashboard_handler(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Json<Dashboard> {
    let state = state.lock().unwrap();
    Json(state.baseline_state.dashboard.clone())
}

// Handler to return list of nurses for UI
async fn get_nurses_handler(State(state): State<Arc<Mutex<AppState>>>) -> Json<serde_json::Value> {
    let state = state.lock().unwrap();
    Json(json!({ "nurses": state.scenario.nurses.clone() }))
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../adapters/ultracrew/tests/data/n030w4");
    let scenario = parse_scenario(base_dir.join("Sc-n030w4.json")).unwrap();
    let week_data = parse_week_data(base_dir.join("WD-n030w4-0.json")).unwrap();
    let requirements = &week_data.requirements;
    
    let mut bottlenecks = Vec::new();
    let mut skill_feasible = true;
    let mut contract_feasible = true;
    let mut structural_feasible = true;
    let mut total_required_assignments = 0;
    let mut skill_demand: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut weekend_demand = 0;
    let mut night_demand = 0;
    
    let num_days = (scenario.number_of_weeks * 7) as usize;
    for d in 0..num_days {
        let weekday = d % 7;
        for req in requirements {
            let req_amt = match weekday {
                0 => req.monday.optimal,
                1 => req.tuesday.optimal,
                2 => req.wednesday.optimal,
                3 => req.thursday.optimal,
                4 => req.friday.optimal,
                5 => req.saturday.optimal,
                6 => req.sunday.optimal,
                _ => 0,
            };
            if req_amt > 0 {
                total_required_assignments += req_amt;
                *skill_demand.entry(req.skill.clone()).or_insert(0) += req_amt;
                if weekday == 5 || weekday == 6 {
                    weekend_demand += req_amt;
                }
                if req.shift_type == "Night" {
                    night_demand += req_amt;
                }
            }
        }
    }
    
    let mut total_capacity = 0;
    let mut skill_capacity: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut weekend_capacity = 0;
    let mut night_capacity = 0;
    
    for nurse in &scenario.nurses {
        let contract = scenario.contracts.iter().find(|c| c.id == nurse.contract).unwrap();
        let scale = 56.0 / (scenario.number_of_weeks as f64 * 7.0);
        let max_assign = (contract.max_assignments as f64 * scale) as usize;
        
        total_capacity += max_assign;
        
        for skill in &nurse.skills {
            *skill_capacity.entry(skill.clone()).or_insert(0) += max_assign;
        }
        
        let max_weekend = (contract.max_working_weekends as f64 * (8.0 / scenario.number_of_weeks as f64)) as usize;
        weekend_capacity += max_weekend * 2;
        night_capacity += max_assign;
    }
    
    if total_required_assignments > total_capacity {
        contract_feasible = false;
        bottlenecks.push(Bottleneck {
            description: format!("Global Capacity Shortfall: Required {}, Capacity {}", total_required_assignments, total_capacity),
            severity: "CRITICAL".to_string(),
        });
    }
    
    for (skill, demand) in &skill_demand {
        let capacity = skill_capacity.get(skill).unwrap_or(&0);
        if demand > capacity {
            skill_feasible = false;
            bottlenecks.push(Bottleneck {
                description: format!("{} Capacity Shortfall: Required {}, Capacity {}", skill, demand, capacity),
                severity: "CRITICAL".to_string(),
            });
        }
    }
    
    if weekend_demand > weekend_capacity {
        structural_feasible = false;
        bottlenecks.push(Bottleneck {
            description: format!("Weekend Capacity Shortfall: Required {}, Capacity {}", weekend_demand, weekend_capacity),
            severity: "HIGH".to_string(),
        });
    }
    
    let overall_feasible = skill_feasible && contract_feasible && structural_feasible;
    
    let feasibility_report = FeasibilityReport {
        overall_feasible,
        skill_feasible,
        contract_feasible,
        structural_feasible,
        bottlenecks,
    };
    
    let startup = ultracrew::pipeline::run_inrc_startup_pipeline(
        &base_dir.join("Sc-n030w4.json"),
        &base_dir.join("WD-n030w4-0.json"),
        100,
    ).unwrap_or_else(|e| {
        println!("Startup pipeline error: {:?}", e);
        ultracrew::public_contracts::InrcStartupResult {
            schedule: HashMap::new(),
            pareto_solutions: Vec::new(),
        }
    });

    let pareto_solutions: Vec<ParetoFrontierSolution> = startup.pareto_solutions.into_iter().map(|p| {
        ParetoFrontierSolution {
            s6_assignment_penalty: p.s6_assignment_penalty,
            s7_weekend_penalty: p.s7_weekend_penalty,
            recovery_penalty: p.recovery_penalty,
            workload_balance: p.workload_balance,
            temporal_load_balance: p.temporal_load_balance,
            schedule: p.schedule,
        }
    }).collect();

    let validation_report = validate_schedule(&startup.schedule, &scenario);
    let schedule = startup.schedule;
    
    let mut balances = Vec::new();
    for (i, nurse) in scenario.nurses.iter().enumerate() {
        let nurse_id = nurse.id.clone();
        let balance = if i == 0 { -4 } else if i == 1 { 2 } else if i == 2 { 1 } else if i == 3 { 1 } else { 0 };
        let exp = if balance < 0 { vec!["Slightly under target workload".to_string()] } 
                  else if balance > 0 { vec!["Slightly over target workload".to_string()] } 
                  else { vec!["On track with target workload".to_string()] };
        balances.push(NurseBalance { nurse_id, balance, explanation: exp });
    }
    
    let initial_sum: i32 = balances.iter().map(|b| b.balance).sum();
    assert_eq!(initial_sum, 0, "Initial balances must sum to zero");
    
    let mut workload_audit = Vec::new();
    for nurse in &scenario.nurses {
        let contract = scenario.contracts.iter().find(|c| c.id == nurse.contract).unwrap();
        let scale = 56.0 / (scenario.number_of_weeks as f64 * 7.0);
        let min_assign = (contract.min_assignments as f64 * scale) as i32;
        let max_assign = (contract.max_assignments as f64 * scale) as i32;
        let expected = (min_assign + max_assign) / 2;
        let actual = schedule[&nurse.id].iter().filter(|s| !s.is_empty()).count() as i32;
        let mut max_work = 0;
        let mut max_off = 0;
        let mut curr_work = 0;
        let mut curr_off = 0;
        for d in 0..num_days {
            let shift = &schedule[&nurse.id][d];
            if !shift.is_empty() {
                curr_work += 1;
                curr_off = 0;
                if curr_work > max_work { max_work = curr_work; }
            } else {
                curr_off += 1;
                curr_work = 0;
                if curr_off > max_off { max_off = curr_off; }
            }
        }
        
        workload_audit.push(WorkloadAudit {
            nurse_id: nurse.id.clone(),
            expected_assignments: expected,
            actual_assignments: actual,
            deviation: actual - expected,
            max_work_streak: max_work,
            max_off_streak: max_off,
        });
    }

    let mut daily_assignments = vec![0; num_days];
    let required_assignments = 16 * num_days;
    let mut actual_assignments = 0;
    for shifts in schedule.values() {
        for d in 0..num_days {
            if !shifts[d].is_empty() {
                actual_assignments += 1;
                daily_assignments[d] += 1;
            }
        }
    }
    let coverage_percentage = (actual_assignments as f64 / required_assignments as f64) * 100.0;

    let mut skill_deficits = Vec::new();
    let mut total_req_slots = 0;
    let mut filled_req_slots = 0;
    
    for d in 0..num_days {
        let weekday = d % 7;
        for req in &week_data.requirements {
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
            
            if required > 0 {
                let mapped_shift = match req.shift_type.as_str() {
                    "Early" => "E",
                    "Day" => "D",
                    "Late" => "L",
                    "Night" => "N",
                    _ => "",
                };
                
                let mut assigned = 0;
                for nurse in &scenario.nurses {
                    if schedule[&nurse.id][d] == mapped_shift || schedule[&nurse.id][d].ends_with(&format!("-{}", mapped_shift)) {
                        if nurse.skills.contains(&req.skill) {
                            assigned += 1;
                        }
                    }
                }
                
                total_req_slots += required;
                filled_req_slots += std::cmp::min(assigned, required);
                
                let deficit = required as i32 - assigned as i32;
                if deficit > 0 {
                    skill_deficits.push(SkillDeficit {
                        day: d,
                        shift: req.shift_type.clone(),
                        skill: req.skill.clone(),
                        required,
                        assigned,
                        deficit,
                    });
                }
            }
        }
    }
    
    let mut skill_counts = std::collections::HashMap::new();
    let mut shift_counts = std::collections::HashMap::new();
    for d in &skill_deficits {
        *skill_counts.entry(d.skill.clone()).or_insert(0) += d.deficit;
        *shift_counts.entry(d.shift.clone()).or_insert(0) += d.deficit;
    }
    
    let worst_skill = skill_counts.into_iter().max_by_key(|&(_, v)| v).map(|(k, _)| k).unwrap_or_else(|| "None".to_string());
    let worst_shift = shift_counts.into_iter().max_by_key(|&(_, v)| v).map(|(k, _)| k).unwrap_or_else(|| "None".to_string());
    
    let skill_coverage_percentage = if total_req_slots > 0 {
        (filled_req_slots as f64 / total_req_slots as f64) * 100.0
    } else {
        100.0
    };
    
    let skill_coverage_audit = SkillCoverageAudit {
        skill_coverage_percentage,
        total_skill_deficits: skill_deficits.len(),
        worst_skill,
        worst_shift,
        deficits: skill_deficits,
    };
    
    let coverage_audit = CoverageAudit {
        required_assignments,
        actual_assignments,
        coverage_percentage,
        daily_assignments,
    };
        
    let mut constraint_audit = Vec::new();
    for nurse in &scenario.nurses {
        let mut min_work = 0;
        let mut max_work = 0;
        let mut min_off = 0;
        let mut max_off = 0;
        
        for det in &validation_report.details {
            if det.nurse_id == nurse.id {
                if det.constraint == "min_consecutive_working_days" { min_work += 1; }
                if det.constraint == "max_consecutive_working_days" { max_work += 1; }
                if det.constraint == "min_consecutive_days_off" { min_off += 1; }
                if det.constraint == "max_consecutive_days_off" { max_off += 1; }
            }
        }
        constraint_audit.push(ConstraintAudit {
            nurse_id: nurse.id.clone(),
            min_work_streak_violations: min_work,
            max_work_streak_violations: max_work,
            min_off_streak_violations: min_off,
            max_off_streak_violations: max_off,
        });
    }

    let legality_score = if validation_report.is_legal { 100 } else { 
        let v = validation_report.details.len() as i32;
        std::cmp::max(0, 100 - (v * 5))
    };
    let coverage_score = coverage_percentage as i32;
    
    let mut max_dev = 0;
    for wa in &workload_audit {
        if wa.deviation.abs() > max_dev { max_dev = wa.deviation.abs(); }
    }
    let balance_score = std::cmp::max(0, 100 - (max_dev * 10));
    let fragmentation_score = std::cmp::max(0, 100 - (validation_report.details.len() as i32 * 2));
    let recovery_score = 100;
    
    let roster_health = RosterHealth {
        legality_score,
        coverage_score,
        balance_score,
        fragmentation_score,
        recovery_score,
    };
    
    let baseline_status = BaselineStatus {
        state: if validation_report.is_legal { "Legal".to_string() } else { "RepairFailed".to_string() },
        is_legal: validation_report.is_legal,
        repair_attempts: 50,
        exhausted_search: !validation_report.is_legal,
    };

    let dashboard = make_dynamic_dashboard(
        &schedule,
        &scenario,
        &week_data,
        &validation_report,
        Some(feasibility_report),
        Some(pareto_solutions),
    );
    let verification_reports = VerificationReports {
        baseline: Some(validation_report.clone()),
        sickness: None,
        recovery: None,
    };
    
    let baseline_state = SimulationState {
        schedule: schedule.clone(),
        dashboard,
        balances,
        recovery_plan: None,
        verification_reports,
    };
    
    let app_state = Arc::new(Mutex::new(AppState {
        scenario,
        baseline_state: baseline_state.clone(),
        original_state: baseline_state,
        last_solution: None,
        last_request: None,
        decisions: Vec::new(),
        schedule_versions: Vec::new(),
    }));

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/scenario", get(get_scenario))
        .route("/api/state", get(get_state))
        .route("/api/simulations/sick-leave", post(simulate_sick_leave))
        .route("/api/simulations/reset", post(reset_simulation))
        .route("/api/schedule", post(schedule_handler))
        .route("/api/reschedule", post(reschedule_handler))
        .route("/api/validate", post(validate_handler))
        .route("/api/recommendations", get(recommendations_handler))
        .route("/api/metrics", get(metrics_handler))
        .route("/api/balance", get(get_balance_handler))
        .route("/api/dashboard", get(get_dashboard_handler))
        .route("/api/nurses", get(get_nurses_handler))
        // Generic Export endpoints (Phase A Step 3)
        .route("/api/export/formats", get(export_formats_handler))
        .route("/api/export/{format}", post(export_solution_handler))
        // Decision workspace endpoints
        .route("/api/decision_cases", get(list_decision_cases).post(create_decision_case))
        .route("/api/decision_cases/{id}", get(get_decision_case).put(update_decision_case).delete(delete_decision_case))
        .route("/api/decision_cases/{id}/commit", post(commit_schedule_version))
        .route("/api/decision_cases/{id}/export", get(export_decision_case_csv))
        .with_state(app_state)
        .layer(cors);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let bind_addr = format!("0.0.0.0:{}", port);
    let addr = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
    println!("UltraCrew Server running on http://0.0.0.0:{}", port);
    axum::serve(addr, app).await.unwrap();
}

#[cfg(test)]
mod server_endpoints_tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt; // for oneshot

    fn setup_test_app() -> Router {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers(Any);
            
        let scenario = InrcScenario {
            id: "test-sc".to_string(),
            number_of_weeks: 1,
            skills: vec!["Forklift".to_string()],
            shift_types: vec![],
            contracts: vec![],
            nurses: vec![],
            forbidden_shift_type_successions: vec![],
        };
        
        let baseline_state = SimulationState {
            schedule: HashMap::new(),
            dashboard: Dashboard {
                feasibility_report: None,
                skill_coverage_audit: None,
                coverage: Coverage { covered: 0, understaffed: 0, critical: 0 },
                coverage_audit: CoverageAudit {
                    required_assignments: 0,
                    actual_assignments: 0,
                    coverage_percentage: 0.0,
                    daily_assignments: vec![],
                },
                alerts: vec![],
                recommendations: vec![],
                validation_report: ValidationReport {
                    max_consecutive_work_violations: 0,
                    min_consecutive_work_violations: 0,
                    min_days_off_violations: 0,
                    max_days_off_violations: 0,
                    forbidden_successions: 0,
                    coverage_achieved: 0.0,
                    is_legal: true,
                    details: vec![],
                },
                workload_audit: vec![],
                constraint_audit: vec![],
                roster_health: RosterHealth {
                    legality_score: 100,
                    coverage_score: 100,
                    balance_score: 100,
                    fragmentation_score: 100,
                    recovery_score: 100,
                },
                baseline_status: BaselineStatus {
                    state: "Legal".to_string(),
                    is_legal: true,
                    repair_attempts: 0,
                    exhausted_search: false,
                },
                pareto_frontier: None,
            },
            balances: vec![],
            recovery_plan: None,
            verification_reports: VerificationReports {
                baseline: None,
                sickness: None,
                recovery: None,
            },
        };

        let app_state = Arc::new(Mutex::new(AppState {
            scenario,
            baseline_state: baseline_state.clone(),
            original_state: baseline_state,
            last_solution: None,
            last_request: None,
            decisions: Vec::new(),
            schedule_versions: Vec::new(),
        }));

        Router::new()
            .route("/api/health", get(health_check))
            .route("/api/schedule", post(schedule_handler))
            .route("/api/reschedule", post(reschedule_handler))
            .route("/api/validate", post(validate_handler))
            .route("/api/recommendations", get(recommendations_handler))
            .route("/api/metrics", get(metrics_handler))
            .route("/api/nurses", get(get_nurses_handler))
            .with_state(app_state)
            .layer(cors)
    }

    #[tokio::test]
    async fn test_health_check_endpoint() {
        let app = setup_test_app();
        let response = app
            .oneshot(Request::builder().uri("/api/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 10000).await.unwrap();
        let resp: StatusResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(resp.status, "ok");
    }

    #[tokio::test]
    async fn test_schedule_endpoint_validation_failure() {
        let app = setup_test_app();
        
        let request = ultracrew::public_contracts::ScheduleRequest {
            workers: vec![],
            shifts: vec![],
            historical_workloads: None,
            rng_seed: None,
            generation_limit: None,
            scenario: None,
        };
        
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/schedule")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_schedule_endpoint_success() {
        let app = setup_test_app();
        
        use ultracrew::models::{Worker, Shift, Skill};
        let request = ultracrew::public_contracts::ScheduleRequest {
            workers: vec![
                Worker { id: 1, skills: vec![Skill::new("Forklift")] }
            ],
            shifts: vec![
                Shift { id: 101, start_hour: 8, duration_hours: 8, required_skill: Skill::new("Forklift") }
            ],
            historical_workloads: None,
            rng_seed: Some(42),
            generation_limit: None,
            scenario: None,
        };
        
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/schedule")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), 50000).await.unwrap();
        let resp: ScheduleResponse = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(resp.schedule.len(), 1);
        assert!(!resp.metrics.is_empty());
        assert!(resp.constraint_report.is_some());
        assert_eq!(resp.recommendations.len(), 0);
    }

    #[tokio::test]
    async fn test_validate_endpoint() {
        let app = setup_test_app();
        
        use ultracrew::models::{Worker, Shift, Skill};
        let request = ultracrew::public_contracts::ScheduleRequest {
            workers: vec![
                Worker { id: 1, skills: vec![Skill::new("Forklift")] }
            ],
            shifts: vec![
                Shift { id: 101, start_hour: 8, duration_hours: 8, required_skill: Skill::new("Forklift") }
            ],
            historical_workloads: None,
            rng_seed: Some(42),
            generation_limit: None,
            scenario: None,
        };
        
        let mut assignments = std::collections::HashMap::new();
        assignments.insert(101, 1);

        let validate_req = ultracrew::public_contracts::ValidateRequest {
            request,
            assignments,
        };
        
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/validate")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&validate_req).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), 50000).await.unwrap();
        let resp: ScheduleResponse = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(resp.schedule.len(), 1);
        assert_eq!(*resp.schedule.get(&101).unwrap(), 1);
        assert!(resp.constraint_report.is_some());
        assert_eq!(resp.constraint_report.unwrap().hard_violations, 0);
    }

    #[tokio::test]
    async fn test_reschedule_endpoint() {
        let app = setup_test_app();
        
        use ultracrew::models::{Worker, Shift, Skill};
        let request = ultracrew::public_contracts::ScheduleRequest {
            workers: vec![
                Worker { id: 1, skills: vec![Skill::new("Forklift")] },
                Worker { id: 2, skills: vec![Skill::new("Forklift")] }
            ],
            shifts: vec![
                Shift { id: 101, start_hour: 8, duration_hours: 8, required_skill: Skill::new("Forklift") },
                Shift { id: 102, start_hour: 16, duration_hours: 8, required_skill: Skill::new("Forklift") }
            ],
            historical_workloads: None,
            rng_seed: Some(42),
            generation_limit: None,
            scenario: None,
        };
        
        let mut existing_assignments = std::collections::HashMap::new();
        existing_assignments.insert(101, 1);
        existing_assignments.insert(102, 2);

        let reschedule_req = ultracrew::public_contracts::RescheduleRequest {
            request,
            existing_assignments,
            locked_shift_ids: Some(vec![101]), // Shift 101 is locked to Worker 1
            generation_limit: None,
            tournament_size: None,
            population_size: None,
            mutation_rate: None,
            crossover_rate: None,
            elite_count: None,
        };
        
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/reschedule")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&reschedule_req).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), 50000).await.unwrap();
        let resp: ScheduleResponse = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(resp.schedule.len(), 2);
        // Verify locked assignment is preserved
        assert_eq!(*resp.schedule.get(&101).unwrap(), 1);
    }
}

