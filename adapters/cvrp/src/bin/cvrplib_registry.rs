/// CVRPLIB Benchmark Metadata Registry — Two-Layer Architecture
///
/// Layer 1: BenchmarkMetadata — operational data the solver needs to execute a benchmark.
///   Compact, rarely changes, optimized for runtime lookup.
///
/// Layer 2: QualificationMetadata — provenance, confidence, and release evidence.
///   Sparse: family defaults cover ~95% of cases; instance overrides for genuine exceptions.
///   Evolves independently as Coralys learns more about benchmark provenance.
///
/// Sources:
///   https://galgos.inf.puc-rio.br/cvrplib/index.php/en/instances/1

// =============================================================================
// LAYER 1 — OPERATIONAL METADATA
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum BenchmarkFamily {
    Augerat,       // A, B, E, P
    Fisher,        // F
    Christofides,  // M
    CMT,           // Christofides, Mingozzi, Toth (1979)
    Taillard,      // Tai
    Golden,        // Golden et al (1998)
    Li,            // Li et al (2005)
    Uchoa,         // X — Uchoa et al (2017)
    Unknown,
}

#[derive(Debug, Clone)]
pub struct BenchmarkMetadata {
    pub name: &'static str,
    pub vehicles: usize,
    pub bks: f64,
    pub family: BenchmarkFamily,
    pub source: &'static str,
}

/// Query the registry for a named instance.
/// Returns None if the instance is not in the registry (e.g. Augerat instances
/// that encode vehicle count in the COMMENT — they don't need the registry).
pub fn benchmark_metadata(name: &str) -> Option<BenchmarkMetadata> {
    REGISTRY.iter().find(|m| m.name == name).cloned()
}

