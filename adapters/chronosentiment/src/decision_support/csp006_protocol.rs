//! CS-P-006-B research protocol constants.
//!
//! Protocol rules are frozen. Chronological development / selection / evaluation
//! partitions are frozen. This module does not search, evolve, or score policies.

/// Intended first Coralys discovery universe. Not a silent substitute for B4.
pub const RESEARCH_UNIVERSE: [&str; 7] = [
    "HDFCBANK.NS",
    "ICICIBANK.NS",
    "INFY.NS",
    "RELIANCE.NS",
    "TCS.NS",
    "IDEA.NS",
    "MAHABANK.NS",
];

/// Instruments actually present on certified B4 / CS-P-005 historical rows.
/// Not the CS-P-006 research universe.
pub const CERTIFIED_FIVE_INSTRUMENT_SNAPSHOT: [&str; 5] = [
    "HDFCBANK.NS",
    "ICICIBANK.NS",
    "INFY.NS",
    "RELIANCE.NS",
    "TCS.NS",
];

pub const B4_DUMP_SHA256: &str = "f74e576e8e98b24058cc913b14a567d9ff4b3eabc75662ab9a96901b102f8cd6";

/// Disposable CS-P-006 research snapshot. Not B4. Not B5.
pub const RESEARCH_SNAPSHOT_DIR: &str =
    "product_validation/CS-P-006/snapshot/20260814T183851Z_7instrument";
pub const RESEARCH_SNAPSHOT_IDENTITY_HASH: &str =
    "c21ec256133fb63656b35e68c5e1e72b72751ad2fb45f11c12f99ddb34a628c6";
pub const RESEARCH_SNAPSHOT_MANIFEST_SHA256: &str =
    "80e5b82fa7c089b487f99deb2b6f064de87e9173bc8b6766ffd8c03cbb04cc1d";
pub const RESEARCH_SNAPSHOT_CERTIFIED: bool = true;

pub const CSP005_ROW_COUNT: usize = 195;
pub const MAX_RULES_FIRST_DISCOVERY: usize = 16;

/// Chronological development / selection / evaluation partition is frozen (CS-P-006-B.1).
pub const CHRONOLOGICAL_PARTITION_FROZEN: bool = true;
pub const CHRONOLOGICAL_PARTITION_HASH: &str =
    "4354c81ef546003b1d11ec98cba83dd5f8c56b13c8b6055b8451614abdc4cfca";
/// Alias retained for existing protocol tests; prefer `CHRONOLOGICAL_PARTITION_FROZEN`.
pub const SPLIT_DATES_FROZEN: bool = CHRONOLOGICAL_PARTITION_FROZEN;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoverageAudit {
    Sufficient,
    Insufficient { missing: Vec<&'static str> },
}

/// Compare the intended seven-name universe to certified B4 / CS-P-005 rows.
/// Those dumps remain 5/7. They must not stand in for the research snapshot.
pub fn audit_b4_coverage() -> CoverageAudit {
    let missing: Vec<&'static str> = RESEARCH_UNIVERSE
        .iter()
        .copied()
        .filter(|ticker| {
            !CERTIFIED_FIVE_INSTRUMENT_SNAPSHOT
                .iter()
                .any(|known| known == ticker)
        })
        .collect();
    if missing.is_empty() {
        CoverageAudit::Sufficient
    } else {
        CoverageAudit::Insufficient { missing }
    }
}

pub fn audit_research_universe_coverage() -> CoverageAudit {
    audit_b4_coverage()
}

pub fn coralys_search_is_authorized() -> bool {
    CHRONOLOGICAL_PARTITION_FROZEN && RESEARCH_SNAPSHOT_CERTIFIED
}

/// First CS-P-006-C discovery evidence directory. Not B5.
pub const RESEARCH_DISCOVERY_DIR: &str = "product_validation/CS-P-006/discovery/20260814T195327Z";
pub const RESEARCH_DISCOVERY_ARTIFACT_HASH: &str =
    "9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0";
pub const RESEARCH_DISCOVERY_METHODOLOGY_HASH: &str =
    "6e92ef3e097d52f923b6028258f6442bcb5de6163c45a94628dead9aa954e3a5";

/// One authorized C.3-R Search #2 evidence directory. Not Search #1. Not B5.
pub const RESEARCH_DISCOVERY_TWO_DIR: &str =
    "product_validation/CS-P-006/discovery/20260815T051900Z_c3";
pub const RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH: &str =
    "5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121";
pub const RESEARCH_DISCOVERY_TWO_METHODOLOGY_HASH: &str =
    "eff198957d799419035a5b86f6adceee6233bfa626f5ff2fee39d59132d99a99";

/// C.3-G: next target is regime persistence at T. No experiment. Not Search #3.
pub const REGIME_PERSISTENCE_EXPERIMENT_AUTHORIZED: bool = false;
