/// Generic Import Adapter — Phase A
///
/// Converts customer workforce data (CSV or JSON) into a `ScheduleRequest`
/// that the UltraCrew optimization pipeline can consume directly.
///
/// # Supported formats
///
/// ## JSON (direct)
/// A single JSON file containing a serialized `ScheduleRequest`.
/// Use this when the customer system can produce structured JSON.
///
/// ## CSV (two-file)
/// Two CSV files: one for workers, one for shifts.
/// Use this when the customer system exports spreadsheet data.
///
/// ### Workers CSV columns (required)
/// `id,skills`
/// - `id`: unsigned integer, unique per worker
/// - `skills`: comma-separated skill names within the field, e.g. `"Nurse,ICU"`
///
/// ### Shifts CSV columns (required)
/// `id,start_hour,duration_hours,required_skill`
/// - `id`: unsigned integer, unique per shift
/// - `start_hour`: hour of the planning week (0–167 for a 7-day week)
/// - `duration_hours`: shift length in hours
/// - `required_skill`: single skill name
///
/// ### Optional columns (workers CSV)
/// `historical_hours` — semicolon-separated list of past weekly hours, e.g. `"40;38;42"`
/// Used to seed the ecology fatigue model.
///
/// # Error handling
/// All functions return `ImportError` with a descriptive message.
/// No `unwrap()` or `expect()` calls — all errors are propagated.

use std::collections::HashMap;
use std::path::Path;
use std::fs;

use crate::models::{Skill, Worker, Shift};
use crate::public_contracts::{ScheduleRequest, Scenario};

// ─── Error type ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ImportError {
    pub message: String,
}

impl ImportError {
    fn new(msg: impl Into<String>) -> Self {
        Self { message: msg.into() }
    }
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ImportError: {}", self.message)
    }
}

impl std::error::Error for ImportError {}

// ─── JSON import ─────────────────────────────────────────────────────────────

/// Load a `ScheduleRequest` directly from a JSON file.
///
/// The JSON must match the `ScheduleRequest` schema exactly.
/// Use `export_request_template()` to generate a template.
pub fn load_from_json<P: AsRef<Path>>(path: P) -> Result<ScheduleRequest, ImportError> {
    let path = path.as_ref();
    let data = fs::read_to_string(path).map_err(|e| {
        ImportError::new(format!("Cannot read file '{}': {}", path.display(), e))
    })?;
    serde_json::from_str::<ScheduleRequest>(&data).map_err(|e| {
        ImportError::new(format!(
            "JSON parse error in '{}': {}. Use export_request_template() to see the expected schema.",
            path.display(), e
        ))
    })
}

// ─── CSV import ──────────────────────────────────────────────────────────────

/// Load a `ScheduleRequest` from two CSV files: workers and shifts.
///
/// `workers_path`: path to the workers CSV (columns: id, skills, [historical_hours])
/// `shifts_path`: path to the shifts CSV (columns: id, start_hour, duration_hours, required_skill)
/// `scenario`: optional planning scenario (horizon, max hours per worker)
/// `rng_seed`: optional seed for deterministic optimization
/// `generation_limit`: optional GA generation limit
pub fn load_from_csv<P: AsRef<Path>>(
    workers_path: P,
    shifts_path: P,
    scenario: Option<Scenario>,
    rng_seed: Option<u64>,
    generation_limit: Option<usize>,
) -> Result<ScheduleRequest, ImportError> {
    let workers_path = workers_path.as_ref();
    let shifts_path = shifts_path.as_ref();

    let workers_data = fs::read_to_string(workers_path).map_err(|e| {
        ImportError::new(format!("Cannot read workers file '{}': {}", workers_path.display(), e))
    })?;
    let shifts_data = fs::read_to_string(shifts_path).map_err(|e| {
        ImportError::new(format!("Cannot read shifts file '{}': {}", shifts_path.display(), e))
    })?;

    let (workers, historical_workloads) = parse_workers_csv(&workers_data, workers_path)?;
    let shifts = parse_shifts_csv(&shifts_data, shifts_path)?;

    validate_workers(&workers)?;
    validate_shifts(&shifts)?;
    validate_skill_coverage(&workers, &shifts)?;

    Ok(ScheduleRequest {
        workers,
        shifts,
        historical_workloads: if historical_workloads.is_empty() { None } else { Some(historical_workloads) },
        rng_seed,
        generation_limit,
        scenario,
    })
}

// ─── CSV parsers ─────────────────────────────────────────────────────────────

