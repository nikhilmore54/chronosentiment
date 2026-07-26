// src/config/mod.rs

pub mod optimization_profiles;
pub mod optimizer_config;

pub use optimization_profiles::OptimizationProfile;
pub use optimizer_config::{OptimizerConfig, OptimizerParams, ScenarioParams, load_config, parse_config, ConfigFormat};
