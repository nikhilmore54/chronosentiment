pub mod execution_certificate;
/// Coralys Qualification Subsystem — CVRP adapter
///
/// Implements the Feasibility Certification Framework (GOV-009) and
/// Benchmark Qualification (GOV-008) as a reusable platform capability.
///
/// Module structure:
///   feasibility            — FC-1, FC-2.5, FC-2, FC-3 heuristic; FC-4, FC-5 (future)
///   fleet_utilization      — FUC-001 Fleet Utilization Certificate (post-optimization)
///   fleet_semantics        — FCS Fleet Constraint Semantics check
///   execution_certificate  — M18.3 per-instance portable evidence artifact
///
/// Usage in campaign.rs:
///   use cvrp::qualification::feasibility::{run_pre_optimization_fcf_with_fc3, BenchmarkMeta};
///   use cvrp::qualification::fleet_utilization::FleetUtilizationCertificate;
///   use cvrp::qualification::execution_certificate::{ExecutionCertificate, CertificateInput};
pub mod feasibility;
pub mod fleet_semantics;
pub mod fleet_utilization;

pub use feasibility::{
    BenchmarkMeta, BinPackResult, CapacityCertificate, FcResult, FeasibilityCertificate,
    FeasibilityStatus, fc1_structural, fc2_5_benchmark, fc2_capacity, fc3_bin_pack_ffd,
    run_pre_optimization_fcf, run_pre_optimization_fcf_with_fc3,
};

pub use fleet_utilization::{FleetUtilizationCertificate, PackingClassification, RouteLoad};

pub use fleet_semantics::{
    FamilyFleetConstraintType, FamilyFleetEvidenceLevel, FamilyFleetSemantics, FleetConstraint,
    FleetSemanticCheck, FleetSemanticOutcome, derive_fleet_constraint,
    family_fleet_semantics_registry, get_family_fleet_semantics,
};
