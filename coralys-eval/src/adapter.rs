//! The `BenchmarkAdapter` trait — the single interface all Coralys adapters implement.
//!
//! # Design rationale
//!
//! The trait is generic over `Problem` and `Solution` so that:
//! - Each adapter defines its own domain types (no shared airline/benchmark types leak in).
//! - The framework can call `evaluate` without knowing anything about the domain.
//! - The `AdapterRegistry` erases these generics via `ErasedAdapter` (see `registry.rs`).
//!
//! # Implementing an adapter
//!
//! ```rust,ignore
//! use coralys_eval::adapter::BenchmarkAdapter;
//! use coralys_eval::types::EvaluationResult;
//!
//! pub struct MyCVD001Adapter;
//!
//! impl BenchmarkAdapter for MyCVD001Adapter {
//!     type Problem = cvd001::Problem;
//!     type Solution = cvd001::Solution;
//!
//!     fn adapter_id(&self) -> &'static str { "cvd001" }
//!     fn adapter_name(&self) -> &'static str { "CVD-001 Reference Adapter" }
//!     fn adapter_version(&self) -> &'static str { "1.0.0" }
//!
//!     fn evaluate(
//!         &self,
//!         problem: &Self::Problem,
//!         solution: &Self::Solution,
//!     ) -> EvaluationResult {
//!         // ... map cvd001::EvaluationResult → coralys_eval::EvaluationResult
//!     }
//! }
//! ```

use crate::types::EvaluationResult;

/// The core trait that every Coralys adapter must implement.
///
/// An adapter is a bridge between a specific problem domain (benchmark or
/// production scheduling problem) and the Coralys Evaluation Framework.
///
/// Adapters are responsible for:
/// 1. Defining their own `Problem` and `Solution` types.
/// 2. Implementing `evaluate` to produce a framework-standard `EvaluationResult`.
/// 3. Providing stable identity metadata (`adapter_id`, `adapter_name`, `adapter_version`).
///
/// Adapters must NOT:
/// - Modify the Coralys Evaluation Framework.
/// - Assume they are the only adapter registered.
/// - Embed framework-level logic (pipeline orchestration, registry management).
pub trait BenchmarkAdapter: Send + Sync {
    /// The problem representation for this adapter's domain.
    type Problem: Send + Sync;

    /// The solution representation for this adapter's domain.
    type Solution: Send + Sync;

    /// A stable, unique identifier for this adapter (e.g. `"cvd001"`, `"inrc2010"`).
    ///
    /// This string is used as the registry key. It must be unique across all
    /// registered adapters and must not change between versions without a
    /// deliberate versioned update.
    fn adapter_id(&self) -> &'static str;

    /// A human-readable name for this adapter (e.g. `"CVD-001 Reference Adapter"`).
    fn adapter_name(&self) -> &'static str;

    /// The version of this adapter implementation (e.g. `"1.0.0"`).
    fn adapter_version(&self) -> &'static str;

    /// Evaluate a solution against a problem instance.
    ///
    /// Returns a framework-standard `EvaluationResult`. The `adapter_id` field
    /// of the result must match `self.adapter_id()`.
    fn evaluate(&self, problem: &Self::Problem, solution: &Self::Solution) -> EvaluationResult;
}

/// Metadata about a registered adapter, independent of its generic types.
///
/// Used by the registry to list available adapters without requiring type erasure
/// of the full adapter.
#[derive(Debug, Clone)]
pub struct AdapterInfo {
    /// Stable unique identifier.
    pub id: &'static str,
    /// Human-readable name.
    pub name: &'static str,
    /// Version string.
    pub version: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EvaluationResult, ObjectiveValue};

    // A minimal stub adapter for testing the trait contract.
    struct StubAdapter;

    struct StubProblem;
    struct StubSolution {
        pub value: f64,
    }

    impl BenchmarkAdapter for StubAdapter {
        type Problem = StubProblem;
        type Solution = StubSolution;

        fn adapter_id(&self) -> &'static str {
            "stub"
        }
        fn adapter_name(&self) -> &'static str {
            "Stub Adapter"
        }
        fn adapter_version(&self) -> &'static str {
            "0.0.1"
        }

        fn evaluate(
            &self,
            _problem: &Self::Problem,
            solution: &Self::Solution,
        ) -> EvaluationResult {
            EvaluationResult::new(
                self.adapter_id(),
                vec![ObjectiveValue::new("obj", "Objective", solution.value)],
                vec![],
            )
        }
    }

    #[test]
    fn stub_adapter_returns_correct_id() {
        let adapter = StubAdapter;
        assert_eq!(adapter.adapter_id(), "stub");
        assert_eq!(adapter.adapter_name(), "Stub Adapter");
        assert_eq!(adapter.adapter_version(), "0.0.1");
    }

    #[test]
    fn stub_adapter_evaluate_produces_correct_result() {
        let adapter = StubAdapter;
        let problem = StubProblem;
        let solution = StubSolution { value: 42.0 };
        let result = adapter.evaluate(&problem, &solution);
        assert_eq!(result.adapter_id, "stub");
        assert!(result.feasible);
        assert!((result.primary_objective() - 42.0).abs() < 1e-9);
    }
}
