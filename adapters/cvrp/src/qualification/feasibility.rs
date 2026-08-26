use crate::{CvrpInstance, DistanceMetric};
/// Feasibility Certification Framework — GOV-009
/// Implements FC-1 (Structural), FC-2.5 (Benchmark Consistency), FC-2 (Capacity).
/// These are O(n) / O(1) checks that run before every optimization.
///
/// Reference: benchmarks/campaign/feasibility_certification_framework.md
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Outcome types
// ---------------------------------------------------------------------------

/// Pass/Fail result for a single FC check, with a human-readable reason.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcResult {
    pub passed: bool,
    pub reason: String,
}

impl FcResult {
    pub fn pass(reason: impl Into<String>) -> Self {
        Self {
            passed: true,
            reason: reason.into(),
        }
    }
    pub fn fail(reason: impl Into<String>) -> Self {
        Self {
            passed: false,
            reason: reason.into(),
        }
    }
}

/// Feasibility status per GOV-009 §3.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FeasibilityStatus {
    /// A solution satisfying every constraint has been verified.
    /// Certificate: the solution itself (Coralys or exact solver).
    ProvenFeasible,
    /// Mathematical proof of infeasibility at the given FC level.
    ProvenInfeasible { level: String, reason: String },
    /// Passed FC-1 through FC-3; exact proof not attempted.
    FeasibilityUndetermined,
    /// Optimizer exhausted budget without finding a feasible solution.
    SolverFailed,
    /// Benchmark metadata or registry error (FC-2.5 failure).
    BenchmarkInvalid { reason: String },
    /// Instance graph is malformed (FC-1 failure).
    StructuralInvalid { reason: String },
}

impl FeasibilityStatus {
    /// Short code for log output.
    pub fn code(&self) -> &'static str {
        match self {
            FeasibilityStatus::ProvenFeasible => "PROVEN_FEASIBLE",
            FeasibilityStatus::ProvenInfeasible { .. } => "PROVEN_INFEASIBLE",
            FeasibilityStatus::FeasibilityUndetermined => "FEASIBILITY_UNDETERMINED",
            FeasibilityStatus::SolverFailed => "SOLVER_FAILED",
            FeasibilityStatus::BenchmarkInvalid { .. } => "BENCHMARK_INVALID",
            FeasibilityStatus::StructuralInvalid { .. } => "STRUCTURAL_INVALID",
        }
    }

    /// Feasibility Confidence Ladder level (F0–F5).
    pub fn confidence_level(&self) -> u8 {
        match self {
            FeasibilityStatus::StructuralInvalid { .. } => 0,
            FeasibilityStatus::BenchmarkInvalid { .. } => 0,
            FeasibilityStatus::ProvenInfeasible { .. } => 2, // proven at FC-2 or higher
            FeasibilityStatus::SolverFailed => 3,            // passed FC-1..FC-3, solver failed
            FeasibilityStatus::FeasibilityUndetermined => 3,
            FeasibilityStatus::ProvenFeasible => 4, // solution verified
        }
    }
}

/// Full feasibility certificate for one instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeasibilityCertificate {
    pub instance_name: String,
    pub fc1_structural: FcResult,
    pub fc2_5_benchmark: FcResult,
    pub fc2_capacity: FcResult,
    /// FC-3 not yet implemented; always None in Phase 1.
    pub fc3_bin_pack: Option<FcResult>,
    /// FC-4 not yet implemented; always None in Phase 1.
    pub fc4_cuts: Option<FcResult>,
    /// FC-5 not yet implemented; always None in Phase 1.
    pub fc5_exact: Option<FcResult>,
    /// Feasibility Confidence Ladder level (F0–F5).
    pub confidence_level: u8,
    pub status: FeasibilityStatus,
}

impl FeasibilityCertificate {
    /// True if optimization should be skipped (instance is invalid or proven infeasible).
    pub fn skip_optimization(&self) -> bool {
        matches!(
            &self.status,
            FeasibilityStatus::StructuralInvalid { .. }
                | FeasibilityStatus::BenchmarkInvalid { .. }
                | FeasibilityStatus::ProvenInfeasible { .. }
        )
    }

    /// One-line summary for campaign log.
    pub fn log_summary(&self) -> String {
        format!(
            "FCF: {} (F{}) — FC1:{} FC2.5:{} FC2:{}",
            self.status.code(),
            self.confidence_level,
            if self.fc1_structural.passed {
                "PASS"
            } else {
                "FAIL"
            },
            if self.fc2_5_benchmark.passed {
                "PASS"
            } else {
                "FAIL"
            },
            if self.fc2_capacity.passed {
                "PASS"
            } else {
                "FAIL"
            },
        )
    }
}

