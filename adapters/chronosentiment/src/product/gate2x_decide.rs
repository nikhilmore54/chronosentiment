//! Gate 2X decide — hybrid Coralys-heavy authority (adapter-owned).
//!
//! Authority chain (deterministic):
//! 1. Eligibility (Hold-only): disabled / not armed / min hold
//! 2. Soft-breach + dwell / HIGH immediate (feature presentation)
//! 3. Always call `evaluate_coralys_direction_aligned_state` (frozen engine)
//! 4. Remap Buy|Sell→Hold, Watch→Protect, NoTrade→Exit
//! 5. Insufficient fallback: Layer-2 Reversing + armed soft-breach → Protect
//!
//! Python/Rust executors never invent Exit; frozen SL/TP remain separate.

use coralys_decision::recommendation::{Rec001hStore, RecommendationAction, RecommendationRecordV1};
use serde::{Deserialize, Serialize};

use super::coralys_state_bridge::{
    evaluate_coralys_direction_aligned_state, CoralysPositionAction, MarketPositionTrajectoryState,
    MomentumTrajectoryRegime,
};
use super::gate2x_causal_map::map_gate2x_to_causal;
use super::gate2x_features::{
    Gate2xFeatureSnapshot, Gate2xSoftBreachState, ProtectFloorMode, VolSurgeBucket,
};

/// Decision output for shadow executors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate2xDecision {
    pub action: CoralysPositionAction,
    pub reason: String,
    pub coralys_action: String,
    pub recommendation_action: String,
    pub decision_id: String,
    pub protocol: String,
    pub soft_breach: Gate2xSoftBreachState,
    pub engine_insufficient: bool,
    pub authority: String,
}

fn action_name(a: CoralysPositionAction) -> &'static str {
    match a {
        CoralysPositionAction::Hold => "Hold",
        CoralysPositionAction::Protect => "Protect",
        CoralysPositionAction::Exit => "Exit",
    }
}

fn rec_action_name(a: RecommendationAction) -> &'static str {
    match a {
        RecommendationAction::Buy => "Buy",
        RecommendationAction::Sell => "Sell",
        RecommendationAction::Watch => "Watch",
        RecommendationAction::NoTrade => "NoTrade",
    }
}

fn hold_decision(
    snap: &Gate2xFeatureSnapshot,
    reason: &str,
    soft: Gate2xSoftBreachState,
) -> Gate2xDecision {
    Gate2xDecision {
        action: CoralysPositionAction::Hold,
        reason: reason.to_string(),
        coralys_action: "Hold".into(),
        recommendation_action: "HoldEligibility".into(),
        decision_id: snap.decision_id.clone(),
        protocol: snap.params.protocol.clone(),
        soft_breach: soft,
        engine_insufficient: false,
        authority: "adapter_eligibility".into(),
    }
}

fn is_immediate_surge(snap: &Gate2xFeatureSnapshot) -> bool {
    let surge = snap.vol_surge.as_str();
    snap.params
        .immediate_vol_surges
        .iter()
        .any(|s| s.eq_ignore_ascii_case(surge))
}

fn dwell_ms_for(snap: &Gate2xFeatureSnapshot, giveback_frac: f64) -> u64 {
    let mut dwell = if snap.vol_surge == VolSurgeBucket::Med {
        snap.params.med_vol_dwell_ms
    } else {
        snap.params.low_vol_dwell_ms
    };
    if giveback_frac >= snap.params.deep_giveback_frac {
        dwell = ((dwell as f64) * snap.params.deep_giveback_dwell_scale).round() as u64;
        dwell = dwell.max(5_000);
    }
    dwell
}

