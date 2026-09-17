//! Action vocabulary and state → action mapping — Intelligence Contract v1.
//!
//! Translates an [`OpportunityState`] at a given [`Checkpoint`] into a
//! [`TradingAction`] with confidence and horizon.
//!
//! # Resolved conflict: WaitLate
//!
//! The state machine marks `WaitLate` as `is_act() == true` because it has
//! PF 6.47x historically (N=18). However, the action layer must distinguish
//! it from a confident ACT:
//!
//! - `Enter` / `WaitHigh` / `EnterLate` → `Action::Act` (HIGH confidence)
//! - `WaitLate` → `Action::Continue` (MODERATE confidence, still positive)
//! - `WaitMid` → `Action::Monitor` (MODERATE confidence, reassess at H120)
//! - `WaitLow` / `Avoid` / `AvoidLate` → `Action::Avoid` (HIGH confidence)
//!
//! This prevents the product from presenting `WaitLate` with the same urgency
//! as `Enter`. The user sees "continue monitoring" rather than "act now".
//!
//! # Checkpoint semantics
//!
//! - `Entry`: initial classification from T0 information only
//! - `H60`: path update (informational; state does not change)
//! - `H120`: reassessment checkpoint (LONG WAIT-MID may upgrade/downgrade)
//! - `H180`: confirmation (informational; state does not change)

use serde::{Deserialize, Serialize};

use super::state_machine::OpportunityState;

// ── Checkpoint ────────────────────────────────────────────────────────────────

/// Decision checkpoint. Each checkpoint may only use information available
/// at that point in time (see Intelligence Contract v1 §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Checkpoint {
    Entry,
    H60,
    H120,
    H180,
}

impl Checkpoint {
    pub fn label(&self) -> &'static str {
        match self {
            Checkpoint::Entry => "ENTRY",
            Checkpoint::H60 => "H60",
            Checkpoint::H120 => "H120",
            Checkpoint::H180 => "H180",
        }
    }
}

// ── Action vocabulary ─────────────────────────────────────────────────────────

/// Product action code (Intelligence Contract v1 §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// Enter or maintain position. High-confidence ACT states only.
    Act,
    /// Observe path; do not act yet. Used for WaitMid at entry.
    Monitor,
    /// Do not enter; exit if already in.
    Avoid,
    /// Promote from Monitor to Act (H120 FAV for WaitMid → EnterLate).
    Upgrade,
    /// Maintain current monitoring posture (WaitLate — still positive but not urgent).
    Continue,
}

impl Action {
    pub fn label(&self) -> &'static str {
        match self {
            Action::Act => "ACT",
            Action::Monitor => "MONITOR",
            Action::Avoid => "AVOID",
            Action::Upgrade => "UPGRADE",
            Action::Continue => "CONTINUE",
        }
    }
}

// ── Confidence ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    High,
    Moderate,
    Low,
}

impl Confidence {
    pub fn label(&self) -> &'static str {
        match self {
            Confidence::High => "HIGH",
            Confidence::Moderate => "MODERATE",
            Confidence::Low => "LOW",
        }
    }
}

// ── Trading action (full output) ──────────────────────────────────────────────

/// Complete product action output for a single checkpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingAction {
    pub checkpoint: Checkpoint,
    pub state: OpportunityState,
    pub action: Action,
    pub confidence: Confidence,
    /// Human-readable next decision point.
    pub horizon: &'static str,
    /// One-sentence explanation.
    pub why: &'static str,
    /// One-sentence risk note.
    pub risk: &'static str,
}

// ── State → action mapping ────────────────────────────────────────────────────

/// Map an [`OpportunityState`] at a given [`Checkpoint`] to a [`TradingAction`].
///
/// The `prior_state` parameter is used at H120 to detect upgrades
/// (WaitMid → EnterLate produces `Action::Upgrade`).
pub fn resolve_action(
    checkpoint: Checkpoint,
    state: OpportunityState,
    prior_state: Option<OpportunityState>,
) -> TradingAction {
    let (action, confidence, horizon, why, risk) =
        action_for_state(checkpoint, state, prior_state);

    TradingAction {
        checkpoint,
        state,
        action,
        confidence,
        horizon,
        why,
        risk,
    }
}

