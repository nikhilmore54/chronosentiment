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
    /// Soft-breach / arm latch carried across manage ticks (per open position).
    last_soft: Gate2xSoftBreachState,
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
    }

    /// One manage tick: merge prior soft-breach latch if snapshot omits it, then decide.
    pub fn manage_tick(&mut self, mut snap: Gate2xFeatureSnapshot) -> Gate2xDecision {
        if snap.params.protocol.is_empty() {
            snap.params.protocol = self.book.default_protocol().to_string();
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
        snap.peak_mfe = 3.0;
        snap.current_mfe = 1.6;
        let d1 = mgr.manage_tick(snap.clone());
        assert_eq!(d1.reason, "GATE2X_DWELL_START");
        assert!(mgr.last_soft_breach().soft_breach_since_ms.is_some());

        // Next tick: omit since_ms — manager must latch from prior soft state.
        snap.soft_breach_since_ms = None;
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
}
