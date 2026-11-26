// adapters/ultracrew/src/config/fatigue_config.rs

use serde::{Deserialize, Serialize};

/// Experimental configuration for the fatigue feature.
///
/// * `enable_fatigue` – experimental switch; when `true` the fatigue penalty is
///   applied according to `fatigue_weight`. Default is `false` (OFF).
/// * `fatigue_weight` – scaling factor for the raw fatigue signal. No default
///   is provided; the value must be supplied explicitly in experiments.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FatigueConfig {
    /// Experimental switch – disables the fatigue contribution when `false`.
    #[serde(default)]
    pub enable_fatigue: bool,
    /// Scaling factor for the raw fatigue signal. No default; must be supplied
    /// explicitly in experiments.
    #[serde(default)]
    pub fatigue_weight: f64,
}