static REGISTRY: &[BenchmarkMetadata] = &[
    // =========================================================================
    // CMT — Christofides, Mingozzi, Toth (1979)
    // 14 instances in two sets (CMT1-5 and CMT6-10 are paired, CMT11-14 extra)
    // =========================================================================
    BenchmarkMetadata { name: "CMT1",  vehicles: 5,  bks: 524.61,  family: BenchmarkFamily::CMT, source: "Christofides et al. 1979" },
    BenchmarkMetadata { name: "CMT2",  vehicles: 10, bks: 835.26,  family: BenchmarkFamily::CMT, source: "Christofides et al. 1979" },
    BenchmarkMetadata { name: "CMT3",  vehicles: 8,  bks: 826.14,  family: BenchmarkFamily::CMT, source: "Christofides et al. 1979" },
    BenchmarkMetadata { name: "CMT4",  vehicles: 12, bks: 1028.42, family: BenchmarkFamily::CMT, source: "Christofides et al. 1979" },
    BenchmarkMetadata { name: "CMT5",  vehicles: 17, bks: 1291.29, family: BenchmarkFamily::CMT, source: "Christofides et al. 1979" },
    BenchmarkMetadata { name: "CMT6",  vehicles: 6,  bks: 555.43,  family: BenchmarkFamily::CMT, source: "Christofides et al. 1979" },
    BenchmarkMetadata { name: "CMT7",  vehicles: 11, bks: 909.68,  family: BenchmarkFamily::CMT, source: "Christofides et al. 1979" },
    BenchmarkMetadata { name: "CMT8",  vehicles: 9,  bks: 865.94,  family: BenchmarkFamily::CMT, source: "Christofides et al. 1979" },
    BenchmarkMetadata { name: "CMT9",  vehicles: 14, bks: 1162.55, family: BenchmarkFamily::CMT, source: "Christofides et al. 1979" },
    BenchmarkMetadata { name: "CMT10", vehicles: 18, bks: 1395.85, family: BenchmarkFamily::CMT, source: "Christofides et al. 1979" },
    BenchmarkMetadata { name: "CMT11", vehicles: 11, bks: 1042.11, family: BenchmarkFamily::CMT, source: "Christofides et al. 1979" },
    BenchmarkMetadata { name: "CMT12", vehicles: 10, bks: 819.56,  family: BenchmarkFamily::CMT, source: "Christofides et al. 1979" },
    BenchmarkMetadata { name: "CMT13", vehicles: 11, bks: 1541.14, family: BenchmarkFamily::CMT, source: "Christofides et al. 1979" },
    BenchmarkMetadata { name: "CMT14", vehicles: 10, bks: 866.37,  family: BenchmarkFamily::CMT, source: "Christofides et al. 1979" },

    // =========================================================================
    // Tai — Taillard (1993)
    // Per-instance vehicle counts differ within size groups
    // =========================================================================
    BenchmarkMetadata { name: "Tai75a",  vehicles: 10, bks: 1618.36, family: BenchmarkFamily::Taillard, source: "Taillard 1993" },
    BenchmarkMetadata { name: "Tai75b",  vehicles: 9,  bks: 1407.89, family: BenchmarkFamily::Taillard, source: "Taillard 1993" },
    BenchmarkMetadata { name: "Tai75c",  vehicles: 10, bks: 1166.69, family: BenchmarkFamily::Taillard, source: "Taillard 1993" },
    BenchmarkMetadata { name: "Tai75d",  vehicles: 9,  bks: 1468.73, family: BenchmarkFamily::Taillard, source: "Taillard 1993" },
    BenchmarkMetadata { name: "Tai100a", vehicles: 11, bks: 2141.07, family: BenchmarkFamily::Taillard, source: "Taillard 1993" },
    BenchmarkMetadata { name: "Tai100b", vehicles: 11, bks: 1940.55, family: BenchmarkFamily::Taillard, source: "Taillard 1993" },
    BenchmarkMetadata { name: "Tai100c", vehicles: 11, bks: 1406.94, family: BenchmarkFamily::Taillard, source: "Taillard 1993" },
    BenchmarkMetadata { name: "Tai100d", vehicles: 11, bks: 1575.03, family: BenchmarkFamily::Taillard, source: "Taillard 1993" },
    BenchmarkMetadata { name: "Tai150a", vehicles: 12, bks: 2470.47, family: BenchmarkFamily::Taillard, source: "Taillard 1993" },
    BenchmarkMetadata { name: "Tai150b", vehicles: 12, bks: 2197.45, family: BenchmarkFamily::Taillard, source: "Taillard 1993" },
    BenchmarkMetadata { name: "Tai150c", vehicles: 12, bks: 2097.04, family: BenchmarkFamily::Taillard, source: "Taillard 1993" },
    BenchmarkMetadata { name: "Tai150d", vehicles: 12, bks: 2222.35, family: BenchmarkFamily::Taillard, source: "Taillard 1993" },
    BenchmarkMetadata { name: "Tai385",  vehicles: 24, bks: 24420.0, family: BenchmarkFamily::Taillard, source: "Taillard 1993" },

    // =========================================================================
    // Golden — Golden et al (1998)
    // All instances have >200 customers; will be skipped by MAX_CUSTOMERS limit.
    // =========================================================================
    BenchmarkMetadata { name: "Golden_1",  vehicles: 9,  bks: 5623.47,  family: BenchmarkFamily::Golden, source: "Golden et al. 1998" },
    BenchmarkMetadata { name: "Golden_2",  vehicles: 9,  bks: 8404.61,  family: BenchmarkFamily::Golden, source: "Golden et al. 1998" },
    BenchmarkMetadata { name: "Golden_3",  vehicles: 10, bks: 11036.22, family: BenchmarkFamily::Golden, source: "Golden et al. 1998" },
    BenchmarkMetadata { name: "Golden_4",  vehicles: 11, bks: 13624.55, family: BenchmarkFamily::Golden, source: "Golden et al. 1998" },
    BenchmarkMetadata { name: "Golden_5",  vehicles: 5,  bks: 6460.98,  family: BenchmarkFamily::Golden, source: "Golden et al. 1998" },
    BenchmarkMetadata { name: "Golden_6",  vehicles: 6,  bks: 8404.26,  family: BenchmarkFamily::Golden, source: "Golden et al. 1998" },
    BenchmarkMetadata { name: "Golden_7",  vehicles: 7,  bks: 10102.68, family: BenchmarkFamily::Golden, source: "Golden et al. 1998" },
    BenchmarkMetadata { name: "Golden_8",  vehicles: 8,  bks: 11635.34, family: BenchmarkFamily::Golden, source: "Golden et al. 1998" },
    BenchmarkMetadata { name: "Golden_9",  vehicles: 14, bks: 579.71,   family: BenchmarkFamily::Golden, source: "Golden et al. 1998" },
    BenchmarkMetadata { name: "Golden_10", vehicles: 16, bks: 736.26,   family: BenchmarkFamily::Golden, source: "Golden et al. 1998" },
    BenchmarkMetadata { name: "Golden_11", vehicles: 18, bks: 912.84,   family: BenchmarkFamily::Golden, source: "Golden et al. 1998" },
    BenchmarkMetadata { name: "Golden_12", vehicles: 20, bks: 1102.69,  family: BenchmarkFamily::Golden, source: "Golden et al. 1998" },
    BenchmarkMetadata { name: "Golden_13", vehicles: 22, bks: 857.19,   family: BenchmarkFamily::Golden, source: "Golden et al. 1998" },
    BenchmarkMetadata { name: "Golden_14", vehicles: 24, bks: 1080.55,  family: BenchmarkFamily::Golden, source: "Golden et al. 1998" },
    BenchmarkMetadata { name: "Golden_15", vehicles: 26, bks: 1337.92,  family: BenchmarkFamily::Golden, source: "Golden et al. 1998" },
    BenchmarkMetadata { name: "Golden_16", vehicles: 28, bks: 1612.50,  family: BenchmarkFamily::Golden, source: "Golden et al. 1998" },
    BenchmarkMetadata { name: "Golden_17", vehicles: 22, bks: 707.76,   family: BenchmarkFamily::Golden, source: "Golden et al. 1998" },
    BenchmarkMetadata { name: "Golden_18", vehicles: 26, bks: 995.13,   family: BenchmarkFamily::Golden, source: "Golden et al. 1998" },
    BenchmarkMetadata { name: "Golden_19", vehicles: 30, bks: 1365.60,  family: BenchmarkFamily::Golden, source: "Golden et al. 1998" },
    BenchmarkMetadata { name: "Golden_20", vehicles: 34, bks: 1818.32,  family: BenchmarkFamily::Golden, source: "Golden et al. 1998" },

    // =========================================================================
    // Li — Li et al (2005)
    // All instances have >200 customers; will be skipped by MAX_CUSTOMERS limit.
    // =========================================================================
    BenchmarkMetadata { name: "Li_21", vehicles: 10, bks: 21532.0,  family: BenchmarkFamily::Li, source: "Li et al. 2005" },
    BenchmarkMetadata { name: "Li_22", vehicles: 10, bks: 22814.0,  family: BenchmarkFamily::Li, source: "Li et al. 2005" },
    BenchmarkMetadata { name: "Li_23", vehicles: 10, bks: 24613.0,  family: BenchmarkFamily::Li, source: "Li et al. 2005" },
    BenchmarkMetadata { name: "Li_24", vehicles: 10, bks: 27591.0,  family: BenchmarkFamily::Li, source: "Li et al. 2005" },
    BenchmarkMetadata { name: "Li_25", vehicles: 10, bks: 29368.0,  family: BenchmarkFamily::Li, source: "Li et al. 2005" },
    BenchmarkMetadata { name: "Li_26", vehicles: 10, bks: 31742.0,  family: BenchmarkFamily::Li, source: "Li et al. 2005" },
    BenchmarkMetadata { name: "Li_27", vehicles: 10, bks: 33609.0,  family: BenchmarkFamily::Li, source: "Li et al. 2005" },
    BenchmarkMetadata { name: "Li_28", vehicles: 10, bks: 35627.0,  family: BenchmarkFamily::Li, source: "Li et al. 2005" },
    BenchmarkMetadata { name: "Li_29", vehicles: 10, bks: 39360.0,  family: BenchmarkFamily::Li, source: "Li et al. 2005" },
    BenchmarkMetadata { name: "Li_30", vehicles: 10, bks: 31742.51, family: BenchmarkFamily::Li, source: "Li et al. 2005" },
    BenchmarkMetadata { name: "Li_31", vehicles: 10, bks: 43748.0,  family: BenchmarkFamily::Li, source: "Li et al. 2005" },
    BenchmarkMetadata { name: "Li_32", vehicles: 10, bks: 48217.0,  family: BenchmarkFamily::Li, source: "Li et al. 2005" },
];

