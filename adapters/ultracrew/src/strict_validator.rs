/// S2-01 — Strict-mode data import validator
///
/// Validates a fully-parsed `ScheduleRequest` and returns a structured
/// `ValidationReport` containing every violation found in a single pass.
/// Unlike the per-field checks in `generic_import`, this module:
///
/// - Collects ALL violations before returning (no early exit).
/// - Classifies each violation by severity (`Error` vs `Warning`).
/// - Assigns a stable error code to every violation class.
/// - Provides a machine-readable `ValidationReport` suitable for
///   REST API responses, CLI output, and log ingestion.
///
/// # Usage
///
/// ```rust
/// use ultracrew::strict_validator::validate_request;
///
/// let report = validate_request(&request);
/// if !report.is_valid() {
///     eprintln!("{}", report.display());
///     std::process::exit(1);
/// }
/// ```
///
/// # Error codes
///
/// | Code   | Severity | Description |
/// |--------|----------|-------------|
/// | V-001  | Error    | Workers list is empty |
/// | V-002  | Error    | Shifts list is empty |
/// | V-003  | Error    | Duplicate worker ID |
/// | V-004  | Error    | Duplicate shift ID |
/// | V-005  | Error    | Worker has no skills |
/// | V-006  | Error    | Shift duration is zero |
/// | V-007  | Error    | Shift end exceeds planning horizon |
/// | V-008  | Error    | Required skill has no qualified worker |
/// | V-009  | Warning  | Worker has no shifts that match their skills |
/// | V-010  | Warning  | Shift start_hour is unusually large (> 10 000 h) |
/// | V-011  | Warning  | generation_limit is very low (< 50) |
/// | V-012  | Warning  | max_hours_per_worker is less than the shortest shift duration |
/// | V-013  | Error    | Skill name is blank or whitespace-only |
/// | V-014  | Warning  | Historical workload references an unknown worker ID |

use std::collections::{HashMap, HashSet};
use serde::Serialize;

use crate::public_contracts::ScheduleRequest;
use crate::models::Skill;

// ─── Severity ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error   => write!(f, "ERROR"),
            Severity::Warning => write!(f, "WARN "),
        }
    }
}

// ─── ValidationIssue ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ValidationIssue {
    /// Stable error code, e.g. "V-003".
    pub code: &'static str,
    /// Human-readable description of the violation.
    pub message: String,
    /// Severity level.
    pub severity: Severity,
    /// Optional context: which entity triggered the issue.
    pub context: Option<String>,
}

impl ValidationIssue {
    fn error(code: &'static str, message: impl Into<String>, context: Option<String>) -> Self {
        Self { code, message: message.into(), severity: Severity::Error, context }
    }

    fn warning(code: &'static str, message: impl Into<String>, context: Option<String>) -> Self {
        Self { code, message: message.into(), severity: Severity::Warning, context }
    }
}

impl std::fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ctx = self.context.as_deref().unwrap_or("");
        if ctx.is_empty() {
            write!(f, "[{}] {} — {}", self.severity, self.code, self.message)
        } else {
            write!(f, "[{}] {} ({}) — {}", self.severity, self.code, ctx, self.message)
        }
    }
}

// ─── ValidationReport ────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ValidationReport {
    /// True only when there are zero Error-severity issues.
    pub valid: bool,
    /// Total number of Error-severity issues.
    pub error_count: usize,
    /// Total number of Warning-severity issues.
    pub warning_count: usize,
    /// All issues found, in discovery order.
    pub issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    fn new(issues: Vec<ValidationIssue>) -> Self {
        let error_count   = issues.iter().filter(|i| i.severity == Severity::Error).count();
        let warning_count = issues.iter().filter(|i| i.severity == Severity::Warning).count();
        Self { valid: error_count == 0, error_count, warning_count, issues }
    }

    /// Returns `true` when there are no Error-severity issues.
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// One-line summary suitable for CLI output.
    pub fn summary(&self) -> String {
        if self.valid {
            format!(
                "Validation passed ({} warning{})",
                self.warning_count,
                if self.warning_count == 1 { "" } else { "s" }
            )
        } else {
            format!(
                "Validation FAILED: {} error{}, {} warning{}",
                self.error_count,
                if self.error_count == 1 { "" } else { "s" },
                self.warning_count,
                if self.warning_count == 1 { "" } else { "s" }
            )
        }
    }

    /// Multi-line human-readable report for CLI display.
    pub fn display(&self) -> String {
        let mut out = String::new();
        out.push_str("── Validation Report ──────────────────────────────\n");
        out.push_str(&format!("  Status:   {}\n", if self.valid { "PASS ✓" } else { "FAIL ✗" }));
        out.push_str(&format!("  Errors:   {}\n", self.error_count));
        out.push_str(&format!("  Warnings: {}\n", self.warning_count));
        if !self.issues.is_empty() {
            out.push_str("────────────────────────────────────────────────────\n");
            for issue in &self.issues {
                out.push_str(&format!("  {}\n", issue));
            }
        }
        out.push_str("────────────────────────────────────────────────────\n");
        out
    }

    /// Serialise to a pretty-printed JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

