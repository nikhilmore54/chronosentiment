
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
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
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
    /// INRC scenario — None until loaded (pilot portal does not require this)
    scenario: Option<InrcScenario>,
    /// Simulation state — None until an INRC scenario is loaded
    baseline_state: Option<SimulationState>,
    /// Original simulation state for reset — None until loaded
    original_state: Option<SimulationState>,
    last_solution: Option<ultracrew::schedule_solution::ScheduleSolution>,
    last_request: Option<ultracrew::public_contracts::ScheduleRequest>,
    decisions: Vec<DecisionCase>,
    schedule_versions: Vec<ScheduleVersion>,
    /// Current CSRF token (double-submit cookie pattern)
    csrf_token: String,
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

// ─── CSRF Token Endpoint ──────────────────────────────────────────────────────

#[derive(Serialize)]
struct CsrfTokenResponse {
    csrf_token: String,
}

/// Issue a CSRF token. The client must:
/// 1. Call GET /api/csrf-token to receive the token.
/// 2. Store it and send it as the X-CSRF-Token header on all POST requests.
/// 3. The server validates the header matches the issued token.
///
/// For PX-001 pilot (single-server, no user sessions), we use a simple
/// per-request token stored in a shared atomic string. This is sufficient
/// for the pilot environment and can be upgraded to a proper session-based
/// CSRF scheme before production deployment.
async fn csrf_token_handler(
    State(state): State<Arc<Mutex<AppState>>>,
) -> impl IntoResponse {
    let token = uuid::Uuid::new_v4().to_string();
    {
        let mut s = state.lock().unwrap();
        s.csrf_token = token.clone();
    }
    // Set as a cookie (SameSite=Strict) AND return in body for double-submit
    let cookie = format!(
        "csrf_token={}; Path=/; SameSite=Strict; HttpOnly=false",
        token
    );
    (
        axum::http::StatusCode::OK,
        [(axum::http::header::SET_COOKIE, cookie)],
        Json(CsrfTokenResponse { csrf_token: token }),
    )
}

async fn get_scenario(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let state = state.lock().unwrap();
    match &state.scenario {
        Some(sc) => (StatusCode::OK, Json(serde_json::to_value(sc).unwrap())).into_response(),
        None => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"error": "INRC scenario not loaded. This endpoint is not used by the pilot portal."}))).into_response(),
    }
}

async fn get_state(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let state = state.lock().unwrap();
    match &state.baseline_state {
        Some(s) => (StatusCode::OK, Json(serde_json::to_value(s).unwrap())).into_response(),
        None => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"error": "Simulation state not loaded. This endpoint is not used by the pilot portal."}))).into_response(),
    }
}

async fn reset_simulation(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let mut state = state.lock().unwrap();
    match state.original_state.clone() {
        Some(orig) => {
            state.baseline_state = Some(orig.clone());
            println!("Reset simulation state on server successfully.");
            (StatusCode::OK, Json(serde_json::to_value(&orig).unwrap())).into_response()
        }
        None => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"error": "Simulation state not loaded. This endpoint is not used by the pilot portal."}))).into_response(),
    }
}

async fn simulate_sick_leave(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(req): Json<SickLeaveRequest>,
) -> impl IntoResponse {
    println!("Received SickLeaveRequest for employee {}", req.employee_id);
    let mut state = state.lock().unwrap();
    // Guard: simulation state requires INRC fixture — not used by pilot portal
    let mut current_state = match state.baseline_state.clone() {
        Some(s) => s,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"error": "Simulation state not loaded. This endpoint is not used by the pilot portal."}))).into_response(),
    };
    let scenario = match state.scenario.as_ref() {
        Some(s) => s,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"error": "INRC scenario not loaded. This endpoint is not used by the pilot portal."}))).into_response(),
    };
    let scenario = scenario.clone();
    let scenario = &scenario;
    let today_index = 14;

    let nurse_id = &req.employee_id;
    let sick_nurse = match scenario.nurses.iter().find(|n| n.id == *nurse_id) {
        Some(n) => n,
        None => {
            println!("Nurse {} not found in scenario!", nurse_id);
            return Json(current_state).into_response();
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
    
    state.baseline_state = Some(current_state.clone());
    
    Json(current_state).into_response()
}

// ─── Load INRC Scenario Endpoint ─────────────────────────────────────────────

#[derive(Deserialize)]
struct LoadScenarioRequest {
    /// Absolute or relative path to the scenario directory.
    /// Defaults to the bundled n030w4 fixture when omitted.
    base_dir: Option<String>,
}

#[derive(Serialize)]
struct LoadScenarioResponse {
    loaded: bool,
    nurses: usize,
    weeks: usize,
    message: String,
}

async fn load_scenario_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(req): Json<LoadScenarioRequest>,
) -> impl IntoResponse {
    let base_dir = match req.base_dir {
        Some(ref p) => PathBuf::from(p),
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../adapters/ultracrew/tests/data/n030w4"),
    };

    let scenario_path = base_dir.join("Sc-n030w4.json");
    let week_data_path = base_dir.join("WD-n030w4-0.json");

    let scenario = match parse_scenario(scenario_path) {
        Ok(s) => s,
        Err(e) => return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": format!("Failed to parse scenario: {}", e)})),
        ).into_response(),
    };

    let week_data = match parse_week_data(week_data_path) {
        Ok(w) => w,
        Err(e) => return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": format!("Failed to parse week data: {}", e)})),
        ).into_response(),
    };

    // Build a baseline schedule from the scenario + requirements
    let genome = match ultracrew_server::simulation::generate_baseline_schedule(
        &scenario,
        &week_data.requirements,
    ) {
        Ok(g) => g,
        Err(e) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Failed to generate baseline schedule: {}", e)})),
        ).into_response(),
    };

    // Convert genome → HashMap<nurse_id, Vec<shift_per_day>> using built-in method
    let schedule = genome.to_flat_schedule();

    let validation_report = ultracrew::inrc::validator::validate_schedule(&schedule, &scenario);

    let balances: Vec<ultracrew_server::simulation::NurseBalance> = scenario.nurses.iter().map(|n| {
        ultracrew_server::simulation::NurseBalance {
            nurse_id: n.id.clone(),
            balance: 0,
            explanation: vec!["Baseline — no adjustments yet".to_string()],
        }
    }).collect();

    let dashboard = make_dynamic_dashboard(
        &schedule,
        &scenario,
        &week_data,
        &validation_report,
        None,
        None,
    );

    let sim_state = SimulationState {
        schedule,
        dashboard,
        balances,
        recovery_plan: None,
        verification_reports: VerificationReports {
            baseline: Some(validation_report),
            sickness: None,
            recovery: None,
        },
    };

    let nurses = scenario.nurses.len();
    let weeks = scenario.number_of_weeks as usize;

    let mut app = state.lock().unwrap();
    app.scenario = Some(scenario);
    app.baseline_state = Some(sim_state.clone());
    app.original_state = Some(sim_state);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "loaded": true,
            "nurses": nurses,
            "weeks": weeks,
            "message": format!("INRC scenario loaded: {} nurses, {} weeks", nurses, weeks)
        })),
    ).into_response()
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