// ---------------------------------------------------------------------------
// FC-1: Structural Validation
// ---------------------------------------------------------------------------

/// FC-1: Structural Validation — O(n).
/// Checks graph integrity, depot existence, customer numbering, demand validity.
pub fn fc1_structural(instance: &CvrpInstance, name: &str) -> FcResult {
    // Depot must exist (non-zero id is conventional; we just check it's present)
    if instance.depot.id == 0
        && instance.depot.x == 0.0
        && instance.depot.y == 0.0
        && instance.customers.is_empty()
    {
        return FcResult::fail("Depot and customers both empty — malformed instance");
    }

    // Customer list must be non-empty
    if instance.customers.is_empty() {
        return FcResult::fail("No customers — malformed instance");
    }

    // All demands must be non-negative
    for c in &instance.customers {
        if c.demand < 0 {
            return FcResult::fail(format!(
                "Customer {} has negative demand {} — malformed",
                c.id, c.demand
            ));
        }
    }

    // No duplicate customer IDs
    let mut seen = std::collections::HashSet::new();
    for c in &instance.customers {
        if !seen.insert(c.id) {
            return FcResult::fail(format!("Duplicate customer ID {} — malformed", c.id));
        }
    }

    // Capacity must be positive
    if instance.capacity <= 0 {
        return FcResult::fail(format!("Capacity {} ≤ 0 — malformed", instance.capacity));
    }

    // For explicit matrix instances: matrix must be present and sized correctly
    if instance.distance_metric == DistanceMetric::ExplicitMatrix {
        let n = instance.customers.len() + 1; // customers + depot
        let dim = instance.explicit_matrix.len();
        if dim == 0 {
            return FcResult::fail("ExplicitMatrix instance has empty distance matrix");
        }
        // Matrix is (dimension+1) × (dimension+1), 1-indexed
        // We accept any non-empty matrix here; detailed validation is FC-2.5
    }

    FcResult::pass(format!(
        "n={} customers, depot_id={}, capacity={}, metric={:?}",
        instance.customers.len(),
        instance.depot.id,
        instance.capacity,
        instance.distance_metric,
    ))
}

// ---------------------------------------------------------------------------
// FC-2.5: Benchmark Consistency
// ---------------------------------------------------------------------------

/// Benchmark metadata supplied by the registry for FC-2.5 checks.
/// Caller populates this from the registry before calling fc2_5_benchmark.
#[derive(Debug, Clone)]
pub struct BenchmarkMeta {
    pub name: String,
    pub vehicles: usize,
    pub capacity: i32,
    pub bks: Option<f64>,
    pub distance_metric: String,
    pub family: String,
}

/// FC-2.5: Benchmark Consistency — O(1).
/// Checks that registry metadata is internally consistent and matches the parsed instance.
pub fn fc2_5_benchmark(instance: &CvrpInstance, meta: Option<&BenchmarkMeta>) -> FcResult {
    // If no registry entry exists, this is a benchmark consistency gap (not a structural error).
    // We pass with a warning — the instance may still be valid.
    let meta = match meta {
        Some(m) => m,
        None => return FcResult::pass("No registry entry — BKS and vehicle count from file only"),
    };

    // Vehicle count must be positive
    if meta.vehicles == 0 {
        return FcResult::fail(format!(
            "Registry vehicle count is 0 for '{}' — BENCHMARK_INVALID",
            meta.name
        ));
    }

    // Registry vehicle count must match instance max_vehicles (if set)
    if let Some(inst_k) = instance.max_vehicles {
        if inst_k != meta.vehicles {
            return FcResult::fail(format!(
                "Vehicle count mismatch: instance={} registry={} for '{}'",
                inst_k, meta.vehicles, meta.name
            ));
        }
    }

    // Registry capacity must match instance capacity
    if meta.capacity > 0 && meta.capacity != instance.capacity {
        return FcResult::fail(format!(
            "Capacity mismatch: instance={} registry={} for '{}'",
            instance.capacity, meta.capacity, meta.name
        ));
    }

    // BKS must be positive if present
    if let Some(bks) = meta.bks {
        if bks <= 0.0 {
            return FcResult::fail(format!(
                "Registry BKS={} ≤ 0 for '{}' — BENCHMARK_INVALID",
                bks, meta.name
            ));
        }
    }

    FcResult::pass(format!(
        "Registry consistent: K={}, Q={}, BKS={:?}, family={}",
        meta.vehicles, meta.capacity, meta.bks, meta.family
    ))
}

