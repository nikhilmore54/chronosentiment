/// Execution Certificate — M18.3
///
/// The canonical per-instance evidence artifact produced by every Coralys optimization run.
///
/// An Execution Certificate bundles:
///   - Identity: who ran what, when, with which version
///   - Instance: benchmark metadata
///   - Objective: distance, gap, quality class
///   - Qualification: FCF, FCS, FUC-001 outcomes
///   - Benchmark Context: provenance and fleet semantics registry versions
///   - Evidence: campaign reference, termination reason, configuration hash
///   - Governance: capability versions, overall status, content hash
///
/// The certificate is serializable to JSON (via serde) and can be rendered as
/// a human-readable TOML-style text block via `render_text()`.
///
/// The `content_hash` field is computed over all other fields using FNV-1a 64-bit.
/// It is appended last and covers the canonical text representation of all preceding fields.
///
/// Design principles:
///   - Domain-independent: the structure is valid for any optimization domain
///   - Immutable: once generated, the certificate is not modified
///   - Portable: can be archived, compared, signed, or exchanged independently of the campaign log
///   - Self-describing: every field carries enough context to be interpreted without the campaign log

use serde::{Deserialize, Serialize};
use chrono::Utc;

use crate::qualification::feasibility::{FeasibilityCertificate, FeasibilityStatus};
use crate::qualification::fleet_utilization::FleetUtilizationCertificate;
use crate::qualification::fleet_semantics::{FleetSemanticCheck, FleetSemanticOutcome};

// =============================================================================
// CERTIFICATE VERSION
// =============================================================================

/// Semantic version of the Execution Certificate schema.
/// Increment MINOR when new optional fields are added.
/// Increment MAJOR when the schema changes in a breaking way.
pub const CERTIFICATE_VERSION: &str = "1.0.0";

/// Provenance Registry version referenced by this certificate.
pub const PROVENANCE_REGISTRY_VERSION: &str = "1.0";

/// Fleet Semantics Registry version referenced by this certificate.
pub const FLEET_SEMANTICS_REGISTRY_VERSION: &str = "1.0";

/// Qualification subsystem version (FCF + FCS + FUC-001).
pub const QUALIFICATION_VERSION: &str = "1.0";

// =============================================================================
// CERTIFICATE STATUS
// =============================================================================

/// Overall qualification status of the certificate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificateStatus {
    /// All qualification gates passed; gap is a valid comparison.
    Qualified,
    /// FCF passed; gap comparison validity pending (Stage B not yet complete).
    ProvisionallyQualified,
    /// FCF failed — instance is infeasible or structurally invalid.
    Infeasible,
    /// Optimization was skipped (e.g. >200 customers, unsupported feature).
    Skipped,
    /// Gap comparison is not valid (fleet semantics mismatch, BKS provenance issue).
    NotComparable,
    /// Under investigation — negative gap or other anomaly observed.
    UnderInvestigation,
    /// No BKS available — gap cannot be computed.
    NoReference,
}

impl CertificateStatus {
    pub fn code(&self) -> &'static str {
        match self {
            CertificateStatus::Qualified              => "QUALIFIED",
            CertificateStatus::ProvisionallyQualified => "PROVISIONALLY_QUALIFIED",
            CertificateStatus::Infeasible             => "INFEASIBLE",
            CertificateStatus::Skipped                => "SKIPPED",
            CertificateStatus::NotComparable          => "NOT_COMPARABLE",
            CertificateStatus::UnderInvestigation     => "UNDER_INVESTIGATION",
            CertificateStatus::NoReference            => "NO_REFERENCE",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            CertificateStatus::Qualified              => "✓",
            CertificateStatus::ProvisionallyQualified => "~",
            CertificateStatus::Infeasible             => "✗",
            CertificateStatus::Skipped                => "—",
            CertificateStatus::NotComparable          => "⚠",
            CertificateStatus::UnderInvestigation     => "?",
            CertificateStatus::NoReference            => "·",
        }
    }
}