// ─── FTA Constants (TC CAR 700 — Air Canada / Air Transat) ───────────────────
//
// Transport Canada Commercial Air Services, Subpart 700.15 (Flight Duty Period)
// and 700.17 (Rest Period).  These are the rules used by the GERAD G-2014-22
// benchmark dataset (Kasirzadeh, Saddoune & Soumis 2014, HEC Montréal).
//
// Key differences from DGCA CAR Section 7 / EASA ORO.FTL:
//   • Night band is 00:00–05:59 only (not 18:00–05:59).
//     Reporting 06:00–23:59 is treated as "day" (no afternoon sub-band).
//   • FDP limits: 1-2 sectors 13h/12h, 3-4 sectors 12h/11h, 5+ sectors 11h/10h
//     (day/night respectively).
//   • Minimum rest = max(8h, preceding FDP duration).
//     TC CAR 700.17(1): rest period ≥ 8h free from duty.
//     TC CAR 700.17(2): if FDP > 8h, rest ≥ FDP.
//   • Home-base rest threshold: 34h (industry convention, same as DGCA).

/// Minimum absolute rest between consecutive FDPs (TC CAR 700.17(1)).
const LAYOVER_REST_HOURS: f64 = 8.0;

/// Home-base rest threshold: rest gap >= this value ends a pairing.
const HOME_BASE_REST_HOURS: f64 = 34.0;

/// Maximum hours in any 7-day window (TC CAR 700 cumulative limit).
const MAX_WEEKLY_HOURS: f64 = 60.0;

/// Maximum consecutive duty days (TC CAR 700).
const MAX_CONSECUTIVE_DAYS: u64 = 6;

/// TC CAR 700.15 Table: maximum FDP hours by sector count and reporting time.
///
/// Reporting time bands (TC CAR 700):
///   Night: 00:00–05:59  (report_hour in 0..=5)
///   Day:   06:00–23:59  (all other hours)
///
/// Sector count:
///   1-2 sectors: Day 13h, Night 12h
///   3-4 sectors: Day 12h, Night 11h
///   5+ sectors:  Day 11h, Night 10h
fn max_fdp_hours(sector_count: usize, report_hour: u64) -> f64 {
    // Night band: 00:00–05:59 only (TC CAR 700 definition)
    let is_night = report_hour <= 5;
    match (sector_count, is_night) {
        (0..=2, false) => 13.0,
        (0..=2, true)  => 12.0,
        (3..=4, false) => 12.0,
        (3..=4, true)  => 11.0,
        (_,     false) => 11.0,
        (_,     true)  => 10.0,
    }
}

/// TC CAR 700.17: minimum rest after an FDP.
/// rest >= max(LAYOVER_REST_HOURS=8h, preceding FDP duration).
fn min_rest_after_fdp(fdp_hours: f64) -> f64 {
    f64::max(LAYOVER_REST_HOURS, fdp_hours)
}

// ─── Shared request type for pairings / duties / swap_exchanges ──────────────

#[derive(Deserialize)]
struct ScheduleAnalysisRequest {
    /// shift_id → worker_id (the schedule produced by /api/schedule)
    schedule: HashMap<u64, u64>,
    /// The same shifts array sent to /api/schedule
    shifts: Vec<ShiftInput>,
    /// The same workers array sent to /api/schedule
    workers: Vec<WorkerInput>,
}

#[derive(Deserialize, Clone)]
struct ShiftInput {
    id: u64,
    start_hour: u64,
    duration_hours: u64,
    required_skill: String,
}

#[derive(Deserialize, Clone)]
struct WorkerInput {
    id: u64,
    skills: Vec<String>,
}

// ─── Pairing: a sequence of consecutive duties for one worker ─────────────────

// ─── FTA data model ───────────────────────────────────────────────────────────

/// A single Flight Duty Period: from crew report to last block-off.
/// Each sector (shift) within the FDP is separated by a ground time < LAYOVER_REST_HOURS.
#[derive(Serialize)]
struct FdpPeriod {
    /// Shifts (sectors) in this FDP, in chronological order.
    sectors: Vec<SectorInFdp>,
    /// Hour at which the crew reports for duty (= start_hour of first sector).
    report_hour: u64,
    /// Hour at which the crew is released (= end_hour of last sector).
    release_hour: u64,
    /// FDP duration in hours (release_hour - report_hour).
    fdp_hours: f64,
    /// Number of sectors in this FDP.
    sector_count: usize,
    /// Maximum FDP allowed for this sector count and reporting time (DGCA CAR Section 7).
    fdp_limit_hours: f64,
    /// Whether this FDP is within the regulatory limit.
    fdp_compliant: bool,
    /// Violation message if not compliant.
    fdp_violation: Option<String>,
    /// Rest gap after this FDP before the next FDP (None if this is the last FDP in the pairing).
    rest_after_hours: Option<f64>,
    /// Minimum required rest after this FDP: max(fdp_hours, LAYOVER_REST_HOURS).
    /// TC CAR 700.17: rest >= max(8h, preceding FDP duration).
    min_rest_required_hours: f64,
    /// Whether the rest after this FDP meets the minimum requirement.
    rest_compliant: bool,
}

#[derive(Serialize)]
struct SectorInFdp {
    shift_id: u64,
    start_hour: u64,
    end_hour: u64,
    duration_hours: u64,
    required_skill: String,
}

/// A crew pairing: a sequence of FDPs from home-base departure to home-base return.
/// A pairing boundary is a rest gap >= HOME_BASE_REST_HOURS (34h).
/// Within a pairing, FDPs are separated by layover rests (10h–33h).
#[derive(Serialize)]
struct Pairing {
    pairing_id: String,
    worker_id: u64,
    worker_skill: String,
    /// Individual FDPs within this pairing (each validated separately).
    fdp_periods: Vec<FdpPeriod>,
    /// Total block hours across all FDPs in this pairing.
    total_block_hours: f64,
    /// Total layover hours within this pairing (time away from base between FDPs).
    total_layover_hours: f64,
    /// Number of FDPs in this pairing.
    fdp_count: usize,
    /// Rest gap after this pairing (home-base rest). None if this is the last pairing.
    home_base_rest_hours: Option<f64>,
    /// True only if ALL FDPs in this pairing are FDP-compliant.
    fdp_compliant: bool,
    /// True only if ALL inter-FDP rests within this pairing are rest-compliant.
    rest_compliant: bool,
    /// Combined violation messages.
    violations: Vec<String>,
}

