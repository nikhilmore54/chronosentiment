//! Gate 2X in-process position manage (Rust shadow cutover path).
//!
//! Holds a book-local `Rec001hStore` and calls [`gate2x_decide`] on each manage
//! tick — no Python decision authority, no per-tick CLI spawn required when the
//! Rust daemon (or `--manage-loop`) owns the process.
//!
//! Books / evidence never pooled (crypto vs NSE). coralys-* frozen (PVP-003).
//! Aligned with `.cursor/rules/chronosentiment-core.mdc` §3c.

use coralys_decision::recommendation::Rec001hStore;

use super::gate2x_decide::{gate2x_decide, Gate2xDecision};
use super::gate2x_features::{Gate2xFeatureSnapshot, Gate2xParams, Gate2xSoftBreachState};

/// Which REC001h evidence book this manager serves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate2xBookKind {
    Crypto,
    Nse,
}

impl Gate2xBookKind {
    pub fn default_protocol(self) -> &'static str {
        match self {
            Self::Crypto => "RP-608-GATE2X-CRYPTO-MFE-GIVEBACK-001",
            Self::Nse => "RP-608-GATE2X-NSE-MFE-GIVEBACK-001",
        }
    }

    pub fn default_params(self) -> Gate2xParams {
        match self {
            Self::Crypto => Gate2xParams::crypto_defaults(),
            Self::Nse => Gate2xParams::nse_defaults(),
        }
    }
}

/// In-process Gate 2X manage session for one book.
pub struct Gate2xPositionManager {
    store: Rec001hStore,
    book: Gate2xBookKind,
    /// Soft-breach / arm latch carried across manage ticks (**same plan only**).
    last_soft: Gate2xSoftBreachState,
    /// `decision_id` / plan_id that owns `last_soft` — never inherit across symbols.
    last_plan_id: Option<String>,
    ticks: u64,
}

impl Gate2xPositionManager {
    pub fn load(evidence_dir: &str, book: Gate2xBookKind) -> Result<Self, String> {
        let store = Rec001hStore::load_from_dir(evidence_dir)
            .map_err(|e| format!("Rec001hStore load from {evidence_dir}: {e}"))?;
        Ok(Self {
            store,
            book,
            last_soft: Gate2xSoftBreachState::default(),
            last_plan_id: None,
            ticks: 0,
        })
    }

    pub fn book(&self) -> Gate2xBookKind {
        self.book
    }

    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    pub fn last_soft_breach(&self) -> &Gate2xSoftBreachState {
        &self.last_soft
    }

    /// Clear latch when the executor closes / replaces the open position.
    pub fn reset_position_latch(&mut self) {
        self.last_soft = Gate2xSoftBreachState::default();
        self.last_plan_id = None;
    }