impl std::fmt::Display for CertificateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.symbol(), self.code())
    }
}

// =============================================================================
// CERTIFICATE SECTIONS
// =============================================================================

/// Identity section — who ran what, when.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateIdentity {
    /// Unique certificate ID (UUID v4).
    pub certificate_id: String,
    /// Certificate schema version.
    pub certificate_version: String,
    /// ISO 8601 timestamp of certificate generation.
    pub generated_at: String,
    /// Campaign identifier (e.g. "campaign_v1.5").
    pub campaign_id: String,
    /// Coralys solver version string.
    pub solver_version: String,
}

/// Instance section — benchmark metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInstance {
    /// Instance name (e.g. "A-n32-k5").
    pub name: String,
    /// Benchmark family (e.g. "A", "CMT", "Tai").
    pub family: String,
    /// Number of customers (excluding depot).
    pub customers: usize,
    /// Vehicle capacity.
    pub capacity: i32,
    /// Benchmark vehicle count K.
    pub benchmark_vehicles: usize,
    /// How the vehicle count was resolved (COMMENT, NAME, REGISTRY, etc.).
    pub vehicle_source: String,
    /// Distance metric used (e.g. "TspLibEuc2D").
    pub distance_metric: String,
}

/// Objective section — solution quality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateObjective {
    /// Best distance found (integer-rounded, matching BKS comparison convention).
    pub best_distance: f64,
    /// Best distance (float, for transparency when different from integer).
    pub best_distance_float: Option<f64>,
    /// Published BKS value (None if no BKS available).
    pub bks: Option<f64>,
    /// Gap percentage: (best_distance - bks) / bks * 100. None if no BKS.
    pub gap_pct: Option<f64>,
    /// Gap using float distance (for fractional BKS instances). None if not applicable.
    pub gap_fp_pct: Option<f64>,
    /// Quality classification (Solved / NearOptimal / Competitive / Weak / Poor / NoRef).
    pub quality_class: String,
    /// Number of routes used in the best solution.
    pub routes_used: usize,
    /// Runtime in milliseconds.
    pub runtime_ms: u128,
    /// Number of generations run.
    pub generations: usize,
    /// Termination reason (e.g. "NoImprovement(30)", "GenerationLimit").
    pub termination_reason: String,
}

/// FCF qualification outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateFcf {
    /// Overall FCF status code (PROVEN_FEASIBLE, PROVEN_INFEASIBLE, etc.).
    pub status: String,
    /// Confidence level (0–4).
    pub confidence_level: u8,
    /// FC-1 structural check: passed/failed + reason.
    pub fc1_passed: bool,
    pub fc1_reason: String,
    /// FC-2.5 benchmark consistency check: passed/failed + reason.
    pub fc2_5_passed: bool,
    pub fc2_5_reason: String,
    /// FC-2 capacity feasibility: passed/failed + reason.
    pub fc2_passed: bool,
    pub fc2_reason: String,
    /// FC-3 bin-pack FFD lower bound: passed/failed + reason (non-blocking).
    pub fc3_passed: bool,
    pub fc3_reason: String,
}

/// FCS qualification outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateFcs {
    /// Fleet constraint type (ATMOST, EXACT, UNSPECIFIED, etc.).
    pub constraint_type: String,
    /// Declared K value.
    pub declared_k: Option<usize>,
    /// Routes actually used.
    pub routes_used: usize,
    /// FCS outcome (Valid, Invalid, PendingVerification).
    pub outcome: String,
    /// Whether the gap comparison is valid under current fleet semantics.
    pub comparison_valid: bool,
}

