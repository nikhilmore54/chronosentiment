/// S2-05 — Health check endpoint
///
/// Provides a `health_check()` function that returns a structured
/// `HealthResponse` suitable for:
///
/// - CLI: `ultracrew-cli --health` prints the JSON response to stdout
/// - HTTP: a future `/health` route returns the serialised struct
/// - Monitoring: any process that needs to verify the adapter is alive
///
/// # Response schema
///
/// ```json
/// {
///   "status": "ok",
///   "version": "0.1.0",
///   "adapter": "ultracrew",
///   "checks": {
///     "config": "ok",
///     "validator": "ok"
///   }
/// }
/// ```
///
/// `status` is `"ok"` when all checks pass, `"degraded"` when at least one
/// check fails but the adapter can still serve requests, or `"error"` when
/// the adapter cannot function.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Version constant ─────────────────────────────────────────────────────────

/// The adapter version, sourced from `Cargo.toml` at compile time.
pub const ADAPTER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The adapter name.
pub const ADAPTER_NAME: &str = "ultracrew";

// ─── Response types ───────────────────────────────────────────────────────────

/// Overall health status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Ok,
    Degraded,
    Error,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Ok      => write!(f, "ok"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Error   => write!(f, "error"),
        }
    }
}

/// The structured health check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Overall status: `"ok"`, `"degraded"`, or `"error"`.
    pub status: HealthStatus,

    /// Adapter version string (from `Cargo.toml`).
    pub version: String,

    /// Adapter name (`"ultracrew"`).
    pub adapter: String,

    /// Per-subsystem check results. Each value is `"ok"` or an error message.
    pub checks: HashMap<String, String>,
}

impl HealthResponse {
    /// Serialise to a pretty-printed JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self)
            .unwrap_or_else(|_| r#"{"status":"error","version":"unknown"}"#.to_string())
    }

    /// Return `true` if the overall status is `Ok`.
    pub fn is_ok(&self) -> bool {
        self.status == HealthStatus::Ok
    }
}

// ─── Health check logic ───────────────────────────────────────────────────────

/// Run all health checks and return a `HealthResponse`.
///
/// Currently checks:
/// - `config`: verifies the default `OptimizerConfig` can be constructed
/// - `validator`: verifies the strict validator module is reachable
pub fn health_check() -> HealthResponse {
    let mut checks: HashMap<String, String> = HashMap::new();

    // Check 1: config subsystem
    let config_ok = check_config();
    checks.insert(
        "config".to_string(),
        if config_ok { "ok".to_string() } else { "config subsystem unavailable".to_string() },
    );

    // Check 2: validator subsystem
    let validator_ok = check_validator();
    checks.insert(
        "validator".to_string(),
        if validator_ok { "ok".to_string() } else { "validator subsystem unavailable".to_string() },
    );

    let all_ok = config_ok && validator_ok;
    let status = if all_ok { HealthStatus::Ok } else { HealthStatus::Degraded };

    HealthResponse {
        status,
        version: ADAPTER_VERSION.to_string(),
        adapter: ADAPTER_NAME.to_string(),
        checks,
    }
}

/// Verify the config subsystem by constructing a default `OptimizerConfig`.
fn check_config() -> bool {
    // parse_config with empty TOML should always succeed (all fields optional).
    crate::config::optimizer_config::parse_config("", crate::config::optimizer_config::ConfigFormat::Toml).is_ok()
}

/// Verify the validator subsystem by running a minimal valid request through it.
fn check_validator() -> bool {
    use crate::models::{Shift, Worker};
    use crate::public_contracts::{ScheduleRequest, Scenario};
    use crate::strict_validator::validate_request;
    use crate::models::Skill;

    let req = ScheduleRequest {
        workers: vec![Worker { id: 1, skills: vec![Skill::new("Captain")] }],
        shifts:  vec![Shift  { id: 1, start_hour: 6, duration_hours: 8, required_skill: Skill::new("Captain"), flight_id: None, crew_role: None }],
        historical_workloads: None,
        rng_seed: Some(42),
        generation_limit: Some(10),
        scenario: Some(Scenario {
            planning_horizon_hours: Some(168.0),
            max_hours_per_worker: Some(48.0),
        }),
    };
    validate_request(&req).is_valid()
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_returns_ok() {
        let resp = health_check();
        assert_eq!(resp.status, HealthStatus::Ok, "Health check must return ok: {:?}", resp);
        assert!(resp.is_ok());
    }

    #[test]
    fn test_health_check_version_is_set() {
        let resp = health_check();
        assert!(!resp.version.is_empty(), "Version must not be empty");
        // Version must look like semver (at least one dot).
        assert!(resp.version.contains('.'), "Version must be semver: {}", resp.version);
    }

    #[test]
    fn test_health_check_adapter_name() {
        let resp = health_check();
        assert_eq!(resp.adapter, "ultracrew");
    }

    #[test]
    fn test_health_check_all_subsystems_present() {
        let resp = health_check();
        assert!(resp.checks.contains_key("config"), "config check missing");
        assert!(resp.checks.contains_key("validator"), "validator check missing");
    }

    #[test]
    fn test_health_check_subsystems_ok() {
        let resp = health_check();
        assert_eq!(resp.checks["config"], "ok");
        assert_eq!(resp.checks["validator"], "ok");
    }

    #[test]
    fn test_health_response_to_json() {
        let resp = health_check();
        let json = resp.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json)
            .expect("health response JSON must be valid");
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["adapter"], "ultracrew");
        assert!(parsed["version"].is_string());
        assert!(parsed["checks"].is_object());
    }

    #[test]
    fn test_health_status_display() {
        assert_eq!(HealthStatus::Ok.to_string(), "ok");
        assert_eq!(HealthStatus::Degraded.to_string(), "degraded");
        assert_eq!(HealthStatus::Error.to_string(), "error");
    }

    #[test]
    fn test_adapter_version_constant() {
        assert!(!ADAPTER_VERSION.is_empty());
        assert!(ADAPTER_VERSION.contains('.'));
    }
}