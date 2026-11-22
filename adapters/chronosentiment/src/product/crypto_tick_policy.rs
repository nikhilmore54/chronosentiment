//! RP-608-SHADOW-TICK-001 — crypto tick Policy B/C (Rust port).
//!
//! Behavioural reference: `scripts/rp608_crypto_tick_policy.py`
//! Deterministic: same quote sequence → same factors/actions.
//! Tick micro imbalance threshold 0.30 is **not** baseline OF paper ±0.30 construct.
//! Authority 0%. Symbol-agnostic (BTC/ETH/SOL).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::VecDeque;

pub const PROTOCOL_ID: &str = "RP-608-SHADOW-TICK-001";
pub const DECISION_CLOCK: &str = "tick_book_ticker";

pub const MAX_VALID_SPREAD_BPS: f64 = 5.0;
pub const MAX_QUOTE_AGE_MS: f64 = 500.0;
pub const MIN_HISTORY_WARMUP_SEC: f64 = 5.0;
pub const MIN_REFRACTORY_MS: f64 = 1000.0;

pub const IMBALANCE_THRESHOLD: f64 = 0.30;
pub const VELOCITY_5S_THRESHOLD_BPS: f64 = 1.0;
pub const TREND_FILTER_BPS: f64 = 15.0;
pub const TICK_VOL_WINDOW_SEC: f64 = 30.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickQuote {
    pub update_id: i64,
    pub event_time_ms: i64,
    pub receive_time_ms: i64,
    pub bid_price: f64,
    pub bid_qty: f64,
    pub ask_price: f64,
    pub ask_qty: f64,
    #[serde(default = "default_symbol")]
    pub symbol: String,
}

fn default_symbol() -> String {
    "BTCUSDT".to_string()
}

impl TickQuote {
    pub fn mid_price(&self) -> f64 {
        (self.bid_price + self.ask_price) / 2.0
    }

    pub fn spread_bps(&self) -> f64 {
        let mid = self.mid_price();
        if mid <= 0.0 {
            return 0.0;
        }
        ((self.ask_price - self.bid_price) / mid) * 10_000.0
    }

    pub fn imbalance(&self) -> f64 {
        let denom = self.bid_qty + self.ask_qty;
        if denom <= 0.0 {
            return 0.0;
        }
        (self.bid_qty - self.ask_qty) / denom
    }