fn parse_workers_csv(
    data: &str,
    source: &Path,
) -> Result<(Vec<Worker>, HashMap<u64, Vec<f64>>), ImportError> {
    let mut workers = Vec::new();
    let mut historical_workloads: HashMap<u64, Vec<f64>> = HashMap::new();
    let mut lines = data.lines().enumerate();

    // Header line
    let (_, header) = lines.next().ok_or_else(|| {
        ImportError::new(format!("Workers file '{}' is empty", source.display()))
    })?;

    let headers: Vec<&str> = header.split(',').map(str::trim).collect();
    let id_col = find_column(&headers, "id", source)?;
    let skills_col = find_column(&headers, "skills", source)?;
    let hist_col = headers.iter().position(|h| *h == "historical_hours");

    for (line_num, line) in lines {
        let line = line.trim();
        if line.is_empty() { continue; }

        // Handle quoted fields (skills may contain commas inside quotes)
        let fields = split_csv_line(line);

        let id_str = fields.get(id_col).ok_or_else(|| {
            ImportError::new(format!(
                "Workers file '{}' line {}: missing 'id' column",
                source.display(), line_num + 1
            ))
        })?;
        let id: u64 = id_str.trim().parse().map_err(|_| {
            ImportError::new(format!(
                "Workers file '{}' line {}: 'id' must be an unsigned integer, got '{}'",
                source.display(), line_num + 1, id_str
            ))
        })?;

        let skills_str = fields.get(skills_col).ok_or_else(|| {
            ImportError::new(format!(
                "Workers file '{}' line {}: missing 'skills' column",
                source.display(), line_num + 1
            ))
        })?;
        let skills: Vec<Skill> = skills_str
            .trim_matches('"')
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(Skill::new)
            .collect();

        if skills.is_empty() {
            return Err(ImportError::new(format!(
                "Workers file '{}' line {}: worker {} has no skills. At least one skill is required.",
                source.display(), line_num + 1, id
            )));
        }

        // Optional historical hours
        if let Some(hcol) = hist_col {
            if let Some(hist_str) = fields.get(hcol) {
                let hist_str = hist_str.trim().trim_matches('"');
                if !hist_str.is_empty() {
                    let hours: Result<Vec<f64>, _> = hist_str
                        .split(';')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(|s| s.parse::<f64>())
                        .collect();
                    match hours {
                        Ok(h) => { historical_workloads.insert(id, h); }
                        Err(_) => {
                            return Err(ImportError::new(format!(
                                "Workers file '{}' line {}: 'historical_hours' must be semicolon-separated numbers, got '{}'",
                                source.display(), line_num + 1, hist_str
                            )));
                        }
                    }
                }
            }
        }

        workers.push(Worker { id, skills });
    }

    Ok((workers, historical_workloads))
}

fn parse_shifts_csv(data: &str, source: &Path) -> Result<Vec<Shift>, ImportError> {
    let mut shifts = Vec::new();
    let mut lines = data.lines().enumerate();

    let (_, header) = lines.next().ok_or_else(|| {
        ImportError::new(format!("Shifts file '{}' is empty", source.display()))
    })?;

    let headers: Vec<&str> = header.split(',').map(str::trim).collect();
    let id_col = find_column(&headers, "id", source)?;
    let start_col = find_column(&headers, "start_hour", source)?;
    let dur_col = find_column(&headers, "duration_hours", source)?;
    let skill_col = find_column(&headers, "required_skill", source)?;

    for (line_num, line) in lines {
        let line = line.trim();
        if line.is_empty() { continue; }

        let fields = split_csv_line(line);

        let id: u64 = parse_u64_field(&fields, id_col, "id", line_num + 1, source)?;
        let start_hour: u64 = parse_u64_field(&fields, start_col, "start_hour", line_num + 1, source)?;
        let duration_hours: u64 = parse_u64_field(&fields, dur_col, "duration_hours", line_num + 1, source)?;

        let skill_str = fields.get(skill_col).ok_or_else(|| {
            ImportError::new(format!(
                "Shifts file '{}' line {}: missing 'required_skill' column",
                source.display(), line_num + 1
            ))
        })?;
        let required_skill = Skill::new(skill_str.trim().trim_matches('"'));

        if duration_hours == 0 {
            return Err(ImportError::new(format!(
                "Shifts file '{}' line {}: shift {} has duration_hours=0. Duration must be at least 1.",
                source.display(), line_num + 1, id
            )));
        }

        shifts.push(Shift { id, start_hour, duration_hours, required_skill, flight_id: None, crew_role: None });
    }

    Ok(shifts)
}