#[derive(Serialize)]
struct PairingsResponse {
    pairings: Vec<Pairing>,
    total_pairings: usize,
    /// Number of pairings with at least one FDP violation.
    fdp_violations: usize,
    /// Number of pairings with at least one rest violation.
    rest_violations: usize,
    /// Number of fully compliant pairings (no FDP or rest violations).
    compliant_pairings: usize,
}

/// POST /api/pairings
///
/// Correct FTA model (DGCA CAR Section 7 / EASA ORO.FTL):
///
/// 1. Group each worker's shifts into FDPs: consecutive sectors with inter-sector
///    ground time < LAYOVER_REST_HOURS (8h) belong to the same FDP.
///
/// 2. Group FDPs into pairings: consecutive FDPs with inter-FDP rest
///    < HOME_BASE_REST_HOURS (34h) belong to the same pairing. A rest >= 34h
///    ends the pairing (crew returns to home base).
///
/// 3. Validate each FDP:
///    - FDP limit: max_fdp_hours(sector_count, report_hour) per TC CAR 700.15 Table.
///    - Rest compliance: rest_after >= min_rest_after_fdp(fdp_hours).
///      (TC CAR 700.17: rest >= max(8h, preceding FDP duration).)
async fn pairings_handler(
    Json(req): Json<ScheduleAnalysisRequest>,
) -> Result<Json<PairingsResponse>, (StatusCode, String)> {
    let shift_map: HashMap<u64, ShiftInput> = req.shifts.iter().map(|s| (s.id, s.clone())).collect();
    let worker_map: HashMap<u64, WorkerInput> = req.workers.iter().map(|w| (w.id, w.clone())).collect();

    // Group shifts by worker
    let mut worker_shifts: HashMap<u64, Vec<ShiftInput>> = HashMap::new();
    for (shift_id, worker_id) in &req.schedule {
        if let Some(shift) = shift_map.get(shift_id) {
            worker_shifts.entry(*worker_id).or_default().push(shift.clone());
        }
    }

    let mut pairings: Vec<Pairing> = Vec::new();
    let mut pairing_counter = 0u64;

    for (worker_id, mut shifts) in worker_shifts {
        shifts.sort_by_key(|s| s.start_hour);
        let skill = worker_map.get(&worker_id)
            .and_then(|w| w.skills.first())
            .cloned()
            .unwrap_or_default();

        // ── Step 1: group sectors into FDPs ──────────────────────────────────
        // A new FDP starts when the ground time between consecutive sectors >= LAYOVER_REST_HOURS.
        let mut fdp_groups: Vec<Vec<ShiftInput>> = Vec::new();
        let mut current_fdp: Vec<ShiftInput> = Vec::new();
        for shift in &shifts {
            if current_fdp.is_empty() {
                current_fdp.push(shift.clone());
            } else {
                let last = current_fdp.last().unwrap();
                let ground_time = shift.start_hour as f64 - (last.start_hour + last.duration_hours) as f64;
                if ground_time >= LAYOVER_REST_HOURS {
                    fdp_groups.push(current_fdp.clone());
                    current_fdp = vec![shift.clone()];
                } else {
                    current_fdp.push(shift.clone());
                }
            }
        }
        if !current_fdp.is_empty() { fdp_groups.push(current_fdp); }

        // ── Step 2: group FDPs into pairings ─────────────────────────────────
        // A new pairing starts when the rest between consecutive FDPs >= HOME_BASE_REST_HOURS.
        let mut pairing_fdp_groups: Vec<Vec<Vec<ShiftInput>>> = Vec::new();
        let mut current_pairing: Vec<Vec<ShiftInput>> = Vec::new();
        for fdp in &fdp_groups {
            if current_pairing.is_empty() {
                current_pairing.push(fdp.clone());
            } else {
                let prev_fdp = current_pairing.last().unwrap();
                let prev_release = prev_fdp.last().map(|s| s.start_hour + s.duration_hours).unwrap_or(0);
                let next_report = fdp.first().map(|s| s.start_hour).unwrap_or(0);
                let rest_gap = next_report as f64 - prev_release as f64;
                if rest_gap >= HOME_BASE_REST_HOURS {
                    pairing_fdp_groups.push(current_pairing.clone());
                    current_pairing = vec![fdp.clone()];
                } else {
                    current_pairing.push(fdp.clone());
                }
            }
        }
        if !current_pairing.is_empty() { pairing_fdp_groups.push(current_pairing); }

        // ── Step 3: build and validate each pairing ───────────────────────────
        for (pi, fdp_list) in pairing_fdp_groups.iter().enumerate() {
            pairing_counter += 1;

            // Compute rest gap to next pairing (home-base rest)
            let home_base_rest_hours: Option<f64> = pairing_fdp_groups.get(pi + 1).map(|next_pairing| {
                let this_release = fdp_list.last()
                    .and_then(|fdp| fdp.last())
                    .map(|s| s.start_hour + s.duration_hours)
                    .unwrap_or(0);
                let next_report = next_pairing.first()
                    .and_then(|fdp| fdp.first())
                    .map(|s| s.start_hour)
                    .unwrap_or(0);
                next_report as f64 - this_release as f64
            });

            // Build FdpPeriod structs with compliance checks
            let mut fdp_periods: Vec<FdpPeriod> = Vec::new();
            for (fi, fdp_sectors) in fdp_list.iter().enumerate() {
                let report_hour = fdp_sectors.first().map(|s| s.start_hour).unwrap_or(0);
                let release_hour = fdp_sectors.last().map(|s| s.start_hour + s.duration_hours).unwrap_or(0);
                let fdp_hours = (release_hour - report_hour) as f64;
                let sector_count = fdp_sectors.len();
                let report_hour_of_day = report_hour % 24;
                let fdp_limit = max_fdp_hours(sector_count, report_hour_of_day);
                let fdp_compliant = fdp_hours <= fdp_limit;
                let fdp_violation = if !fdp_compliant {
                    Some(format!(
                        "FDP {:.1}h exceeds limit {:.1}h ({} sector{}, report {:02}:00)",
                        fdp_hours, fdp_limit, sector_count,
                        if sector_count == 1 { "" } else { "s" },
                        report_hour_of_day
                    ))
                } else {
                    None
                };

                // Rest after this FDP (gap to next FDP within the pairing)
                let rest_after_hours: Option<f64> = fdp_list.get(fi + 1).map(|next_fdp| {
                    let next_report = next_fdp.first().map(|s| s.start_hour).unwrap_or(0);
                    next_report as f64 - release_hour as f64
                });

                // TC CAR 700.17: rest >= max(8h, preceding FDP duration)
                let min_rest_required = min_rest_after_fdp(fdp_hours);
                let rest_compliant = rest_after_hours
                    .map(|r| r >= min_rest_required)
                    .unwrap_or(true); // last FDP in pairing: rest compliance is home-base rest (checked separately)

                fdp_periods.push(FdpPeriod {
                    sectors: fdp_sectors.iter().map(|s| SectorInFdp {
                        shift_id: s.id,
                        start_hour: s.start_hour,
                        end_hour: s.start_hour + s.duration_hours,
                        duration_hours: s.duration_hours,
                        required_skill: s.required_skill.clone(),
                    }).collect(),
                    report_hour,
                    release_hour,
                    fdp_hours,
                    sector_count,
                    fdp_limit_hours: fdp_limit,
                    fdp_compliant,
                    fdp_violation,
                    rest_after_hours,
                    min_rest_required_hours: min_rest_required,
                    rest_compliant,
                });
            }

            let total_block_hours: f64 = fdp_periods.iter()
                .flat_map(|fp| fp.sectors.iter())
                .map(|s| s.duration_hours as f64)
                .sum();
            let total_layover_hours: f64 = fdp_periods.iter()
                .filter_map(|fp| fp.rest_after_hours)
                .sum();
            let fdp_count = fdp_periods.len();
            let pairing_fdp_compliant = fdp_periods.iter().all(|fp| fp.fdp_compliant);
            let pairing_rest_compliant = fdp_periods.iter().all(|fp| fp.rest_compliant);
            let mut violations: Vec<String> = Vec::new();
            for fp in &fdp_periods {
                if let Some(v) = &fp.fdp_violation { violations.push(v.clone()); }
                if !fp.rest_compliant {
                    if let Some(r) = fp.rest_after_hours {
                        violations.push(format!(
                            "Rest {:.1}h < required {:.1}h (must be >= preceding FDP {:.1}h)",
                            r, fp.min_rest_required_hours, fp.fdp_hours
                        ));
                    }
                }
            }

            pairings.push(Pairing {
                pairing_id: format!("P{:04}", pairing_counter),
                worker_id,
                worker_skill: skill.clone(),
                fdp_periods,
                total_block_hours,
                total_layover_hours,
                fdp_count,
                home_base_rest_hours,
                fdp_compliant: pairing_fdp_compliant,
                rest_compliant: pairing_rest_compliant,
                violations,
            });
        }
    }

    pairings.sort_by(|a, b| a.worker_id.cmp(&b.worker_id)
        .then(
            a.fdp_periods.first().and_then(|fp| fp.sectors.first()).map(|s| s.start_hour).unwrap_or(0)
            .cmp(&b.fdp_periods.first().and_then(|fp| fp.sectors.first()).map(|s| s.start_hour).unwrap_or(0))
        ));

    let fdp_violations = pairings.iter().filter(|p| !p.fdp_compliant).count();
    let rest_violations = pairings.iter().filter(|p| !p.rest_compliant).count();
    let compliant_pairings = pairings.iter().filter(|p| p.fdp_compliant && p.rest_compliant).count();
    let total_pairings = pairings.len();

    Ok(Json(PairingsResponse { pairings, total_pairings, fdp_violations, rest_violations, compliant_pairings }))
}

