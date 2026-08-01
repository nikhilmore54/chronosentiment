//! Milestone 1 — Experimental Infrastructure
//!
//! Reusable experiment harness for all Section 3 experiments (Experiments 0–10).
//! Every experiment is a configuration, not a custom executable.
//!
//! # Work Packages
//!
//! - WP1: Experiment harness (registration, benchmark loading, repeat execution, persistence)
//! - WP2: Standardized experiment schema (JSON metadata)
//! - WP3: Machine-readable results (experiment.json, metrics.csv, generations.csv, summary.json)
//! - WP4: Generation metrics struct
//! - WP5: Logging API (structured events)
//! - WP6: Automatic report generator
//! - WP7: Reproducibility layer
//! - WP8: Baseline freeze marker

pub mod schema;
pub mod logging;
pub mod persistence;
pub mod report;
pub mod reproducibility;

pub use schema::{
    ExperimentConfig, ExperimentResult, GenerationRecord, RunSummary,
    InitStrategy, OperatorConfig,
};
pub use logging::{EventLogger, ExperimentEvent};
pub use persistence::ResultPersistence;
pub use report::ReportGenerator;
pub use reproducibility::ReproducibilityInfo;