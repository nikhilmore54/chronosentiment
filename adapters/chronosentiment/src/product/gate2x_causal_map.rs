//! Map Gate 2X tick features into Coralys causal state (adapter-owned).
//!
//! Giveback / volume / dwell geometry is encoded into labels the frozen
//! `RecommendationEngineV1` already consumes (trend, momentum, volatility,
//! relative_volume). `coralys-*` remains untouched.

use super::coralys_state_bridge::{CausalPositionState, CausalStateVelocity};
use super::gate2x_features::{Gate2xFeatureSnapshot, VolSurgeBucket};

/// Relative volume scale fed to Coralys from surge bucket.
pub fn relative_volume_from_surge(surge: VolSurgeBucket) -> f64 {
    match surge {
        VolSurgeBucket::High => 2.2,
        VolSurgeBucket::Med => 1.35,
        VolSurgeBucket::Low => 0.75,
        VolSurgeBucket::Unk => 1.0,
    }
}

/// Build direction-aligned causal (S_t, ΔS_t) from a Gate 2X snapshot.
///
/// When `adverse` is true (armed soft-breach past dwell / HIGH immediate),
/// momentum velocity and pressure are set adverse so the engine sees
/// Negative momentum / Bearish-leaning labels after direction alignment.
pub fn map_gate2x_to_causal(
    snap: &Gate2xFeatureSnapshot,
    adverse: bool,
) -> (CausalPositionState, CausalStateVelocity) {
    let peak = snap.peak_mfe.max(0.0);
    let cur = snap.current_mfe;
    let giveback = (peak - cur).max(0.0);
    let giveback_frac = if peak > 1e-9 { giveback / peak } else { 0.0 };
    let notional = snap.reference_price.unwrap_or(1.0).max(1e-9);
    // Normalize currency MFE into return-like units for DTO fields.
    let mfe_ret = peak / notional;
    let giveback_ret = giveback / notional;
    let cur_ret = cur / notional;

    let rel_vol = relative_volume_from_surge(snap.vol_surge);

    // Favorable path: positive aligned momentum. Adverse: negative.
    let (mom, pressure, d_mom, d_pressure, giveback_vel) = if adverse {
        let strength = (0.15 + giveback_frac).min(1.0);
        (
            -0.002 * strength,
            -0.25 * strength,
            -0.0015 * strength,
            -0.12 * strength,
            giveback_frac * 0.002,
        )
    } else {
        let strength = (snap.peak_tp_frac.max(0.0)).min(1.0);
        (
            0.0015 * strength.max(0.2),
            0.18 * strength.max(0.2),
            0.0008 * strength.max(0.2),
            0.06 * strength.max(0.2),
            0.0,
        )
    };

    let state = CausalPositionState {
        current_signed_return: cur_ret,
        mfe_to_date: mfe_ret,
        giveback_from_mfe: giveback_ret,
        position_age: snap.hold_s.max(0.0) as u32,
        recent_momentum: mom,
        directional_pressure_5: pressure,
        return_volatility_5: if adverse {
            snap.train_vol_median * 1.4
        } else {
            snap.train_vol_median * 0.9
        },
        relative_volume_5: rel_vol,
    };

    let velocity = CausalStateVelocity {
        delta_signed_return: if adverse { -0.0005 } else { 0.0002 },
        delta_momentum_5: d_mom,
        delta2_momentum_5: if adverse { -0.0002 } else { 0.0 },
        delta_directional_pressure_5: d_pressure,
        delta2_directional_pressure_5: if adverse { -0.03 } else { 0.0 },
        delta_return_volatility_5: if adverse { 0.0002 } else { 0.0 },
        delta_relative_volume_5: if adverse { 0.25 } else { 0.0 },
        giveback_velocity: giveback_vel,
        bars_since_mfe: if giveback > 0.0 {
            ((snap.hold_s / 60.0).max(1.0) as u32).min(60)
        } else {
            0
        },
    };

    (state, velocity)
}
