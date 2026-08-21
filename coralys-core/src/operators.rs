use std::error::Error;

/// Defines the hard constraint evaluation interface for a candidate genome.
pub trait ConstraintModel<G> {
    type Violation: std::fmt::Debug + Clone;

    /// Evaluates the candidate and returns all active violations.
    fn evaluate_violations(&self, candidate: &G) -> Vec<Self::Violation>;

    /// Returns true if zero hard constraints are violated.
    fn is_feasible(&self, candidate: &G) -> bool {
        self.evaluate_violations(candidate).is_empty()
    }
}

pub struct NoOpConstraintModel<G>(std::marker::PhantomData<G>);
impl<G> Default for NoOpConstraintModel<G> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}
impl<G> ConstraintModel<G> for NoOpConstraintModel<G> {
    type Violation = ();
    fn evaluate_violations(&self, _candidate: &G) -> Vec<Self::Violation> {
        vec![]
    }
}

/// Execution budget bounding operator runtime.
#[derive(Debug, Clone)]
pub struct OperatorBudget {
    pub max_iterations: usize,
    pub max_time_ms: u64,
}

/// REPAIR OPERATOR: R : X -> X U X_F
/// Transforms a potentially invalid candidate toward a feasible state.
pub trait RepairOperator<G, M: ConstraintModel<G>> {
    type Error: Error + Send + Sync + 'static;

    /// Attempts to resolve violations. Returns Ok(true) if fully repaired,
    /// Ok(false) if budget was exhausted before full feasibility, or Err on failure.
    fn repair(
        &self,
        candidate: &mut G,
        model: &M,
        budget: &OperatorBudget,
    ) -> Result<bool, Self::Error>;
}

/// IMPROVEMENT OPERATOR: I: X_F -> X_F
/// Performs local search within the feasible space, preserving feasibility.
pub trait ImprovementOperator<G, M: ConstraintModel<G>> {
    type Error: Error + Send + Sync + 'static;

    /// Applies local search heuristics. 
    /// INVARIANT: If candidate was feasible before improve(), it MUST remain feasible.
    /// Note: The Rust type system expresses this intended contract; runtime verification 
    /// via assertions enforces the preservation property.
    fn improve(
        &self,
        candidate: &mut G,
        model: &M,
        budget: &OperatorBudget,
    ) -> Result<bool, Self::Error>;
}