// =============================================================================
// LAYER 2 — QUALIFICATION METADATA
// =============================================================================
// Answers two distinct questions:
//   verification_status  → Has the metadata been checked?
//   qualification_level  → Can this benchmark be used as release evidence?
//
// Hierarchy:
//   1. default_verified()      — validated families (Augerat, Fisher, Christofides)
//   2. family_default(name)    — family-level findings from qualification campaigns
//   3. instance_override(name) — genuine per-instance exceptions (rare)
// =============================================================================

/// Has the metadata (vehicle count, BKS) been verified?
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationStatus {
    /// Verified against official CVRPLIB catalog.
    Verified,
    /// Extracted from instance file; not cross-checked externally.
    FileExtracted,
    /// From original publication only; not cross-checked against CVRPLIB.
    PublicationOnly,
    /// Flagged for further investigation.
    PendingVerification,
}

impl std::fmt::Display for VerificationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerificationStatus::Verified => write!(f, "Verified"),
            VerificationStatus::FileExtracted => write!(f, "FileExtracted"),
            VerificationStatus::PublicationOnly => write!(f, "PublicationOnly"),
            VerificationStatus::PendingVerification => write!(f, "PendingVerification"),
        }
    }
}

/// Can this benchmark be used as release evidence?
#[derive(Debug, Clone, PartialEq)]
pub enum QualificationLevel {
    /// Fully qualified — contributes to release evidence.
    Verified,
    /// Partially qualified — usable but with caveats noted.
    PartiallyVerified,
    /// Under investigation — do not use as release evidence yet.
    UnderInvestigation,
    /// Unsupported — engine does not implement required capability.
    Unsupported,
    /// Excluded — outside current qualification scope (e.g. >200 customers).
    Excluded,
}