// ─── Validation ──────────────────────────────────────────────────────────────

fn validate_workers(workers: &[Worker]) -> Result<(), ImportError> {
    if workers.is_empty() {
        return Err(ImportError::new("Workers list is empty. At least one worker is required."));
    }
    let mut seen_ids = std::collections::HashSet::new();
    for w in workers {
        if !seen_ids.insert(w.id) {
            return Err(ImportError::new(format!(
                "Duplicate worker id: {}. Worker ids must be unique.", w.id
            )));
        }
    }
    Ok(())
}

fn validate_shifts(shifts: &[Shift]) -> Result<(), ImportError> {
    if shifts.is_empty() {
        return Err(ImportError::new("Shifts list is empty. At least one shift is required."));
    }
    let mut seen_ids = std::collections::HashSet::new();
    for s in shifts {
        if !seen_ids.insert(s.id) {
            return Err(ImportError::new(format!(
                "Duplicate shift id: {}. Shift ids must be unique.", s.id
            )));
        }
    }
    Ok(())
}

/// Warn if any shift requires a skill that no worker possesses.
/// This is a soft validation — it returns an error to surface the issue early
/// rather than letting the optimizer silently produce HC1 violations.
fn validate_skill_coverage(workers: &[Worker], shifts: &[Shift]) -> Result<(), ImportError> {
    let all_skills: std::collections::HashSet<&Skill> = workers
        .iter()
        .flat_map(|w| w.skills.iter())
        .collect();

    let uncovered: Vec<String> = shifts
        .iter()
        .filter(|s| !all_skills.contains(&s.required_skill))
        .map(|s| format!("shift {} requires skill '{}'", s.id, s.required_skill.0))
        .collect();

    if !uncovered.is_empty() {
        return Err(ImportError::new(format!(
            "No worker possesses the required skill for the following shifts: {}. \
             Add workers with these skills or correct the skill names.",
            uncovered.join("; ")
        )));
    }
    Ok(())
}

// ─── Template export ─────────────────────────────────────────────────────────

/// Return a JSON string showing the expected `ScheduleRequest` schema with example values.
/// Use this to help customers understand the JSON import format.
pub fn export_request_template() -> String {
    let template = ScheduleRequest {
        workers: vec![
            crate::models::Worker {
                id: 1,
                skills: vec![Skill::new("Nurse"), Skill::new("ICU")],
            },
            crate::models::Worker {
                id: 2,
                skills: vec![Skill::new("Nurse")],
            },
        ],
        shifts: vec![
            crate::models::Shift {
                id: 1,
                start_hour: 0,
                duration_hours: 8,
                required_skill: Skill::new("Nurse"),
                flight_id: None,
                crew_role: None,
            },
            crate::models::Shift {
                id: 2,
                start_hour: 8,
                duration_hours: 8,
                required_skill: Skill::new("ICU"),
                flight_id: None,
                crew_role: None,
            },
        ],
        historical_workloads: Some({
            let mut m = HashMap::new();
            m.insert(1u64, vec![40.0, 38.0]);
            m.insert(2u64, vec![36.0]);
                m
        }),
        rng_seed: Some(42),
        generation_limit: Some(200),
        scenario: Some(Scenario {
            planning_horizon_hours: Some(168.0),
            max_hours_per_worker: Some(40.0),
            minimum_rest_hours: Some(11),
        }),
    };
    serde_json::to_string_pretty(&template).unwrap_or_else(|_| "{}".to_string())
}

