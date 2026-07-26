// Generic Export Adapter — Phase A Step 3
//
// Serialises a `ScheduleSolution` (or the raw assignments map from a
// `ScheduleResponse`) to vendor-neutral CSV or JSON so that any downstream
// system can consume UltraCrew output without knowing Coralys internals.
//
// Design mirrors `generic_import.rs`:
//   • `ExportFormat`   — supported output formats
//   • `ExportConfig`   — caller-supplied options
//   • `ExportResult`   — the serialised payload + metadata
//   • `GenericExporter`— stateless entry-point with one method per format
//
// No `unwrap()` or `expect()` — all errors propagate via `ExportError`.

use std::collections::HashMap;
use std::fmt;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::schedule_solution::ScheduleSolution;

// ─── Error type ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum ExportError {
    /// I/O failure while writing to a file.
    Io(io::Error),
    /// JSON serialisation failure.
    Json(serde_json::Error),
    /// The requested format is not supported.
    UnsupportedFormat(String),
    /// The solution data is structurally invalid for export.
    InvalidSolution(String),
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportError::Io(e) => write!(f, "I/O error: {}", e),
            ExportError::Json(e) => write!(f, "JSON serialisation error: {}", e),
            ExportError::UnsupportedFormat(s) => write!(f, "Unsupported export format: {}", s),
            ExportError::InvalidSolution(s) => write!(f, "Invalid solution: {}", s),
        }
    }
}

impl std::error::Error for ExportError {}

impl From<io::Error> for ExportError {
    fn from(e: io::Error) -> Self {
        ExportError::Io(e)
    }
}

impl From<serde_json::Error> for ExportError {
    fn from(e: serde_json::Error) -> Self {
        ExportError::Json(e)
    }
}

// ─── Format enum ─────────────────────────────────────────────────────────────

/// Output formats supported by the Generic Export layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    /// Newline-delimited JSON — the full `ScheduleSolution` struct.
    Json,
    /// Two-section CSV:
    ///   Section 1 — assignments (shift_id, worker_id)
    ///   Section 2 — summary metrics (fitness, violations, penalties)
    Csv,
}

impl ExportFormat {
    /// Parse a format string (case-insensitive) into an `ExportFormat`.
    pub fn from_str(s: &str) -> Result<Self, ExportError> {
        match s.to_lowercase().as_str() {
            "json" => Ok(ExportFormat::Json),
            "csv" => Ok(ExportFormat::Csv),
            other => Err(ExportError::UnsupportedFormat(other.to_string())),
        }
    }

    /// Human-readable label used in API responses.
    pub fn label(&self) -> &'static str {
        match self {
            ExportFormat::Json => "json",
            ExportFormat::Csv => "csv",
        }
    }

    /// MIME type for HTTP Content-Type headers.
    pub fn mime_type(&self) -> &'static str {
        match self {
            ExportFormat::Json => "application/json",
            ExportFormat::Csv => "text/csv",
        }
    }
}

// ─── Config ──────────────────────────────────────────────────────────────────

/// Caller-supplied options for an export operation.
#[derive(Debug, Clone)]
pub struct ExportConfig {
    /// Target format.
    pub format: ExportFormat,
    /// If `true`, pretty-print JSON output (ignored for CSV).
    pub pretty_json: bool,
    /// Optional column separator for CSV (default: comma).
    pub csv_separator: char,
    /// Whether to include the telemetry section in JSON output.
    /// When `false`, the `telemetry` field is stripped before serialisation.
    pub include_telemetry: bool,
    /// Whether to include the recommendations section in JSON output.
    pub include_recommendations: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        ExportConfig {
            format: ExportFormat::Json,
            pretty_json: false,
            csv_separator: ',',
            include_telemetry: true,
            include_recommendations: true,
        }
    }
}

// ─── Result ──────────────────────────────────────────────────────────────────