impl std::fmt::Display for QualificationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QualificationLevel::Verified => write!(f, "Verified"),
            QualificationLevel::PartiallyVerified => write!(f, "PartiallyVerified"),
            QualificationLevel::UnderInvestigation => write!(f, "UnderInvestigation"),
            QualificationLevel::Unsupported => write!(f, "Unsupported"),
            QualificationLevel::Excluded => write!(f, "Excluded"),
        }
    }
}

/// Fleet semantics: what the vehicle count means for this instance.
#[derive(Debug, Clone, PartialEq)]
pub enum FleetSemantics {
    /// Minimum fleet: optimizer may not use fewer vehicles.
    Minimum,
    /// Maximum fleet: optimizer may use fewer vehicles.
    Maximum,
    /// Exact fleet: optimizer must use exactly this many vehicles.
    Exact,
    /// Unknown — requires investigation.
    Unknown,
}

impl std::fmt::Display for FleetSemantics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FleetSemantics::Minimum => write!(f, "Minimum"),
            FleetSemantics::Maximum => write!(f, "Maximum"),
            FleetSemantics::Exact => write!(f, "Exact"),
            FleetSemantics::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Distance semantics for this instance.
#[derive(Debug, Clone, PartialEq)]
pub enum DistanceSemantics {
    /// TSPLIB EUC_2D: round(sqrt(dx²+dy²))
    TspLibEuc2D,
    /// Explicit integer distance matrix (TSPLIB EXPLICIT format)
    ExplicitInteger,
    /// Unknown
    Unknown,
}

impl std::fmt::Display for DistanceSemantics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DistanceSemantics::TspLibEuc2D => write!(f, "TspLibEuc2D"),
            DistanceSemantics::ExplicitInteger => write!(f, "ExplicitInteger"),
            DistanceSemantics::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Source of the BKS value.
#[derive(Debug, Clone, PartialEq)]
pub enum BksProvenance {
    /// BKS from CVRPLIB catalog (galgos.inf.puc-rio.br).
    CvrplibCatalog,
    /// BKS from instance file COMMENT field.
    FileComment,
    /// BKS from original publication.
    OriginalPublication,
    /// BKS unverified or unknown.
    Unverified,
}

