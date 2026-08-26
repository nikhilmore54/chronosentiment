/// Fleet Constraint Semantics — FCS
/// GOV-009 / GOV-008 — Benchmark Qualification
///
/// Fleet cardinality is part of the optimization problem definition.
/// If the benchmark and the optimizer use different fleet semantics,
/// the objective functions are different and the gap is not comparable.
///
/// Reference: benchmarks/campaign/QDR-FINAL.md §6.2
///            benchmarks/campaign/qualification_decision_register.md §Category A
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Fleet Constraint Semantics
// ---------------------------------------------------------------------------

/// Declares the fleet cardinality constraint for a benchmark instance.
///
/// This is part of the optimization problem definition — not a solver parameter.
/// Two solvers producing the same objective value under different fleet semantics
/// are solving different problems and their results are not directly comparable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FleetConstraint {
    /// Exactly K vehicles must be used. |R| = K.
    /// Empty routes are not permitted.
    Exact(usize),

    /// Up to K vehicles may be used. |R| ≤ K.
    /// Unused vehicles are permitted. Fleet minimization is valid.
    AtMost(usize),

    /// At least K vehicles must be used. |R| ≥ K.
    /// Rare in standard benchmarks.
    AtLeast(usize),

    /// Fleet size is a decision variable. K is optimized.
    /// No upper bound on vehicle count (beyond feasibility).
    Variable,

    /// Benchmark does not state fleet semantics.
    /// Requires qualification before comparison is valid.
    Unspecified,
}

impl FleetConstraint {
    /// Returns the declared vehicle count K, if applicable.
    pub fn declared_k(&self) -> Option<usize> {
        match self {
            FleetConstraint::Exact(k)
            | FleetConstraint::AtMost(k)
            | FleetConstraint::AtLeast(k) => Some(*k),
            FleetConstraint::Variable | FleetConstraint::Unspecified => None,
        }
    }

    /// Returns true if using `routes_used` vehicles satisfies this constraint.
    pub fn is_satisfied(&self, routes_used: usize) -> bool {
        match self {
            FleetConstraint::Exact(k) => routes_used == *k,
            FleetConstraint::AtMost(k) => routes_used <= *k,
            FleetConstraint::AtLeast(k) => routes_used >= *k,
            FleetConstraint::Variable => true,
            FleetConstraint::Unspecified => true, // cannot determine
        }
    }

    /// Returns true if the comparison is valid given `routes_used`.
    /// A comparison is valid only when fleet semantics are known and satisfied.
    pub fn comparison_valid(&self, routes_used: usize) -> bool {
        match self {
            FleetConstraint::Unspecified => false, // unknown semantics → not comparable
            other => other.is_satisfied(routes_used),
        }
    }

