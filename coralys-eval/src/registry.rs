//! Adapter registry for the Coralys Evaluation Framework.
//!
//! The registry holds type-erased adapter entries. Each entry stores:
//! - The adapter's identity metadata (`AdapterInfo`).
//! - A type-erased `evaluate` function that accepts serialised JSON problem/solution
//!   payloads and returns an `EvaluationResult`.
//!
//! # Type erasure strategy
//!
//! `BenchmarkAdapter` is generic over `Problem` and `Solution`. The registry
//! cannot store `Box<dyn BenchmarkAdapter<Problem=?, Solution=?>>` directly
//! because the associated types differ per adapter.
//!
//! Instead, each adapter is wrapped in a `RegisteredAdapter` that captures the
//! concrete types at registration time and exposes a uniform
//! `fn(&[u8], &[u8]) -> Result<EvaluationResult, RegistryError>` interface.
//! Callers serialise their problem/solution to JSON; the registry deserialises
//! them into the concrete types and calls the adapter.
//!
//! This approach:
//! - Requires `Problem: DeserializeOwned` and `Solution: DeserializeOwned`.
//! - Avoids `unsafe` and does not require `Any` downcasting.
//! - Allows the registry to be used across crate boundaries without exposing
//!   concrete adapter types.
//!
//! For M4.2, the CVD-001 adapter will be registered here.

use crate::adapter::{AdapterInfo, BenchmarkAdapter};
use crate::types::EvaluationResult;
use serde::de::DeserializeOwned;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur during registry operations.
#[derive(Debug, Clone, PartialEq)]
pub enum RegistryError {
    /// No adapter with the given id is registered.
    AdapterNotFound(String),
    /// An adapter with the given id is already registered.
    DuplicateAdapter(String),
    /// The problem payload could not be deserialised.
    ProblemDeserializationError(String),
    /// The solution payload could not be deserialised.
    SolutionDeserializationError(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::AdapterNotFound(id) => write!(f, "No adapter registered with id '{id}'"),
            RegistryError::DuplicateAdapter(id) => {
                write!(f, "Adapter '{id}' is already registered")
            }
            RegistryError::ProblemDeserializationError(msg) => {
                write!(f, "Failed to deserialise problem: {msg}")
            }
            RegistryError::SolutionDeserializationError(msg) => {
                write!(f, "Failed to deserialise solution: {msg}")
            }
        }
    }
}

impl std::error::Error for RegistryError {}

// ---------------------------------------------------------------------------
// Internal erased entry
// ---------------------------------------------------------------------------

/// A type-erased adapter entry stored in the registry.
struct RegisteredAdapter {
    info: AdapterInfo,
    /// Evaluate a (problem_json, solution_json) pair.
    evaluate_fn: Box<dyn Fn(&[u8], &[u8]) -> Result<EvaluationResult, RegistryError> + Send + Sync>,
}

// ---------------------------------------------------------------------------
// AdapterRegistry
// ---------------------------------------------------------------------------

/// The Coralys adapter registry.
///
/// Holds all registered adapters and provides a uniform evaluation interface.
/// The registry is the single point of extension for adding new benchmark
/// adapters or production scheduling evaluators.
///
/// # Usage
///
/// ```rust,ignore
/// let mut registry = AdapterRegistry::new();
/// registry.register(MyCVD001Adapter)?;
///
/// let result = registry.evaluate("cvd001", problem_json, solution_json)?;
/// ```
pub struct AdapterRegistry {
    adapters: HashMap<&'static str, RegisteredAdapter>,
}