/// FUC-001 qualification outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateFuc {
    /// Average fleet utilization (0.0–1.0).
    pub avg_utilization: f64,
    /// Median fleet utilization (0.0–1.0).
    pub median_utilization: f64,
    /// Coefficient of variation of route loads.
    pub cv: f64,
    /// Residual Concentration Ratio (largest_slack / total_residual_capacity).
    pub rcr: f64,
    /// Packing classification label.
    pub packing_classification: String,
    /// Number of capacity violations (should be 0 for valid solutions).
    pub capacity_violations: usize,
    /// Total demand served.
    pub total_demand: i32,
    /// Total fleet capacity (routes_used * capacity).
    pub total_fleet_capacity: i64,
    /// Fleet capacity utilization (total_demand / total_fleet_capacity).
    pub fleet_capacity_used: f64,
}

/// Benchmark context section — registry versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateBenchmarkContext {
    /// Provenance Registry version.
    pub provenance_registry_version: String,
    /// Fleet Semantics Registry version.
    pub fleet_semantics_registry_version: String,
    /// Qualification subsystem version.
    pub qualification_version: String,
    /// BKS source description (e.g. "CVRPLIB catalog (current)").
    pub bks_source: String,
    /// Fleet semantics evidence level (Hypothesis, Verified, etc.).
    pub fleet_semantics_evidence: String,
}

/// Governance section — capability versions and certificate integrity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateGovernance {
    /// FCF capability version.
    pub fcf_version: String,
    /// FCS capability version.
    pub fcs_version: String,
    /// FUC-001 capability version.
    pub fuc_version: String,
    /// Overall certificate status.
    pub status: CertificateStatus,
    /// Status reason (human-readable explanation of the status).
    pub status_reason: String,
    /// FNV-1a 64-bit content hash over all preceding fields.
    /// Computed from the canonical text representation of the certificate body.
    pub content_hash: String,
}

// =============================================================================
// EXECUTION CERTIFICATE
// =============================================================================

/// The canonical per-instance evidence artifact.
///
/// Generated by `ExecutionCertificate::generate()` after every optimization run.
/// Serializable to JSON. Renderable as human-readable text via `render_text()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionCertificate {
    pub identity: CertificateIdentity,
    pub instance: CertificateInstance,
    pub objective: CertificateObjective,
    pub fcf: CertificateFcf,
    pub fcs: CertificateFcs,
    pub fuc: Option<CertificateFuc>,
    pub benchmark_context: CertificateBenchmarkContext,
    pub governance: CertificateGovernance,
}

// =============================================================================
// BUILDER INPUT
// =============================================================================

/// All inputs required to generate an Execution Certificate.
/// Collected by the campaign runner after optimization completes.
pub struct CertificateInput<'a> {
    pub campaign_id: &'a str,
    pub solver_version: &'a str,
    pub instance_name: &'a str,
    pub family: &'a str,
    pub customers: usize,
    pub capacity: i32,
    pub benchmark_vehicles: usize,
    pub vehicle_source: &'a str,
    pub distance_metric: &'a str,
    pub best_distance_integer: f64,
    pub best_distance_float: f64,
    pub bks: Option<f64>,
    pub gap_pct: Option<f64>,
    pub gap_fp_pct: Option<f64>,
    pub quality_class: &'a str,
    pub routes_used: usize,
    pub runtime_ms: u128,
    pub generations: usize,
    pub termination_reason: &'a str,
    pub fcf: &'a FeasibilityCertificate,
    pub fcs: &'a FleetSemanticCheck,
    pub fuc: Option<&'a FleetUtilizationCertificate>,
    pub bks_source: &'a str,
    pub fleet_semantics_evidence: &'a str,
}

// =============================================================================
// GENERATION
// =============================================================================