    /// Short code for logging.
    pub fn code(&self) -> &'static str {
        match self {
            FleetConstraint::Exact(_) => "EXACT",
            FleetConstraint::AtMost(_) => "ATMOST",
            FleetConstraint::AtLeast(_) => "ATLEAST",
            FleetConstraint::Variable => "VARIABLE",
            FleetConstraint::Unspecified => "UNSPECIFIED",
        }
    }

    /// Human-readable description.
    pub fn description(&self) -> String {
        match self {
            FleetConstraint::Exact(k) => format!("EXACT({}) — exactly {} vehicles required", k, k),
            FleetConstraint::AtMost(k) => format!("ATMOST({}) — up to {} vehicles permitted", k, k),
            FleetConstraint::AtLeast(k) => {
                format!("ATLEAST({}) — at least {} vehicles required", k, k)
            }
            FleetConstraint::Variable => "VARIABLE — fleet size is a decision variable".to_string(),
            FleetConstraint::Unspecified => {
                "UNSPECIFIED — fleet semantics not declared".to_string()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Fleet Semantics Verification Result
// ---------------------------------------------------------------------------

/// Result of checking fleet constraint against actual routes used.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetSemanticCheck {
    /// Declared fleet constraint for this benchmark.
    pub constraint: FleetConstraint,
    /// Routes actually used in the best solution.
    pub routes_used: usize,
    /// Whether the constraint is satisfied.
    pub satisfied: bool,
    /// Whether the gap comparison is valid.
    pub comparison_valid: bool,
    /// Qualification outcome for this instance.
    pub outcome: FleetSemanticOutcome,
}

/// Qualification outcome from fleet semantics check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FleetSemanticOutcome {
    /// Fleet constraint satisfied; comparison is valid.
    Valid,
    /// Fleet constraint violated; comparison is not valid.
    NotComparable,
    /// Fleet semantics unspecified; comparison cannot be determined.
    PendingVerification,
}

impl FleetSemanticCheck {
    pub fn evaluate(constraint: FleetConstraint, routes_used: usize) -> Self {
        let satisfied = constraint.is_satisfied(routes_used);
        let comparison_valid = constraint.comparison_valid(routes_used);
        let outcome = match &constraint {
            FleetConstraint::Unspecified => FleetSemanticOutcome::PendingVerification,
            _ if satisfied => FleetSemanticOutcome::Valid,
            _ => FleetSemanticOutcome::NotComparable,
        };
        FleetSemanticCheck {
            constraint,
            routes_used,
            satisfied,
            comparison_valid,
            outcome,
        }
    }

    /// One-line log entry.
    pub fn log_line(&self) -> String {
        let outcome_str = match &self.outcome {
            FleetSemanticOutcome::Valid => "VALID",
            FleetSemanticOutcome::NotComparable => "NOT_COMPARABLE",
            FleetSemanticOutcome::PendingVerification => "PENDING_VERIFICATION",
        };
        format!(
            "FCS: {} routes_used={} satisfied={} comparison={} outcome={}",
            self.constraint.description(),
            self.routes_used,
            self.satisfied,
            self.comparison_valid,
            outcome_str,
        )
    }
}

// ---------------------------------------------------------------------------
// Benchmark Family Fleet Semantics Registry
// ---------------------------------------------------------------------------

/// Known fleet semantics per benchmark family.
///
/// Evidence levels follow GOV-008:
///   Verified   — confirmed from benchmark specification document
///   Observed   — inferred from campaign evidence
///   Hypothesis — not yet confirmed
///
/// Sources:
///   Augerat A/B/E/P: Augerat et al. 1995 — "at most K" semantics
///     (instances named A-nN-kK where K is the minimum fleet size)
///   CMT: Christofides et al. 1979 — "exact K" semantics (original paper)
///   Tai: Taillard 1993 — "at most K" semantics (paper uses minimum fleet)
///   M: Christofides & Eilon 1969 / Augerat 1995 — "at most K" semantics
///   X: Uchoa et al. 2017 — "exact K" semantics (instances named X-nN-kK)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyFleetSemantics {
    pub family: String,
    pub constraint_type: FamilyFleetConstraintType,
    pub evidence_level: FamilyFleetEvidenceLevel,
    pub source: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FamilyFleetConstraintType {
    /// All instances in this family use EXACT(K) semantics.
    AllExact,
    /// All instances in this family use ATMOST(K) semantics.
    AllAtMost,
    /// Fleet semantics vary by instance within this family.
    Mixed,
    /// Fleet semantics not yet determined for this family.
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FamilyFleetEvidenceLevel {
    Verified,
    Observed,
    Hypothesis,
}

/// Returns the known fleet semantics for each benchmark family.
/// Evidence levels are conservative — only Verified where confirmed from spec.
pub fn family_fleet_semantics_registry() -> Vec<FamilyFleetSemantics> {
    vec![
        FamilyFleetSemantics {
            family: "A".to_string(),
            constraint_type: FamilyFleetConstraintType::AllAtMost,
            evidence_level: FamilyFleetEvidenceLevel::Hypothesis,
            source: "Augerat et al. 1995 — instance names encode minimum K".to_string(),
            notes: "Instances named A-nN-kK. K is the minimum number of vehicles needed. \
                    Standard interpretation is ATMOST(K) — optimizer may use fewer. \
                    Pending verification against benchmark specification document.".to_string(),
        },
        FamilyFleetSemantics {
            family: "B".to_string(),
            constraint_type: FamilyFleetConstraintType::AllAtMost,
            evidence_level: FamilyFleetEvidenceLevel::Hypothesis,
            source: "Augerat et al. 1995 — instance names encode minimum K".to_string(),
            notes: "Same naming convention as A-family. ATMOST(K) hypothesis. \
                    Pending verification.".to_string(),
        },
        FamilyFleetSemantics {
            family: "E".to_string(),
            constraint_type: FamilyFleetConstraintType::AllAtMost,
            evidence_level: FamilyFleetEvidenceLevel::Hypothesis,
            source: "Augerat et al. 1995 — instance names encode minimum K".to_string(),
            notes: "Same naming convention as A-family. ATMOST(K) hypothesis. \
                    Pending verification.".to_string(),
        },
        FamilyFleetSemantics {
            family: "P".to_string(),
            constraint_type: FamilyFleetConstraintType::AllAtMost,
            evidence_level: FamilyFleetEvidenceLevel::Hypothesis,
            source: "Augerat et al. 1995 — instance names encode minimum K".to_string(),
            notes: "Same naming convention as A-family. ATMOST(K) hypothesis. \
                    P-n55-k8 uses 7/8 routes — consistent with ATMOST semantics. \
                    Pending verification.".to_string(),
        },
        FamilyFleetSemantics {
            family: "M".to_string(),
            constraint_type: FamilyFleetConstraintType::AllAtMost,
            evidence_level: FamilyFleetEvidenceLevel::Hypothesis,
            source: "Christofides & Eilon 1969 / Augerat 1995".to_string(),
            notes: "M-family instances use ATMOST(K) hypothesis. \
                    Both M-n151-k12 and M-n200-k17 use exact K routes — consistent with either semantics. \
                    Pending verification.".to_string(),
        },
        FamilyFleetSemantics {
            family: "CMT".to_string(),
            constraint_type: FamilyFleetConstraintType::Unknown,
            evidence_level: FamilyFleetEvidenceLevel::Hypothesis,
            source: "Christofides, Mingozzi & Toth 1979".to_string(),
            notes: "CMT instances show fleet semantics difference in 7 of 14 instances. \
                    Original paper may use EXACT(K) semantics. \
                    7 instances where Coralys uses fewer routes than K are classified Not Comparable. \
                    Pending verification against original paper and CVRPLIB.org specification.".to_string(),
        },
        FamilyFleetSemantics {
            family: "Tai".to_string(),
            constraint_type: FamilyFleetConstraintType::AllAtMost,
            evidence_level: FamilyFleetEvidenceLevel::Hypothesis,
            source: "Taillard 1993".to_string(),
            notes: "All 5 negative-gap Tai instances use exact K routes — consistent with ATMOST semantics. \
                    No fleet semantics difference observed in Tai family. \
                    Pending verification against original paper.".to_string(),
        },
        FamilyFleetSemantics {
            family: "X".to_string(),
            constraint_type: FamilyFleetConstraintType::AllExact,
            evidence_level: FamilyFleetEvidenceLevel::Hypothesis,
            source: "Uchoa et al. 2017 — instance names encode exact K".to_string(),
            notes: "X-family instances named X-nN-kK. K is the exact vehicle count. \
                    EXACT(K) hypothesis. All X instances are INFEASIBLE in current campaign \
                    (no BKS in registry). Capability Boundary classification.".to_string(),
        },
    ]
}

/// Look up fleet semantics for a given family code.
pub fn get_family_fleet_semantics(family: &str) -> Option<FamilyFleetSemantics> {
    family_fleet_semantics_registry()
        .into_iter()
        .find(|f| f.family == family)
}

/// Derive a FleetConstraint for a specific instance given family and K.
/// Returns Unspecified if family semantics are Unknown.
pub fn derive_fleet_constraint(family: &str, k: usize) -> FleetConstraint {
    match get_family_fleet_semantics(family) {
        Some(fam) => match fam.constraint_type {
            FamilyFleetConstraintType::AllExact => FleetConstraint::Exact(k),
            FamilyFleetConstraintType::AllAtMost => FleetConstraint::AtMost(k),
            FamilyFleetConstraintType::Mixed | FamilyFleetConstraintType::Unknown => {
                FleetConstraint::Unspecified
            }
        },
        None => FleetConstraint::Unspecified,
    }
}