// ─── Duties: per-worker duty periods with FDP compliance ─────────────────────

#[derive(Serialize)]
struct DutyPeriod {
    duty_id: String,
    worker_id: u64,
    worker_skill: String,
    shift_ids: Vec<u64>,
    report_hour: u64,
    release_hour: u64,
    fdp_hours: f64,
    rest_after_hours: Option<f64>,
    fdp_compliant: bool,
    rest_compliant: bool,
    weekly_hours_compliant: bool,
    violations: Vec<String>,
}

#[derive(Serialize)]
struct DutiesResponse {
    duties: Vec<DutyPeriod>,
    total_duties: usize,
    fdp_violations: usize,
    rest_violations: usize,
    weekly_violations: usize,
}

/// POST /api/duties
/// Returns per-worker duty periods with full FDP compliance checking.
async fn duties_handler(
    Json(req): Json<ScheduleAnalysisRequest>,
) -> Result<Json<DutiesResponse>, (StatusCode, String)> {
    let shift_map: HashMap<u64, ShiftInput> = req.shifts.iter().map(|s| (s.id, s.clone())).collect();
    let worker_map: HashMap<u64, WorkerInput> = req.workers.iter().map(|w| (w.id, w.clone())).collect();

    let mut worker_shifts: HashMap<u64, Vec<ShiftInput>> = HashMap::new();
    for (shift_id, worker_id) in &req.schedule {
        if let Some(shift) = shift_map.get(shift_id) {
            worker_shifts.entry(*worker_id).or_default().push(shift.clone());
        }
    }

    let mut duties: Vec<DutyPeriod> = Vec::new();
    let mut duty_counter = 0u64;

    for (worker_id, mut shifts) in worker_shifts {
        shifts.sort_by_key(|s| s.start_hour);
        let skill = worker_map.get(&worker_id)
            .and_then(|w| w.skills.first())
            .cloned()
            .unwrap_or_default();

        // Group into duty periods (same logic as pairings)
        let mut groups: Vec<Vec<ShiftInput>> = Vec::new();
        let mut current: Vec<ShiftInput> = Vec::new();
        for shift in &shifts {
            if current.is_empty() {
                current.push(shift.clone());
            } else {
                let last = current.last().unwrap();
                let gap = shift.start_hour as f64 - (last.start_hour + last.duration_hours) as f64;
                if gap >= LAYOVER_REST_HOURS {
                    groups.push(current.clone());
                    current = vec![shift.clone()];
                } else {
                    current.push(shift.clone());
                }
            }
        }
        if !current.is_empty() { groups.push(current); }

        // Compute weekly hours for this worker
        let total_hours: f64 = shifts.iter().map(|s| s.duration_hours as f64).sum();
        let weekly_hours_compliant = total_hours <= MAX_WEEKLY_HOURS;

        for (i, group) in groups.iter().enumerate() {
            duty_counter += 1;
            let report_hour = group.first().map(|s| s.start_hour).unwrap_or(0);
            let release_hour = group.last().map(|s| s.start_hour + s.duration_hours).unwrap_or(0);
            let fdp_hours = (release_hour - report_hour) as f64;
            let sector_count = group.len();
            let report_hour_of_day = report_hour % 24;
            let fdp_limit = max_fdp_hours(sector_count, report_hour_of_day);
            let fdp_compliant = fdp_hours <= fdp_limit;

            // Rest after = gap to next duty group
            let rest_after_hours = groups.get(i + 1).map(|next| {
                let next_report = next.first().map(|s| s.start_hour).unwrap_or(0);
                next_report as f64 - release_hour as f64
            });
            // TC CAR 700.17: rest >= max(8h, preceding FDP duration)
            let min_rest_required = min_rest_after_fdp(fdp_hours);
            let rest_compliant = rest_after_hours.map(|r| r >= min_rest_required).unwrap_or(true);

            let mut violations = Vec::new();
            if !fdp_compliant {
                violations.push(format!(
                    "FDP {:.1}h > limit {:.1}h ({} sector{}, report {:02}:00)",
                    fdp_hours, fdp_limit, sector_count,
                    if sector_count == 1 { "" } else { "s" },
                    report_hour_of_day
                ));
            }
            if !rest_compliant {
                violations.push(format!(
                    "Rest {:.1}h < required {:.1}h (>= max(FDP {:.1}h, 10h))",
                    rest_after_hours.unwrap_or(0.0), min_rest_required, fdp_hours
                ));
            }
            if !weekly_hours_compliant {
                violations.push(format!("Weekly hours {:.1}h > max {:.1}h", total_hours, MAX_WEEKLY_HOURS));
            }

            duties.push(DutyPeriod {
                duty_id: format!("D{:05}", duty_counter),
                worker_id,
                worker_skill: skill.clone(),
                shift_ids: group.iter().map(|s| s.id).collect(),
                report_hour,
                release_hour,
                fdp_hours,
                rest_after_hours,
                fdp_compliant,
                rest_compliant,
                weekly_hours_compliant,
                violations,
            });
        }
    }

    duties.sort_by(|a, b| a.worker_id.cmp(&b.worker_id).then(a.report_hour.cmp(&b.report_hour)));

    let fdp_violations = duties.iter().filter(|d| !d.fdp_compliant).count();
    let rest_violations = duties.iter().filter(|d| !d.rest_compliant).count();
    let weekly_violations = duties.iter().filter(|d| !d.weekly_hours_compliant).count();
    let total_duties = duties.len();

    Ok(Json(DutiesResponse { duties, total_duties, fdp_violations, rest_violations, weekly_violations }))
}