impl ExecutionCertificate {
    /// Generate an Execution Certificate from all qualification outputs.
    pub fn generate(input: &CertificateInput<'_>) -> Self {
        let now = Utc::now().to_rfc3339();
        let cert_id = generate_cert_id(&input.instance_name, &now);

        // ── Identity ──────────────────────────────────────────────────────────
        let identity = CertificateIdentity {
            certificate_id: cert_id,
            certificate_version: CERTIFICATE_VERSION.to_string(),
            generated_at: now,
            campaign_id: input.campaign_id.to_string(),
            solver_version: input.solver_version.to_string(),
        };

        // ── Instance ──────────────────────────────────────────────────────────
        let instance = CertificateInstance {
            name: input.instance_name.to_string(),
            family: input.family.to_string(),
            customers: input.customers,
            capacity: input.capacity,
            benchmark_vehicles: input.benchmark_vehicles,
            vehicle_source: input.vehicle_source.to_string(),
            distance_metric: input.distance_metric.to_string(),
        };

        // ── Objective ─────────────────────────────────────────────────────────
        let best_distance_float = if (input.best_distance_float - input.best_distance_integer).abs() > 0.01 {
            Some(input.best_distance_float)
        } else {
            None
        };

        let objective = CertificateObjective {
            best_distance: input.best_distance_integer,
            best_distance_float,
            bks: input.bks,
            gap_pct: input.gap_pct,
            gap_fp_pct: input.gap_fp_pct,
            quality_class: input.quality_class.to_string(),
            routes_used: input.routes_used,
            runtime_ms: input.runtime_ms,
            generations: input.generations,
            termination_reason: input.termination_reason.to_string(),
        };

        // ── FCF ───────────────────────────────────────────────────────────────
        let fcf = build_fcf_section(input.fcf);

        // ── FCS ───────────────────────────────────────────────────────────────
        let fcs = build_fcs_section(input.fcs);

        // ── FUC ───────────────────────────────────────────────────────────────
        let fuc = input.fuc.map(build_fuc_section);

        // ── Benchmark Context ─────────────────────────────────────────────────
        let benchmark_context = CertificateBenchmarkContext {
            provenance_registry_version: PROVENANCE_REGISTRY_VERSION.to_string(),
            fleet_semantics_registry_version: FLEET_SEMANTICS_REGISTRY_VERSION.to_string(),
            qualification_version: QUALIFICATION_VERSION.to_string(),
            bks_source: input.bks_source.to_string(),
            fleet_semantics_evidence: input.fleet_semantics_evidence.to_string(),
        };

        // ── Governance ────────────────────────────────────────────────────────
        let (status, status_reason) = derive_status(input, &fcs);

        // Build governance without hash first, then compute hash over body
        let body_text = build_body_text(&identity, &instance, &objective, &fcf, &fcs, &fuc, &benchmark_context, &status, &status_reason);
        let content_hash = fnv1a_64_hex(&body_text);

        let governance = CertificateGovernance {
            fcf_version: "1.0".to_string(),
            fcs_version: "1.0".to_string(),
            fuc_version: "1.0".to_string(),
            status,
            status_reason,
            content_hash,
        };

        ExecutionCertificate { identity, instance, objective, fcf, fcs, fuc, benchmark_context, governance }
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
    }

