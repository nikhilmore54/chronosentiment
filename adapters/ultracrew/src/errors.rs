/// S2-04 — Unified error taxonomy for the UltraCrew pipeline
///
/// Provides a single `UltraCrewError` enum that covers every failure mode
/// across the three pipeline stages:
///
/// | Stage  | Variants |
/// |--------|----------|
/// | Import | `Io`, `ParseJson`, `ParseCsv`, `ValidationFailed` |
/// | Engine | `InvalidConfig`, `OptimizationFailed` |
/// | Export | `Io`, `SerializationFailed`, `UnsupportedFormat` |
///
/// All variants carry a human-readable message and, where applicable, the
/// underlying source error. The enum implements `std::error::Error` and
/// `Display` so it can be used with `?` throughout the codebase.
///
/// # Usage
///
/// ```rust
/// use ultracrew::errors::{UltraCrewError, Result};
///
/// fn load(path: &str) -> Result<String> {
///     std::fs::read_to_string(path)
///         .map_err(|e| UltraCrewError::io(path, e))
/// }
/// ```
use std::fmt;

// ─── Result alias ─────────────────────────────────────────────────────────────

/// Convenience alias: `Result<T>` in the UltraCrew crate.
pub type Result<T> = std::result::Result<T, UltraCrewError>;

// ─── Error enum ───────────────────────────────────────────────────────────────

/// All error conditions that can arise in the UltraCrew pipeline.
#[derive(Debug)]
pub enum UltraCrewError {
    // ── Import stage ──────────────────────────────────────────────────────────
    /// A file or stream could not be read or written.
    Io { path: String, message: String },

    /// JSON input could not be parsed.
    ParseJson { path: String, message: String },

    /// CSV input could not be parsed.
    ParseCsv { path: String, message: String },

    /// Input passed JSON/CSV parsing but failed strict semantic validation.
    /// The `issues` field contains the full `ValidationReport` display string.
    ValidationFailed { path: String, issues: String },

    // ── Engine stage ──────────────────────────────────────────────────────────
    /// The optimizer configuration is invalid (e.g. population_size = 0).
    InvalidConfig { message: String },

    /// The optimizer encountered an unrecoverable error during evolution.
    OptimizationFailed { message: String },

    // ── Export stage ──────────────────────────────────────────────────────────
    /// The output could not be serialised to the requested format.
    SerializationFailed { format: String, message: String },

    /// The requested export format is not supported.
    UnsupportedFormat { format: String },

    // ── Config stage ──────────────────────────────────────────────────────────
    /// A configuration file could not be loaded or parsed.
    ConfigError { path: String, message: String },
}

// ─── Constructors ─────────────────────────────────────────────────────────────

impl UltraCrewError {
    /// Construct an `Io` error from a path and a `std::io::Error`.
    pub fn io(path: impl Into<String>, err: impl fmt::Display) -> Self {
        UltraCrewError::Io {
            path: path.into(),
            message: err.to_string(),
        }
    }

    /// Construct a `ParseJson` error.
    pub fn parse_json(path: impl Into<String>, err: impl fmt::Display) -> Self {
        UltraCrewError::ParseJson {
            path: path.into(),
            message: err.to_string(),
        }
    }

    /// Construct a `ParseCsv` error.
    pub fn parse_csv(path: impl Into<String>, err: impl fmt::Display) -> Self {
        UltraCrewError::ParseCsv {
            path: path.into(),
            message: err.to_string(),
        }
    }

    /// Construct a `ValidationFailed` error from a path and a display string.
    pub fn validation_failed(path: impl Into<String>, issues: impl Into<String>) -> Self {
        UltraCrewError::ValidationFailed {
            path: path.into(),
            issues: issues.into(),
        }
    }

    /// Construct an `InvalidConfig` error.
    pub fn invalid_config(message: impl Into<String>) -> Self {
        UltraCrewError::InvalidConfig {
            message: message.into(),
        }
    }

    /// Construct an `OptimizationFailed` error.
    pub fn optimization_failed(message: impl Into<String>) -> Self {
        UltraCrewError::OptimizationFailed {
            message: message.into(),
        }
    }

    /// Construct a `SerializationFailed` error.
    pub fn serialization_failed(format: impl Into<String>, err: impl fmt::Display) -> Self {
        UltraCrewError::SerializationFailed {
            format: format.into(),
            message: err.to_string(),
        }
    }