// ─── Swap Exchanges: feasible worker swaps for a given shift ─────────────────

#[derive(Serialize)]
struct SwapCandidate {
    shift_id: u64,
    current_worker_id: u64,
    candidate_worker_id: u64,
    candidate_skill: String,
    feasible: bool,
    reason: Option<String>,
    /// Estimated FDP hours for candidate after swap
    candidate_fdp_after: f64,
    /// Whether candidate's FDP would remain compliant after swap
    candidate_fdp_compliant: bool,
}

#[derive(Serialize)]
struct SwapExchangesResponse {
    swaps: Vec<SwapCandidate>,
    total_candidates: usize,
    feasible_swaps: usize,
}

/// POST /api/swap_exchanges
/// For each assigned shift, finds workers who could swap in while remaining FDP-compliant.
async fn swap_exchanges_handler(
    Json(req): Json<ScheduleAnalysisRequest>,
) -> Result<Json<SwapExchangesResponse>, (StatusCode, String)> {
    let shift_map: HashMap<u64, ShiftInput> = req.shifts.iter().map(|s| (s.id, s.clone())).collect();
    let worker_map: HashMap<u64, WorkerInput> = req.workers.iter().map(|w| (w.id, w.clone())).collect();

    // Build per-worker shift lists
    let mut worker_shifts: HashMap<u64, Vec<ShiftInput>> = HashMap::new();
    for (shift_id, worker_id) in &req.schedule {
        if let Some(shift) = shift_map.get(shift_id) {
            worker_shifts.entry(*worker_id).or_default().push(shift.clone());
        }
    }
    // Sort each worker's shifts
    for shifts in worker_shifts.values_mut() {
        shifts.sort_by_key(|s| s.start_hour);
    }

    let mut swaps: Vec<SwapCandidate> = Vec::new();

    for (shift_id, current_worker_id) in &req.schedule {
        let shift = match shift_map.get(shift_id) { Some(s) => s, None => continue };

        // Find candidate workers: same skill, not already assigned at this time
        for worker in &req.workers {
            if worker.id == *current_worker_id { continue; }
            if !worker.skills.contains(&shift.required_skill) { continue; }

            // Check if candidate is free during this shift
            let candidate_shifts = worker_shifts.get(&worker.id).cloned().unwrap_or_default();
            let conflict = candidate_shifts.iter().any(|cs| {
                let cs_end = cs.start_hour + cs.duration_hours;
                let s_end = shift.start_hour + shift.duration_hours;
                // Overlap check
                cs.start_hour < s_end && shift.start_hour < cs_end
            });

            if conflict {
                swaps.push(SwapCandidate {
                    shift_id: *shift_id,
                    current_worker_id: *current_worker_id,
                    candidate_worker_id: worker.id,
                    candidate_skill: worker.skills.first().cloned().unwrap_or_default(),
                    feasible: false,
                    reason: Some("Scheduling conflict: candidate already assigned during this period".to_string()),
                    candidate_fdp_after: 0.0,
                    candidate_fdp_compliant: false,
                });
                continue;
            }

            // Check FDP: would adding this shift violate FDP for the candidate?
            // Find adjacent shifts for candidate to compute new FDP span
            let prev_shift = candidate_shifts.iter().rev()
                .find(|cs| cs.start_hour + cs.duration_hours <= shift.start_hour);
            let next_shift = candidate_shifts.iter()
                .find(|cs| cs.start_hour >= shift.start_hour + shift.duration_hours);

            // Check rest before
            let rest_before_ok = prev_shift.map(|ps| {
                (shift.start_hour as f64 - (ps.start_hour + ps.duration_hours) as f64) >= LAYOVER_REST_HOURS
            }).unwrap_or(true);

            // Check rest after
            let rest_after_ok = next_shift.map(|ns| {
                (ns.start_hour as f64 - (shift.start_hour + shift.duration_hours) as f64) >= LAYOVER_REST_HOURS
            }).unwrap_or(true);

            // Compute FDP span if this shift is added (worst case: adjacent to prev/next)
            let fdp_start = prev_shift
                .filter(|ps| (shift.start_hour as f64 - (ps.start_hour + ps.duration_hours) as f64) < LAYOVER_REST_HOURS)
                .map(|ps| ps.start_hour)
                .unwrap_or(shift.start_hour);
            let fdp_end = next_shift
                .filter(|ns| (ns.start_hour as f64 - (shift.start_hour + shift.duration_hours) as f64) < LAYOVER_REST_HOURS)
                .map(|ns| ns.start_hour + ns.duration_hours)
                .unwrap_or(shift.start_hour + shift.duration_hours);
            let candidate_fdp_after = (fdp_end - fdp_start) as f64;
            // Use conservative 1-sector limit at the candidate's report hour
            let candidate_report_hod = fdp_start % 24;
            let candidate_fdp_limit = max_fdp_hours(1, candidate_report_hod);
            let candidate_fdp_compliant = candidate_fdp_after <= candidate_fdp_limit;

            let feasible = rest_before_ok && rest_after_ok && candidate_fdp_compliant;
            let reason = if !feasible {
                let mut reasons = Vec::new();
                if !rest_before_ok { reasons.push(format!("Insufficient rest before ({:.1}h < {:.1}h)",
                    prev_shift.map(|ps| shift.start_hour as f64 - (ps.start_hour + ps.duration_hours) as f64).unwrap_or(0.0), LAYOVER_REST_HOURS)); }
                if !rest_after_ok { reasons.push(format!("Insufficient rest after ({:.1}h < {:.1}h)",
                    next_shift.map(|ns| ns.start_hour as f64 - (shift.start_hour + shift.duration_hours) as f64).unwrap_or(0.0), LAYOVER_REST_HOURS)); }
                if !candidate_fdp_compliant { reasons.push(format!("FDP would be {:.1}h > {:.1}h", candidate_fdp_after, candidate_fdp_limit)); }
                Some(reasons.join("; "))
            } else {
                None
            };

            swaps.push(SwapCandidate {
                shift_id: *shift_id,
                current_worker_id: *current_worker_id,
                candidate_worker_id: worker.id,
                candidate_skill: worker.skills.first().cloned().unwrap_or_default(),
                feasible,
                reason,
                candidate_fdp_after,
                candidate_fdp_compliant,
            });
        }
    }

    swaps.sort_by(|a, b| a.shift_id.cmp(&b.shift_id).then(b.feasible.cmp(&a.feasible)));

    let feasible_swaps = swaps.iter().filter(|s| s.feasible).count();
    let total_candidates = swaps.len();

    Ok(Json(SwapExchangesResponse { swaps, total_candidates, feasible_swaps }))
}