/// The serialised payload returned by an export operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    /// The serialised content (UTF-8 string).
    pub content: String,
    /// Format that was used.
    pub format: ExportFormat,
    /// MIME type suitable for an HTTP Content-Type header.
    pub mime_type: String,
    /// Number of assignments included in the export.
    pub assignment_count: usize,
}

// ─── Exporter ────────────────────────────────────────────────────────────────

/// Stateless entry-point for all export operations.
pub struct GenericExporter;

impl GenericExporter {
    // ── Public API ────────────────────────────────────────────────────────

    /// Export a `ScheduleSolution` using the given config.
    ///
    /// Returns an `ExportResult` containing the serialised payload.
    pub fn export(
        solution: &ScheduleSolution,
        config: &ExportConfig,
    ) -> Result<ExportResult, ExportError> {
        match config.format {
            ExportFormat::Json => Self::export_json(solution, config),
            ExportFormat::Csv => Self::export_csv(solution, config),
        }
    }

    /// Export a raw assignments map (shift_id → worker_id) plus summary
    /// metrics to the requested format.  Used by the REST handler which
    /// already has the assignments and metrics separately.
    pub fn export_from_parts(
        assignments: &HashMap<u64, u64>,
        metrics: &HashMap<String, f64>,
        config: &ExportConfig,
    ) -> Result<ExportResult, ExportError> {
        // Build a minimal ScheduleSolution from the parts so we can reuse
        // the same serialisation paths.
        let solution = ScheduleSolution {
            assignments: assignments.clone(),
            fitness: metrics.get("fitness").copied().unwrap_or(0.0),
            hard_violations: metrics
                .get("hard_violations")
                .copied()
                .unwrap_or(0.0) as usize,
            fairness_penalty: metrics.get("fairness_penalty").copied().unwrap_or(0.0),
            fatigue_penalty: metrics.get("fatigue_penalty").copied().unwrap_or(0.0),
            rest_violations: metrics
                .get("rest_violations")
                .copied()
                .unwrap_or(0.0) as usize,
            recommendations: None,
            telemetry: None,
        };
        Self::export(&solution, config)
    }