fn action_for_state(
    checkpoint: Checkpoint,
    state: OpportunityState,
    prior_state: Option<OpportunityState>,
) -> (Action, Confidence, &'static str, &'static str, &'static str) {
    match state {
        // ── High-confidence ACT states ────────────────────────────────────────
        OpportunityState::Enter => (
            Action::Act,
            Confidence::High,
            "Act within 5-hour session",
            "H15 and H60 both favourable — strong intraday signal.",
            "Monitor for early adverse reversal; avg loss ~0.5–0.9% if wrong.",
        ),

        OpportunityState::WaitHigh => (
            Action::Act,
            Confidence::High,
            "Act within 5-hour session",
            "High-quality WAIT with sustained favourable path.",
            "PF >6x historically; avg loss small.",
        ),

        OpportunityState::EnterLate => {
            // Detect upgrade: was WaitMid, now EnterLate
            let is_upgrade = prior_state == Some(OpportunityState::WaitMid);
            if is_upgrade {
                (
                    Action::Upgrade,
                    Confidence::High,
                    "Act now — H120 confirms opportunity",
                    "H120 FAV upgrades WAIT-MID to ENTER-LATE. PF 11.67x historically.",
                    "Avg loss small; 70% win rate at H300.",
                )
            } else {
                (
                    Action::Act,
                    Confidence::High,
                    "Act within remaining session",
                    "H120 FAV confirmed — opportunity developing as expected.",
                    "PF 11.67x historically; 70% win rate.",
                )
            }
        }

        // ── Monitoring state ──────────────────────────────────────────────────
        OpportunityState::WaitMid => (
            Action::Monitor,
            Confidence::Moderate,
            "Reassess at H120 (2 hours)",
            "Moderate-quality LONG opportunity — insufficient early signal to act.",
            "Winners and losers indistinguishable before H120; do not act at entry.",
        ),

        // ── Continue monitoring (still positive, not urgent) ──────────────────
        OpportunityState::WaitLate => (
            Action::Continue,
            Confidence::Moderate,
            "Continue monitoring; reassess at H180",
            "H120 FLAT — opportunity still developing. PF 6.47x historically.",
            "50% win rate; do not exit on flat path alone.",
        ),

        // ── Avoid states ──────────────────────────────────────────────────────
        OpportunityState::WaitLow => (
            Action::Avoid,
            Confidence::High,
            "No action required",
            "Low-quality WAIT — path not moving favourably during session.",
            "PF 0.04–0.67x historically; acting here destroys value.",
        ),

        OpportunityState::Avoid => (
            Action::Avoid,
            Confidence::High,
            "No action required",
            "H15 and H60 both adverse — confirmed negative intraday path.",
            "PF 0.08–0.23x historically; strong avoidance signal.",
        ),

        OpportunityState::AvoidLate => (
            Action::Avoid,
            Confidence::Moderate,
            "No action required",
            "H120 ADV or MFE below floor — adverse path at 2-hour mark.",
            "N=1 historically; avoidance rule not yet fully evidenced.",
        ),

        OpportunityState::Unknown => (
            Action::Avoid,
            Confidence::Low,
            "No action required",
            "Unclassifiable state — insufficient information.",
            "No historical reference available.",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_produces_act() {
        let ta = resolve_action(Checkpoint::Entry, OpportunityState::Enter, None);
        assert_eq!(ta.action, Action::Act);
        assert_eq!(ta.confidence, Confidence::High);
    }

    #[test]
    fn wait_mid_produces_monitor() {
        let ta = resolve_action(Checkpoint::Entry, OpportunityState::WaitMid, None);
        assert_eq!(ta.action, Action::Monitor);
        assert_eq!(ta.confidence, Confidence::Moderate);
    }

    #[test]
    fn wait_late_produces_continue_not_act() {
        // Critical: WaitLate must NOT produce Action::Act even though
        // state_machine::is_act() returns true for it.
        let ta = resolve_action(Checkpoint::H120, OpportunityState::WaitLate, None);
        assert_eq!(ta.action, Action::Continue);
        assert_eq!(ta.confidence, Confidence::Moderate);
    }

    #[test]
    fn enter_late_upgrade_from_wait_mid() {
        let ta = resolve_action(
            Checkpoint::H120,
            OpportunityState::EnterLate,
            Some(OpportunityState::WaitMid),
        );
        assert_eq!(ta.action, Action::Upgrade);
        assert_eq!(ta.confidence, Confidence::High);
    }

    #[test]
    fn enter_late_without_prior_is_act() {
        let ta = resolve_action(Checkpoint::H120, OpportunityState::EnterLate, None);
        assert_eq!(ta.action, Action::Act);
    }

    #[test]
    fn avoid_states_produce_avoid() {
        for state in [
            OpportunityState::Avoid,
            OpportunityState::WaitLow,
            OpportunityState::AvoidLate,
            OpportunityState::Unknown,
        ] {
            let ta = resolve_action(Checkpoint::Entry, state, None);
            assert_eq!(ta.action, Action::Avoid, "Expected Avoid for {state:?}");
        }
    }

    #[test]
    fn checkpoint_labels_match_python() {
        assert_eq!(Checkpoint::Entry.label(), "ENTRY");
        assert_eq!(Checkpoint::H60.label(), "H60");
        assert_eq!(Checkpoint::H120.label(), "H120");
        assert_eq!(Checkpoint::H180.label(), "H180");
    }

    #[test]
    fn action_labels_match_python() {
        assert_eq!(Action::Act.label(), "ACT");
        assert_eq!(Action::Monitor.label(), "MONITOR");
        assert_eq!(Action::Avoid.label(), "AVOID");
        assert_eq!(Action::Upgrade.label(), "UPGRADE");
        assert_eq!(Action::Continue.label(), "CONTINUE");
    }
}