// ─── INRC Compliance Endpoint ─────────────────────────────────────────────────

/// Request body for POST /api/inrc/compliance.
/// Accepts a schedule as nurse_id → [shift_per_day] (same format as /api/schedule output).
#[derive(Deserialize)]
struct InrcComplianceRequest {
    /// nurse_id → list of shift strings, one per day (empty string = day off).
    schedule: HashMap<String, Vec<String>>,
}

/// Per-nurse compliance summary returned by POST /api/inrc/compliance.
#[derive(Serialize)]
struct NurseComplianceSummary {
    nurse_id: String,
    is_compliant: bool,
    forbidden_succession_violations: usize,
    max_consecutive_work_violations: usize,
    min_consecutive_work_violations: usize,
    min_days_off_violations: usize,
    max_days_off_violations: usize,
    total_violations: usize,
}

/// Response body for POST /api/inrc/compliance.
#[derive(Serialize)]
struct InrcComplianceResponse {
    /// True only if the schedule has zero hard-constraint violations.
    is_legal: bool,
    /// Total hard-constraint violations across all nurses.
    total_violations: usize,
    /// Breakdown by violation type.
    forbidden_succession_violations: usize,
    max_consecutive_work_violations: usize,
    min_consecutive_work_violations: usize,
    min_days_off_violations: usize,
    max_days_off_violations: usize,
    /// Coverage percentage (assigned shifts / required slots × 100).
    coverage_achieved: f64,
    /// Per-nurse compliance summaries.
    nurses: Vec<NurseComplianceSummary>,
    /// Flat list of all violation details.
    violations: Vec<serde_json::Value>,
}

/// POST /api/inrc/compliance
///
/// Validates an INRC nurse roster against the loaded scenario's hard constraints:
///   HC1 — forbidden shift-type successions (e.g. Night → Early violates EU WTD 11h rest)
///   HC2 — max consecutive working days
///   HC3 — min consecutive working days
///   HC4 — min consecutive days off
///   HC5 — max consecutive days off
///
/// Requires a scenario to be loaded first via POST /api/load-scenario.
/// The schedule format is identical to the output of POST /api/schedule:
///   { "schedule": { "nurse_id": ["Early", "", "Night", ...] } }
async fn inrc_compliance_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(req): Json<InrcComplianceRequest>,
) -> impl IntoResponse {
    let app_state = state.lock().unwrap();
    let scenario = match app_state.scenario.as_ref() {
        Some(s) => s,
        None => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "No INRC scenario loaded. Call POST /api/load-scenario first."
            })),
        ).into_response(),
    };

    let report = ultracrew::inrc::validator::validate_schedule(&req.schedule, scenario);

    // Build per-nurse summaries
    let mut nurse_map: HashMap<String, NurseComplianceSummary> = scenario.nurses.iter().map(|n| {
        (n.id.clone(), NurseComplianceSummary {
            nurse_id: n.id.clone(),
            is_compliant: true,
            forbidden_succession_violations: 0,
            max_consecutive_work_violations: 0,
            min_consecutive_work_violations: 0,
            min_days_off_violations: 0,
            max_days_off_violations: 0,
            total_violations: 0,
        })
    }).collect();

    let mut violations_json: Vec<serde_json::Value> = Vec::new();

    for detail in &report.details {
        violations_json.push(serde_json::json!({
            "nurse_id": detail.nurse_id,
            "day": detail.day,
            "constraint": detail.constraint,
            "actual": detail.actual,
            "required": detail.required,
        }));

        if let Some(ns) = nurse_map.get_mut(&detail.nurse_id) {
            ns.total_violations += 1;
            ns.is_compliant = false;
            match detail.constraint.as_str() {
                "forbidden_shift_type_successions"  => ns.forbidden_succession_violations += 1,
                "max_consecutive_working_days"      => ns.max_consecutive_work_violations += 1,
                "min_consecutive_working_days"      => ns.min_consecutive_work_violations += 1,
                "min_consecutive_days_off"          => ns.min_days_off_violations += 1,
                "max_consecutive_days_off"          => ns.max_days_off_violations += 1,
                _ => {}
            }
        }
    }

    let nurses: Vec<NurseComplianceSummary> = scenario.nurses.iter()
        .filter_map(|n| nurse_map.remove(&n.id))
        .collect();

    let total_violations = report.forbidden_successions
        + report.max_consecutive_work_violations
        + report.min_consecutive_work_violations
        + report.min_days_off_violations
        + report.max_days_off_violations;

    let response = InrcComplianceResponse {
        is_legal: report.is_legal,
        total_violations,
        forbidden_succession_violations: report.forbidden_successions,
        max_consecutive_work_violations: report.max_consecutive_work_violations,
        min_consecutive_work_violations: report.min_consecutive_work_violations,
        min_days_off_violations: report.min_days_off_violations,
        max_days_off_violations: report.max_days_off_violations,
        coverage_achieved: report.coverage_achieved,
        nurses,
        violations: violations_json,
    };

    (StatusCode::OK, Json(serde_json::to_value(response).unwrap())).into_response()
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