    pub fn micro_price(&self) -> f64 {
        let denom = self.bid_qty + self.ask_qty;
        if denom <= 0.0 {
            return self.mid_price();
        }
        (self.bid_price * self.ask_qty + self.ask_price * self.bid_qty) / denom
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickFeatureSnapshot {
    pub mid_price: f64,
    pub spread_bps: f64,
    pub imbalance: f64,
    pub micro_vs_mid_bps: f64,
    pub velocity_5s_bps: Option<f64>,
    pub velocity_30s_bps: Option<f64>,
    pub tick_vol_30s_bps: Option<f64>,
    pub macro_trend_16_bps: Option<f64>,
    pub macro_trend_64_bps: Option<f64>,
    pub history_sec: f64,
    pub quote_age_ms: f64,
    pub update_id: i64,
    pub event_time_ms: i64,
    pub receive_time_ms: i64,
}

#[derive(Debug, Clone)]
pub struct TickFeatureBuffer {
    window_sec: f64,
    buf: VecDeque<TickQuote>,
}

impl Default for TickFeatureBuffer {
    fn default() -> Self {
        Self {
            window_sec: 65.0,
            buf: VecDeque::new(),
        }
    }
}

impl TickFeatureBuffer {
    pub fn new(window_sec: f64) -> Self {
        Self {
            window_sec,
            buf: VecDeque::new(),
        }
    }

    pub fn append(&mut self, quote: TickQuote) {
        let cutoff = quote.event_time_ms - (self.window_sec * 1000.0) as i64;
        self.buf.push_back(quote);
        while self
            .buf
            .front()
            .map(|q| q.event_time_ms < cutoff)
            .unwrap_or(false)
        {
            self.buf.pop_front();
        }
    }

    pub fn history_sec(&self) -> f64 {
        if self.buf.len() < 2 {
            return 0.0;
        }
        let first = self.buf.front().unwrap().event_time_ms;
        let last = self.buf.back().unwrap().event_time_ms;
        (last - first) as f64 / 1000.0
    }

    pub fn velocity_bps(&self, lookback_sec: f64) -> Option<f64> {
        let latest = self.buf.back()?;
        let target_ms = latest.event_time_ms - (lookback_sec * 1000.0) as i64;
        if self.buf.front()?.event_time_ms > target_ms {
            return None;
        }
        let hist = self.buf.iter().find(|q| q.event_time_ms >= target_ms)?;
        if hist.mid_price() <= 0.0 {
            return None;
        }
        Some(((latest.mid_price() - hist.mid_price()) / hist.mid_price()) * 10_000.0)
    }

    pub fn realized_range_bps(&self, lookback_sec: f64) -> Option<f64> {
        let latest = self.buf.back()?;
        let cutoff = latest.event_time_ms - (lookback_sec * 1000.0) as i64;
        let mids: Vec<f64> = self
            .buf
            .iter()
            .filter(|q| q.event_time_ms >= cutoff && q.mid_price() > 0.0)
            .map(|q| q.mid_price())
            .collect();
        if mids.len() < 2 {
            return None;
        }
        let lo = mids.iter().cloned().fold(f64::INFINITY, f64::min);
        let hi = mids.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let mid = latest.mid_price();
        if mid <= 0.0 {
            return None;
        }
        Some(((hi - lo) / mid) * 10_000.0)
    }
}

pub fn macro_trend_bps(closes: &[f64], lookback_bars: usize) -> Option<f64> {
    if lookback_bars < 1 || closes.len() < lookback_bars {
        return None;
    }
    let past = closes[closes.len() - lookback_bars];
    let latest = *closes.last()?;
    if past <= 0.0 {
        return None;
    }
    Some(((latest - past) / past) * 10_000.0)
}

pub fn validate_tick_quote(
    quote: &TickQuote,
    prev_update_id: Option<i64>,
    max_age_ms: f64,
) -> (bool, &'static str) {
    if quote.bid_price <= 0.0 || quote.ask_price <= 0.0 {
        return (false, "INVALID_PRICE");
    }
    if quote.ask_price <= quote.bid_price {
        return (false, "INVERTED_BOOK");
    }
    if quote.bid_qty <= 0.0 || quote.ask_qty <= 0.0 {
        return (false, "INVALID_QTY");
    }
    if quote.spread_bps() > MAX_VALID_SPREAD_BPS {
        return (false, "EXCESSIVE_SPREAD");
    }
    let age = (quote.receive_time_ms - quote.event_time_ms) as f64;
    if age > max_age_ms {
        return (false, "STALE_QUOTE");
    }
    if let Some(prev) = prev_update_id {
        if quote.update_id < prev {
            return (false, "OUT_OF_ORDER");
        }
    }
    (true, "VALID")
}

pub fn build_tick_snapshot(
    quote: &TickQuote,
    buf: &TickFeatureBuffer,
    completed_closes: &[f64],
) -> TickFeatureSnapshot {
    let mid = quote.mid_price();
    let micro_bps = if mid > 0.0 {
        ((quote.micro_price() - mid) / mid) * 10_000.0
    } else {
        0.0
    };
    TickFeatureSnapshot {
        mid_price: mid,
        spread_bps: quote.spread_bps(),
        imbalance: quote.imbalance(),
        micro_vs_mid_bps: micro_bps,
        velocity_5s_bps: buf.velocity_bps(5.0),
        velocity_30s_bps: buf.velocity_bps(30.0),
        tick_vol_30s_bps: buf.realized_range_bps(TICK_VOL_WINDOW_SEC),
        macro_trend_16_bps: macro_trend_bps(completed_closes, 16),
        macro_trend_64_bps: macro_trend_bps(completed_closes, 64),
        history_sec: buf.history_sec(),
        quote_age_ms: (quote.receive_time_ms - quote.event_time_ms) as f64,
        update_id: quote.update_id,
        event_time_ms: quote.event_time_ms,
        receive_time_ms: quote.receive_time_ms,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceFactor {
    pub concept: String,
    pub present: bool,
    #[serde(default)]
    pub direction: Option<String>,
}

pub fn tick_features_to_evidence(snap: &TickFeatureSnapshot, alphabet: &str) -> Vec<EvidenceFactor> {
    let mut factors = Vec::new();

    let macro_v = snap.macro_trend_64_bps.or(snap.macro_trend_16_bps);
    let trend_dir = match macro_v {
        None => "Neutral",
        Some(m) if m > TREND_FILTER_BPS => "Bullish",
        Some(m) if m < -TREND_FILTER_BPS => "Bearish",
        Some(_) => "Neutral",
    };
    factors.push(EvidenceFactor {
        concept: "Trend".into(),
        present: true,
        direction: Some(trend_dir.into()),
    });

    let v5 = snap.velocity_5s_bps;
    let mom_dir = match v5 {
        None => "Neutral",
        Some(v) if v >= VELOCITY_5S_THRESHOLD_BPS => "Positive",
        Some(v) if v <= -VELOCITY_5S_THRESHOLD_BPS => "Negative",
        Some(_) => "Neutral",
    };
    factors.push(EvidenceFactor {
        concept: "Momentum".into(),
        present: true,
        direction: Some(mom_dir.into()),
    });

    let mut vol_present = false;
    if snap.tick_vol_30s_bps.map(|v| v >= 2.0).unwrap_or(false) {
        vol_present = true;
    }
    if snap.imbalance.abs() >= IMBALANCE_THRESHOLD {
        vol_present = true;
    }
    if v5.map(|v| v.abs() >= VELOCITY_5S_THRESHOLD_BPS).unwrap_or(false) {
        vol_present = true;
    }
    factors.push(EvidenceFactor {
        concept: "Volatility".into(),
        present: vol_present,
        direction: None,
    });

    let micro_long = snap.imbalance >= IMBALANCE_THRESHOLD && snap.micro_vs_mid_bps >= 0.0;
    let micro_short = snap.imbalance <= -IMBALANCE_THRESHOLD && snap.micro_vs_mid_bps <= 0.0;
    if micro_long {
        factors.push(EvidenceFactor {
            concept: "MicroConfirm".into(),
            present: true,
            direction: Some("BidHeavy".into()),
        });
    } else if micro_short {
        factors.push(EvidenceFactor {
            concept: "MicroConfirm".into(),
            present: true,
            direction: Some("AskHeavy".into()),
        });
    } else {
        factors.push(EvidenceFactor {
            concept: "MicroConfirm".into(),
            present: false,
            direction: None,
        });
    }

    if alphabet == "TickWithExhaustion" {
        if snap.imbalance >= 0.55 && snap.micro_vs_mid_bps > 1.0 {
            factors.push(EvidenceFactor {
                concept: "PriceRangeStretch".into(),
                present: true,
                direction: Some("ExhaustedHigh".into()),
            });
        } else if snap.imbalance <= -0.55 && snap.micro_vs_mid_bps < -1.0 {
            factors.push(EvidenceFactor {
                concept: "PriceRangeStretch".into(),
                present: true,
                direction: Some("ExhaustedLow".into()),
            });
        } else {
            factors.push(EvidenceFactor {
                concept: "PriceRangeStretch".into(),
                present: false,
                direction: None,
            });
        }

        let v30 = snap.velocity_30s_bps;
        if v30.map(|v| v > 8.0).unwrap_or(false) {
            factors.push(EvidenceFactor {
                concept: "VWAPStretch".into(),
                present: true,
                direction: Some("Overbought".into()),
            });
        } else if v30.map(|v| v < -8.0).unwrap_or(false) {
            factors.push(EvidenceFactor {
                concept: "VWAPStretch".into(),
                present: true,
                direction: Some("Oversold".into()),
            });
        } else {
            factors.push(EvidenceFactor {
                concept: "VWAPStretch".into(),
                present: false,
                direction: None,
            });
        }

        let burst = snap.imbalance.abs() >= 0.50;
        factors.push(EvidenceFactor {
            concept: "VolumeBurst".into(),
            present: burst,
            direction: if burst {
                Some("Burst".into())
            } else {
                None
            },
        });
    }

    factors
}

fn factor_present_for_pred(factor: Option<&EvidenceFactor>, pred_present: Option<bool>) -> bool {
    match pred_present {
        Some(false) => factor.map(|f| !f.present).unwrap_or(true),
        Some(true) => factor.map(|f| f.present).unwrap_or(false),
        None => true,
    }
}

fn predicate_matches(pred: &Value, factors: &[EvidenceFactor]) -> bool {
    let target = pred.get("concept").and_then(|c| c.as_str()).unwrap_or("");
    let factor = factors.iter().find(|f| f.concept == target);
    let pred_present = pred.get("present").and_then(|p| {
        if p.is_null() {
            None
        } else {
            p.as_bool()
        }
    });
    // JSON may omit present
    let pred_present = if pred.get("present").is_none() {
        None
    } else {
        pred_present
    };
    let pred_dir = pred
        .get("direction")
        .and_then(|d| if d.is_null() { None } else { d.as_str() });

    if pred_present == Some(false) && factor.is_none() {
        return pred_dir.is_none();
    }
    if pred_present == Some(false) {
        if let Some(f) = factor {
            return !f.present && pred_dir.is_none();
        }
    }
    if pred_present == Some(true) {
        if let Some(f) = factor {
            if f.present {
                return match pred_dir {
                    None => true,
                    Some(d) => f.direction.as_deref() == Some(d),
                };
            }
        }
        return false;
    }
    if pred_present.is_none() {
        if let Some(f) = factor {
            return match pred_dir {
                None => true,
                Some(d) => f.present && f.direction.as_deref() == Some(d),
            };
        }
        return pred_dir.is_none();
    }
    let _ = factor_present_for_pred;
    false
}

pub fn evaluate_policy_rules(genome: &Value, factors: &[EvidenceFactor]) -> String {
    let unmatched = genome
        .get("unmatched_action")
        .and_then(|a| a.as_str())
        .unwrap_or("NO_TRADE")
        .to_string();
    let Some(rules) = genome.get("rules").and_then(|r| r.as_array()) else {
        return unmatched;
    };
    for rule in rules {
        let when = rule
            .get("when")
            .and_then(|w| w.as_array())
            .cloned()
            .unwrap_or_default();
        if when.iter().all(|p| predicate_matches(p, factors)) {
            return rule
                .get("action")
                .and_then(|a| a.as_str())
                .unwrap_or("NO_TRADE")
                .to_string();
        }
    }
    unmatched
}

pub fn evaluate_tick_policies(
    snap: &TickFeatureSnapshot,
    policy_b_genome: &Value,
    policy_c_genome: &Value,
) -> (String, String, Vec<EvidenceFactor>, Vec<EvidenceFactor>) {
    let factors_b = tick_features_to_evidence(snap, "TickBase");
    let factors_c = tick_features_to_evidence(snap, "TickWithExhaustion");
    let action_b = evaluate_policy_rules(policy_b_genome, &factors_b);
    let action_c = evaluate_policy_rules(policy_c_genome, &factors_c);
    (action_b, action_c, factors_b, factors_c)
}

#[derive(Debug, Clone)]
pub struct TickEmissionGate {
    pub refractory_ms: f64,
    pub last_emit_ms: i64,
    pub last_action_b: Option<String>,
    pub last_action_c: Option<String>,
}

impl Default for TickEmissionGate {
    fn default() -> Self {
        Self {
            refractory_ms: MIN_REFRACTORY_MS,
            last_emit_ms: 0,
            last_action_b: None,
            last_action_c: None,
        }
    }
}

impl TickEmissionGate {
    pub fn should_emit(&self, now_ms: i64, action_b: &str, action_c: &str) -> (bool, &'static str) {
        let changed = self.last_action_b.as_deref() != Some(action_b)
            || self.last_action_c.as_deref() != Some(action_c);
        if !changed {
            return (false, "UNCHANGED");
        }
        if self.last_emit_ms > 0
            && ((now_ms - self.last_emit_ms) as f64) < self.refractory_ms
        {
            return (false, "REFRACTORY");
        }
        (true, "OK")
    }

    pub fn mark_emitted(&mut self, now_ms: i64, action_b: &str, action_c: &str) {
        self.last_emit_ms = now_ms;
        self.last_action_b = Some(action_b.to_string());
        self.last_action_c = Some(action_c.to_string());
    }
}

pub fn snapshot_to_json(snap: &TickFeatureSnapshot) -> Value {
    serde_json::json!({
        "mid_price": (snap.mid_price * 1e8).round() / 1e8,
        "spread_bps": (snap.spread_bps * 1e6).round() / 1e6,
        "imbalance": (snap.imbalance * 1e6).round() / 1e6,
        "micro_vs_mid_bps": (snap.micro_vs_mid_bps * 1e4).round() / 1e4,
        "velocity_5s_bps": snap.velocity_5s_bps.map(|v| (v * 1e4).round() / 1e4),
        "velocity_30s_bps": snap.velocity_30s_bps.map(|v| (v * 1e4).round() / 1e4),
        "tick_vol_30s_bps": snap.tick_vol_30s_bps.map(|v| (v * 1e4).round() / 1e4),
        "macro_trend_16_bps": snap.macro_trend_16_bps.map(|v| (v * 1e4).round() / 1e4),
        "macro_trend_64_bps": snap.macro_trend_64_bps.map(|v| (v * 1e4).round() / 1e4),
        "history_sec": (snap.history_sec * 1e3).round() / 1e3,
        "quote_age_ms": (snap.quote_age_ms * 1e2).round() / 1e2,
        "update_id": snap.update_id,
        "event_time_ms": snap.event_time_ms,
        "receive_time_ms": snap.receive_time_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    }

    fn load_genome(name: &str) -> Value {
        let p = workspace_root()
            .join("datasets/crypto_tick")
            .join(name);
        serde_json::from_str(&fs::read_to_string(p).unwrap()).unwrap()
    }

    fn quote(u: i64, t_ms: i64, bid: f64, ask: f64, bq: f64, aq: f64, skew: i64) -> TickQuote {
        TickQuote {
            update_id: u,
            event_time_ms: t_ms,
            receive_time_ms: t_ms + skew,
            bid_price: bid,
            bid_qty: bq,
            ask_price: ask,
            ask_qty: aq,
            symbol: "BTCUSDT".into(),
        }
    }

    #[test]
    fn validate_rejects_stale_and_inverted() {
        let q = quote(1, 1_000_000, 100.0, 100.01, 1.0, 1.0, 600);
        let (ok, reason) = validate_tick_quote(&q, None, MAX_QUOTE_AGE_MS);
        assert!(!ok);
        assert_eq!(reason, "STALE_QUOTE");

        let q2 = quote(2, 1_000_000, 100.2, 100.1, 1.0, 1.0, 10);
        let (ok2, reason2) = validate_tick_quote(&q2, None, MAX_QUOTE_AGE_MS);
        assert!(!ok2);
        assert_eq!(reason2, "INVERTED_BOOK");

        let q3 = quote(3, 1_000_000, 100.0, 100.01, 1.0, 1.0, 10);
        let (ok3, reason3) = validate_tick_quote(&q3, Some(5), MAX_QUOTE_AGE_MS);
        assert!(!ok3);
        assert_eq!(reason3, "OUT_OF_ORDER");
    }

    #[test]
    fn same_quote_sequence_same_actions() {
        let art_b = load_genome("frozen_policy_b_tick_champion.json");
        let art_c = load_genome("frozen_policy_c_tick_champion.json");
        let genome_b = art_b.get("champion_genome").cloned().unwrap();
        let genome_c = art_c.get("champion_genome").cloned().unwrap();

        let closes: Vec<f64> = (0..70).map(|i| 100.0 + i as f64 * 0.05).collect();
        let t0 = 1_700_000_000_000i64;
        let mut buf_a = TickFeatureBuffer::default();
        let mut buf_b = TickFeatureBuffer::default();
        let mut actions_a = Vec::new();
        let mut actions_b = Vec::new();

        for i in 0..40 {
            let mid = 100.0 + i as f64 * 0.02;
            let bid = mid - 0.05;
            let ask = mid + 0.05;
            let q = quote(i + 1, t0 + i * 200, bid, ask, 3.0, 1.0, 10);
            buf_a.append(q.clone());
            buf_b.append(q.clone());
            let snap_a = build_tick_snapshot(&q, &buf_a, &closes);
            let snap_b = build_tick_snapshot(&q, &buf_b, &closes);
            let (ba, ca, _, _) = evaluate_tick_policies(&snap_a, &genome_b, &genome_c);
            let (bb, cb, _, _) = evaluate_tick_policies(&snap_b, &genome_b, &genome_c);
            actions_a.push((ba, ca));
            actions_b.push((bb, cb));
        }
        assert_eq!(actions_a, actions_b);
    }

    #[test]
    fn emission_gate_refractory() {
        let mut gate = TickEmissionGate::default();
        let (ok, reason) = gate.should_emit(1000, "LONG", "NO_TRADE");
        assert!(ok);
        assert_eq!(reason, "OK");
        gate.mark_emitted(1000, "LONG", "NO_TRADE");
        let (ok2, reason2) = gate.should_emit(1500, "SHORT", "NO_TRADE");
        assert!(!ok2);
        assert_eq!(reason2, "REFRACTORY");
        let (ok3, _) = gate.should_emit(2100, "SHORT", "NO_TRADE");
        assert!(ok3);
    }
}
