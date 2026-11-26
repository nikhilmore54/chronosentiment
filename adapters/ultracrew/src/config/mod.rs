// src/config/mod.rs

pub mod fatigue_config;
pub mod optimization_profiles;
pub mod optimizer_config;

pub use fatigue_config::FatigueConfig;
pub use optimization_profiles::OptimizationProfile;
pub use optimizer_config::{
    load_config, parse_config, ConfigFormat, OptimizerConfig, OptimizerParams, ScenarioParams,
};
