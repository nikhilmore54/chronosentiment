//! Opportunity state vocabulary and H120 reassessment — Intelligence Contract v1.
//!
//! Defines [`OpportunityState`] (the complete state vocabulary) and the
//! [`reassess_at_h120`] function that transitions LONG WAIT-MID to a late state.
//!
//! # State vocabulary
//!
//! | State | Meaning |
//! |---|---|
//! | `Enter` | Strong intraday signal — act at entry |
//! | `WaitHigh` | High-quality WAIT — act at entry |
//! | `WaitMid` | Moderate-quality LONG WAIT — monitor, reassess at H120 |
//! | `WaitLow` | Low-quality WAIT — do not act |
//! | `Avoid` | Confirmed adverse path — do not act |
//! | `EnterLate` | WAIT-MID upgraded at H120 (FAV) — act at H120 |
//! | `WaitLate` | WAIT-MID at H120 (FLAT) — continue monitoring |
//! | `AvoidLate` | WAIT-MID at H120 (ADV or MFE<0.1%) — do not act (N=1, low evidence) |
//! | `Unknown` | Unclassifiable — do not act |
//!
//! # H120 reassessment
//!
//! Only `LONG WAIT-MID` is reassessed at H120. All other states pass through
//! unchanged. The reassessment uses `h120_ret` and `mfe_h120` (both available
//! at the H120 checkpoint — see Intelligence Contract v1 §4).

use serde::{Deserialize, Serialize};

use super::classification::{Direction, MFE_FLOOR, bucket, ReturnBucket};

// ── State vocabulary ──────────────────────────────────────────────────────────

/// Complete opportunity state vocabulary (Intelligence Contract v1 §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpportunityState {
    /// Strong intraday signal — act at entry.
    Enter,
    /// High-quality WAIT — act at entry.
    WaitHigh,
    /// Moderate-quality LONG WAIT — monitor, reassess at H120.
    WaitMid,
    /// Low-quality WAIT — do not act.
    WaitLow,
    /// Confirmed adverse path — do not act.
    Avoid,
    /// WAIT-MID upgraded at H120 (FAV) — act at H120.
    EnterLate,
    /// WAIT-MID at H120 (FLAT) — continue monitoring (still positive historically).
    WaitLate,
    /// WAIT-MID at H120 (ADV or MFE<0.1%) — do not act.
    /// NOTE: N=1 in historical data. Implement but treat as low-evidence.
    AvoidLate,
    /// Unclassifiable — do not act.
    Unknown,
}

impl OpportunityState {
    /// Returns true if the product should act on this state.
    ///
    /// ACT states: Enter, WaitHigh, EnterLate, WaitLate.
    /// WaitLate is included because it has PF 6.47x historically (N=18).
    pub fn is_act(&self) -> bool {
        matches!(
            self,
            OpportunityState::Enter
                | OpportunityState::WaitHigh
                | OpportunityState::EnterLate
                | OpportunityState::WaitLate
        )
    }

    /// Returns true if this is a monitoring state (product watches but does not act yet).
    pub fn is_monitor(&self) -> bool {
        matches!(self, OpportunityState::WaitMid)
    }

    /// Returns true if the product should not act on this state.
    pub fn is_avoid(&self) -> bool {
        matches!(
            self,
            OpportunityState::WaitLow
                | OpportunityState::Avoid
                | OpportunityState::AvoidLate
                | OpportunityState::Unknown
        )
    }

    /// Human-readable label matching the Python reference implementation.
    pub fn label(&self) -> &'static str {
        match self {
            OpportunityState::Enter => "ENTER",
            OpportunityState::WaitHigh => "WAIT-HIGH",
            OpportunityState::WaitMid => "WAIT-MID",
            OpportunityState::WaitLow => "WAIT-LOW",
            OpportunityState::Avoid => "AVOID",
            OpportunityState::EnterLate => "ENTER-LATE",
            OpportunityState::WaitLate => "WAIT-LATE",
            OpportunityState::AvoidLate => "AVOID-LATE",
            OpportunityState::Unknown => "UNKNOWN",
        }
    }
}

// ── H120 reassessment ─────────────────────────────────────────────────────────

/// H120 path observation inputs. Available at the H120 checkpoint only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H120Input {
    /// Return from T0 to H120 (direction-adjusted).
    pub h120_ret: Option<f64>,
    /// Maximum favourable excursion from T0 to H120.
    pub mfe_h120: Option<f64>,
}