/// Core Gate 2X decide — Coralys engine path with thin adapter eligibility.
pub fn gate2x_decide(snap: &Gate2xFeatureSnapshot, store: &Rec001hStore) -> Gate2xDecision {
    let mut soft = Gate2xSoftBreachState {
        soft_breach_since_ms: snap.soft_breach_since_ms,
        ..Gate2xSoftBreachState::default()
    };

    if !snap.params.enabled {
        return hold_decision(snap, "GATE2X_DISABLED", soft);
    }

    let armed = snap.already_armed
        || (snap.peak_mfe > snap.arm_threshold
            && snap.peak_tp_frac >= snap.params.arm_min_tp_frac);
    soft.armed = armed;
    if armed {
        soft.protocol = Some(snap.params.protocol.clone());
    }

    if !armed {
        return hold_decision(snap, "GATE2X_NOT_ARMED", soft);
    }

    if snap.hold_s < snap.params.min_hold_s_before_exit {
        return hold_decision(snap, "GATE2X_MIN_HOLD", soft);
    }

    let base_floor = match snap.params.protect_floor {
        ProtectFloorMode::Entry => 0.0,
        ProtectFloorMode::CostClear => snap.arm_threshold,
    };
    let ratchet_floor = if snap.peak_mfe > 0.0 {
        snap.peak_mfe * snap.params.ratchet_peak_frac
    } else {
        0.0
    };
    let protect_floor = base_floor.max(ratchet_floor);
    soft.protect_floor = Some(protect_floor);
    soft.ratchet_floor = Some(ratchet_floor);

    let detail_base = match snap.params.protect_floor {
        ProtectFloorMode::Entry => "MFE_GIVEBACK_TO_ENTRY",
        ProtectFloorMode::CostClear => "MFE_GIVEBACK_TO_COST_CLEAR",
    };

    // Above ratchet/protect floor → clear soft-breach; Hold.
    if snap.current_mfe > protect_floor {
        soft.soft_breach_since_ms = None;
        soft.soft_breach_vol_surge = None;
        soft.giveback_frac = None;
        return hold_decision(snap, "GATE2X_ABOVE_FLOOR", soft);
    }

    let giveback_frac = if snap.peak_mfe > 1e-9 {
        (snap.peak_mfe - snap.current_mfe) / snap.peak_mfe
    } else {
        0.0
    };
    soft.giveback_frac = Some(giveback_frac);
    soft.soft_breach_vol_surge = Some(snap.vol_surge.as_str().to_string());

    // v1.5 hard floor: ratchet starts dwell for wick filter, but once MFE falls
    // through cost-clear / entry (base_floor) do not keep dwelling into a loss.
    // Session 2026-11-21: two SHORT givebacks (peak +$5.9/+13.7) exited −$3.5/−$5
    // after ~36s LOW deep-GB dwell while still bleeding through cost-clear.
    let hard_floor = base_floor;
    if snap.current_mfe <= hard_floor {
        soft.soft_breach_since_ms = None;
        let reason = format!("{detail_base}_HARD_FLOOR");
        return invoke_coralys(snap, store, true, soft, &reason);
    }

    if !snap.params.reversal_volume_confirm {
        // Immediate adverse presentation → Coralys decides Protect/Exit.
        soft.soft_breach_since_ms = None;
        return invoke_coralys(
            snap,
            store,
            true,
            soft,
            detail_base,
        );
    }

    if is_immediate_surge(snap) {
        soft.soft_breach_since_ms = None;
        let reason = format!("{detail_base}_VOL_{}", snap.vol_surge.as_str());
        return invoke_coralys(snap, store, true, soft, &reason);
    }

    let dwell = dwell_ms_for(snap, giveback_frac);
    soft.soft_breach_dwell_ms = Some(dwell);
    let mut detail = detail_base.to_string();
    if giveback_frac >= snap.params.deep_giveback_frac {
        detail = format!("{detail}_DEEP_GB");
    }

    let since = soft.soft_breach_since_ms.unwrap_or(0);
    if since <= 0 {
        soft.soft_breach_since_ms = Some(snap.now_ms);
        // Start dwell — Hold this tick (do not present adverse yet).
        return hold_decision(snap, "GATE2X_DWELL_START", soft);
    }
    if snap.now_ms - since < dwell as i64 {
        return hold_decision(snap, "GATE2X_DWELLING", soft);
    }

    soft.soft_breach_since_ms = None;
    let reason = format!("{detail}_VOL_DWELL_{}", snap.vol_surge.as_str());
    invoke_coralys(snap, store, true, soft, &reason)
}

fn normalize_instrument(instrument: &str) -> String {
    // Rec001hStore indexes NSE as TICKER_NS; crypto tickers (e.g. BTCUSDT) unchanged.
    instrument.replace(".NS", "_NS")
}

