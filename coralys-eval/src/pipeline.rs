//! The Coralys Evaluation Pipeline.
//!
//! The pipeline is the top-level orchestrator for evaluation. It holds a
//! reference to an `AdapterRegistry` and provides a single entry point:
//! `EvaluationPipeline::run`.
//!
//! # Responsibilities
//!
//! The pipeline is responsible for:
//! 1. Looking up the requested adapter in the registry.
//! 2. Delegating evaluation to the adapter.
//! 3. Returning the `EvaluationResult` to the caller.
//!
//! The pipeline is intentionally thin. It does not implement constraint logic,
//! objective logic, or domain-specific behaviour. Those belong in adapters.
//!
//! # Future extension points
//!
//! The pipeline is the right place to add cross-cutting concerns that apply to
//! all evaluations, such as:
//! - Timing / telemetry (record evaluation duration in `result.metrics`).
//! - Caching (return a cached result for identical inputs).
//! - Logging / audit trail.
//! - Pre/post-evaluation hooks for multi-objective aggregation.
//!
//! These are not implemented in M4.1 to avoid premature generalisation.

use crate::registry::{AdapterRegistry, RegistryError};
use crate::types::EvaluationResult;

/// The Coralys Evaluation Pipeline.
///
/// Wraps an `AdapterRegistry` and provides a uniform evaluation entry point.
/// The pipeline is the primary interface for Coralys scheduling engines and
/// product layers to invoke evaluation without knowing which adapter is in use.
pub struct EvaluationPipeline {
    registry: AdapterRegistry,
}

impl EvaluationPipeline {
    /// Create a pipeline backed by the given registry.
    pub fn new(registry: AdapterRegistry) -> Self {
        Self { registry }
    }

    /// Run evaluation using the named adapter.
    ///
    /// # Arguments
    ///
    /// - `adapter_id`: The stable id of the adapter to use (e.g. `"cvd001"`).
    /// - `problem_json`: JSON-encoded problem instance.
    /// - `solution_json`: JSON-encoded solution to evaluate.
    ///
    /// # Returns
    ///
    /// `Ok(EvaluationResult)` on success, or a `RegistryError` if the adapter
    /// is not found or the inputs cannot be deserialised.
    pub fn run(
        &self,
        adapter_id: &str,
        problem_json: &[u8],
        solution_json: &[u8],
    ) -> Result<EvaluationResult, RegistryError> {
        self.registry
            .evaluate(adapter_id, problem_json, solution_json)
    }

    /// Return the underlying registry (read-only).
    pub fn registry(&self) -> &AdapterRegistry {
        &self.registry
    }

    /// Return `true` if the named adapter is available in this pipeline.
    pub fn has_adapter(&self, adapter_id: &str) -> bool {
        self.registry.contains(adapter_id)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::BenchmarkAdapter;
    use crate::registry::AdapterRegistry;
    use crate::types::{EvaluationResult, ObjectiveValue};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct P {
        pub scale: f64,
    }

    #[derive(Serialize, Deserialize)]
    struct S {
        pub value: f64,
    }

    struct PipelineStubAdapter;

    impl BenchmarkAdapter for PipelineStubAdapter {
        type Problem = P;
        type Solution = S;
        fn adapter_id(&self) -> &'static str {
            "pipeline-stub"
        }
        fn adapter_name(&self) -> &'static str {
            "Pipeline Stub"
        }
        fn adapter_version(&self) -> &'static str {
            "0.0.1"
        }
        fn evaluate(&self, problem: &P, solution: &S) -> EvaluationResult {
            EvaluationResult::new(
                self.adapter_id(),
                vec![ObjectiveValue::new(
                    "obj",
                    "Objective",
                    solution.value * problem.scale,
                )],
                vec![],
            )
        }
    }

    fn make_pipeline() -> EvaluationPipeline {
        let mut registry = AdapterRegistry::new();
        registry.register(PipelineStubAdapter).unwrap();
        EvaluationPipeline::new(registry)
    }

    #[test]
    fn pipeline_run_produces_correct_result() {
        let pipeline = make_pipeline();
        let problem = serde_json::to_vec(&P { scale: 3.0 }).unwrap();
        let solution = serde_json::to_vec(&S { value: 14.0 }).unwrap();
        let result = pipeline.run("pipeline-stub", &problem, &solution).unwrap();
        assert!(result.feasible);
        assert!((result.primary_objective() - 42.0).abs() < 1e-9);
    }

    #[test]
    fn pipeline_run_unknown_adapter_returns_error() {
        let pipeline = make_pipeline();
        let err = pipeline.run("unknown", b"{}", b"{}").unwrap_err();
        assert!(matches!(err, RegistryError::AdapterNotFound(_)));
    }

    #[test]
    fn pipeline_has_adapter_returns_correct_values() {
        let pipeline = make_pipeline();
        assert!(pipeline.has_adapter("pipeline-stub"));
        assert!(!pipeline.has_adapter("missing"));
    }
}