impl std::fmt::Display for BksProvenance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BksProvenance::CvrplibCatalog => write!(f, "CvrplibCatalog"),
            BksProvenance::FileComment => write!(f, "FileComment"),
            BksProvenance::OriginalPublication => write!(f, "OriginalPublication"),
            BksProvenance::Unverified => write!(f, "Unverified"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QualificationMetadata {
    /// Has the metadata been checked?
    pub verification_status: VerificationStatus,
    /// Can this benchmark be used as release evidence?
    pub qualification_level: QualificationLevel,
    /// Primary source for metadata lookup.
    pub metadata_source: &'static str,
    /// Original publication for this benchmark family.
    pub source_publication: &'static str,
    /// Fleet semantics: minimum/maximum/exact/unknown.
    pub fleet_semantics: FleetSemantics,
    /// Distance semantics for this instance.
    pub distance_semantics: DistanceSemantics,
    /// Source of the BKS value.
    pub bks_provenance: BksProvenance,
    /// Qualification notes — family-level findings from qualification campaigns.
    pub qualification_notes: &'static str,
}

// ── Family defaults ───────────────────────────────────────────────────────────

/// Default for validated families: Augerat (A/B/E/P), Fisher (F), Christofides (M).
/// Vehicle count from COMMENT; BKS from COMMENT. Fully qualified.
fn default_verified() -> QualificationMetadata {
    QualificationMetadata {
        verification_status: VerificationStatus::FileExtracted,
        qualification_level: QualificationLevel::Verified,
        metadata_source: "CVRPLIB catalog (galgos.inf.puc-rio.br)",
        source_publication: "Augerat et al. 1995",
        fleet_semantics: FleetSemantics::Minimum,
        distance_semantics: DistanceSemantics::TspLibEuc2D,
        bks_provenance: BksProvenance::FileComment,
        qualification_notes: "Vehicle count from COMMENT 'No of trucks: N'. BKS from COMMENT 'Optimal value: N'. Fully qualified for release evidence.",
    }
}

fn family_cmt() -> QualificationMetadata {
    QualificationMetadata {
        verification_status: VerificationStatus::Verified,
        qualification_level: QualificationLevel::PartiallyVerified,
        metadata_source: "CVRPLIB catalog (galgos.inf.puc-rio.br)",
        source_publication: "Christofides, Mingozzi, Toth 1979",
        fleet_semantics: FleetSemantics::Minimum,
        distance_semantics: DistanceSemantics::TspLibEuc2D,
        bks_provenance: BksProvenance::CvrplibCatalog,
        qualification_notes: "Vehicle count from registry (not in .vrp file). \
                              Per-instance counts verified against CVRPLIB catalog 2026-07. \
                              BKS provenance under verification — some values may differ from original publication. \
                              Fleet semantics under review. Qualification status: Partially Verified.",
    }
}

fn family_taillard() -> QualificationMetadata {
    QualificationMetadata {
        verification_status: VerificationStatus::Verified,
        qualification_level: QualificationLevel::PartiallyVerified,
        metadata_source: "CVRPLIB catalog (galgos.inf.puc-rio.br)",
        source_publication: "Taillard 1993",
        fleet_semantics: FleetSemantics::Minimum,
        distance_semantics: DistanceSemantics::TspLibEuc2D,
        bks_provenance: BksProvenance::CvrplibCatalog,
        qualification_notes: "Vehicle count from registry (not in .vrp file). \
                              Per-instance counts differ within size groups (e.g. Tai75b=9, Tai75d=9 vs Tai75a/c=10). \
                              Tai150 fleet semantics require confirmation. \
                              Qualification status: Partially Verified.",
    }
}

fn family_golden() -> QualificationMetadata {
    QualificationMetadata {
        verification_status: VerificationStatus::Verified,
        qualification_level: QualificationLevel::Excluded,
        metadata_source: "CVRPLIB catalog (galgos.inf.puc-rio.br)",
        source_publication: "Golden et al. 1998",
        fleet_semantics: FleetSemantics::Minimum,
        distance_semantics: DistanceSemantics::TspLibEuc2D,
        bks_provenance: BksProvenance::CvrplibCatalog,
        qualification_notes: "All instances >200 customers. Excluded from current qualification scope (MAX_CUSTOMERS=200). \
                              Registry metadata verified. Qualification status: Excluded.",
    }
}

fn family_li() -> QualificationMetadata {
    QualificationMetadata {
        verification_status: VerificationStatus::PublicationOnly,
        qualification_level: QualificationLevel::Excluded,
        metadata_source: "Li et al. 2005 (publication)",
        source_publication: "Li et al. 2005",
        fleet_semantics: FleetSemantics::Unknown,
        distance_semantics: DistanceSemantics::TspLibEuc2D,
        bks_provenance: BksProvenance::OriginalPublication,
        qualification_notes: "All instances >200 customers. Not yet qualified — outside campaign scope. \
                              Qualification status: Excluded.",
    }
}

fn family_uchoa() -> QualificationMetadata {
    QualificationMetadata {
        verification_status: VerificationStatus::FileExtracted,
        qualification_level: QualificationLevel::PartiallyVerified,
        metadata_source: "CVRPLIB catalog (galgos.inf.puc-rio.br)",
        source_publication: "Uchoa et al. 2017",
        fleet_semantics: FleetSemantics::Minimum,
        distance_semantics: DistanceSemantics::TspLibEuc2D,
        bks_provenance: BksProvenance::FileComment,
        qualification_notes: "Vehicle count from -kN in instance name. BKS from COMMENT. \
                              Fleet semantics require confirmation against Uchoa et al. 2017. \
                              Qualification status: Partially Verified.",
    }
}

fn family_eilon() -> QualificationMetadata {
    QualificationMetadata {
        verification_status: VerificationStatus::Verified,
        qualification_level: QualificationLevel::PartiallyVerified,
        metadata_source: "CVRPLIB catalog (galgos.inf.puc-rio.br)",
        source_publication: "Eilon et al. 1971",
        fleet_semantics: FleetSemantics::Minimum,
        distance_semantics: DistanceSemantics::ExplicitInteger,
        bks_provenance: BksProvenance::FileComment,
        qualification_notes: "EXPLICIT LOWER_ROW distance matrix — not EUC_2D. \
                              Vehicle count from COMMENT. ExplicitMatrix support added in v1.1. \
                              Qualification status: Partially Verified (new capability, requires validation run).",
    }
}

/// Return qualification metadata for a named instance.
///
/// Hierarchy:
///   1. Instance-specific overrides (genuine per-instance exceptions — currently none)
///   2. Family defaults (covers ~95% of cases)
///   3. default_verified() for validated families (Augerat, Fisher, Christofides)
///
/// Every instance has provenance — this function always returns a value.
pub fn qualification_metadata(name: &str) -> QualificationMetadata {
    // ── Step 1: Instance-specific overrides ──────────────────────────────────
    // Reserved for genuine per-instance exceptions discovered by qualification campaigns.
    // Currently none — all known issues are family-level.
    // Future examples: "Tai150a" | "Tai150b" | "CMT13" if individual issues are found.

    // ── Step 2: Family dispatch ───────────────────────────────────────────────
    if name.starts_with("CMT") {
        return family_cmt();
    }
    if name.starts_with("Tai") {
        return family_taillard();
    }
    if name.starts_with("Golden") {
        return family_golden();
    }
    if name.starts_with("Li_") {
        return family_li();
    }
    // Uchoa X-family: "X-n..." pattern
    if name.starts_with("X-") || name.starts_with("X_") {
        return family_uchoa();
    }
    // Eilon instances with EXPLICIT matrix (E-n13-k4, E-n31-k7)
    // These are in the Augerat E-family but use EXPLICIT distance matrix
    if name == "E-n13-k4" || name == "E-n31-k7" {
        return family_eilon();
    }

    // ── Step 3: Default — validated families (Augerat A/B/E/P, Fisher F, Christofides M) ──
    default_verified()
}fn main() {}