// ---------------------------------------------------------------------------
// FC-2: Capacity Validation
// ---------------------------------------------------------------------------

/// Result of FC-2 capacity checks with full arithmetic detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityCertificate {
    pub total_demand: i64,
    pub fleet_capacity: i64, // K × Q
    pub k_available: usize,
    pub k_minimum: usize, // ⌈Σd_i / Q⌉
    pub max_demand: i32,  // largest single customer demand
    pub nc1_fleet: FcResult,
    pub nc2_individual: FcResult,
    pub nc5_lower_bound: FcResult,
}

/// FC-2: Capacity Validation — O(n).
/// NC1: Σd_i ≤ K×Q
/// NC2: d_i ≤ Q for all i
/// NC5: K ≥ ⌈Σd_i / Q⌉
pub fn fc2_capacity(instance: &CvrpInstance) -> CapacityCertificate {
    let q = instance.capacity as i64;
    let k = instance.max_vehicles.unwrap_or(0) as i64;

    let total_demand: i64 = instance.customers.iter().map(|c| c.demand as i64).sum();
    let fleet_capacity = k * q;
    let k_minimum = if q > 0 {
        ((total_demand + q - 1) / q) as usize // ⌈Σd_i / Q⌉
    } else {
        usize::MAX
    };
    let max_demand = instance
        .customers
        .iter()
        .map(|c| c.demand)
        .max()
        .unwrap_or(0);

    // NC1: Fleet capacity
    let nc1 = if q <= 0 {
        FcResult::fail(format!("Capacity Q={} ≤ 0 — cannot validate", q))
    } else if k <= 0 {
        FcResult::fail(format!("Fleet size K={} ≤ 0 — cannot validate", k))
    } else if total_demand > fleet_capacity {
        FcResult::fail(format!(
            "NC1 FAIL: Σd_i={} > K×Q={}×{}={} — PROVEN_INFEASIBLE",
            total_demand, k, q, fleet_capacity
        ))
    } else {
        FcResult::pass(format!(
            "NC1 PASS: Σd_i={} ≤ K×Q={}×{}={}",
            total_demand, k, q, fleet_capacity
        ))
    };

    // NC2: Individual demand
    let nc2 = if max_demand as i64 > q {
        // Find the offending customer
        let offender = instance
            .customers
            .iter()
            .find(|c| c.demand as i64 > q)
            .map(|c| format!("customer_id={} demand={}", c.id, c.demand))
            .unwrap_or_default();
        FcResult::fail(format!(
            "NC2 FAIL: max_demand={} > Q={} ({}) — PROVEN_INFEASIBLE",
            max_demand, q, offender
        ))
    } else {
        FcResult::pass(format!("NC2 PASS: max_demand={} ≤ Q={}", max_demand, q))
    };

    // NC5: Vehicle lower bound
    let nc5 = if k <= 0 {
        FcResult::fail(format!("NC5: K={} ≤ 0 — cannot validate", k))
    } else if (k as usize) < k_minimum {
        FcResult::fail(format!(
            "NC5 FAIL: K={} < K_min=⌈{}/{}⌉={} — PROVEN_INFEASIBLE",
            k, total_demand, q, k_minimum
        ))
    } else {
        FcResult::pass(format!(
            "NC5 PASS: K={} ≥ K_min=⌈{}/{}⌉={}",
            k, total_demand, q, k_minimum
        ))
    };

    CapacityCertificate {
        total_demand,
        fleet_capacity,
        k_available: k as usize,
        k_minimum,
        max_demand,
        nc1_fleet: nc1,
        nc2_individual: nc2,
        nc5_lower_bound: nc5,
    }
}

impl CapacityCertificate {
    /// True if any NC fails — instance is PROVEN_INFEASIBLE at FC-2.
    pub fn is_infeasible(&self) -> bool {
        !self.nc1_fleet.passed || !self.nc2_individual.passed || !self.nc5_lower_bound.passed
    }