/// Reassess a LONG WAIT-MID opportunity at H120.
///
/// Rules (Intelligence Contract v1 §4.2):
/// 1. If `mfe_h120 < 0.001` (0.1%) → `AvoidLate` (MFE floor override)
/// 2. If `h120_ret > +0.002` (+0.2%) → `EnterLate`
/// 3. If `h120_ret < -0.002` (-0.2%) → `AvoidLate`
/// 4. Otherwise → `WaitLate`
///
/// For all other states, the state passes through unchanged.
pub fn reassess_at_h120(
    direction: Direction,
    current_state: OpportunityState,
    h120: &H120Input,
) -> OpportunityState {
    // Only LONG WAIT-MID is reassessed.
    if direction != Direction::Long || current_state != OpportunityState::WaitMid {
        return current_state;
    }

    let ret = match h120.h120_ret {
        Some(r) => r,
        None => return OpportunityState::WaitLate, // no data → continue monitoring
    };

    // MFE floor override: if max favourable excursion is negligible, avoid.
    if let Some(mfe) = h120.mfe_h120 {
        if mfe < MFE_FLOOR {
            return OpportunityState::AvoidLate;
        }
    }

    match bucket(ret) {
        ReturnBucket::Fav => OpportunityState::EnterLate,
        ReturnBucket::Adv => OpportunityState::AvoidLate,
        ReturnBucket::Flat => OpportunityState::WaitLate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn act_states_correct() {
        assert!(OpportunityState::Enter.is_act());
        assert!(OpportunityState::WaitHigh.is_act());
        assert!(OpportunityState::EnterLate.is_act());
        assert!(OpportunityState::WaitLate.is_act());
        assert!(!OpportunityState::WaitMid.is_act());
        assert!(!OpportunityState::WaitLow.is_act());
        assert!(!OpportunityState::Avoid.is_act());
        assert!(!OpportunityState::AvoidLate.is_act());
    }

    #[test]
    fn h120_fav_upgrades_wait_mid() {
        let h120 = H120Input {
            h120_ret: Some(0.005),
            mfe_h120: Some(0.006),
        };
        assert_eq!(
            reassess_at_h120(Direction::Long, OpportunityState::WaitMid, &h120),
            OpportunityState::EnterLate
        );
    }

    #[test]
    fn h120_adv_downgrades_wait_mid() {
        let h120 = H120Input {
            h120_ret: Some(-0.005),
            mfe_h120: Some(0.002),
        };
        assert_eq!(
            reassess_at_h120(Direction::Long, OpportunityState::WaitMid, &h120),
            OpportunityState::AvoidLate
        );
    }

    #[test]
    fn h120_flat_stays_wait_late() {
        let h120 = H120Input {
            h120_ret: Some(0.001),
            mfe_h120: Some(0.002),
        };
        assert_eq!(
            reassess_at_h120(Direction::Long, OpportunityState::WaitMid, &h120),
            OpportunityState::WaitLate
        );
    }

    #[test]
    fn mfe_floor_override() {
        let h120 = H120Input {
            h120_ret: Some(0.005), // would be FAV
            mfe_h120: Some(0.0005), // but MFE < floor → AVOID-LATE
        };
        assert_eq!(
            reassess_at_h120(Direction::Long, OpportunityState::WaitMid, &h120),
            OpportunityState::AvoidLate
        );
    }

    #[test]
    fn non_wait_mid_passes_through() {
        let h120 = H120Input {
            h120_ret: Some(0.010),
            mfe_h120: Some(0.012),
        };
        // SHORT WAIT-HIGH should not be reassessed
        assert_eq!(
            reassess_at_h120(Direction::Short, OpportunityState::WaitHigh, &h120),
            OpportunityState::WaitHigh
        );
        // LONG ENTER should not be reassessed
        assert_eq!(
            reassess_at_h120(Direction::Long, OpportunityState::Enter, &h120),
            OpportunityState::Enter
        );
    }

    #[test]
    fn missing_h120_ret_gives_wait_late() {
        let h120 = H120Input {
            h120_ret: None,
            mfe_h120: Some(0.005),
        };
        assert_eq!(
            reassess_at_h120(Direction::Long, OpportunityState::WaitMid, &h120),
            OpportunityState::WaitLate
        );
    }

    #[test]
    fn labels_match_python_reference() {
        assert_eq!(OpportunityState::Enter.label(), "ENTER");
        assert_eq!(OpportunityState::WaitHigh.label(), "WAIT-HIGH");
        assert_eq!(OpportunityState::WaitMid.label(), "WAIT-MID");
        assert_eq!(OpportunityState::WaitLow.label(), "WAIT-LOW");
        assert_eq!(OpportunityState::Avoid.label(), "AVOID");
        assert_eq!(OpportunityState::EnterLate.label(), "ENTER-LATE");
        assert_eq!(OpportunityState::WaitLate.label(), "WAIT-LATE");
        assert_eq!(OpportunityState::AvoidLate.label(), "AVOID-LATE");
        assert_eq!(OpportunityState::Unknown.label(), "UNKNOWN");
    }
}