    /// Render as a human-readable text certificate (TOML-style, for logs and reports).
    pub fn render_text(&self) -> String {
        let mut out = String::new();

        out.push_str("╔══════════════════════════════════════════════════════════════╗\n");
        out.push_str(&format!("║  EXECUTION CERTIFICATE  {}  ║\n",
            pad_or_truncate(&self.instance.name, 32)));
        out.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");

        // Identity
        out.push_str("── Identity ─────────────────────────────────────────────────────\n");
        out.push_str(&format!("  Certificate ID   {}\n", self.identity.certificate_id));
        out.push_str(&format!("  Version          {}\n", self.identity.certificate_version));
        out.push_str(&format!("  Generated        {}\n", self.identity.generated_at));
        out.push_str(&format!("  Campaign         {}\n", self.identity.campaign_id));
        out.push_str(&format!("  Solver           {}\n\n", self.identity.solver_version));

        // Instance
        out.push_str("── Instance ─────────────────────────────────────────────────────\n");
        out.push_str(&format!("  Name             {}\n", self.instance.name));
        out.push_str(&format!("  Family           {}\n", self.instance.family));
        out.push_str(&format!("  Customers        {}\n", self.instance.customers));
        out.push_str(&format!("  Capacity         {}\n", self.instance.capacity));
        out.push_str(&format!("  Vehicles (K)     {}\n", self.instance.benchmark_vehicles));
        out.push_str(&format!("  Vehicle source   {}\n", self.instance.vehicle_source));
        out.push_str(&format!("  Distance metric  {}\n\n", self.instance.distance_metric));

        // Objective
        out.push_str("── Objective ────────────────────────────────────────────────────\n");
        out.push_str(&format!("  Best distance    {:.0}\n", self.objective.best_distance));
        if let Some(fp) = self.objective.best_distance_float {
            out.push_str(&format!("  Best (float)     {:.4}\n", fp));
        }
        if let Some(bks) = self.objective.bks {
            out.push_str(&format!("  BKS              {:.2}\n", bks));
        } else {
            out.push_str("  BKS              — (no reference)\n");
        }
        if let Some(gap) = self.objective.gap_pct {
            out.push_str(&format!("  Gap              {:+.2}%\n", gap));
        }
        if let Some(gap_fp) = self.objective.gap_fp_pct {
            out.push_str(&format!("  Gap (float)      {:+.2}%\n", gap_fp));
        }
        out.push_str(&format!("  Quality class    {}\n", self.objective.quality_class));
        out.push_str(&format!("  Routes used      {}\n", self.objective.routes_used));
        out.push_str(&format!("  Runtime          {} ms\n", self.objective.runtime_ms));
        out.push_str(&format!("  Generations      {}\n", self.objective.generations));
        out.push_str(&format!("  Termination      {}\n\n", self.objective.termination_reason));

        // Qualification
        out.push_str("── Qualification ────────────────────────────────────────────────\n");
        let fcf_sym = if self.fcf.fc2_passed { "✓" } else { "✗" };
        out.push_str(&format!("  FCF              {} {} (confidence F{})\n",
            fcf_sym, self.fcf.status, self.fcf.confidence_level));
        out.push_str(&format!("    FC-1           {} {}\n",
            if self.fcf.fc1_passed { "✓" } else { "✗" }, self.fcf.fc1_reason));
        out.push_str(&format!("    FC-2.5         {} {}\n",
            if self.fcf.fc2_5_passed { "✓" } else { "✗" }, self.fcf.fc2_5_reason));
        out.push_str(&format!("    FC-2           {} {}\n",
            if self.fcf.fc2_passed { "✓" } else { "✗" }, self.fcf.fc2_reason));
        out.push_str(&format!("    FC-3           {} {}\n",
            if self.fcf.fc3_passed { "✓" } else { "~" }, self.fcf.fc3_reason));

        let fcs_sym = if self.fcs.comparison_valid { "✓" } else { "⚠" };
        out.push_str(&format!("  FCS              {} {} ({})\n",
            fcs_sym, self.fcs.outcome,
            if let Some(k) = self.fcs.declared_k {
                format!("{}({}), used={}", self.fcs.constraint_type, k, self.fcs.routes_used)
            } else {
                format!("{}, used={}", self.fcs.constraint_type, self.fcs.routes_used)
            }
        ));

        if let Some(fuc) = &self.fuc {
            out.push_str(&format!("  FUC-001          ✓ {} (util={:.1}% CV={:.3} RCR={:.3})\n",
                fuc.packing_classification,
                fuc.avg_utilization * 100.0,
                fuc.cv,
                fuc.rcr));
            if fuc.capacity_violations > 0 {
                out.push_str(&format!("    ⚠ Capacity violations: {}\n", fuc.capacity_violations));
            }
        } else {
            out.push_str("  FUC-001          — (not computed)\n");
        }
        out.push('\n');

        // Benchmark Context
        out.push_str("── Benchmark Context ────────────────────────────────────────────\n");
        out.push_str(&format!("  Provenance reg.  v{}\n", self.benchmark_context.provenance_registry_version));
        out.push_str(&format!("  Fleet sem. reg.  v{}\n", self.benchmark_context.fleet_semantics_registry_version));
        out.push_str(&format!("  Qualification    v{}\n", self.benchmark_context.qualification_version));
        out.push_str(&format!("  BKS source       {}\n", self.benchmark_context.bks_source));
        out.push_str(&format!("  Fleet evidence   {}\n\n", self.benchmark_context.fleet_semantics_evidence));

        // Governance
        out.push_str("── Governance ───────────────────────────────────────────────────\n");
        out.push_str(&format!("  FCF              v{}\n", self.governance.fcf_version));
        out.push_str(&format!("  FCS              v{}\n", self.governance.fcs_version));
        out.push_str(&format!("  FUC-001          v{}\n", self.governance.fuc_version));
        out.push_str(&format!("  Status           {}\n", self.governance.status));
        out.push_str(&format!("  Reason           {}\n", self.governance.status_reason));
        out.push_str(&format!("  Content hash     {}\n", self.governance.content_hash));

        out
    }

