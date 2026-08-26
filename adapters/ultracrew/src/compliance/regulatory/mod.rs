// Regulatory rule packs — one sub-module per regional workforce authority.
//
// Each sub-module implements [`crate::compliance::traits::CompliancePack`] and
// encodes the hard limits published by that authority.  The optimizer never
// imports a specific authority; it only calls `pack.load_into(registry)`.
//
// # Planned (stub modules to be filled in)
// - `eu_wtd`          — EU Working Time Directive (Europe)
// - `osha`            — OSHA guidelines (United States)
//
// Reserved for future domain-specific regulatory packs
