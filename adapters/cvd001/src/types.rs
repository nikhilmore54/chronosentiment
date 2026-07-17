/// A single flight leg within a duty.
///
/// # Fields
/// - `id`: unique flight leg identifier (f ∈ F)
/// - `credit`: credited minutes c_f for this leg (pre-computed by loader)
/// - `duration`: actual block time d_f in minutes
///
/// # Caller responsibility
/// `credit >= 0.0`, `duration >= 0.0`.
#[derive(Debug, Clone)]
pub struct FlightLeg {
    pub id: u32,
    /// c_f — credited minutes for this leg.
    pub credit: f64,
    /// d_f — actual block minutes.
    pub duration: f64,
}

/// A duty (work period) composed of one or more flight legs.
///
/// # Fields
/// - `id`: unique duty identifier (t ∈ T)
/// - `credit`: credited minutes c_t for this duty (pre-computed by loader;
///   may differ from the sum of leg credits due to qualification rules)
/// - `legs`: the flight legs comprising this duty
///
/// # Caller responsibility
/// `credit >= 0.0`.
#[derive(Debug, Clone)]
pub struct Duty {
    pub id: u32,
    /// c_t — duty-level credited minutes (authoritative; pre-computed by loader).
    pub credit: f64,
    pub legs: Vec<FlightLeg>,
}

/// A crew member with their contract parameters and assigned duties.
///
/// # Fields
/// - `id`: unique crew member identifier (n ∈ N)
/// - `min_workload`: W^min_n — contractual minimum (soft, enforced via Δ_n)
/// - `max_workload`: W^max_n — hard cap; HC3-A requires W_n <= max_workload
/// - `target_workload`: t_n — target credited minutes; Δ_n = |W_n − t_n|
/// - `duties`: duties assigned to this crew member in this solution
///
/// # Caller responsibility
/// `min_workload <= max_workload`. `target_workload >= 0.0`.
#[derive(Debug, Clone)]
pub struct CrewMember {
    pub id: u32,
    /// W^min_n — contractual minimum (soft enforcement via Δ_n).
    pub min_workload: f64,
    /// W^max_n — hard cap; HC3-A requires W_n <= max_workload.
    pub max_workload: f64,
    /// t_n — target workload; Δ_n = |W_n − t_n|.
    pub target_workload: f64,
    pub duties: Vec<Duty>,
}

/// A complete solution: the full crew roster for one scheduling period.
///
/// `crew` is indexed 0..N-1. The evaluator treats this as the complete input.
#[derive(Debug, Clone)]
pub struct Solution {
    pub crew: Vec<CrewMember>,
}

/// A structured record of a single constraint violation.
///
/// Carrying violations as structured data (rather than only a boolean flag)
/// allows Coralys to consume constraint information uniformly across benchmarks
/// and to support richer diagnostics and multi-constraint evaluation in future.
#[derive(Debug, Clone)]
pub struct ConstraintViolation {
    /// Human-readable constraint identifier (e.g. "HC3").
    pub constraint: &'static str,
    /// Index into `EvaluationResult::workloads` of the violating crew member.
    pub crew_member_index: usize,
    /// The crew member's `id` field.
    pub crew_member_id: u32,
    /// The computed workload W_n that caused the violation.
    pub workload: f64,
    /// The threshold that was exceeded (W^max_n for HC3).
    pub threshold: f64,
}

/// The result of evaluating a solution against the CVD-001 benchmark.
///
/// # Fields
/// - `workloads`: W_n for each crew member, same order as `solution.crew`
/// - `violations`: structured list of constraint violations (empty if feasible)
/// - `feasible`: true iff `violations` is empty
/// - `objective`: Z = Σ_n |W_n − t_n|; `f64::INFINITY` when infeasible
///
/// # Design
/// `violations` is always populated with full diagnostic information even when
/// `feasible` is false. This allows Coralys to inspect which constraints were
/// violated and by how much, without re-evaluating the solution.
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    /// W_n per crew member (always populated, even when infeasible).
    pub workloads: Vec<f64>,
    /// Structured constraint violations; empty iff feasible.
    pub violations: Vec<ConstraintViolation>,
    /// true iff `violations` is empty.
    pub feasible: bool,
    /// Z = Σ_n Δ_n if feasible; f64::INFINITY otherwise.
    pub objective: f64,
}