    /// One-line summary for campaign log.
    pub fn log_line(&self) -> String {
        let gap_str = match self.objective.gap_pct {
            Some(g) => format!(" gap={:+.2}%", g),
            None => " gap=N/A".to_string(),
        };
        format!(
            "[CERT] {} | {} | dist={:.0}{} | {} | hash={}",
            self.instance.name,
            self.governance.status.code(),
            self.objective.best_distance,
            gap_str,
            self.objective.quality_class,
            &self.governance.content_hash[..12],
        )
    }
}

// =============================================================================
// INTERNAL HELPERS
// =============================================================================

fn build_fcf_section(fcf: &FeasibilityCertificate) -> CertificateFcf {
    let fc3 = fcf.fc3_bin_pack.as_ref();
    CertificateFcf {
        status: fcf.status.code().to_string(),
        confidence_level: fcf.confidence_level,
        fc1_passed: fcf.fc1_structural.passed,
        fc1_reason: fcf.fc1_structural.reason.clone(),
        fc2_5_passed: fcf.fc2_5_benchmark.passed,
        fc2_5_reason: fcf.fc2_5_benchmark.reason.clone(),
        fc2_passed: fcf.fc2_capacity.passed,
        fc2_reason: fcf.fc2_capacity.reason.clone(),
        fc3_passed: fc3.map(|r| r.passed).unwrap_or(true),
        fc3_reason: fc3.map(|r| r.reason.clone()).unwrap_or_else(|| "not run".to_string()),
    }
}

fn build_fcs_section(fcs: &FleetSemanticCheck) -> CertificateFcs {
    let outcome = match &fcs.outcome {
        FleetSemanticOutcome::Valid               => "Valid",
        FleetSemanticOutcome::NotComparable       => "NotComparable",
        FleetSemanticOutcome::PendingVerification => "PendingVerification",
    };
    let comparison_valid = matches!(&fcs.outcome, FleetSemanticOutcome::Valid);

    CertificateFcs {
        constraint_type: fcs.constraint.code().to_string(),
        declared_k: fcs.constraint.declared_k(),
        routes_used: fcs.routes_used,
        outcome: outcome.to_string(),
        comparison_valid,
    }
}

fn build_fuc_section(fuc: &FleetUtilizationCertificate) -> CertificateFuc {
    let total_fleet_capacity = fuc.routes_used as i64 * fuc.capacity as i64;
    CertificateFuc {
        avg_utilization: fuc.avg_utilization,
        median_utilization: fuc.median_utilization,
        cv: fuc.load_cv,
        rcr: fuc.residual_concentration_ratio,
        packing_classification: fuc.packing_classification.label().to_string(),
        capacity_violations: fuc.capacity_violations,
        total_demand: fuc.total_demand,
        total_fleet_capacity,
        fleet_capacity_used: fuc.fleet_capacity_used,
    }
}