    /// Construct an `UnsupportedFormat` error.
    pub fn unsupported_format(format: impl Into<String>) -> Self {
        UltraCrewError::UnsupportedFormat {
            format: format.into(),
        }
    }

    /// Construct a `ConfigError`.
    pub fn config_error(path: impl Into<String>, message: impl Into<String>) -> Self {
        UltraCrewError::ConfigError {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Return the pipeline stage this error belongs to.
    pub fn stage(&self) -> &'static str {
        match self {
            UltraCrewError::Io { .. }
            | UltraCrewError::ParseJson { .. }
            | UltraCrewError::ParseCsv { .. }
            | UltraCrewError::ValidationFailed { .. } => "import",

            UltraCrewError::InvalidConfig { .. } | UltraCrewError::OptimizationFailed { .. } => {
                "engine"
            }

            UltraCrewError::SerializationFailed { .. }
            | UltraCrewError::UnsupportedFormat { .. } => "export",

            UltraCrewError::ConfigError { .. } => "config",
        }
    }

    /// Return a stable error code string for logging and API responses.
    pub fn code(&self) -> &'static str {
        match self {
            UltraCrewError::Io { .. } => "UC-IO-001",
            UltraCrewError::ParseJson { .. } => "UC-IMP-001",
            UltraCrewError::ParseCsv { .. } => "UC-IMP-002",
            UltraCrewError::ValidationFailed { .. } => "UC-IMP-003",
            UltraCrewError::InvalidConfig { .. } => "UC-ENG-001",
            UltraCrewError::OptimizationFailed { .. } => "UC-ENG-002",
            UltraCrewError::SerializationFailed { .. } => "UC-EXP-001",
            UltraCrewError::UnsupportedFormat { .. } => "UC-EXP-002",
            UltraCrewError::ConfigError { .. } => "UC-CFG-001",
        }
    }
}

// ─── Display ──────────────────────────────────────────────────────────────────

impl fmt::Display for UltraCrewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UltraCrewError::Io { path, message } => {
                write!(f, "[{}] I/O error for '{}': {}", self.code(), path, message)
            }

            UltraCrewError::ParseJson { path, message } => write!(
                f,
                "[{}] JSON parse error in '{}': {}",
                self.code(),
                path,
                message
            ),

            UltraCrewError::ParseCsv { path, message } => write!(
                f,
                "[{}] CSV parse error in '{}': {}",
                self.code(),
                path,
                message
            ),

            UltraCrewError::ValidationFailed { path, issues } => write!(
                f,
                "[{}] Validation failed for '{}':\n{}",
                self.code(),
                path,
                issues
            ),

            UltraCrewError::InvalidConfig { message } => write!(
                f,
                "[{}] Invalid optimizer configuration: {}",
                self.code(),
                message
            ),

            UltraCrewError::OptimizationFailed { message } => {
                write!(f, "[{}] Optimization failed: {}", self.code(), message)
            }

            UltraCrewError::SerializationFailed { format, message } => write!(
                f,
                "[{}] Serialization to '{}' failed: {}",
                self.code(),
                format,
                message
            ),

            UltraCrewError::UnsupportedFormat { format } => write!(
                f,
                "[{}] Unsupported export format: '{}'. Supported: json, csv",
                self.code(),
                format
            ),

            UltraCrewError::ConfigError { path, message } => write!(
                f,
                "[{}] Config error for '{}': {}",
                self.code(),
                path,
                message
            ),
        }
    }
}

// ─── std::error::Error ───────────────────────────────────────────────────────

impl std::error::Error for UltraCrewError {}

// ─── From conversions ─────────────────────────────────────────────────────────

impl From<std::io::Error> for UltraCrewError {
    fn from(e: std::io::Error) -> Self {
        UltraCrewError::Io {
            path: "<unknown>".to_string(),
            message: e.to_string(),
        }
    }
}