// ─── Pilot Portal Evidence Endpoint ──────────────────────────────────────────

/// DSP evidence record — written to disk as JSON for EL-001 ingestion.
#[derive(Debug, Serialize, Deserialize, Clone)]
struct PilotSessionRecord {
    /// Evidence ID (DSP-NNN)
    id: String,
    /// ISO 8601 timestamp
    timestamp: String,
    /// Dispatcher identifier (anonymised)
    dispatcher_id: String,
    /// Dispatcher role and experience level
    dispatcher_role: String,
    /// Scenario dataset used
    scenario_id: String,
    /// Software version
    adapter_version: String,
    /// Coverage from optimizer output
    coverage_pct: f64,
    /// Hard violations from optimizer output
    hard_violations: u32,
    /// Rest violations from optimizer output
    rest_violations: u32,
    /// Fitness score from optimizer output
    fitness: f64,
    /// Optimizer runtime in seconds
    runtime_secs: f64,
    /// Time from disruption event to accepted recovery plan (seconds), if measured
    disruption_recovery_secs: Option<f64>,
    /// Number of manual edits made after optimization
    manual_edits: u32,
    /// Recommendations presented to dispatcher
    recommendations_presented: u32,
    /// Recommendations accepted by dispatcher
    recommendations_accepted: u32,
    /// Recommendations rejected by dispatcher
    recommendations_rejected: u32,
    /// Per-recommendation decisions: [{id, action, reason}]
    recommendation_decisions: Vec<RecommendationDecision>,
    /// Explanation usefulness rating (1–5)
    explanation_usefulness: u8,
    /// Free-text qualitative comments from dispatcher
    dispatcher_comments: String,
    /// Session completed successfully
    session_complete: bool,
    // ── Commercial evidence fields (Stream 4) ────────────────────────────────
    /// Organisation name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    org_name: Option<String>,
    /// Current manual scheduling time in minutes (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    baseline_scheduling_mins: Option<f64>,
    /// Current disruption recovery time in minutes (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    baseline_disruption_mins: Option<f64>,
    /// Product gaps / what was missing (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    product_gaps: Option<String>,
    /// Agreed next step (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    next_steps: Option<String>,
    /// Willingness to run a paid pilot (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    willing_to_pilot: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct RecommendationDecision {
    recommendation_text: String,
    action: String, // "accepted" | "rejected"
    rejection_reason: Option<String>,
}

#[derive(Deserialize)]
struct PilotSessionInput {
    dispatcher_id: String,
    dispatcher_role: String,
    scenario_id: String,
    coverage_pct: f64,
    hard_violations: u32,
    rest_violations: u32,
    fitness: f64,
    runtime_secs: f64,
    disruption_recovery_secs: Option<f64>,
    manual_edits: u32,
    recommendations_presented: u32,
    recommendations_accepted: u32,
    recommendations_rejected: u32,
    recommendation_decisions: Vec<RecommendationDecision>,
    explanation_usefulness: u8,
    dispatcher_comments: String,
    session_complete: bool,
    // ── Commercial evidence fields (Stream 4) ────────────────────────────────
    #[serde(default)]
    org_name: Option<String>,
    #[serde(default)]
    baseline_scheduling_mins: Option<f64>,
    #[serde(default)]
    baseline_disruption_mins: Option<f64>,
    #[serde(default)]
    product_gaps: Option<String>,
    #[serde(default)]
    next_steps: Option<String>,
    #[serde(default)]
    willing_to_pilot: Option<String>,
}

async fn pilot_session_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: axum::http::HeaderMap,
    Json(input): Json<PilotSessionInput>,
) -> Result<Json<PilotSessionRecord>, (StatusCode, String)> {
    // CSRF validation — double-submit: X-CSRF-Token header must match stored token
    {
        let s = state.lock().unwrap();
        let stored = &s.csrf_token;
        let provided = headers
            .get("x-csrf-token")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if stored.is_empty() || provided != stored.as_str() {
            return Err((StatusCode::FORBIDDEN, "CSRF token invalid or missing. Call GET /api/csrf-token first.".to_string()));
        }
    }

    let session_id = uuid::Uuid::new_v4().to_string();
    let dsp_id = format!("DSP-{}", &session_id[..8].to_uppercase());

    // ISO 8601 timestamp (UTC)
    let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let record = PilotSessionRecord {
        id: dsp_id.clone(),
        timestamp,
        dispatcher_id: input.dispatcher_id,
        dispatcher_role: input.dispatcher_role,
        scenario_id: input.scenario_id,
        adapter_version: ultracrew::health::ADAPTER_VERSION.to_string(),
        coverage_pct: input.coverage_pct,
        hard_violations: input.hard_violations,
        rest_violations: input.rest_violations,
        fitness: input.fitness,
        runtime_secs: input.runtime_secs,
        disruption_recovery_secs: input.disruption_recovery_secs,
        manual_edits: input.manual_edits,
        recommendations_presented: input.recommendations_presented,
        recommendations_accepted: input.recommendations_accepted,
        recommendations_rejected: input.recommendations_rejected,
        recommendation_decisions: input.recommendation_decisions,
        explanation_usefulness: input.explanation_usefulness,
        dispatcher_comments: input.dispatcher_comments,
        session_complete: input.session_complete,
        org_name: input.org_name,
        baseline_scheduling_mins: input.baseline_scheduling_mins,
        baseline_disruption_mins: input.baseline_disruption_mins,
        product_gaps: input.product_gaps,
        next_steps: input.next_steps,
        willing_to_pilot: input.willing_to_pilot,
    };

    // ── Persist: Supabase REST (preferred) or local disk (fallback) ──────────
    let supabase_url = std::env::var("SUPABASE_URL").ok();
    let supabase_key = std::env::var("SUPABASE_ANON_KEY").ok();

    match (supabase_url, supabase_key) {
        (Some(url), Some(key)) => {
            // Supabase PostgREST insert
            let endpoint = format!("{}/rest/v1/pilot_sessions", url.trim_end_matches('/'));
            let client = reqwest::Client::new();
            let resp = client
                .post(&endpoint)
                .header("apikey", &key)
                .header("Authorization", format!("Bearer {}", key))
                .header("Content-Type", "application/json")
                .header("Prefer", "return=minimal")
                .json(&record)
                .send()
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Supabase request failed: {}", e)))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err((StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Supabase insert failed ({}): {}", status, body)));
            }
            println!("Pilot session persisted to Supabase: {}", dsp_id);
        }
        _ => {
            // Local disk fallback (development / no Supabase configured)
            let dir = std::path::Path::new("pilot_sessions");
            std::fs::create_dir_all(dir)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create pilot_sessions dir: {}", e)))?;
            let file_path = dir.join(format!("{}.json", dsp_id));
            let json = serde_json::to_string_pretty(&record)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Serialization failed: {}", e)))?;
            std::fs::write(&file_path, &json)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to write session record: {}", e)))?;
            println!("Pilot session recorded to disk (no Supabase): {} -> {:?}", dsp_id, file_path);
        }
    }

    Ok(Json(record))
}