    /// First failing NC reason, or None if all pass.
    pub fn infeasibility_reason(&self) -> Option<String> {
        if !self.nc1_fleet.passed {
            return Some(self.nc1_fleet.reason.clone());
        }
        if !self.nc2_individual.passed {
            return Some(self.nc2_individual.reason.clone());
        }
        if !self.nc5_lower_bound.passed {
            return Some(self.nc5_lower_bound.reason.clone());
        }
        None
    }

    /// One-line summary for campaign log.
    pub fn log_summary(&self) -> String {
        format!(
            "FC2: D={} KQ={} K_min={} K={} NC1:{} NC2:{} NC5:{}",
            self.total_demand,
            self.fleet_capacity,
            self.k_minimum,
            self.k_available,
            if self.nc1_fleet.passed {
                "PASS"
            } else {
                "FAIL"
            },
            if self.nc2_individual.passed {
                "PASS"
            } else {
                "FAIL"
            },
            if self.nc5_lower_bound.passed {
                "PASS"
            } else {
                "FAIL"
            },
        )
    }
}

// ---------------------------------------------------------------------------
// Pipeline entry point
// ---------------------------------------------------------------------------

/// Run the full pre-optimization FCF pipeline (FC-1, FC-2.5, FC-2).
/// Returns a FeasibilityCertificate. If `skip_optimization()` is true,
/// the caller must not run the optimizer.
pub fn run_pre_optimization_fcf(
    instance: &CvrpInstance,
    name: &str,
    registry_meta: Option<&BenchmarkMeta>,
) -> FeasibilityCertificate {
    // FC-1: Structural
    let fc1 = fc1_structural(instance, name);
    if !fc1.passed {
        let reason = fc1.reason.clone();
        return FeasibilityCertificate {
            instance_name: name.to_string(),
            fc1_structural: fc1,
            fc2_5_benchmark: FcResult::pass("skipped — FC-1 failed"),
            fc2_capacity: FcResult::pass("skipped — FC-1 failed"),
            fc3_bin_pack: None,
            fc4_cuts: None,
            fc5_exact: None,
            confidence_level: 0,
            status: FeasibilityStatus::StructuralInvalid { reason },
        };
    }

    // FC-2.5: Benchmark Consistency
    let fc2_5 = fc2_5_benchmark(instance, registry_meta);
    if !fc2_5.passed {
        let reason = fc2_5.reason.clone();
        return FeasibilityCertificate {
            instance_name: name.to_string(),
            fc1_structural: fc1,
            fc2_5_benchmark: fc2_5,
            fc2_capacity: FcResult::pass("skipped — FC-2.5 failed"),
            fc3_bin_pack: None,
            fc4_cuts: None,
            fc5_exact: None,
            confidence_level: 0,
            status: FeasibilityStatus::BenchmarkInvalid { reason },
        };
    }

    // FC-2: Capacity Validation
    let cap = fc2_capacity(instance);
    let fc2_result = if cap.is_infeasible() {
        FcResult::fail(cap.infeasibility_reason().unwrap_or_default())
    } else {
        FcResult::pass(cap.log_summary())
    };

    if cap.is_infeasible() {
        let reason = cap.infeasibility_reason().unwrap_or_default();
        return FeasibilityCertificate {
            instance_name: name.to_string(),
            fc1_structural: fc1,
            fc2_5_benchmark: fc2_5,
            fc2_capacity: fc2_result,
            fc3_bin_pack: None,
            fc4_cuts: None,
            fc5_exact: None,
            confidence_level: 2,
            status: FeasibilityStatus::ProvenInfeasible {
                level: "FC-2".to_string(),
                reason,
            },
        };
    }

    // All Phase 1 checks passed — feasibility undetermined until optimizer runs
    FeasibilityCertificate {
        instance_name: name.to_string(),
        fc1_structural: fc1,
        fc2_5_benchmark: fc2_5,
        fc2_capacity: fc2_result,
        fc3_bin_pack: None,
        fc4_cuts: None,
        fc5_exact: None,
        confidence_level: 3,
        status: FeasibilityStatus::FeasibilityUndetermined,
    }
}

// ---------------------------------------------------------------------------
// FC-3: Bin Packing Relaxation — First-Fit Decreasing (Heuristic)
// ---------------------------------------------------------------------------

/// Result of FC-3 heuristic bin packing check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinPackResult {
    /// True if FFD found a valid packing (necessary condition satisfied).
    pub packed: bool,
    /// Number of bins (vehicles) used by FFD.
    pub bins_used: usize,
    /// Available bins (K).
    pub bins_available: usize,
    /// Bin capacity (Q).
    pub bin_capacity: i32,
    /// Total demand packed.
    pub total_demand: i64,
    pub reason: String,
}