fn invoke_coralys(
    snap: &Gate2xFeatureSnapshot,
    store: &Rec001hStore,
    adverse: bool,
    soft: Gate2xSoftBreachState,
    reason_hint: &str,
) -> Gate2xDecision {
    let (state, velocity) = map_gate2x_to_causal(snap, adverse);
    let instrument = normalize_instrument(&snap.instrument);
    let (record, action) = evaluate_coralys_direction_aligned_state(
        &snap.decision_id,
        &instrument,
        &snap.direction,
        &state,
        &velocity,
        snap.reference_price,
        snap.train_vol_median,
        store,
    );

    let insufficient = record.evidence_class.eq_ignore_ascii_case("Insufficient")
        || record.sample_size < 15;

    // Engine maps Insufficient → NoTrade. Adverse soft-breach uses trajectory fallback
    // only when analogues are insufficient.
    let (final_action, authority, engine_insufficient) =
        resolve_with_fallback(adverse, &state, &velocity, &record, action);

    let reason = if matches!(
        final_action,
        CoralysPositionAction::Protect | CoralysPositionAction::Exit
    ) {
        // Executor maps Protect/Exit → GATE2X_MFE_GIVEBACK_PROTECT ledger reason.
        reason_hint.to_string()
    } else if adverse {
        format!("{reason_hint}_CORALYS_HOLD")
    } else {
        reason_hint.to_string()
    };

    Gate2xDecision {
        action: final_action,
        reason,
        coralys_action: action_name(final_action).into(),
        recommendation_action: rec_action_name(record.action).into(),
        decision_id: snap.decision_id.clone(),
        protocol: snap.params.protocol.clone(),
        soft_breach: soft,
        engine_insufficient: engine_insufficient || insufficient,
        authority: authority.into(),
    }
}

