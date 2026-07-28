/// Regulatory rule packs — one sub-module per civil aviation authority.
///
/// Each sub-module implements [`crate::compliance::traits::RulePack`] and
/// encodes the hard limits published by that authority.  The optimizer never
/// imports a specific authority; it only calls `pack.load_into(registry)`.
///
/// # Implemented
/// - [`dgca`] — DGCA (India) CAR Section 7 Series J Part III
///
/// # Planned (stub modules to be filled in)
/// - `easa`            — EASA EU-OPS / ORO.FTL (Europe)
/// - `faa_part117`     — FAA Part 117 (United States)
/// - `transport_canada`— Transport Canada CARs Part VII (Canada — separate from FAA)
/// - `casa`            — CASA CAO 48.1 (Australia)
/// - `uk_caa`          — UK CAA CAP 1616 (post-Brexit UK)
/// - `gaca`            — GACA (Saudi Arabia)
/// - `gcaa`            — GCAA (UAE)
/// - `caas`            — CAAS (Singapore)
/// - `caac`            — CAAC (China)
/// - `icao_reference`  — ICAO Annex 6 reference rules (baseline for all others)

pub mod dgca;

// Re-export the primary entry point for the DGCA pack.
pub use dgca::DgcaCompliancePack;
// Backward-compatible alias
pub use dgca::DgcaRulePack;