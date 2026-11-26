use serde::{Deserialize, Serialize};
/// S2-02 — TOML/YAML optimizer configuration loader
///
/// Provides a file-based configuration system for the UltraCrew optimizer.
/// Operators can supply a `.toml` or `.yaml`/`.yml` file to override the
/// default optimizer parameters without recompiling.
///
/// # Supported formats
///
/// | Extension       | Format |
/// |-----------------|--------|
/// | `.toml`         | TOML   |
/// | `.yaml`, `.yml` | YAML   |
///
/// # Example TOML
///
/// ```toml
/// [optimizer]
/// generation_limit = 500
/// population_size  = 50
/// rng_seed         = 42
///
/// [scenario]
/// planning_horizon_hours = 168.0
/// max_hours_per_worker   = 48.0
/// ```
///
/// # Example YAML
///
/// ```yaml
/// optimizer:
///   generation_limit: 500
///   population_size: 50
///   rng_seed: 42
///
/// scenario:
///   planning_horizon_hours: 168.0
///   max_hours_per_worker: 48.0
/// ```
///
/// All fields are optional. Missing fields fall back to built-in defaults.
/// Unknown fields are rejected with a descriptive error (strict parsing).
use std::path::Path;

// ─── Defaults ────────────────────────────────────────────────────────────────

const DEFAULT_GENERATION_LIMIT: usize = 200;
const DEFAULT_POPULATION_SIZE: usize = 50;
const DEFAULT_HORIZON_HOURS: f64 = 168.0;
const DEFAULT_MAX_HOURS: f64 = 48.0;

// ─── Config structs ───────────────────────────────────────────────────────────

/// Optimizer tuning parameters.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct OptimizerParams {
    /// Maximum number of GA generations to run.
    /// Default: 200. SunAir canonical: 500.
    pub generation_limit: Option<usize>,

    /// GA population size (number of candidate schedules per generation).
    /// Default: 50.
    pub population_size: Option<usize>,

    /// RNG seed for deterministic runs. None = non-deterministic.
    pub rng_seed: Option<u64>,
}

impl OptimizerParams {
    pub fn generation_limit(&self) -> usize {
        self.generation_limit.unwrap_or(DEFAULT_GENERATION_LIMIT)
    }

    pub fn population_size(&self) -> usize {
        self.population_size.unwrap_or(DEFAULT_POPULATION_SIZE)
    }
}

/// Planning scenario parameters.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ScenarioParams {
    /// Total planning horizon in hours (e.g. 168.0 for a 7-day week).
    /// Default: 168.0.
    pub planning_horizon_hours: Option<f64>,

    /// Maximum credited hours per worker over the planning horizon.
    /// Default: 48.0.
    pub max_hours_per_worker: Option<f64>,
}

impl ScenarioParams {
    pub fn planning_horizon_hours(&self) -> f64 {
        self.planning_horizon_hours.unwrap_or(DEFAULT_HORIZON_HOURS)
    }

    pub fn max_hours_per_worker(&self) -> f64 {
        self.max_hours_per_worker.unwrap_or(DEFAULT_MAX_HOURS)
    }
}

/// Top-level optimizer configuration file structure.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct OptimizerConfig {
    /// Optimizer tuning parameters.
    #[serde(default)]
    pub optimizer: OptimizerParams,

    /// Planning scenario parameters.
    #[serde(default)]
    pub scenario: ScenarioParams,
}

// ─── Format detection ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigFormat {
    Toml,
    Yaml,
}

impl ConfigFormat {
    /// Detect format from file extension.
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension().and_then(|e| e.to_str()) {
            Some("toml") => Some(ConfigFormat::Toml),
            Some("yaml") | Some("yml") => Some(ConfigFormat::Yaml),
            _ => None,
        }
    }
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Load an `OptimizerConfig` from a TOML or YAML file.
///
/// Format is auto-detected from the file extension.
/// Returns an error if the file cannot be read, the format is unrecognised,
/// or the content does not match the expected schema.
pub fn load_config(path: &Path) -> Result<OptimizerConfig, String> {
    let fmt = ConfigFormat::from_path(path).ok_or_else(|| {
        format!(
            "Unrecognised config file extension for '{}'. \
             Supported: .toml, .yaml, .yml",
            path.display()
        )
    })?;

    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read config file '{}': {}", path.display(), e))?;

    parse_config(&content, fmt)
}