/// Derive the overall certificate status from all qualification outcomes.
fn derive_status(input: &CertificateInput<'_>, fcs: &CertificateFcs) -> (CertificateStatus, String) {
    // Infeasible: FCF failed at FC-2
    if matches!(input.fcf.status, FeasibilityStatus::ProvenInfeasible { .. }) {
        return (
            CertificateStatus::Infeasible,
            format!("FC-2 FAIL: {}", input.fcf.fc2_capacity.reason),
        );
    }

    // Structural/benchmark invalid
    if matches!(input.fcf.status,
        FeasibilityStatus::StructuralInvalid { .. } | FeasibilityStatus::BenchmarkInvalid { .. })
    {
        return (
            CertificateStatus::Infeasible,
            format!("FCF FAIL: {}", input.fcf.status.code()),
        );
    }

    // No BKS — cannot compute gap
    if input.bks.is_none() {
        return (
            CertificateStatus::NoReference,
            "No published BKS available for this instance".to_string(),
        );
    }

    // FCS pending verification — gap comparison validity unknown
    if fcs.outcome == "PendingVerification" {
        return (
            CertificateStatus::UnderInvestigation,
            format!("Fleet semantics unspecified for family '{}' — gap comparison validity unknown", input.family),
        );
    }

    // FCS invalid — routes_used violates fleet constraint
    if !fcs.comparison_valid {
        return (
            CertificateStatus::NotComparable,
            format!("FCS INVALID: routes_used={} violates {}({:?})",
                fcs.routes_used, fcs.constraint_type, fcs.declared_k),
        );
    }

    // Negative gap — under investigation
    if let Some(gap) = input.gap_pct {
        if gap < -0.01 {
            return (
                CertificateStatus::UnderInvestigation,
                format!("Negative gap {:+.2}% — comparison validity pending Stage B certificate (P4/P5)", gap),
            );
        }
    }

    // All gates passed — provisionally qualified (Stage B pending)
    (
        CertificateStatus::ProvisionallyQualified,
        "FCF PASS, FCS VALID, gap computed — Stage B route-count certificate pending".to_string(),
    )
}

