//! L1 order-flow mark (±0.30) — behavioural parity with
//! `scripts/rp608_order_flow_mark.py`.
//!
//! Threshold frozen. No reverse-OF lifecycle / paper / authority changes.

use serde::{Deserialize, Serialize};

use super::event::MarketEvent;
use super::stale::StaleReason;

/// Frozen confirmatory imbalance threshold (do not change).
pub const ORDER_FLOW_IMBALANCE_THRESHOLD: f64 = 0.30;
pub const ORDER_FLOW_LABEL: &str = "ORDER_FLOW_SUPPORTED";
pub const ORDER_FLOW_EXIT_LABEL: &str = "ORDER_FLOW_EXIT";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OfSide {
    Long,
    Short,
}

impl OfSide {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_uppercase().as_str() {
            "LONG" => Some(Self::Long),
            "SHORT" => Some(Self::Short),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Long => "LONG",
            Self::Short => "SHORT",
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::Long => Self::Short,
            Self::Short => Self::Long,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfClassifyResult {
    pub order_flow_supported: bool,
    pub order_flow_label: Option<String>,
    pub order_flow_imbalance: Option<f64>,
    pub order_flow_source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfInspection {
    pub order_flow_supported: bool,
    pub order_flow_label: Option<String>,
    pub order_flow_imbalance: Option<f64>,
    pub order_flow_source: String,
    pub order_flow_side: String,
    pub order_flow_long_supported: bool,
    pub order_flow_short_supported: bool,
    pub order_flow_inspection: bool,
    pub order_flow_imbalance_threshold: f64,
}

/// OF mark durably linked to a market event + depth snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfMarkRecord {
    pub ingest_seq: u64,
    pub content_fp: u64,
    pub instrument_token: u32,
    pub exchange_timestamp_utc_ms: u64,
    pub receipt_utc_ms: u64,
    pub quote_source: String,
    pub inspection: OfInspection,
    /// False while feed is stale/reconciling — must not be shown as current.
    pub is_current: bool,
    pub suppressed_reason: Option<StaleReason>,
}

pub fn l1_imbalance(bid_qty: f64, ask_qty: f64) -> Option<f64> {
    let tot = bid_qty + ask_qty;
    if tot <= 0.0 {
        return None;
    }
    Some((bid_qty - ask_qty) / tot)
}

pub fn micro_vs_mid_bps(
    bid_price: f64,
    ask_price: f64,
    bid_qty: f64,
    ask_qty: f64,
) -> Option<f64> {
    if bid_price <= 0.0 || ask_price <= 0.0 || ask_price <= bid_price {
        return None;
    }
    let mid = 0.5 * (bid_price + ask_price);
    if mid <= 0.0 {
        return None;
    }
    let denom = bid_qty + ask_qty;
    let micro = if denom <= 0.0 {
        mid
    } else {
        (ask_price * bid_qty + bid_price * ask_qty) / denom
    };
    Some(((micro - mid) / mid) * 10_000.0)
}

fn round6(x: f64) -> f64 {
    (x * 1_000_000.0).round() / 1_000_000.0
}

pub fn classify_order_flow_support(
    side: OfSide,
    bid_qty: f64,
    ask_qty: f64,
    bid_price: f64,
    ask_price: f64,
    source: &str,
    threshold: f64,
) -> OfClassifyResult {
    let Some(imb) = l1_imbalance(bid_qty, ask_qty) else {
        return OfClassifyResult {
            order_flow_supported: false,
            order_flow_label: None,
            order_flow_imbalance: None,
            order_flow_source: if source.is_empty() {
                "unavailable".into()
            } else {
                source.into()
            },
        };
    };

    let micro_bps = micro_vs_mid_bps(bid_price, ask_price, bid_qty, ask_qty);
    let (qty_ok, micro_ok) = match side {
        OfSide::Long => (
            imb >= threshold,
            micro_bps.map(|m| m >= 0.0).unwrap_or(true),
        ),
        OfSide::Short => (
            imb <= -threshold,
            micro_bps.map(|m| m <= 0.0).unwrap_or(true),
        ),
    };
    let supported = qty_ok && micro_ok;
    OfClassifyResult {
        order_flow_supported: supported,
        order_flow_label: if supported {
            Some(ORDER_FLOW_LABEL.to_string())
        } else {
            None
        },
        order_flow_imbalance: Some(round6(imb)),
        order_flow_source: source.to_string(),
    }
}

pub fn inspect_order_flow_book(
    bid_qty: f64,
    ask_qty: f64,
    bid_price: f64,
    ask_price: f64,
    source: &str,
    primary_side: OfSide,
    threshold: f64,
) -> OfInspection {
    let long_of = classify_order_flow_support(
        OfSide::Long,
        bid_qty,
        ask_qty,
        bid_price,
        ask_price,
        source,
        threshold,
    );
    let short_of = classify_order_flow_support(
        OfSide::Short,
        bid_qty,
        ask_qty,
        bid_price,
        ask_price,
        source,
        threshold,
    );
    let primary = match primary_side {
        OfSide::Long => &long_of,
        OfSide::Short => &short_of,
    };
    let imb = primary
        .order_flow_imbalance
        .or(long_of.order_flow_imbalance)
        .or(short_of.order_flow_imbalance);
    let supported = primary.order_flow_supported;
    OfInspection {
        order_flow_supported: supported,
        order_flow_label: if supported {
            Some(ORDER_FLOW_LABEL.to_string())
        } else {
            None
        },
        order_flow_imbalance: imb,
        order_flow_source: primary.order_flow_source.clone(),
        order_flow_side: primary_side.as_str().to_string(),
        order_flow_long_supported: long_of.order_flow_supported,
        order_flow_short_supported: short_of.order_flow_supported,
        order_flow_inspection: true,
        order_flow_imbalance_threshold: threshold,
    }
}

pub fn inspect_from_market_event(ev: &MarketEvent, primary_side: OfSide) -> OfInspection {
    inspect_order_flow_book(
        ev.bid_qty,
        ev.ask_qty,
        ev.best_bid,
        ev.best_ask,
        ev.source.as_str(),
        primary_side,
        ORDER_FLOW_IMBALANCE_THRESHOLD,
    )
}

/// Opposite-side confirmatory exit signal (rule frozen; observation only in P1).
pub fn is_order_flow_exit_signal(
    active_side: OfSide,
    bid_qty: f64,
    ask_qty: f64,
    bid_price: f64,
    ask_price: f64,
    source: &str,
) -> (bool, OfClassifyResult) {
    let opp = active_side.opposite();
    let of = classify_order_flow_support(
        opp,
        bid_qty,
        ask_qty,
        bid_price,
        ask_price,
        source,
        ORDER_FLOW_IMBALANCE_THRESHOLD,
    );
    (of.order_flow_supported, of)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threshold_frozen() {
        assert!((ORDER_FLOW_IMBALANCE_THRESHOLD - 0.30).abs() < 1e-12);
    }

    #[test]
    fn long_support_at_boundary() {
        // imb = (90-10)/100 = 0.8
        let r = classify_order_flow_support(OfSide::Long, 90.0, 10.0, 99.9, 100.1, "t", 0.30);
        assert!(r.order_flow_supported);
        // boundary exactly 0.30: bid=65 ask=35 → 0.3
        let b = classify_order_flow_support(OfSide::Long, 65.0, 35.0, 99.9, 100.1, "t", 0.30);
        assert!(b.order_flow_supported);
        let below = classify_order_flow_support(OfSide::Long, 64.0, 36.0, 99.9, 100.1, "t", 0.30);
        assert!(!below.order_flow_supported);
    }
}