// ─── Public entry point ───────────────────────────────────────────────────────

/// Validate a `ScheduleRequest` in strict mode.
///
/// Collects all violations in a single pass and returns a `ValidationReport`.
/// Does not modify the request.
pub fn validate_request(req: &ScheduleRequest) -> ValidationReport {
    let mut issues: Vec<ValidationIssue> = Vec::new();

    // ── V-001: Workers list must not be empty ─────────────────────────────────
    if req.workers.is_empty() {
        issues.push(ValidationIssue::error(
            "V-001",
            "Workers list is empty. At least one worker is required.",
            None,
        ));
    }

    // ── V-002: Shifts list must not be empty ──────────────────────────────────
    if req.shifts.is_empty() {
        issues.push(ValidationIssue::error(
            "V-002",
            "Shifts list is empty. At least one shift is required.",
            None,
        ));
    }

    // ── V-003: Worker IDs must be unique ──────────────────────────────────────
    {
        let mut seen: HashSet<u64> = HashSet::new();
        for w in &req.workers {
            if !seen.insert(w.id) {
                issues.push(ValidationIssue::error(
                    "V-003",
                    format!("Duplicate worker id {}. Worker IDs must be unique.", w.id),
                    Some(format!("worker {}", w.id)),
                ));
            }
        }
    }

    // ── V-004: Shift IDs must be unique ───────────────────────────────────────
    {
        let mut seen: HashSet<u64> = HashSet::new();
        for s in &req.shifts {
            if !seen.insert(s.id) {
                issues.push(ValidationIssue::error(
                    "V-004",
                    format!("Duplicate shift id {}. Shift IDs must be unique.", s.id),
                    Some(format!("shift {}", s.id)),
                ));
            }
        }
    }

    // ── V-005: Each worker must have at least one skill ───────────────────────
    for w in &req.workers {
        if w.skills.is_empty() {
            issues.push(ValidationIssue::error(
                "V-005",
                format!(
                    "Worker {} has no skills. At least one skill is required per worker.",
                    w.id
                ),
                Some(format!("worker {}", w.id)),
            ));
        }
    }

    // ── V-013: Skill names must not be blank ──────────────────────────────────
    for w in &req.workers {
        for skill in &w.skills {
            if skill.0.trim().is_empty() {
                issues.push(ValidationIssue::error(
                    "V-013",
                    format!(
                        "Worker {} has a blank skill name. Skill names must be non-empty strings.",
                        w.id
                    ),
                    Some(format!("worker {}", w.id)),
                ));
            }
        }
    }
    for s in &req.shifts {
        if s.required_skill.0.trim().is_empty() {
            issues.push(ValidationIssue::error(
                "V-013",
                format!(
                    "Shift {} has a blank required_skill. Skill names must be non-empty strings.",
                    s.id
                ),
                Some(format!("shift {}", s.id)),
            ));
        }
    }

    // ── V-006: Shift duration must be > 0 ────────────────────────────────────
    for s in &req.shifts {
        if s.duration_hours == 0 {
            issues.push(ValidationIssue::error(
                "V-006",
                format!(
                    "Shift {} has duration_hours=0. Duration must be at least 1 hour.",
                    s.id
                ),
                Some(format!("shift {}", s.id)),
            ));
        }
    }

    // ── V-007: Shift end must not exceed planning horizon ─────────────────────
    let horizon = req.scenario
        .as_ref()
        .and_then(|sc| sc.planning_horizon_hours)
        .unwrap_or(168.0) as u64;

    for s in &req.shifts {
        let end = s.start_hour + s.duration_hours;
        if end > horizon {
            issues.push(ValidationIssue::error(
                "V-007",
                format!(
                    "Shift {} ends at hour {} which exceeds the planning horizon of {} hours. \
                     Reduce start_hour or duration_hours, or increase planning_horizon_hours.",
                    s.id, end, horizon
                ),
                Some(format!("shift {}", s.id)),
            ));
        }
    }

    // ── V-008: Every required skill must have at least one qualified worker ───
    {
        let worker_skills: HashSet<&Skill> = req.workers
            .iter()
            .flat_map(|w| w.skills.iter())
            .collect();

        let mut uncovered: HashMap<&str, Vec<u64>> = HashMap::new();
        for s in &req.shifts {
            if !worker_skills.contains(&s.required_skill) {
                uncovered
                    .entry(s.required_skill.0.as_str())
                    .or_default()
                    .push(s.id);
            }
        }
        for (skill, shift_ids) in &uncovered {
            let ids_str = shift_ids.iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            issues.push(ValidationIssue::error(
                "V-008",
                format!(
                    "No worker has skill '{}', which is required by shift(s): {}. \
                     Add a worker with this skill or correct the skill name.",
                    skill, ids_str
                ),
                Some(format!("skill '{}'", skill)),
            ));
        }
    }

    // ── V-009 (warning): Worker with no matching shifts ───────────────────────
    {
        let shift_skills: HashSet<&Skill> = req.shifts
            .iter()
            .map(|s| &s.required_skill)
            .collect();

        for w in &req.workers {
            let has_match = w.skills.iter().any(|sk| shift_skills.contains(sk));
            if !has_match {
                issues.push(ValidationIssue::warning(
                    "V-009",
                    format!(
                        "Worker {} has skills [{}] but none of the shifts require those skills. \
                         This worker will not be assigned any shifts.",
                        w.id,
                        w.skills.iter().map(|s| s.0.as_str()).collect::<Vec<_>>().join(", ")
                    ),
                    Some(format!("worker {}", w.id)),
                ));
            }
        }
    }

    // ── V-010 (warning): Unusually large start_hour ───────────────────────────
    for s in &req.shifts {
        if s.start_hour > 10_000 {
            issues.push(ValidationIssue::warning(
                "V-010",
                format!(
                    "Shift {} has start_hour={} which is unusually large (> 10 000 h). \
                     Verify this is intentional.",
                    s.id, s.start_hour
                ),
                Some(format!("shift {}", s.id)),
            ));
        }
    }

    // ── V-011 (warning): Very low generation_limit ────────────────────────────
    if let Some(gen) = req.generation_limit {
        if gen < 50 {
            issues.push(ValidationIssue::warning(
                "V-011",
                format!(
                    "generation_limit={} is very low. Solution quality may be poor. \
                     Recommended minimum: 200. SunAir canonical: 500.",
                    gen
                ),
                None,
            ));
        }
    }

    // ── V-012 (warning): max_hours_per_worker < shortest shift ───────────────
    if let Some(sc) = &req.scenario {
        if let Some(max_h) = sc.max_hours_per_worker {
            if let Some(min_dur) = req.shifts.iter().map(|s| s.duration_hours).min() {
                if (max_h as u64) < min_dur {
                    issues.push(ValidationIssue::warning(
                        "V-012",
                        format!(
                            "max_hours_per_worker={} is less than the shortest shift duration ({} h). \
                             No worker can be assigned any shift. Increase max_hours_per_worker.",
                            max_h, min_dur
                        ),
                        None,
                    ));
                }
            }
        }
    }

    // ── V-014 (warning): Historical workload references unknown worker ─────────
    if let Some(hist) = &req.historical_workloads {
        let worker_ids: HashSet<u64> = req.workers.iter().map(|w| w.id).collect();
        for wid in hist.keys() {
            if !worker_ids.contains(wid) {
                issues.push(ValidationIssue::warning(
                    "V-014",
                    format!(
                        "historical_workloads references worker id {} which does not exist \
                         in the workers list. This entry will be ignored.",
                        wid
                    ),
                    Some(format!("worker {}", wid)),
                ));
            }
        }
    }

    ValidationReport::new(issues)
}

