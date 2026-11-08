//! P3 — Push-driven paper-lifecycle **observations**.
//!
//! Mirrors `scripts/rp608_nse_of_paper_ledger` reverse-OF / persistence rules
//! for differential parity. Does **not** write `of_trade_outcomes.jsonl`.
//! Python remains the authoritative ledger writer until a separate cutover.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::event::MarketEvent;
use super::of_mark::{
    classify_order_flow_support, is_order_flow_exit_signal, OfSide, ORDER_FLOW_EXIT_LABEL,
    ORDER_FLOW_IMBALANCE_THRESHOLD,
};
use super::stale::StaleReason;

pub const OF_PAPER_MIN_HOLD_MS: i64 = 300_000;
pub const OF_PAPER_OPPOSITE_PERSIST_MS: i64 = 30_000;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenPaperObs {
    pub symbol: String,
    pub direction: String,
    pub of_trade_id: String,
    pub activated_ms: i64,
    pub entry_price: f64,
    pub active: bool,
    pub opposite_exit_seen_ms: Option<i64>,
    pub last_book_imbalance: Option<f64>,
    pub order_flow_imbalance: Option<f64>,
    pub close_suppressed_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaperObsAction {
    Hold,
    CloseSuppressed,
    CloseOppositeSideOrderFlow,
    /// Observation skipped — feed not live.
    SuppressedStale,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaperLifecycleObservation {
    pub symbol: String,
    pub of_trade_id: String,
    pub action: PaperObsAction,
    pub deny_reason: Option<String>,
    pub exit_signal: bool,
    pub opposite_side: Option<String>,
    pub order_flow_exit_label: Option<String>,
    pub same_side_supported: bool,
    pub order_flow_imbalance: Option<f64>,
    pub ingest_seq: u64,
    pub content_fp: u64,
    pub exchange_timestamp_utc_ms: u64,
    pub receipt_utc_ms: u64,
    pub now_ms: i64,
    pub is_current: bool,
    pub suppressed_reason: Option<StaleReason>,
    /// Explicit: this module never owns ledger writes.
    pub ledger_writer: String,
}

/// Frozen close gate — matches `NseOfPaperLedger._close_allowed`.
pub fn close_allowed(
    activated_ms: i64,
    opposite_exit_seen_ms: Option<i64>,
    now_ms: i64,
    exit_signal: bool,
    min_hold_ms: i64,
    persist_ms: i64,
) -> (bool, String, Option<i64>) {
    let held = (now_ms - activated_ms).max(0);
    if held < min_hold_ms {
        return (
            false,
            format!("min_hold_not_met_{held}ms<{min_hold_ms}ms"),
            opposite_exit_seen_ms,
        );
    }
    if exit_signal {
        match opposite_exit_seen_ms {
            None => {
                return (
                    false,
                    format!("opposite_persist_arming_need_{persist_ms}ms"),
                    Some(now_ms),
                );
            }
            Some(seen) => {
                let dt = now_ms - seen;
                if dt < persist_ms {
                    return (
                        false,
                        format!("opposite_persist_{dt}ms<{persist_ms}ms"),
                        Some(seen),
                    );
                }
                return (true, "ok".into(), Some(seen));
            }
        }
    }
    (true, "ok".into(), opposite_exit_seen_ms)
}

/// Observation engine: consumes push depth/OF events for open paper trades.
/// Never appends to the production ledger.
#[derive(Debug, Default)]
pub struct PaperLifecycleObsEngine {
    opens: BTreeMap<String, OpenPaperObs>,
    feed_stale: bool,
    stale_reason: Option<StaleReason>,
    observations: Vec<PaperLifecycleObservation>,
}

impl PaperLifecycleObsEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn observations(&self) -> &[PaperLifecycleObservation] {
        &self.observations
    }

    pub fn open(&self, symbol: &str) -> Option<&OpenPaperObs> {
        self.opens.get(symbol)
    }

    /// Seed an already-opened paper trade (opened by Python writer).
    pub fn mirror_open(&mut self, open: OpenPaperObs) {
        self.opens.insert(open.symbol.clone(), open);
    }

    pub fn on_disconnect(&mut self) {
        self.feed_stale = true;
        self.stale_reason = Some(StaleReason::Disconnect);
    }

    pub fn begin_reconcile(&mut self) {
        self.feed_stale = true;
        self.stale_reason = Some(StaleReason::Disconnect);
    }

    pub fn complete_reconcile(&mut self) {
        self.feed_stale = false;
        self.stale_reason = None;
    }

    /// Tick one open paper trade from a linked market event (depth/OF).
    pub fn on_depth_event(
        &mut self,
        symbol: &str,
        ev: &MarketEvent,
        now_ms: i64,
    ) -> Option<PaperLifecycleObservation> {
        let prev = self.opens.get(symbol)?.clone();
        if !prev.active {
            return None;
        }

        if self.feed_stale {
            let obs = PaperLifecycleObservation {
                symbol: symbol.to_string(),
                of_trade_id: prev.of_trade_id.clone(),
                action: PaperObsAction::SuppressedStale,
                deny_reason: Some("feed_stale_or_reconciling".into()),
                exit_signal: false,
                opposite_side: None,
                order_flow_exit_label: None,
                same_side_supported: false,
                order_flow_imbalance: None,
                ingest_seq: ev.ingest_seq,
                content_fp: ev.content_fp,
                exchange_timestamp_utc_ms: ev.exchange_timestamp_utc_ms,
                receipt_utc_ms: ev.receipt_utc_ms,
                now_ms,
                is_current: false,
                suppressed_reason: self.stale_reason,
                ledger_writer: "python_authoritative".into(),
            };
            self.observations.push(obs.clone());
            return Some(obs);
        }

        let side = OfSide::parse(&prev.direction).unwrap_or(OfSide::Long);
        let (exit_fire, exit_cls) = is_order_flow_exit_signal(
            side,
            ev.bid_qty,
            ev.ask_qty,
            ev.best_bid,
            ev.best_ask,
            ev.source.as_str(),
        );
        let same = classify_order_flow_support(
            side,
            ev.bid_qty,
            ev.ask_qty,
            ev.best_bid,
            ev.best_ask,
            ev.source.as_str(),
            ORDER_FLOW_IMBALANCE_THRESHOLD,
        );

        let mut open = prev;
        if let Some(imb) = same.order_flow_imbalance {
            open.last_book_imbalance = Some(imb);
            if same.order_flow_supported {
                open.order_flow_imbalance = Some(imb);
            }
        }

        let mut action = PaperObsAction::Hold;
        let mut deny: Option<String> = None;
        let mut opp_side: Option<String> = None;
        let mut exit_label: Option<String> = None;

        if exit_fire {
            let (allowed, reason, new_seen) = close_allowed(
                open.activated_ms,
                open.opposite_exit_seen_ms,
                now_ms,
                true,
                OF_PAPER_MIN_HOLD_MS,
                OF_PAPER_OPPOSITE_PERSIST_MS,
            );
            open.opposite_exit_seen_ms = new_seen;
            opp_side = Some(side.opposite().as_str().to_string());
            if allowed {
                action = PaperObsAction::CloseOppositeSideOrderFlow;
                exit_label = Some(ORDER_FLOW_EXIT_LABEL.to_string());
                open.active = false;
                open.close_suppressed_reason = None;
            } else {
                action = PaperObsAction::CloseSuppressed;
                deny = Some(reason.clone());
                open.close_suppressed_reason = Some(reason);
            }
        } else if open.opposite_exit_seen_ms.is_some() {
            open.opposite_exit_seen_ms = None;
            open.close_suppressed_reason = None;
        }

        self.opens.insert(symbol.to_string(), open.clone());

        let obs = PaperLifecycleObservation {
            symbol: symbol.to_string(),
            of_trade_id: open.of_trade_id,
            action,
            deny_reason: deny,
            exit_signal: exit_fire,
            opposite_side: opp_side,
            order_flow_exit_label: exit_label,
            same_side_supported: same.order_flow_supported,
            order_flow_imbalance: same.order_flow_imbalance.or(exit_cls.order_flow_imbalance),
            ingest_seq: ev.ingest_seq,
            content_fp: ev.content_fp,
            exchange_timestamp_utc_ms: ev.exchange_timestamp_utc_ms,
            receipt_utc_ms: ev.receipt_utc_ms,
            now_ms,
            is_current: true,
            suppressed_reason: None,
            ledger_writer: "python_authoritative".into(),
        };
        let _ = exit_cls;
        self.observations.push(obs.clone());
        Some(obs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn min_hold_and_persist_frozen() {
        assert_eq!(OF_PAPER_MIN_HOLD_MS, 300_000);
        assert_eq!(OF_PAPER_OPPOSITE_PERSIST_MS, 30_000);
    }

    #[test]
    fn close_allowed_arms_then_fires() {
        let act = 1_000_000_i64;
        let (ok1, r1, seen) = close_allowed(act, None, act + 400_000, true, 300_000, 30_000);
        assert!(!ok1);
        assert!(r1.contains("arming"));
        assert_eq!(seen, Some(act + 400_000));
        let (ok2, _, _) = close_allowed(act, seen, act + 400_000 + 29_000, true, 300_000, 30_000);
        assert!(!ok2);
        let (ok3, r3, _) = close_allowed(act, seen, act + 400_000 + 30_000, true, 300_000, 30_000);
        assert!(ok3);
        assert_eq!(r3, "ok");
    }
}