fn resolve_with_fallback(
    adverse: bool,
    state: &super::coralys_state_bridge::CausalPositionState,
    velocity: &super::coralys_state_bridge::CausalStateVelocity,
    record: &RecommendationRecordV1,
    action: CoralysPositionAction,
) -> (CoralysPositionAction, &'static str, bool) {
    // Non-adverse: trust engine (usually Hold).
    if !adverse {
        return (action, "coralys_recommendation_engine_v1", false);
    }

    match action {
        CoralysPositionAction::Protect | CoralysPositionAction::Exit => {
            (action, "coralys_recommendation_engine_v1", false)
        }
        CoralysPositionAction::Hold => {
            // Engine Hold on adverse breach — still protect via Watch-equivalent
            // only when engine was Insufficient (no analogues). Otherwise honor Hold.
            let traj = MarketPositionTrajectoryState::from_state_and_velocity(state, velocity);
            let reversing = matches!(
                traj.classify_momentum_regime(),
                MomentumTrajectoryRegime::Reversing | MomentumTrajectoryRegime::Weakening
            );
            let looks_insufficient = record.evidence_class.eq_ignore_ascii_case("Insufficient")
                || record.sample_size < 15
                || matches!(record.action, RecommendationAction::NoTrade);

            // Heavy toward Coralys: if engine said Buy/Sell → Hold, keep Hold.
            if matches!(
                record.action,
                RecommendationAction::Buy | RecommendationAction::Sell
            ) {
                return (CoralysPositionAction::Hold, "coralys_recommendation_engine_v1", false);
            }

            if looks_insufficient && reversing {
                (
                    CoralysPositionAction::Protect,
                    "adapter_insufficient_fallback",
                    true,
                )
            } else if matches!(record.action, RecommendationAction::Watch) {
                (
                    CoralysPositionAction::Protect,
                    "coralys_recommendation_engine_v1",
                    false,
                )
            } else {
                // Adverse + NoTrade already remapped to Exit above; Hold path rare.
                (action, "coralys_recommendation_engine_v1", looks_insufficient)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! Gate 2X v1.3 golden vectors — eligibility / ratchet / dwell / deep GB.
    //! Coralys adverse outcomes may be Protect|Exit|Hold(CORALYS_HOLD); reason stamps are golden.

    use super::*;
    use super::super::gate2x_features::{Gate2xParams, ProtectFloorMode};
    use std::io::Write;

    fn write_min_store(dir: &std::path::Path, ticker: &str, n: usize) {
        let path = dir.join(format!("{ticker}.jsonl"));
        let mut f = std::fs::File::create(&path).unwrap();
        for i in 0..n {
            let outcome = if i % 3 == 0 {
                "TARGET_BEFORE_RISK"
            } else {
                "RISK_BEFORE_TARGET"
            };
            let line = format!(
                "{{\"ticker\":\"{ticker}\",\"trend\":\"Bearish\",\"momentum\":\"Negative\",\"volatility\":\"present\",\"direction\":\"SHORT\",\"relative_volume_20\":2.2,\"mfe_pct\":[0.1,0.2,0.3,0.4,0.5,0.5,0.5,0.5,0.5,0.5],\"mae_pct\":[-0.1,-0.1,-0.2,-0.2,-0.3,-0.3,-0.3,-0.3,-0.3,-0.3],\"outcome\":\"{outcome}\",\"sessions_to_outcome\":3.0}}\n"
            );
            f.write_all(line.as_bytes()).unwrap();
        }
    }

    fn write_long_store(dir: &std::path::Path, ticker: &str, n: usize) {
        let path = dir.join(format!("{ticker}.jsonl"));
        let mut f = std::fs::File::create(&path).unwrap();
        for i in 0..n {
            let outcome = if i % 2 == 0 {
                "TARGET_BEFORE_RISK"
            } else {
                "RISK_BEFORE_TARGET"
            };
            let line = format!(
                "{{\"ticker\":\"{ticker}\",\"trend\":\"Bullish\",\"momentum\":\"Positive\",\"volatility\":\"present\",\"direction\":\"LONG\",\"relative_volume_20\":2.2,\"mfe_pct\":[0.1,0.2,0.3,0.4,0.5,0.5,0.5,0.5,0.5,0.5],\"mae_pct\":[-0.1,-0.1,-0.2,-0.2,-0.3,-0.3,-0.3,-0.3,-0.3,-0.3],\"outcome\":\"{outcome}\",\"sessions_to_outcome\":3.0}}\n"
            );
            f.write_all(line.as_bytes()).unwrap();
        }
    }

    fn load_store(tag: &str) -> (std::path::PathBuf, Rec001hStore) {
        let dir = std::env::temp_dir().join(format!("gate2x_v13_{tag}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        write_min_store(&dir, "BTCUSDT", 40);
        let store = Rec001hStore::load_from_dir(dir.to_str().unwrap()).unwrap();
        (dir, store)
    }

    fn base_snap(armed: bool, cur_mfe: f64, surge: VolSurgeBucket) -> Gate2xFeatureSnapshot {
        Gate2xFeatureSnapshot {
            decision_id: "PLAN-TEST-1".into(),
            instrument: "BTCUSDT".into(),
            direction: "SHORT".into(),
            reference_price: Some(60000.0),
            peak_mfe: 10.0,
            current_mfe: cur_mfe,
            peak_tp_frac: 0.55,
            hold_s: 200.0,
            arm_threshold: 1.7,
            already_armed: armed,
            vol_surge: surge,
            soft_breach_since_ms: None,
            now_ms: 1_000_000,
            params: Gate2xParams::crypto_defaults(),
            train_vol_median: 0.001,
            tp_level: 0,
            adaptive_armed: false,
        }
    }

    fn assert_adverse_fire(d: &Gate2xDecision, must_contain: &str) {
        assert!(
            d.reason.contains(must_contain),
            "expected reason containing {must_contain:?}, got {}",
            d.reason
        );
        assert!(
            matches!(
                d.action,
                CoralysPositionAction::Protect
                    | CoralysPositionAction::Exit
                    | CoralysPositionAction::Hold
            ),
            "unexpected action {:?}",
            d.action
        );
        if matches!(d.action, CoralysPositionAction::Hold) {
            assert!(
                d.reason.contains("CORALYS_HOLD"),
                "Hold on adverse must stamp CORALYS_HOLD, got {}",
                d.reason
            );
        }
    }

    // ─── Eligibility (Hold-only) ───────────────────────────────────────────

    #[test]
    fn v13_golden_disabled_holds() {
        let (_dir, store) = load_store("disabled");
        let mut snap = base_snap(true, 1.0, VolSurgeBucket::High);
        snap.params.enabled = false;
        let d = gate2x_decide(&snap, &store);
        assert_eq!(d.action, CoralysPositionAction::Hold);
        assert_eq!(d.reason, "GATE2X_DISABLED");
        assert_eq!(d.authority, "adapter_eligibility");
    }

    #[test]
    fn v13_golden_not_armed_holds() {
        let (_dir, store) = load_store("not_armed");
        let mut snap = base_snap(false, 10.0, VolSurgeBucket::High);
        snap.peak_mfe = 0.5;
        snap.peak_tp_frac = 0.1;
        let d = gate2x_decide(&snap, &store);
        assert_eq!(d.action, CoralysPositionAction::Hold);
        assert_eq!(d.reason, "GATE2X_NOT_ARMED");
        assert!(!d.soft_breach.armed);
    }

    #[test]
    fn v13_golden_min_hold_holds() {
        let (_dir, store) = load_store("min_hold");
        let mut snap = base_snap(true, 1.0, VolSurgeBucket::High);
        snap.hold_s = 30.0;
        let d = gate2x_decide(&snap, &store);
        assert_eq!(d.action, CoralysPositionAction::Hold);
        assert_eq!(d.reason, "GATE2X_MIN_HOLD");
    }

    #[test]
    fn v13_golden_above_floor_holds() {
        let (_dir, store) = load_store("above_floor");
        // peak 10 → ratchet 3.0; cur 5.0 above floor
        let snap = base_snap(true, 5.0, VolSurgeBucket::High);
        let d = gate2x_decide(&snap, &store);
        assert_eq!(d.action, CoralysPositionAction::Hold);
        assert_eq!(d.reason, "GATE2X_ABOVE_FLOOR");
        assert!(d.soft_breach.soft_breach_since_ms.is_none());
        assert!((d.soft_breach.ratchet_floor.unwrap() - 3.0).abs() < 1e-9);
    }

    // ─── v1.3 ratchet ──────────────────────────────────────────────────────

    #[test]
    fn v13_golden_ratchet_floor_stamps() {
        let (_dir, store) = load_store("ratchet");
        // arm_threshold 1.7; peak 10 → ratchet 3.0 → protect_floor 3.0
        // Mid-band (above hard 1.7) so HIGH fires before HARD_FLOOR.
        let snap = base_snap(true, 2.5, VolSurgeBucket::High);
        let d = gate2x_decide(&snap, &store);
        assert!((d.soft_breach.protect_floor.unwrap() - 3.0).abs() < 1e-9);
        assert!((d.soft_breach.ratchet_floor.unwrap() - 3.0).abs() < 1e-9);
        assert_adverse_fire(&d, "VOL_HIGH");
    }

    #[test]
    fn v13_golden_between_cost_clear_and_ratchet_still_above() {
        let (_dir, store) = load_store("between_floors");
        // cost_clear=1.7, ratchet=3.0 → protect=3.0; cur=3.5 → Hold
        let snap = base_snap(true, 3.5, VolSurgeBucket::High);
        let d = gate2x_decide(&snap, &store);
        assert_eq!(d.reason, "GATE2X_ABOVE_FLOOR");
        assert_eq!(d.action, CoralysPositionAction::Hold);
    }

    // ─── Volume / dwell tiers ──────────────────────────────────────────────

    #[test]
    fn v13_golden_high_vol_immediate() {
        let (_dir, store) = load_store("high_vol");
        // Mid-band above hard floor so VOL_HIGH (not HARD_FLOOR) stamps.
        let snap = base_snap(true, 2.5, VolSurgeBucket::High);
        let d = gate2x_decide(&snap, &store);
        assert!(d.soft_breach.soft_breach_since_ms.is_none());
        assert_adverse_fire(&d, "VOL_HIGH");
    }

    #[test]
    fn v13_golden_med_dwell_40s() {
        let (_dir, store) = load_store("med_dwell");
        // v1.5: non-deep band above hard_floor is empty at ratchet 0.30.
        // Lower deep_giveback_frac so mid-band (cur=2.5, peak=10) is non-deep.
        let mut snap = base_snap(true, 2.5, VolSurgeBucket::Med);
        snap.params.deep_giveback_frac = 0.80; // gb=0.75 < 0.80 → full MED 40s
        let d1 = gate2x_decide(&snap, &store);
        assert_eq!(d1.action, CoralysPositionAction::Hold);
        assert_eq!(d1.reason, "GATE2X_DWELL_START");
        assert_eq!(d1.soft_breach.soft_breach_dwell_ms, Some(40_000));
        let since = d1.soft_breach.soft_breach_since_ms.unwrap();

        snap.soft_breach_since_ms = Some(since);
        snap.now_ms = since + 39_999;
        let d_mid = gate2x_decide(&snap, &store);
        assert_eq!(d_mid.reason, "GATE2X_DWELLING");

        snap.now_ms = since + 40_000;
        let d2 = gate2x_decide(&snap, &store);
        assert!(!d2.reason.contains("DEEP_GB"), "got {}", d2.reason);
        assert_adverse_fire(&d2, "VOL_DWELL_MED");
    }

    #[test]
    fn v13_golden_low_dwell_90s() {
        let (_dir, store) = load_store("low_dwell");
        // Mid-band above hard 1.7; raise deep threshold so LOW keeps full 90s.
        let mut snap = base_snap(true, 2.5, VolSurgeBucket::Low);
        snap.params.deep_giveback_frac = 0.80;
        let d1 = gate2x_decide(&snap, &store);
        assert_eq!(d1.reason, "GATE2X_DWELL_START");
        assert_eq!(d1.soft_breach.soft_breach_dwell_ms, Some(90_000));
        let since = d1.soft_breach.soft_breach_since_ms.unwrap();
        snap.soft_breach_since_ms = Some(since);
        snap.now_ms = since + 90_000;
        snap.current_mfe = 2.5; // stay above hard floor through dwell end
        let d2 = gate2x_decide(&snap, &store);
        assert_adverse_fire(&d2, "VOL_DWELL_LOW");
    }

    #[test]
    fn v13_golden_deep_giveback_shortens_dwell() {
        let (_dir, store) = load_store("deep_gb");
        // MED dwell between ratchet and hard floor: peak=10 → ratchet=3, hard=1.7
        // cur=2.0 starts dwell (below ratchet, still above hard).
        let mut snap = base_snap(true, 2.0, VolSurgeBucket::Med); // gb=0.8 deep
        let d1 = gate2x_decide(&snap, &store);
        assert_eq!(d1.reason, "GATE2X_DWELL_START");
        assert_eq!(d1.soft_breach.soft_breach_dwell_ms, Some(16_000));
        let since = d1.soft_breach.soft_breach_since_ms.unwrap();
        snap.soft_breach_since_ms = Some(since);
        snap.now_ms = since + 16_000;
        // Stay above hard floor so dwell can complete (not HARD_FLOOR).
        snap.current_mfe = 2.0;
        let d2 = gate2x_decide(&snap, &store);
        assert!(
            d2.reason.contains("DEEP_GB"),
            "expected DEEP_GB stamp, got {}",
            d2.reason
        );
        assert_adverse_fire(&d2, "VOL_DWELL_MED");
    }

    #[test]
    fn v15_golden_hard_floor_skips_dwell_bleed() {
        let (_dir, store) = load_store("hard_floor");
        // peak 10 → ratchet 3.0; arm_threshold/hard 1.7
        // Soft-breach mid-band then bleed through hard floor → immediate Exit.
        let mut snap = base_snap(true, 2.5, VolSurgeBucket::Low);
        let d1 = gate2x_decide(&snap, &store);
        assert_eq!(d1.reason, "GATE2X_DWELL_START");
        let since = d1.soft_breach.soft_breach_since_ms.unwrap();
        snap.soft_breach_since_ms = Some(since);
        snap.now_ms = since + 5_000; // still inside LOW dwell
        snap.current_mfe = 0.5; // through hard floor (1.7) into loss territory
        let d2 = gate2x_decide(&snap, &store);
        assert!(
            d2.reason.contains("HARD_FLOOR"),
            "expected HARD_FLOOR, got {}",
            d2.reason
        );
        assert_adverse_fire(&d2, "HARD_FLOOR");
    }

    #[test]
    fn v13_golden_vol_confirm_off_fires_without_dwell() {
        let (_dir, store) = load_store("vol_off");
        // Mid-band: vol_confirm off → immediate (not HARD_FLOOR, not dwell).
        let mut snap = base_snap(true, 2.5, VolSurgeBucket::Low);
        snap.params.reversal_volume_confirm = false;
        let d = gate2x_decide(&snap, &store);
        assert!(d.soft_breach.soft_breach_since_ms.is_none());
        assert!(
            d.reason.starts_with("MFE_GIVEBACK_TO_COST_CLEAR"),
            "got {}",
            d.reason
        );
        assert!(!d.reason.contains("DWELL"), "got {}", d.reason);
        assert!(!d.reason.contains("HARD_FLOOR"), "got {}", d.reason);
    }

    #[test]
    fn v13_golden_entry_floor_mode() {
        let (_dir, store) = load_store("entry_floor");
        // ENTRY hard_floor=0; mid-band cur=2.5 with HIGH → VOL_HIGH (not hard).
        let mut snap = base_snap(true, 2.5, VolSurgeBucket::High);
        snap.params.protect_floor = ProtectFloorMode::Entry;
        // peak 10 → ratchet 3; protect = max(0, 3)=3; cur 2.5 below protect
        let d = gate2x_decide(&snap, &store);
        assert_adverse_fire(&d, "MFE_GIVEBACK_TO_ENTRY");
        assert_adverse_fire(&d, "VOL_HIGH");
    }

    #[test]
    fn v13_golden_json_roundtrip_snapshot() {
        let snap = base_snap(true, 1.0, VolSurgeBucket::High);
        let json = serde_json::to_string(&snap).unwrap();
        let back: Gate2xFeatureSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(back.decision_id, snap.decision_id);
        assert_eq!(back.vol_surge, VolSurgeBucket::High);
        assert_eq!(back.params.protocol, "RP-608-GATE2X-CRYPTO-MFE-GIVEBACK-001");
    }

    /// CLI contract: same JSON path as `rp608_position_decide` stdin → stdout.
    #[test]
    fn v13_golden_cli_json_stdin_stdout_contract() {
        let (_dir, store) = load_store("cli_contract");
        let snap = base_snap(true, 1.0, VolSurgeBucket::High);
        let stdin = serde_json::to_string(&snap).expect("snapshot serialize");
        let parsed: Gate2xFeatureSnapshot =
            serde_json::from_str(&stdin).expect("CLI must accept snapshot JSON");
        let decision = gate2x_decide(&parsed, &store);
        let stdout = serde_json::to_string(&decision).expect("decision serialize");
        let back: Gate2xDecision =
            serde_json::from_str(&stdout).expect("Python client must parse decision JSON");
        assert_eq!(back.decision_id, "PLAN-TEST-1");
        assert_eq!(back.protocol, "RP-608-GATE2X-CRYPTO-MFE-GIVEBACK-001");
        assert!(
            matches!(
                back.action,
                CoralysPositionAction::Hold
                    | CoralysPositionAction::Protect
                    | CoralysPositionAction::Exit
            ),
            "action {:?}",
            back.action
        );
        let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        for key in [
            "action",
            "reason",
            "coralys_action",
            "recommendation_action",
            "decision_id",
            "protocol",
            "soft_breach",
            "engine_insufficient",
            "authority",
        ] {
            assert!(v.get(key).is_some(), "missing CLI stdout key {key}");
        }
    }

    #[test]
    fn v13_golden_nse_instrument_normalize() {
        let dir = std::env::temp_dir().join("gate2x_v13_nse_norm");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        write_long_store(&dir, "RELIANCE_NS", 40);
        let store = Rec001hStore::load_from_dir(dir.to_str().unwrap()).unwrap();
        let mut snap = base_snap(true, 2.5, VolSurgeBucket::High);
        snap.instrument = "RELIANCE.NS".into();
        snap.direction = "LONG".into();
        snap.params = Gate2xParams::nse_defaults();
        let d = gate2x_decide(&snap, &store);
        assert_eq!(d.protocol, "RP-608-GATE2X-NSE-MFE-GIVEBACK-001");
        assert_adverse_fire(&d, "VOL_HIGH");
    }

    // legacy aliases kept for filter stability
    #[test]
    fn not_armed_holds() {
        v13_golden_not_armed_holds();
    }
    #[test]
    fn min_hold_holds() {
        v13_golden_min_hold_holds();
    }
    #[test]
    fn high_vol_soft_breach_protects_or_exits() {
        v13_golden_high_vol_immediate();
    }
    #[test]
    fn med_dwell_starts_then_fires() {
        v13_golden_med_dwell_40s();
    }
}