/// Convenience: load JSON from a file path and validate in strict mode.
///
/// Returns `(ScheduleRequest, ValidationReport)` on successful parse,
/// or an error string if the file cannot be read or parsed.
pub fn validate_json_file(
    path: &std::path::Path,
) -> Result<(ScheduleRequest, ValidationReport), String> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read '{}': {}", path.display(), e))?;
    let req: ScheduleRequest = serde_json::from_str(&data)
        .map_err(|e| format!("JSON parse error in '{}': {}", path.display(), e))?;
    let report = validate_request(&req);
    Ok((req, report))
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::models::{Worker, Shift};
    use crate::public_contracts::{ScheduleRequest, Scenario};

    fn minimal_valid() -> ScheduleRequest {
        ScheduleRequest {
            workers: vec![
                Worker { id: 1, skills: vec![Skill::new("Captain")] },
                Worker { id: 2, skills: vec![Skill::new("CabinCrew")] },
            ],
            shifts: vec![
                Shift { id: 1, start_hour: 6,  duration_hours: 8, required_skill: Skill::new("Captain"), crew_role: None, flight_id: None },
                Shift { id: 2, start_hour: 6,  duration_hours: 8, required_skill: Skill::new("CabinCrew"), crew_role: None, flight_id: None },
            ],
            historical_workloads: None,
            rng_seed: Some(42),
            generation_limit: Some(200),
            scenario: Some(Scenario { leave_requests: None, minimum_rest_hours: Some(10), 
                planning_horizon_hours: Some(168.0),
                max_hours_per_worker: Some(48.0),
            }),
        }
    }

    #[test]
    fn test_valid_request_passes() {
        let report = validate_request(&minimal_valid());
        assert!(report.is_valid(), "Expected valid:\n{}", report.display());
        assert_eq!(report.error_count, 0);
    }

    #[test]
    fn test_v001_empty_workers() {
        let mut req = minimal_valid();
        req.workers.clear();
        let report = validate_request(&req);
        assert!(!report.is_valid());
        assert!(report.issues.iter().any(|i| i.code == "V-001"));
    }

    #[test]
    fn test_v002_empty_shifts() {
        let mut req = minimal_valid();
        req.shifts.clear();
        let report = validate_request(&req);
        assert!(!report.is_valid());
        assert!(report.issues.iter().any(|i| i.code == "V-002"));
    }

    #[test]
    fn test_v003_duplicate_worker_id() {
        let mut req = minimal_valid();
        req.workers.push(Worker { id: 1, skills: vec![Skill::new("Captain")] });
        let report = validate_request(&req);
        assert!(!report.is_valid());
        assert!(report.issues.iter().any(|i| i.code == "V-003"));
    }

    #[test]
    fn test_v004_duplicate_shift_id() {
        let mut req = minimal_valid();
        req.shifts.push(Shift { id: 1, start_hour: 20, duration_hours: 8, required_skill: Skill::new("Captain"), crew_role: None, flight_id: None });
        let report = validate_request(&req);
        assert!(!report.is_valid());
        assert!(report.issues.iter().any(|i| i.code == "V-004"));
    }

    #[test]
    fn test_v005_worker_no_skills() {
        let mut req = minimal_valid();
        req.workers[0].skills.clear();
        let report = validate_request(&req);
        assert!(!report.is_valid());
        assert!(report.issues.iter().any(|i| i.code == "V-005"));
    }

    #[test]
    fn test_v006_zero_duration_shift() {
        let mut req = minimal_valid();
        req.shifts[0].duration_hours = 0;
        let report = validate_request(&req);
        assert!(!report.is_valid());
        assert!(report.issues.iter().any(|i| i.code == "V-006"));
    }

    #[test]
    fn test_v007_shift_exceeds_horizon() {
        let mut req = minimal_valid();
        req.shifts[0].start_hour = 165;
        req.shifts[0].duration_hours = 8; // ends at 173 > 168
        let report = validate_request(&req);
        assert!(!report.is_valid());
        assert!(report.issues.iter().any(|i| i.code == "V-007"));
    }

    #[test]
    fn test_v008_uncovered_skill() {
        let mut req = minimal_valid();
        req.shifts.push(Shift {
            id: 99,
            start_hour: 10,
            duration_hours: 8,
            required_skill: Skill::new("FirstOfficer"),
            crew_role: Some("FirstOfficer".to_string()),
            flight_id: Some("FL99".to_string()),
        });
        let report = validate_request(&req);
        assert!(!report.is_valid());
        assert!(report.issues.iter().any(|i| i.code == "V-008"));
    }

    #[test]
    fn test_v009_worker_no_matching_shifts_is_warning() {
        let mut req = minimal_valid();
        req.workers.push(Worker { id: 99, skills: vec![Skill::new("Mechanic")] });
        let report = validate_request(&req);
        // V-009 is a warning — report must still be valid
        assert!(report.is_valid(), "V-009 is a warning, report should be valid");
        assert!(report.issues.iter().any(|i| i.code == "V-009" && i.severity == Severity::Warning));
    }

    #[test]
    fn test_v011_low_generation_limit_is_warning() {
        let mut req = minimal_valid();
        req.generation_limit = Some(10);
        let report = validate_request(&req);
        assert!(report.is_valid()); // warning only
        assert!(report.issues.iter().any(|i| i.code == "V-011"));
    }

    #[test]
    fn test_v013_blank_skill_name() {
        let mut req = minimal_valid();
        req.workers[0].skills[0] = Skill::new("   ");
        let report = validate_request(&req);
        assert!(!report.is_valid());
        assert!(report.issues.iter().any(|i| i.code == "V-013"));
    }

    #[test]
    fn test_v014_unknown_historical_workload_worker_is_warning() {
        let mut req = minimal_valid();
        let mut hist = HashMap::new();
        hist.insert(999u64, vec![40.0, 38.0]);
        req.historical_workloads = Some(hist);
        let report = validate_request(&req);
        assert!(report.is_valid()); // warning only
        assert!(report.issues.iter().any(|i| i.code == "V-014"));
    }

    #[test]
    fn test_multiple_errors_all_collected() {
        // Both V-001 and V-002 must appear — validator must not stop at first error
        let req = ScheduleRequest {
            workers: vec![],
            shifts: vec![],
            historical_workloads: None,
            rng_seed: None,
            generation_limit: None,
            scenario: None,
            // ScheduleRequest has exactly these 6 fields
        };
        let report = validate_request(&req);
        assert!(!report.is_valid());
        assert!(report.issues.iter().any(|i| i.code == "V-001"));
        assert!(report.issues.iter().any(|i| i.code == "V-002"));
        assert!(report.error_count >= 2);
    }

    #[test]
    fn test_summary_pass() {
        let report = validate_request(&minimal_valid());
        assert!(report.summary().contains("passed"));
    }

    #[test]
    fn test_summary_fail() {
        let mut req = minimal_valid();
        req.workers.clear();
        let report = validate_request(&req);
        assert!(report.summary().contains("FAILED"));
    }

    #[test]
    fn test_to_json_is_valid() {
        let report = validate_request(&minimal_valid());
        let json = report.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json)
            .expect("report JSON must be valid");
        assert!(parsed.get("valid").is_some());
        assert!(parsed.get("error_count").is_some());
        assert!(parsed.get("issues").is_some());
    }

    #[test]
    fn test_sunair_demo_passes_strict_validation() {
        // Inline the canonical SunAir scenario as a regression guard.
        // If this test fails, either the dataset or the validator has changed.
        let req = ScheduleRequest {
            workers: (1u64..=20).map(|id| Worker {
                id,
                skills: vec![Skill::new(match id {
                    1..=4  => "Captain",
                    5..=9  => "FirstOfficer",
                    _      => "CabinCrew",
                })],
            }).collect(),
            shifts: (1u64..=42).map(|id| {
                let block_offset: u64 = if id <= 21 { 6 } else { 78 };
                let pos = (id - 1) % 21;
                let skill = match pos {
                    0..=3  => "Captain",
                    4..=6  => "FirstOfficer",
                    _      => "CabinCrew",
                };
                Shift {
                    id,
                    start_hour: block_offset + pos * 2,
                    duration_hours: 8,
                    required_skill: Skill::new(skill),
                    crew_role: Some(skill.to_string()),
                    flight_id: Some(format!("FL{}", id)),
                }
            }).collect(),
            historical_workloads: None,
            rng_seed: Some(42),
            generation_limit: Some(500),
            scenario: Some(Scenario { leave_requests: None, minimum_rest_hours: Some(10), 
                planning_horizon_hours: Some(168.0),
                max_hours_per_worker: Some(48.0),
            }),
        };
        let report = validate_request(&req);
        assert!(
            report.is_valid(),
            "SunAir canonical scenario must pass strict validation.\nReport:\n{}",
            report.display()
        );
    }
}