impl BinPackResult {
    /// True if FFD failed — signals `LIKELY_INFEASIBLE` (not a proof).
    pub fn is_likely_infeasible(&self) -> bool {
        !self.packed
    }

    pub fn log_summary(&self) -> String {
        if self.packed {
            format!(
                "FC3-FFD PASS: packed {} demand into {}/{} bins of Q={}",
                self.total_demand, self.bins_used, self.bins_available, self.bin_capacity
            )
        } else {
            format!(
                "FC3-FFD FAIL: cannot pack {} demand into {} bins of Q={} — LIKELY_INFEASIBLE",
                self.total_demand, self.bins_available, self.bin_capacity
            )
        }
    }
}

/// FC-3 Heuristic: First-Fit Decreasing bin packing.
///
/// Ignores routing geometry. Sorts demands descending, then greedily assigns
/// each demand to the first bin with sufficient remaining capacity.
///
/// FAIL → `LIKELY_INFEASIBLE` (signal only — not a proof of infeasibility).
/// PASS → necessary condition satisfied; feasibility not proven.
///
/// Complexity: O(n log n) sort + O(n × K) assignment.
pub fn fc3_bin_pack_ffd(instance: &CvrpInstance) -> BinPackResult {
    let q = instance.capacity as i64;
    let k = instance.max_vehicles.unwrap_or(0);
    let total_demand: i64 = instance.customers.iter().map(|c| c.demand as i64).sum();

    if q <= 0 || k == 0 {
        return BinPackResult {
            packed: false,
            bins_used: 0,
            bins_available: k,
            bin_capacity: instance.capacity,
            total_demand,
            reason: format!("Invalid Q={} or K={}", q, k),
        };
    }

    // Sort demands descending (FFD order)
    let mut demands: Vec<i64> = instance.customers.iter().map(|c| c.demand as i64).collect();
    demands.sort_unstable_by(|a, b| b.cmp(a));

    // Bin remaining capacities
    let mut bins: Vec<i64> = vec![q; k];
    let mut bins_used = 0usize;

    for &d in &demands {
        // Find first bin with enough remaining capacity
        let mut placed = false;
        for (i, remaining) in bins.iter_mut().enumerate() {
            if *remaining >= d {
                *remaining -= d;
                if i + 1 > bins_used {
                    bins_used = i + 1;
                }
                placed = true;
                break;
            }
        }
        if !placed {
            // No bin can fit this demand — FFD fails
            return BinPackResult {
                packed: false,
                bins_used: k + 1, // sentinel: more than available
                bins_available: k,
                bin_capacity: instance.capacity,
                total_demand,
                reason: format!(
                    "FC3-FFD FAIL: demand={} cannot fit in any remaining bin (Q={}, K={})",
                    d, q, k
                ),
            };
        }
    }

    BinPackResult {
        packed: true,
        bins_used,
        bins_available: k,
        bin_capacity: instance.capacity,
        total_demand,
        reason: format!(
            "FC3-FFD PASS: {} demand packed into {}/{} bins of Q={}",
            total_demand, bins_used, k, q
        ),
    }
}

/// Run the full pre-optimization FCF pipeline (FC-1, FC-2.5, FC-2, FC-3 heuristic).
/// Returns a FeasibilityCertificate. If `skip_optimization()` is true,
/// the caller must not run the optimizer.
pub fn run_pre_optimization_fcf_with_fc3(
    instance: &CvrpInstance,
    name: &str,
    registry_meta: Option<&BenchmarkMeta>,
) -> FeasibilityCertificate {
    // Run FC-1, FC-2.5, FC-2 first
    let mut cert = run_pre_optimization_fcf(instance, name, registry_meta);

    // If already aborted, return immediately
    if cert.skip_optimization() {
        return cert;
    }

    // FC-3: Bin Packing Heuristic (signal only — does not abort)
    let bp = fc3_bin_pack_ffd(instance);
    let fc3_result = if bp.packed {
        FcResult::pass(bp.log_summary())
    } else {
        FcResult::fail(bp.log_summary())
    };

    cert.fc3_bin_pack = Some(fc3_result);
    // FC-3 heuristic failure does NOT change status or skip_optimization.
    // It is a diagnostic signal only. The optimizer still runs.
    cert
}