    /// Write the exported content directly to a file.
    ///
    /// Creates parent directories if they do not exist.
    pub fn export_to_file<P: AsRef<Path>>(
        solution: &ScheduleSolution,
        config: &ExportConfig,
        path: P,
    ) -> Result<(), ExportError> {
        let result = Self::export(solution, config)?;
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, result.content.as_bytes())?;
        Ok(())
    }

    /// Return a list of all supported export formats with their labels and
    /// MIME types.  Used by the `GET /export/formats` endpoint.
    pub fn supported_formats() -> Vec<FormatDescriptor> {
        vec![
            FormatDescriptor {
                id: "json".to_string(),
                label: "JSON".to_string(),
                mime_type: "application/json".to_string(),
                description: "Full ScheduleSolution as a JSON object. \
                              Includes assignments, fitness metrics, \
                              constraint violations, and optional telemetry."
                    .to_string(),
            },
            FormatDescriptor {
                id: "csv".to_string(),
                label: "CSV".to_string(),
                mime_type: "text/csv".to_string(),
                description: "Two-section CSV. Section 1: assignments \
                              (shift_id,worker_id). Section 2: summary \
                              metrics (fitness, violations, penalties)."
                    .to_string(),
            },
        ]
    }

    // ── Private helpers ───────────────────────────────────────────────────

    fn export_json(
        solution: &ScheduleSolution,
        config: &ExportConfig,
    ) -> Result<ExportResult, ExportError> {
        // Optionally strip heavy fields before serialisation.
        let content = if !config.include_telemetry || !config.include_recommendations {
            // Build a filtered view using serde_json::Value so we don't need
            // a separate struct.
            let mut value = serde_json::to_value(solution)?;
            if let Some(obj) = value.as_object_mut() {
                if !config.include_telemetry {
                    obj.remove("telemetry");
                }
                if !config.include_recommendations {
                    obj.remove("recommendations");
                }
            }
            if config.pretty_json {
                serde_json::to_string_pretty(&value)?
            } else {
                serde_json::to_string(&value)?
            }
        } else if config.pretty_json {
            serde_json::to_string_pretty(solution)?
        } else {
            serde_json::to_string(solution)?
        };

        Ok(ExportResult {
            assignment_count: solution.assignments.len(),
            format: ExportFormat::Json,
            mime_type: ExportFormat::Json.mime_type().to_string(),
            content,
        })
    }

    fn export_csv(
        solution: &ScheduleSolution,
        config: &ExportConfig,
    ) -> Result<ExportResult, ExportError> {
        let sep = config.csv_separator;
        let mut lines: Vec<String> = Vec::new();

        // ── Section 1: assignments ────────────────────────────────────────
        lines.push(format!("# UltraCrew Export — Assignments"));
        lines.push(format!("shift_id{}worker_id", sep));

        // Sort by shift_id for deterministic output.
        let mut sorted: Vec<(u64, u64)> = solution.assignments.iter().map(|(&k, &v)| (k, v)).collect();
        sorted.sort_by_key(|(shift_id, _)| *shift_id);

        for (shift_id, worker_id) in &sorted {
            lines.push(format!("{}{}{}", shift_id, sep, worker_id));
        }

        // ── Section 2: summary metrics ────────────────────────────────────
        lines.push(String::new()); // blank separator line
        lines.push(format!("# UltraCrew Export — Summary Metrics"));
        lines.push(format!("metric{}value", sep));
        lines.push(format!("fitness{}{:.6}", sep, solution.fitness));
        lines.push(format!("hard_violations{}{}", sep, solution.hard_violations));
        lines.push(format!("rest_violations{}{}", sep, solution.rest_violations));
        lines.push(format!("fairness_penalty{}{:.6}", sep, solution.fairness_penalty));
        lines.push(format!("fatigue_penalty{}{:.6}", sep, solution.fatigue_penalty));
        lines.push(format!("assignment_count{}{}", sep, solution.assignments.len()));

        let content = lines.join("\n") + "\n";

        Ok(ExportResult {
            assignment_count: solution.assignments.len(),
            format: ExportFormat::Csv,
            mime_type: ExportFormat::Csv.mime_type().to_string(),
            content,
        })
    }
}

// ─── Supporting types ─────────────────────────────────────────────────────────