impl From<serde_json::Error> for UltraCrewError {
    fn from(e: serde_json::Error) -> Self {
        UltraCrewError::ParseJson {
            path: "<unknown>".to_string(),
            message: e.to_string(),
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_display() {
        let e = UltraCrewError::io("input.json", "file not found");
        let s = e.to_string();
        assert!(s.contains("UC-IO-001"), "code missing: {}", s);
        assert!(s.contains("input.json"), "path missing: {}", s);
        assert!(s.contains("file not found"), "message missing: {}", s);
    }

    #[test]
    fn test_parse_json_error_display() {
        let e = UltraCrewError::parse_json("data.json", "unexpected token");
        let s = e.to_string();
        assert!(s.contains("UC-IMP-001"));
        assert!(s.contains("data.json"));
    }

    #[test]
    fn test_parse_csv_error_display() {
        let e = UltraCrewError::parse_csv("workers.csv", "bad column count");
        let s = e.to_string();
        assert!(s.contains("UC-IMP-002"));
        assert!(s.contains("workers.csv"));
    }

    #[test]
    fn test_validation_failed_display() {
        let e = UltraCrewError::validation_failed("input.json", "V-001: workers empty");
        let s = e.to_string();
        assert!(s.contains("UC-IMP-003"));
        assert!(s.contains("V-001"));
    }

    #[test]
    fn test_invalid_config_display() {
        let e = UltraCrewError::invalid_config("population_size must be > 0");
        let s = e.to_string();
        assert!(s.contains("UC-ENG-001"));
        assert!(s.contains("population_size"));
    }

    #[test]
    fn test_optimization_failed_display() {
        let e = UltraCrewError::optimization_failed("engine panicked");
        let s = e.to_string();
        assert!(s.contains("UC-ENG-002"));
    }

    #[test]
    fn test_serialization_failed_display() {
        let e = UltraCrewError::serialization_failed("xml", "unsupported type");
        let s = e.to_string();
        assert!(s.contains("UC-EXP-001"));
        assert!(s.contains("xml"));
    }

    #[test]
    fn test_unsupported_format_display() {
        let e = UltraCrewError::unsupported_format("parquet");
        let s = e.to_string();
        assert!(s.contains("UC-EXP-002"));
        assert!(s.contains("parquet"));
    }

    #[test]
    fn test_config_error_display() {
        let e = UltraCrewError::config_error("config.toml", "unknown field");
        let s = e.to_string();
        assert!(s.contains("UC-CFG-001"));
        assert!(s.contains("config.toml"));
    }

    #[test]
    fn test_stage_classification() {
        assert_eq!(UltraCrewError::io("f", "e").stage(), "import");
        assert_eq!(UltraCrewError::parse_json("f", "e").stage(), "import");
        assert_eq!(UltraCrewError::parse_csv("f", "e").stage(), "import");
        assert_eq!(
            UltraCrewError::validation_failed("f", "e").stage(),
            "import"
        );
        assert_eq!(UltraCrewError::invalid_config("e").stage(), "engine");
        assert_eq!(UltraCrewError::optimization_failed("e").stage(), "engine");
        assert_eq!(
            UltraCrewError::serialization_failed("json", "e").stage(),
            "export"
        );
        assert_eq!(UltraCrewError::unsupported_format("xml").stage(), "export");
        assert_eq!(UltraCrewError::config_error("f", "e").stage(), "config");
    }

    #[test]
    fn test_error_codes_are_stable() {
        // Verify codes don't change accidentally.
        assert_eq!(UltraCrewError::io("f", "e").code(), "UC-IO-001");
        assert_eq!(UltraCrewError::parse_json("f", "e").code(), "UC-IMP-001");
        assert_eq!(UltraCrewError::parse_csv("f", "e").code(), "UC-IMP-002");
        assert_eq!(
            UltraCrewError::validation_failed("f", "e").code(),
            "UC-IMP-003"
        );
        assert_eq!(UltraCrewError::invalid_config("e").code(), "UC-ENG-001");
        assert_eq!(
            UltraCrewError::optimization_failed("e").code(),
            "UC-ENG-002"
        );
        assert_eq!(
            UltraCrewError::serialization_failed("json", "e").code(),
            "UC-EXP-001"
        );
        assert_eq!(
            UltraCrewError::unsupported_format("xml").code(),
            "UC-EXP-002"
        );
        assert_eq!(UltraCrewError::config_error("f", "e").code(), "UC-CFG-001");
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let uc_err: UltraCrewError = io_err.into();
        assert_eq!(uc_err.code(), "UC-IO-001");
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("{bad}").unwrap_err();
        let uc_err: UltraCrewError = json_err.into();
        assert_eq!(uc_err.code(), "UC-IMP-001");
    }

    #[test]
    fn test_error_is_std_error() {
        // Verify UltraCrewError implements std::error::Error.
        let e: Box<dyn std::error::Error> = Box::new(UltraCrewError::io("f", "e"));
        assert!(!e.to_string().is_empty());
    }

    #[test]
    fn test_result_alias() {
        fn returns_result() -> Result<u32> {
            Ok(42)
        }
        assert_eq!(returns_result().unwrap(), 42);
    }
}