async fn list_pilot_sessions_handler() -> Result<Json<Vec<PilotSessionRecord>>, (StatusCode, String)> {
    let supabase_url = std::env::var("SUPABASE_URL").ok();
    let supabase_key = std::env::var("SUPABASE_ANON_KEY").ok();

    match (supabase_url, supabase_key) {
        (Some(url), Some(key)) => {
            // Supabase PostgREST select — ordered by timestamp ascending
            let endpoint = format!(
                "{}/rest/v1/pilot_sessions?order=timestamp.asc",
                url.trim_end_matches('/')
            );
            let client = reqwest::Client::new();
            let resp = client
                .get(&endpoint)
                .header("apikey", &key)
                .header("Authorization", format!("Bearer {}", key))
                .header("Accept", "application/json")
                .send()
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Supabase request failed: {}", e)))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err((StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Supabase select failed ({}): {}", status, body)));
            }

            let records: Vec<PilotSessionRecord> = resp
                .json()
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Supabase response parse failed: {}", e)))?;
            Ok(Json(records))
        }
        _ => {
            // Local disk fallback
            let dir = std::path::Path::new("pilot_sessions");
            if !dir.exists() {
                return Ok(Json(vec![]));
            }
            let mut records = Vec::new();
            let entries = std::fs::read_dir(dir)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read pilot_sessions: {}", e)))?;
            for entry in entries.flatten() {
                if entry.path().extension().map(|e| e == "json").unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if let Ok(record) = serde_json::from_str::<PilotSessionRecord>(&content) {
                            records.push(record);
                        }
                    }
                }
            }
            records.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
            Ok(Json(records))
        }
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
) -> impl IntoResponse {
    let state = state.lock().unwrap();
    match &state.baseline_state {
        Some(s) => (StatusCode::OK, Json(serde_json::to_value(&s.balances).unwrap())).into_response(),
        None => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"error": "Simulation state not loaded. This endpoint is not used by the pilot portal."}))).into_response(),
    }
}

async fn get_dashboard_handler(
    State(state): State<Arc<Mutex<AppState>>>,
) -> impl IntoResponse {
    let state = state.lock().unwrap();
    match &state.baseline_state {
        Some(s) => (StatusCode::OK, Json(serde_json::to_value(&s.dashboard).unwrap())).into_response(),
        None => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"error": "Simulation state not loaded. This endpoint is not used by the pilot portal."}))).into_response(),
    }
}

// Handler to return list of nurses for UI
async fn get_nurses_handler(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let state = state.lock().unwrap();
    match &state.scenario {
        Some(sc) => (StatusCode::OK, Json(json!({ "nurses": sc.nurses.clone() }))).into_response(),
        None => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"error": "INRC scenario not loaded. This endpoint is not used by the pilot portal."}))).into_response(),
    }
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    // ── Lazy initialisation ──────────────────────────────────────────────────
    // The pilot portal does not require the INRC nurse-scheduling scenario.
    // Simulation endpoints (/api/state, /api/simulations/*, /api/balance,
    // /api/dashboard, /api/nurses) return 503 when scenario/baseline_state
    // are None.  The INRC scenario can be loaded at runtime by a future
    // endpoint if needed.
    
    let app_state = Arc::new(Mutex::new(AppState {
        scenario: None,
        baseline_state: None,
        original_state: None,
        last_solution: None,
        last_request: None,
        decisions: Vec::new(),
        schedule_versions: Vec::new(),
        csrf_token: String::new(),
    }));

    // ── DELETED BLOCK (lines removed) ────────────────────────────────────────
    // The INRC scenario loading, feasibility analysis, schedule construction,
    // pareto frontier computation, and SimulationState initialisation that
    // previously lived here have been removed.  The server now starts cleanly
    // without any test fixture dependency.  Simulation endpoints return 503
    // until a scenario is loaded at runtime.
    // ─────────────────────────────────────────────────────────────────────────


    // ── Rate limiting ─────────────────────────────────────────────────────────
    // 10 requests/second per IP, burst of 20.  Permissive for local dev/demo;
    // tighten before production deployment.
    // Returns HTTP 429 Too Many Requests when the limit is exceeded.
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(10)
        .burst_size(20)
        .finish()
        .expect("Invalid governor configuration");
    let governor_conf = std::sync::Arc::new(governor_conf);

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/csrf-token", get(csrf_token_handler))
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
        // Flight duty analysis endpoints (pairings / duties / swap exchanges)
        .route("/api/pairings", post(pairings_handler))
        .route("/api/duties", post(duties_handler))
        .route("/api/swap_exchanges", post(swap_exchanges_handler))
        // Pilot portal evidence endpoints
        .route("/api/pilot/session", post(pilot_session_handler))
        .route("/api/pilot/sessions", get(list_pilot_sessions_handler))
        .route("/api/load-scenario", post(load_scenario_handler))
        // INRC nurse rostering compliance endpoint
        .route("/api/inrc/compliance", post(inrc_compliance_handler))
        .with_state(app_state)
        .layer(GovernorLayer { config: governor_conf })
        .layer(cors);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let bind_addr = format!("0.0.0.0:{}", port);
    let addr = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
    println!("UltraCrew Server running on http://0.0.0.0:{}", port);
    axum::serve(addr, app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await.unwrap();
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