/// Parse an `OptimizerConfig` from a string in the given format.
///
/// Exposed separately to allow unit-testing without touching the filesystem.
pub fn parse_config(content: &str, fmt: ConfigFormat) -> Result<OptimizerConfig, String> {
    match fmt {
        ConfigFormat::Toml => {
            toml::from_str(content).map_err(|e| format!("TOML parse error: {}", e))
        }
        ConfigFormat::Yaml => {
            serde_yaml::from_str(content).map_err(|e| format!("YAML parse error: {}", e))
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── TOML ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_toml_full_config() {
        let toml = r#"
[optimizer]
generation_limit = 500
population_size  = 100
rng_seed         = 42

[scenario]
planning_horizon_hours = 168.0
max_hours_per_worker   = 48.0
"#;
        let cfg = parse_config(toml, ConfigFormat::Toml).unwrap();
        assert_eq!(cfg.optimizer.generation_limit(), 500);
        assert_eq!(cfg.optimizer.population_size(), 100);
        assert_eq!(cfg.optimizer.rng_seed, Some(42));
        assert_eq!(cfg.scenario.planning_horizon_hours(), 168.0);
        assert_eq!(cfg.scenario.max_hours_per_worker(), 48.0);
    }

    #[test]
    fn test_toml_empty_uses_defaults() {
        let cfg = parse_config("", ConfigFormat::Toml).unwrap();
        assert_eq!(cfg.optimizer.generation_limit(), 200);
        assert_eq!(cfg.optimizer.population_size(), 50);
        assert_eq!(cfg.optimizer.rng_seed, None);
        assert_eq!(cfg.scenario.planning_horizon_hours(), 168.0);
        assert_eq!(cfg.scenario.max_hours_per_worker(), 48.0);
    }

    #[test]
    fn test_toml_partial_config() {
        let toml = r#"
[optimizer]
generation_limit = 300
"#;
        let cfg = parse_config(toml, ConfigFormat::Toml).unwrap();
        assert_eq!(cfg.optimizer.generation_limit(), 300);
        assert_eq!(cfg.optimizer.population_size(), 50); // default
    }

    #[test]
    fn test_toml_unknown_field_rejected() {
        let toml = r#"
[optimizer]
unknown_field = 99
"#;
        let result = parse_config(toml, ConfigFormat::Toml);
        assert!(result.is_err(), "Unknown fields must be rejected");
        assert!(result.unwrap_err().contains("TOML parse error"));
    }

    // ── YAML ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_yaml_full_config() {
        let yaml = r#"
optimizer:
  generation_limit: 500
  population_size: 100
  rng_seed: 42

scenario:
  planning_horizon_hours: 168.0
  max_hours_per_worker: 48.0
"#;
        let cfg = parse_config(yaml, ConfigFormat::Yaml).unwrap();
        assert_eq!(cfg.optimizer.generation_limit(), 500);
        assert_eq!(cfg.optimizer.population_size(), 100);
        assert_eq!(cfg.optimizer.rng_seed, Some(42));
        assert_eq!(cfg.scenario.planning_horizon_hours(), 168.0);
        assert_eq!(cfg.scenario.max_hours_per_worker(), 48.0);
    }

    #[test]
    fn test_yaml_empty_uses_defaults() {
        let cfg = parse_config("{}", ConfigFormat::Yaml).unwrap();
        assert_eq!(cfg.optimizer.generation_limit(), 200);
        assert_eq!(cfg.optimizer.population_size(), 50);
    }

    #[test]
    fn test_yaml_partial_config() {
        let yaml = r#"
optimizer:
  rng_seed: 7
"#;
        let cfg = parse_config(yaml, ConfigFormat::Yaml).unwrap();
        assert_eq!(cfg.optimizer.rng_seed, Some(7));
        assert_eq!(cfg.optimizer.generation_limit(), 200); // default
    }

    #[test]
    fn test_yaml_unknown_field_rejected() {
        let yaml = r#"
optimizer:
  mystery_param: 99
"#;
        let result = parse_config(yaml, ConfigFormat::Yaml);
        assert!(result.is_err(), "Unknown fields must be rejected");
        assert!(result.unwrap_err().contains("YAML parse error"));
    }

    // ── Format detection ──────────────────────────────────────────────────────

    #[test]
    fn test_format_detection_toml() {
        use std::path::PathBuf;
        assert_eq!(
            ConfigFormat::from_path(&PathBuf::from("config.toml")),
            Some(ConfigFormat::Toml)
        );
    }

    #[test]
    fn test_format_detection_yaml() {
        use std::path::PathBuf;
        assert_eq!(
            ConfigFormat::from_path(&PathBuf::from("config.yaml")),
            Some(ConfigFormat::Yaml)
        );
        assert_eq!(
            ConfigFormat::from_path(&PathBuf::from("config.yml")),
            Some(ConfigFormat::Yaml)
        );
    }

    #[test]
    fn test_format_detection_unknown() {
        use std::path::PathBuf;
        assert_eq!(ConfigFormat::from_path(&PathBuf::from("config.json")), None);
        assert_eq!(ConfigFormat::from_path(&PathBuf::from("config")), None);
    }

    // ── SunAir canonical fixtures ─────────────────────────────────────────────

    #[test]
    fn test_sunair_toml_fixture_parses() {
        let toml = r#"
[optimizer]
generation_limit = 500
population_size  = 50
rng_seed         = 42

[scenario]
planning_horizon_hours = 168.0
max_hours_per_worker   = 48.0
"#;
        let cfg = parse_config(toml, ConfigFormat::Toml).unwrap();
        assert_eq!(cfg.optimizer.generation_limit(), 500);
        assert_eq!(cfg.optimizer.rng_seed, Some(42));
        assert_eq!(cfg.scenario.planning_horizon_hours(), 168.0);
        assert_eq!(cfg.scenario.max_hours_per_worker(), 48.0);
    }

    #[test]
    fn test_sunair_yaml_fixture_parses() {
        let yaml = r#"
optimizer:
  generation_limit: 500
  population_size: 50
  rng_seed: 42

scenario:
  planning_horizon_hours: 168.0
  max_hours_per_worker: 48.0
"#;
        let cfg = parse_config(yaml, ConfigFormat::Yaml).unwrap();
        assert_eq!(cfg.optimizer.generation_limit(), 500);
        assert_eq!(cfg.optimizer.rng_seed, Some(42));
        assert_eq!(cfg.scenario.planning_horizon_hours(), 168.0);
    }

    #[test]
    fn test_load_config_from_toml_file() {
        // Load the actual SunAir fixture file from disk.
        let path = std::path::Path::new("fixtures/demo/sunair_optimizer.toml");
        if path.exists() {
            let cfg = load_config(path).expect("SunAir TOML fixture must load cleanly");
            assert_eq!(cfg.optimizer.generation_limit(), 500);
            assert_eq!(cfg.optimizer.rng_seed, Some(42));
        }
        // If the file doesn't exist in the test environment, skip silently.
    }

    #[test]
    fn test_load_config_from_yaml_file() {
        let path = std::path::Path::new("fixtures/demo/sunair_optimizer.yaml");
        if path.exists() {
            let cfg = load_config(path).expect("SunAir YAML fixture must load cleanly");
            assert_eq!(cfg.optimizer.generation_limit(), 500);
            assert_eq!(cfg.optimizer.rng_seed, Some(42));
        }
    }

    #[test]
    fn test_load_config_unknown_extension_errors() {
        let path = std::path::Path::new("fixtures/demo/sunair_demo.json");
        let result = load_config(path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Unrecognised config file extension"));
    }
}