/// Build the canonical body text for hashing.
/// All fields except content_hash are included, in deterministic order.
fn build_body_text(
    identity: &CertificateIdentity,
    instance: &CertificateInstance,
    objective: &CertificateObjective,
    fcf: &CertificateFcf,
    fcs: &CertificateFcs,
    fuc: &Option<CertificateFuc>,
    ctx: &CertificateBenchmarkContext,
    status: &CertificateStatus,
    status_reason: &str,
) -> String {
    let mut s = String::new();
    // Identity
    s.push_str(&format!("cert_id={}\n", identity.certificate_id));
    s.push_str(&format!("cert_version={}\n", identity.certificate_version));
    s.push_str(&format!("generated_at={}\n", identity.generated_at));
    s.push_str(&format!("campaign_id={}\n", identity.campaign_id));
    s.push_str(&format!("solver_version={}\n", identity.solver_version));
    // Instance
    s.push_str(&format!("instance={}\n", instance.name));
    s.push_str(&format!("family={}\n", instance.family));
    s.push_str(&format!("customers={}\n", instance.customers));
    s.push_str(&format!("capacity={}\n", instance.capacity));
    s.push_str(&format!("benchmark_vehicles={}\n", instance.benchmark_vehicles));
    s.push_str(&format!("vehicle_source={}\n", instance.vehicle_source));
    s.push_str(&format!("distance_metric={}\n", instance.distance_metric));
    // Objective
    s.push_str(&format!("best_distance={}\n", objective.best_distance));
    s.push_str(&format!("best_distance_float={:?}\n", objective.best_distance_float));
    s.push_str(&format!("bks={:?}\n", objective.bks));
    s.push_str(&format!("gap_pct={:?}\n", objective.gap_pct));
    s.push_str(&format!("gap_fp_pct={:?}\n", objective.gap_fp_pct));
    s.push_str(&format!("quality_class={}\n", objective.quality_class));
    s.push_str(&format!("routes_used={}\n", objective.routes_used));
    s.push_str(&format!("runtime_ms={}\n", objective.runtime_ms));
    s.push_str(&format!("generations={}\n", objective.generations));
    s.push_str(&format!("termination_reason={}\n", objective.termination_reason));
    // FCF
    s.push_str(&format!("fcf_status={}\n", fcf.status));
    s.push_str(&format!("fcf_confidence={}\n", fcf.confidence_level));
    s.push_str(&format!("fc1={}/{}\n", fcf.fc1_passed, fcf.fc1_reason));
    s.push_str(&format!("fc2_5={}/{}\n", fcf.fc2_5_passed, fcf.fc2_5_reason));
    s.push_str(&format!("fc2={}/{}\n", fcf.fc2_passed, fcf.fc2_reason));
    s.push_str(&format!("fc3={}/{}\n", fcf.fc3_passed, fcf.fc3_reason));
    // FCS
    s.push_str(&format!("fcs_constraint={}\n", fcs.constraint_type));
    s.push_str(&format!("fcs_declared_k={:?}\n", fcs.declared_k));
    s.push_str(&format!("fcs_routes_used={}\n", fcs.routes_used));
    s.push_str(&format!("fcs_outcome={}\n", fcs.outcome));
    s.push_str(&format!("fcs_comparison_valid={}\n", fcs.comparison_valid));
    // FUC
    if let Some(f) = fuc {
        s.push_str(&format!("fuc_avg_util={}\n", f.avg_utilization));
        s.push_str(&format!("fuc_median_util={}\n", f.median_utilization));
        s.push_str(&format!("fuc_cv={}\n", f.cv));
        s.push_str(&format!("fuc_rcr={}\n", f.rcr));
        s.push_str(&format!("fuc_packing={}\n", f.packing_classification));
        s.push_str(&format!("fuc_violations={}\n", f.capacity_violations));
        s.push_str(&format!("fuc_total_demand={}\n", f.total_demand));
        s.push_str(&format!("fuc_fleet_capacity={}\n", f.total_fleet_capacity));
        s.push_str(&format!("fuc_fleet_used={}\n", f.fleet_capacity_used));
    } else {
        s.push_str("fuc=none\n");
    }
    // Benchmark context
    s.push_str(&format!("provenance_registry_version={}\n", ctx.provenance_registry_version));
    s.push_str(&format!("fleet_semantics_registry_version={}\n", ctx.fleet_semantics_registry_version));
    s.push_str(&format!("qualification_version={}\n", ctx.qualification_version));
    s.push_str(&format!("bks_source={}\n", ctx.bks_source));
    s.push_str(&format!("fleet_semantics_evidence={}\n", ctx.fleet_semantics_evidence));
    // Governance (status only — hash excluded)
    s.push_str(&format!("status={}\n", status.code()));
    s.push_str(&format!("status_reason={}\n", status_reason));
    s
}

/// FNV-1a 64-bit hash over a string. Returns lowercase hex string.
/// Used for content integrity — not a cryptographic primitive.
fn fnv1a_64_hex(s: &str) -> String {
    const FNV_OFFSET: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;
    let mut hash: u64 = FNV_OFFSET;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{:016x}", hash)
}

/// Generate a deterministic certificate ID from instance name and timestamp.
/// Format: cert-{instance}-{hash8} where hash8 is the first 8 chars of FNV-1a over name+ts.
fn generate_cert_id(instance_name: &str, timestamp: &str) -> String {
    let key = format!("{}|{}", instance_name, timestamp);
    let hash = fnv1a_64_hex(&key);
    // Sanitize instance name for use in ID (replace non-alphanumeric with -)
    let safe_name: String = instance_name.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    format!("cert-{}-{}", safe_name, &hash[..8])
}

/// Pad or truncate a string to exactly `width` chars (ASCII spaces).
fn pad_or_truncate(s: &str, width: usize) -> String {
    if s.len() >= width {
        s[..width].to_string()
    } else {
        format!("{:<width$}", s, width = width)
    }
}