/// Return CSV template strings for the two-file CSV import format.
/// Returns (workers_csv_template, shifts_csv_template).
pub fn export_csv_templates() -> (String, String) {
    let workers = "id,skills,historical_hours\n\
                   1,\"Nurse;ICU\",\"40;38\"\n\
                   2,\"Nurse\",\"36\"\n\
                   3,\"ICU\",\n";

    let shifts = "id,start_hour,duration_hours,required_skill\n\
                  1,0,8,Nurse\n\
                  2,8,8,ICU\n\
                  3,16,8,Nurse\n\
                  4,24,8,ICU\n";

    (workers.to_string(), shifts.to_string())
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn find_column(headers: &[&str], name: &str, source: &Path) -> Result<usize, ImportError> {
    headers.iter().position(|h| *h == name).ok_or_else(|| {
        ImportError::new(format!(
            "File '{}': required column '{}' not found in header. Found: {}",
            source.display(), name, headers.join(", ")
        ))
    })
}

fn parse_u64_field(
    fields: &[String],
    col: usize,
    name: &str,
    line_num: usize,
    source: &Path,
) -> Result<u64, ImportError> {
    let val = fields.get(col).ok_or_else(|| {
        ImportError::new(format!(
            "File '{}' line {}: missing '{}' column",
            source.display(), line_num, name
        ))
    })?;
    val.trim().parse::<u64>().map_err(|_| {
        ImportError::new(format!(
            "File '{}' line {}: '{}' must be an unsigned integer, got '{}'",
            source.display(), line_num, name, val
        ))
    })
}

/// Minimal CSV line splitter that handles double-quoted fields.
/// Fields containing commas must be quoted: `"Nurse,ICU"` or `"Nurse;ICU"`.
fn split_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in line.chars() {
        match ch {
            '"' => { in_quotes = !in_quotes; }
            ',' if !in_quotes => {
                fields.push(current.trim().to_string());
                current = String::new();
            }
            _ => { current.push(ch); }
        }
    }
    fields.push(current.trim().to_string());
    fields
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn dummy_path(name: &str) -> PathBuf {
        PathBuf::from(name)
    }

    #[test]
    fn test_parse_workers_csv_basic() {
        let csv = "id,skills\n1,\"Nurse;ICU\"\n2,Nurse\n";
        let (workers, hist) = parse_workers_csv(csv, &dummy_path("workers.csv")).unwrap();
        assert_eq!(workers.len(), 2);
        assert_eq!(workers[0].id, 1);
        assert_eq!(workers[0].skills.len(), 2);
        assert_eq!(workers[1].skills[0], Skill::new("Nurse"));
        assert!(hist.is_empty());
    }

    #[test]
    fn test_parse_workers_csv_with_history() {
        let csv = "id,skills,historical_hours\n1,Nurse,\"40;38;42\"\n";
        let (workers, hist) = parse_workers_csv(csv, &dummy_path("workers.csv")).unwrap();
        assert_eq!(workers.len(), 1);
        let h = hist.get(&1).unwrap();
        assert_eq!(h, &vec![40.0, 38.0, 42.0]);
    }

    #[test]
    fn test_parse_shifts_csv_basic() {
        let csv = "id,start_hour,duration_hours,required_skill\n1,0,8,Nurse\n2,8,8,ICU\n";
        let shifts = parse_shifts_csv(csv, &dummy_path("shifts.csv")).unwrap();
        assert_eq!(shifts.len(), 2);
        assert_eq!(shifts[0].start_hour, 0);
        assert_eq!(shifts[1].required_skill, Skill::new("ICU"));
    }

    #[test]
    fn test_duplicate_worker_id_rejected() {
        let csv = "id,skills\n1,Nurse\n1,ICU\n";
        let (workers, _) = parse_workers_csv(csv, &dummy_path("workers.csv")).unwrap();
        let result = validate_workers(&workers);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Duplicate worker id"));
    }

    #[test]
    fn test_uncovered_skill_rejected() {
        let workers = vec![Worker { id: 1, skills: vec![Skill::new("Nurse")] }];
        let shifts = vec![Shift { id: 1, start_hour: 0, duration_hours: 8, required_skill: Skill::new("ICU") }];
        let result = validate_skill_coverage(&workers, &shifts);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("No worker possesses"));
    }

    #[test]
    fn test_zero_duration_shift_rejected() {
        let csv = "id,start_hour,duration_hours,required_skill\n1,0,0,Nurse\n";
        let result = parse_shifts_csv(csv, &dummy_path("shifts.csv"));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("duration_hours=0"));
    }

    #[test]
    fn test_export_request_template_is_valid_json() {
        let template = export_request_template();
        let parsed: Result<ScheduleRequest, _> = serde_json::from_str(&template);
        assert!(parsed.is_ok(), "Template JSON should be valid: {:?}", parsed.err());
    }

    #[test]
    fn test_export_csv_templates_parseable() {
        let (workers_csv, shifts_csv) = export_csv_templates();
        let (workers, _) = parse_workers_csv(&workers_csv, &dummy_path("workers.csv")).unwrap();
        let shifts = parse_shifts_csv(&shifts_csv, &dummy_path("shifts.csv")).unwrap();
        assert!(!workers.is_empty());
        assert!(!shifts.is_empty());
    }
}