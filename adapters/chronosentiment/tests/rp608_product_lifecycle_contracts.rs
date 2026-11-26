//! CLEAN-003-D — product Gate2X / plan-lifecycle contracts on the default test surface.
//!
//! Observational Gate2X floor-lock twin remains Python-only
//! (`tests/test_rp608_gate2x_floor_lock_twin.py`); authority 0%. This module
//! locks the **product** protocol identity and default config so default
//! `cargo test -p chronosentiment_adapter` still exercises the load-bearing path.

use chronosentiment_adapter::product::gate2x_features::{Gate2xParams, ProtectFloorMode};
use chronosentiment_adapter::product::rp608_plan_lifecycle::{
    PlanLifecycleConfig, PLAN_LIFECYCLE_PROTOCOL, TRAILING_SL_PROTOCOL,
};

#[test]
fn product_protocol_ids_are_stable() {
    assert_eq!(PLAN_LIFECYCLE_PROTOCOL, "RP-608-PLAN-LIFECYCLE-MANAGE-001");
    assert_eq!(TRAILING_SL_PROTOCOL, "RP-608-TRAILING-SL-001");
    assert_eq!(
        Gate2xParams::crypto_defaults().protocol,
        "RP-608-GATE2X-CRYPTO-MFE-GIVEBACK-001"
    );
}

#[test]
fn default_plan_lifecycle_keeps_frozen_geometry_knobs() {
    let cfg = PlanLifecycleConfig::default();
    // Product default Adaptive + early-scratch are intentional Gate-5 surface;
    // changing them requires Gate 5 — do not flip silently during hygiene.
    assert!(cfg.adaptive_tp_enabled);
    assert!(cfg.early_sl_path_scratch_enabled);
    assert!((cfg.trailing_sl_atr_mult - 0.5).abs() < 1e-12);
    assert!((cfg.trailing_sl_arm_risk_frac - 0.30).abs() < 1e-12);
}

#[test]
fn default_gate2x_protect_floor_is_cost_clear() {
    let p = Gate2xParams::default();
    assert!(matches!(p.protect_floor, ProtectFloorMode::CostClear));
    assert!((p.ratchet_peak_frac - 0.30).abs() < 1e-12);
}
