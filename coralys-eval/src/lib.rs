//! # coralys-eval — Coralys Evaluation Framework
//!
//! This crate provides the benchmark-independent evaluation interfaces,
//! adapter registry, and evaluation pipeline for the Coralys platform.
//!
//! ## Architecture
//!
//! ```text
//! Coralys Evaluation Framework (this crate)
//!         │
//!  ┌──────┴──────────────────────┐
//!  │                             │
//! AdapterRegistry          EvaluationPipeline
//!  │                             │
//!  ├── CVD001Adapter             └── run(adapter_id, problem, solution)
//!  ├── INRCAdapter                         │
//!  └── Future adapters                     ▼
//!                                  EvaluationResult
//! ```
//!
//! ## Crate ownership
//!
//! This crate owns:
//! - [`types::EvaluationResult`] — the authoritative result type
//! - [`types::ConstraintViolation`] — the normative violation type
//! - [`types::ObjectiveValue`] — named objective values
//! - [`adapter::BenchmarkAdapter`] — the interface all adapters implement
//! - [`registry::AdapterRegistry`] — adapter registration and lookup
//! - [`pipeline::EvaluationPipeline`] — evaluation orchestration
//!
//! ## What this crate does NOT own
//!
//! - Benchmark mathematics (owned by individual adapters)
//! - Domain types (airline, nurse, vehicle routing — owned by domain crates)
//! - Optimisation algorithms (owned by `coralys-moga` and future engine crates)
//!
//! ## Adding a new adapter (M4.2 and beyond)
//!
//! 1. Implement [`adapter::BenchmarkAdapter`] for your adapter struct.
//! 2. Ensure your `Problem` and `Solution` types implement `serde::Serialize`
//!    and `serde::de::DeserializeOwned`.
//! 3. Call [`registry::AdapterRegistry::register`] with your adapter instance.
//! 4. The adapter is now available via [`pipeline::EvaluationPipeline::run`].

pub mod types;
pub mod adapter;
pub mod registry;
pub mod pipeline;

// Re-export the most commonly used types at the crate root for ergonomic imports.
pub use types::{
    ConstraintSeverity,
    ConstraintViolation,
    EvaluationResult,
    ObjectiveValue,
};
pub use adapter::{AdapterInfo, BenchmarkAdapter};
pub use registry::{AdapterRegistry, RegistryError};
pub use pipeline::EvaluationPipeline;