/// Metadata about a single supported export format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatDescriptor {
    pub id: String,
    pub label: String,
    pub mime_type: String,
    pub description: String,
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_solution(n: usize) -> ScheduleSolution {
        let mut assignments = HashMap::new();
        for i in 0..n {
            assignments.insert(i as u64, (i % 3) as u64);
        }
        ScheduleSolution {
            assignments,
            fitness: -42.5,
            hard_violations: 1,
            fairness_penalty: 3.14,
            fatigue_penalty: 1.0,
            rest_violations: 2,
            recommendations: None,
            telemetry: None,
        }
    }

    #[test]
    fn test_export_format_from_str_valid() {
        assert_eq!(ExportFormat::from_str("json").unwrap(), ExportFormat::Json);
        assert_eq!(ExportFormat::from_str("JSON").unwrap(), ExportFormat::Json);
        assert_eq!(ExportFormat::from_str("csv").unwrap(), ExportFormat::Csv);
        assert_eq!(ExportFormat::from_str("CSV").unwrap(), ExportFormat::Csv);
    }

    #[test]
    fn test_export_format_from_str_invalid() {
        assert!(matches!(
            ExportFormat::from_str("xml"),
            Err(ExportError::UnsupportedFormat(_))
        ));
    }

    #[test]
    fn test_json_export_contains_assignments() {
        let sol = make_solution(5);
        let config = ExportConfig {
            format: ExportFormat::Json,
            pretty_json: false,
            ..Default::default()
        };
        let result = GenericExporter::export(&sol, &config).unwrap();
        assert_eq!(result.format, ExportFormat::Json);
        assert_eq!(result.assignment_count, 5);
        assert!(result.content.contains("assignments"));
        assert!(result.content.contains("fitness"));
    }

    #[test]
    fn test_json_export_pretty() {
        let sol = make_solution(3);
        let config = ExportConfig {
            format: ExportFormat::Json,
            pretty_json: true,
            ..Default::default()
        };
        let result = GenericExporter::export(&sol, &config).unwrap();
        // Pretty JSON contains newlines.
        assert!(result.content.contains('\n'));
    }

    #[test]
    fn test_json_export_strip_telemetry() {
        let sol = make_solution(2);
        let config = ExportConfig {
            format: ExportFormat::Json,
            include_telemetry: false,
            include_recommendations: false,
            ..Default::default()
        };
        let result = GenericExporter::export(&sol, &config).unwrap();
        assert!(!result.content.contains("telemetry"));
        assert!(!result.content.contains("recommendations"));
    }

    #[test]
    fn test_csv_export_structure() {
        let sol = make_solution(4);
        let config = ExportConfig {
            format: ExportFormat::Csv,
            ..Default::default()
        };
        let result = GenericExporter::export(&sol, &config).unwrap();
        assert_eq!(result.format, ExportFormat::Csv);
        assert_eq!(result.assignment_count, 4);
        // Must contain both section headers.
        assert!(result.content.contains("shift_id,worker_id"));
        assert!(result.content.contains("metric,value"));
        assert!(result.content.contains("fitness"));
        assert!(result.content.contains("hard_violations"));
    }

    #[test]
    fn test_csv_export_sorted_assignments() {
        let sol = make_solution(5);
        let config = ExportConfig {
            format: ExportFormat::Csv,
            ..Default::default()
        };
        let result = GenericExporter::export(&sol, &config).unwrap();
        // Extract shift_id column from assignment rows.
        let ids: Vec<u64> = result
            .content
            .lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty() && !l.starts_with("shift_id") && !l.starts_with("metric"))
            .filter_map(|l| l.split(',').next().and_then(|s| s.parse().ok()))
            .collect();
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted, "CSV assignments must be sorted by shift_id");
    }

    #[test]
    fn test_csv_custom_separator() {
        let sol = make_solution(2);
        let config = ExportConfig {
            format: ExportFormat::Csv,
            csv_separator: ';',
            ..Default::default()
        };
        let result = GenericExporter::export(&sol, &config).unwrap();
        assert!(result.content.contains("shift_id;worker_id"));
    }

    #[test]
    fn test_export_from_parts() {
        let mut assignments = HashMap::new();
        assignments.insert(10u64, 1u64);
        assignments.insert(11u64, 2u64);
        let mut metrics = HashMap::new();
        metrics.insert("fitness".to_string(), -10.0f64);
        metrics.insert("hard_violations".to_string(), 0.0f64);

        let config = ExportConfig {
            format: ExportFormat::Json,
            ..Default::default()
        };
        let result = GenericExporter::export_from_parts(&assignments, &metrics, &config).unwrap();
        assert_eq!(result.assignment_count, 2);
        assert!(result.content.contains("assignments"));
    }

    #[test]
    fn test_supported_formats_list() {
        let formats = GenericExporter::supported_formats();
        assert_eq!(formats.len(), 2);
        let ids: Vec<&str> = formats.iter().map(|f| f.id.as_str()).collect();
        assert!(ids.contains(&"json"));
        assert!(ids.contains(&"csv"));
    }

    #[test]
    fn test_export_to_file_json() {
        let sol = make_solution(3);
        let config = ExportConfig {
            format: ExportFormat::Json,
            ..Default::default()
        };
        let dir = std::env::temp_dir().join("ultracrew_export_test");
        let path = dir.join("solution.json");
        GenericExporter::export_to_file(&sol, &config, &path).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("assignments"));
        // Clean up.
        let _ = std::fs::remove_dir_all(&dir);
    }
}