impl AdapterRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    /// Register an adapter.
    ///
    /// The adapter's `Problem` and `Solution` types must implement
    /// `serde::de::DeserializeOwned` so that the registry can accept
    /// JSON-encoded inputs from callers that do not have access to the
    /// concrete types.
    ///
    /// Returns `Err(RegistryError::DuplicateAdapter)` if an adapter with the
    /// same `adapter_id` is already registered.
    pub fn register<A>(&mut self, adapter: A) -> Result<(), RegistryError>
    where
        A: BenchmarkAdapter + 'static,
        A::Problem: DeserializeOwned,
        A::Solution: DeserializeOwned,
    {
        let id = adapter.adapter_id();
        if self.adapters.contains_key(id) {
            return Err(RegistryError::DuplicateAdapter(id.to_string()));
        }

        let info = AdapterInfo {
            id,
            name: adapter.adapter_name(),
            version: adapter.adapter_version(),
        };

        // Capture the adapter in a closure that erases the generic types.
        let evaluate_fn = Box::new(move |problem_bytes: &[u8], solution_bytes: &[u8]| {
            let problem: A::Problem = serde_json::from_slice(problem_bytes)
                .map_err(|e| RegistryError::ProblemDeserializationError(e.to_string()))?;
            let solution: A::Solution = serde_json::from_slice(solution_bytes)
                .map_err(|e| RegistryError::SolutionDeserializationError(e.to_string()))?;
            Ok(adapter.evaluate(&problem, &solution))
        });

        self.adapters
            .insert(id, RegisteredAdapter { info, evaluate_fn });
        Ok(())
    }

    /// Evaluate a problem/solution pair using the named adapter.
    ///
    /// `problem_json` and `solution_json` must be valid JSON representations
    /// of the adapter's `Problem` and `Solution` types respectively.
    pub fn evaluate(
        &self,
        adapter_id: &str,
        problem_json: &[u8],
        solution_json: &[u8],
    ) -> Result<EvaluationResult, RegistryError> {
        let entry = self
            .adapters
            .get(adapter_id)
            .ok_or_else(|| RegistryError::AdapterNotFound(adapter_id.to_string()))?;
        (entry.evaluate_fn)(problem_json, solution_json)
    }

    /// Return metadata for all registered adapters, sorted by id.
    pub fn list(&self) -> Vec<AdapterInfo> {
        let mut infos: Vec<AdapterInfo> = self.adapters.values().map(|e| e.info.clone()).collect();
        infos.sort_by_key(|i| i.id);
        infos
    }

    /// Return metadata for a specific adapter, or `None` if not registered.
    pub fn info(&self, adapter_id: &str) -> Option<&AdapterInfo> {
        self.adapters.get(adapter_id).map(|e| &e.info)
    }

    /// Return `true` if an adapter with the given id is registered.
    pub fn contains(&self, adapter_id: &str) -> bool {
        self.adapters.contains_key(adapter_id)
    }

    /// Return the number of registered adapters.
    pub fn len(&self) -> usize {
        self.adapters.len()
    }

    /// Return `true` if no adapters are registered.
    pub fn is_empty(&self) -> bool {
        self.adapters.is_empty()
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::BenchmarkAdapter;
    use crate::types::{EvaluationResult, ObjectiveValue};
    use serde::{Deserialize, Serialize};

    // Minimal stub adapter for registry tests.
    #[derive(Serialize, Deserialize)]
    struct StubProblem {
        pub scale: f64,
    }

    #[derive(Serialize, Deserialize)]
    struct StubSolution {
        pub value: f64,
    }

    struct StubAdapter;

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

        fn evaluate(&self, problem: &Self::Problem, solution: &Self::Solution) -> EvaluationResult {
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

    fn make_registry() -> AdapterRegistry {
        let mut r = AdapterRegistry::new();
        r.register(StubAdapter).unwrap();
        r
    }

    #[test]
    fn register_and_list() {
        let r = make_registry();
        assert_eq!(r.len(), 1);
        let infos = r.list();
        assert_eq!(infos[0].id, "stub");
        assert_eq!(infos[0].name, "Stub Adapter");
    }

    #[test]
    fn duplicate_registration_returns_error() {
        let mut r = AdapterRegistry::new();
        r.register(StubAdapter).unwrap();
        let err = r.register(StubAdapter).unwrap_err();
        assert_eq!(err, RegistryError::DuplicateAdapter("stub".to_string()));
    }

    #[test]
    fn evaluate_unknown_adapter_returns_error() {
        let r = make_registry();
        let err = r.evaluate("unknown", b"{}", b"{}").unwrap_err();
        assert_eq!(err, RegistryError::AdapterNotFound("unknown".to_string()));
    }

    #[test]
    fn evaluate_via_registry_produces_correct_result() {
        let r = make_registry();
        let problem = serde_json::to_vec(&StubProblem { scale: 2.0 }).unwrap();
        let solution = serde_json::to_vec(&StubSolution { value: 21.0 }).unwrap();
        let result = r.evaluate("stub", &problem, &solution).unwrap();
        assert_eq!(result.adapter_id, "stub");
        assert!(result.feasible);
        assert!((result.primary_objective() - 42.0).abs() < 1e-9);
    }

    #[test]
    fn contains_returns_correct_values() {
        let r = make_registry();
        assert!(r.contains("stub"));
        assert!(!r.contains("missing"));
    }

    #[test]
    fn info_returns_correct_metadata() {
        let r = make_registry();
        let info = r.info("stub").unwrap();
        assert_eq!(info.id, "stub");
        assert_eq!(info.version, "0.0.1");
    }
}