    /// One manage tick: merge prior soft-breach latch if snapshot omits it, then decide.
    pub fn manage_tick(&mut self, mut snap: Gate2xFeatureSnapshot) -> Gate2xDecision {
        if snap.params.protocol.is_empty() {
            snap.params.protocol = self.book.default_protocol().to_string();
        }
        // Lifecycle loop multiplexes many plans on one manager — latch is per plan_id.
        if self.last_plan_id.as_deref() != Some(snap.decision_id.as_str()) {
            self.last_soft = Gate2xSoftBreachState::default();
            self.last_plan_id = Some(snap.decision_id.clone());
        }
        if snap.soft_breach_since_ms.is_none() {
            snap.soft_breach_since_ms = self.last_soft.soft_breach_since_ms;
        }
        if !snap.already_armed && self.last_soft.armed {
            snap.already_armed = true;
        }
        let decision = gate2x_decide(&snap, &self.store);
        self.last_soft = decision.soft_breach.clone();
        self.ticks = self.ticks.saturating_add(1);
        decision
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::gate2x_features::VolSurgeBucket;
    use super::super::coralys_state_bridge::CoralysPositionAction;
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

    fn base_snap(book: Gate2xBookKind) -> Gate2xFeatureSnapshot {
        Gate2xFeatureSnapshot {
            decision_id: "PLAN-MANAGE-1".into(),
            instrument: if book == Gate2xBookKind::Crypto {
                "BTCUSDT".into()
            } else {
                "RELIANCE.NS".into()
            },
            direction: "SHORT".into(),
            reference_price: Some(60000.0),
            peak_mfe: 10.0,
            current_mfe: 1.0,
            peak_tp_frac: 0.55,
            hold_s: 200.0,
            arm_threshold: 1.7,
            already_armed: true,
            vol_surge: VolSurgeBucket::Med,
            soft_breach_since_ms: None,
            now_ms: 1_000_000,
            params: book.default_params(),
            train_vol_median: 0.001,
            tp_level: 0,
            adaptive_armed: false,
        }
    }

    #[test]
    fn manage_tick_crypto_in_process_dwells_then_latches() {
        let dir = std::env::temp_dir().join("gate2x_manage_crypto");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        write_min_store(&dir, "BTCUSDT", 40);
        let mut mgr =
            Gate2xPositionManager::load(dir.to_str().unwrap(), Gate2xBookKind::Crypto).unwrap();
        let mut snap = base_snap(Gate2xBookKind::Crypto);
        // Mid-band above hard cost-clear: peak 10 → ratchet 3, hard≈friction; cur 2.5.
        snap.peak_mfe = 10.0;
        snap.current_mfe = 2.5;
        snap.peak_tp_frac = 0.55;
        snap.params.deep_giveback_frac = 0.80; // keep full MED 40s dwell
        let d1 = mgr.manage_tick(snap.clone());
        assert_eq!(d1.reason, "GATE2X_DWELL_START");
        assert!(mgr.last_soft_breach().soft_breach_since_ms.is_some());

        // Next tick: omit since_ms — manager must latch from prior soft state.
        snap.soft_breach_since_ms = None;
        snap.current_mfe = 2.5;
        snap.now_ms = d1.soft_breach.soft_breach_since_ms.unwrap() + 40_000;
        let d2 = mgr.manage_tick(snap);
        assert!(
            d2.reason.contains("VOL_DWELL_MED")
                || matches!(
                    d2.action,
                    CoralysPositionAction::Protect
                        | CoralysPositionAction::Exit
                        | CoralysPositionAction::Hold
                ),
            "got {}",
            d2.reason
        );
        assert_eq!(mgr.ticks(), 2);
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

    #[test]
    fn manage_tick_nse_book_protocol() {
        let dir = std::env::temp_dir().join("gate2x_manage_nse");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        write_long_store(&dir, "RELIANCE_NS", 40);
        let mut mgr =
            Gate2xPositionManager::load(dir.to_str().unwrap(), Gate2xBookKind::Nse).unwrap();
        let mut snap = base_snap(Gate2xBookKind::Nse);
        snap.direction = "LONG".into();
        snap.vol_surge = VolSurgeBucket::High;
        let d = mgr.manage_tick(snap);
        assert_eq!(d.protocol, "RP-608-GATE2X-NSE-MFE-GIVEBACK-001");
        assert_eq!(mgr.book(), Gate2xBookKind::Nse);
    }

    #[test]
    fn manage_tick_does_not_inherit_armed_across_plan_ids() {
        // Lifecycle multiplexes many plans on one manager — armed latch must not
        // leak from PLAN-A into PLAN-B (else premature HARD_FLOOR Coralys exits).
        let dir = std::env::temp_dir().join("gate2x_manage_plan_iso");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        write_min_store(&dir, "BTCUSDT", 40);
        let mut mgr =
            Gate2xPositionManager::load(dir.to_str().unwrap(), Gate2xBookKind::Crypto).unwrap();

        let mut armed = base_snap(Gate2xBookKind::Crypto);
        armed.decision_id = "PLAN-A".into();
        armed.already_armed = true;
        armed.peak_mfe = 10.0;
        armed.current_mfe = 0.5; // through hard floor
        armed.peak_tp_frac = 0.55;
        armed.vol_surge = VolSurgeBucket::High;
        let d_a = mgr.manage_tick(armed);
        assert!(
            matches!(
                d_a.action,
                CoralysPositionAction::Protect | CoralysPositionAction::Exit
            ) || d_a.soft_breach.armed,
            "PLAN-A should be armed/exit path: {} {:?}",
            d_a.reason,
            d_a.action
        );

        let mut other = base_snap(Gate2xBookKind::Crypto);
        other.decision_id = "PLAN-B".into();
        other.already_armed = false;
        other.peak_mfe = 0.5; // below arm threshold
        other.current_mfe = -1.0;
        other.peak_tp_frac = 0.05; // below arm_min_tp_frac 0.40
        other.vol_surge = VolSurgeBucket::Unk;
        let d_b = mgr.manage_tick(other);
        assert_eq!(d_b.reason, "GATE2X_NOT_ARMED");
        assert!(!d_b.soft_breach.armed);
        assert_eq!(d_b.action, CoralysPositionAction::Hold);
    }
}
