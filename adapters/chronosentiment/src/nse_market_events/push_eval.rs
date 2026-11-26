//! P2 — Push-driven watchlist (≤20) + recommendation evaluation.
//!
//! Consumes P0/P1 ordered events. Frozen qualification rules (traj15 ≥ 0.05,
//! bars, entry cutoff, OF ±0.30 confirmatory only). No production cutover.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::event::MarketEvent;
use super::of_mark::{
    inspect_from_market_event, OfInspection, OfMarkRecord, OfSide,
};
use super::stale::StaleReason;

pub const TRAJ15_ENTRY_THRESHOLD: f64 = 0.05;
pub const MIN_BARS_FOR_ENTRY: usize = 5;
pub const MAX_NEW_ENTRY_BAR: u32 = 19;
pub const MANDATORY_SQUARE_OFF_BAR: u32 = 23;
pub const WATCHLIST_TOP_N: usize = 20;
pub const MAX_QUOTE_AGE_MS: f64 = 60_000.0;

/// Live traj15 — matches `run_rp608_nse_shadow_daemon.evaluate_symbol_bar`.
pub fn traj15_live(closes: &[f64]) -> f64 {
    if closes.len() < 2 {
        return 0.0;
    }
    let lookback = closes.len().min(15);
    let c0 = closes[closes.len() - lookback];
    let c1 = closes[closes.len() - 1];
    let ret = (c1 - c0) / (c0 + 1e-9);
    let mut sum_sq = 0.0;
    let mut n = 0usize;
    let start = closes.len() - lookback + 1;
    for i in start..closes.len() {
        let prev = closes[i - 1];
        let d = (closes[i] - prev) / (prev + 1e-9);
        sum_sq += d * d;
        n += 1;
    }
    let vol = if n > 0 {
        (sum_sq / n as f64).sqrt()
    } else {
        1e-3
    };
    let denom = (vol * (lookback as f64).sqrt() * 2.5) + 1e-9;
    (ret / denom).tanh()
}

/// EOD / watch traj15 — matches `rp608_nse_next_session_recs._traj15` (last ≤5).
pub fn traj15_watch_eod(closes: &[f64]) -> f64 {
    if closes.len() < 2 {
        return 0.0;
    }
    let window: Vec<f64> = if closes.len() >= 5 {
        closes[closes.len() - 5..].to_vec()
    } else {
        closes.to_vec()
    };
    if window.len() < 2 || window[0] <= 0.0 {
        return 0.0;
    }
    let ret = (window[window.len() - 1] / window[0]) - 1.0;
    let denom = 0.01 * ((window.len() - 1).max(1) as f64).sqrt();
    if denom <= 0.0 {
        return 0.0;
    }
    (ret / denom).tanh()
}

pub fn session_return_bps(closes: &[f64]) -> f64 {
    if closes.len() < 2 || closes[0] <= 0.0 {
        return 0.0;
    }
    ((closes[closes.len() - 1] / closes[0]) - 1.0) * 10_000.0
}

pub fn watch_score(traj15_eod: f64, session_return_bps: f64) -> f64 {
    traj15_eod + (session_return_bps / 10_000.0) * 0.25
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchCandidateInput {
    pub symbol: String,
    pub closes: Vec<f64>,
    pub bid_qty: f64,
    pub ask_qty: f64,
    pub bid_price: f64,
    pub ask_price: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchlistEntry {
    pub symbol: String,
    pub side: String,
    pub decision: String,
    pub status: String,
    pub score: f64,
    pub traj15_eod: f64,
    pub session_return_bps: f64,
    pub close_price: f64,
    pub session_bars: usize,
    pub order_flow_supported: bool,
    pub order_flow_label: Option<String>,
    pub order_flow_imbalance: Option<f64>,
    pub order_flow_source: String,
}

/// Rank next-session watchlist (LONG-only, top_n ≤ 20). Deterministic.
pub fn rank_watchlist(inputs: &[WatchCandidateInput], top_n: usize) -> Vec<WatchlistEntry> {
    let mut cands: Vec<WatchlistEntry> = Vec::new();
    for inp in inputs {
        if inp.closes.len() < MIN_BARS_FOR_ENTRY {
            continue;
        }
        let traj = traj15_watch_eod(&inp.closes);
        let sess = session_return_bps(&inp.closes);
        if traj < TRAJ15_ENTRY_THRESHOLD && sess < 25.0 {
            continue;
        }
        let score = watch_score(traj, sess);
        let of = inspect_from_market_event(
            &MarketEvent {
                instrument_token: 0,
                last_price: *inp.closes.last().unwrap_or(&0.0),
                volume: 0.0,
                exchange_timestamp_sec: 0,
                exchange_timestamp_utc_ms: 0,
                receipt_utc_ms: 0,
                quote_age_ms: 0.0,
                best_bid: inp.bid_price,
                best_ask: inp.ask_price,
                bid_qty: inp.bid_qty,
                ask_qty: inp.ask_qty,
                depth_buy: [super::event::DepthLevel {
                    price: inp.bid_price,
                    quantity: inp.bid_qty,
                }; 5],
                depth_sell: [super::event::DepthLevel {
                    price: inp.ask_price,
                    quantity: inp.ask_qty,
                }; 5],
                source: super::event::SourceProvenance::FixtureReplay,
                ingest_seq: 0,
                content_fp: 0,
            },
            OfSide::Long,
        );
        cands.push(WatchlistEntry {
            symbol: inp.symbol.clone(),
            side: "LONG".into(),
            decision: "WATCH_NEXT_SESSION".into(),
            status: "WATCH_NEXT_SESSION".into(),
            score: round6(score),
            traj15_eod: round6(traj),
            session_return_bps: round4(sess),
            close_price: round4(*inp.closes.last().unwrap_or(&0.0)),
            session_bars: inp.closes.len(),
            order_flow_supported: of.order_flow_supported,
            order_flow_label: of.order_flow_label,
            order_flow_imbalance: of.order_flow_imbalance,
            order_flow_source: of.order_flow_source,
        });
    }
    cands.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.symbol.cmp(&b.symbol))
    });
    let n = top_n.min(WATCHLIST_TOP_N);
    cands.into_iter().take(n).collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecDecision {
    RecommendLong,
    NoTrade,
}

impl RecDecision {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RecommendLong => "RECOMMEND_LONG",
            Self::NoTrade => "NO_TRADE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecommendationEval {
    pub symbol: String,
    pub bar_idx: u32,
    pub traj15: f64,
    pub signal_entry: bool,
    pub decision: String,
    pub decision_code: String,
    pub explanation: String,
    pub ingest_seq: Option<u64>,
    pub content_fp: Option<u64>,
    pub order_flow: Option<OfInspection>,
    pub is_current: bool,
    pub suppressed_reason: Option<StaleReason>,
}

/// Frozen recommendation decision tree (observational; no Estimand A open/close).
pub fn evaluate_recommendation(
    symbol: &str,
    bar_idx: u32,
    closes: &[f64],
    has_valid_fresh_quote: bool,
    quote_missing: bool,
    quote_stale: bool,
    quote_bad_prices: bool,
    quote_crossed: bool,
    existing_position_from_prior_bar: bool,
    prior_entry_bar: Option<u32>,
) -> RecommendationEval {
    let traj15 = traj15_live(closes);
    let signal_entry = traj15 >= TRAJ15_ENTRY_THRESHOLD && closes.len() >= MIN_BARS_FOR_ENTRY;

    let (decision_code, explanation, decision) = if existing_position_from_prior_bar {
        (
            "EXISTING_POSITION_ACTIVE",
            format!(
                "Existing active trade open since bar {}",
                prior_entry_bar.unwrap_or(0)
            ),
            RecDecision::NoTrade,
        )
    } else if bar_idx >= MANDATORY_SQUARE_OFF_BAR {
        (
            "INTRADAY_BOUNDARY_EXCLUDED",
            format!(
                "Session bar {bar_idx} >= {MANDATORY_SQUARE_OFF_BAR} (15:00–15:15 closing window / mandatory square-off)"
            ),
            RecDecision::NoTrade,
        )
    } else if bar_idx > MAX_NEW_ENTRY_BAR {
        (
            "INTRADAY_BOUNDARY_EXCLUDED",
            format!(
                "Session bar {bar_idx} > {MAX_NEW_ENTRY_BAR} (entry cutoff so 60m horizon completes by 15:00 close-window)"
            ),
            RecDecision::NoTrade,
        )
    } else if closes.len() < MIN_BARS_FOR_ENTRY {
        (
            "COLD_START_INSUFFICIENT_HISTORY",
            format!("Warmup history: {}/5 bars required", closes.len()),
            RecDecision::NoTrade,
        )
    } else if traj15 < TRAJ15_ENTRY_THRESHOLD {
        (
            "SUB_THRESHOLD_MOMENTUM",
            format!("Momentum traj15={traj15:+.4} below entry threshold +0.0500"),
            RecDecision::NoTrade,
        )
    } else if !has_valid_fresh_quote {
        let explanation = if quote_missing {
            "Missing top-of-book quote fails Gate G4 execution feasibility".into()
        } else if quote_stale {
            format!(
                "Stale top-of-book quote (age > {MAX_QUOTE_AGE_MS}ms) fails Gate G4 execution feasibility"
            )
        } else if quote_bad_prices {
            "Zero top-of-book bid/ask fails Gate G4 execution feasibility".into()
        } else if quote_crossed {
            "Crossed or locked top-of-book quote fails Gate G4 execution feasibility".into()
        } else {
            "Invalid top-of-book quote fails Gate G4 execution feasibility".into()
        };
        (
            "QUOTE_FEASIBILITY_FAILURE",
            explanation,
            RecDecision::NoTrade,
        )
    } else {
        (
            "QUALIFIED_MOMENTUM_BREAKOUT",
            format!(
                "Momentum breakout traj15={traj15:+.4} >= +0.0500 with {} bars history",
                closes.len()
            ),
            RecDecision::RecommendLong,
        )
    };

    RecommendationEval {
        symbol: symbol.to_string(),
        bar_idx,
        traj15: round6(traj15),
        signal_entry,
        decision: decision.as_str().to_string(),
        decision_code: decision_code.to_string(),
        explanation,
        ingest_seq: None,
        content_fp: None,
        order_flow: None,
        is_current: true,
        suppressed_reason: None,
    }
}

fn round6(x: f64) -> f64 {
    (x * 1_000_000.0).round() / 1_000_000.0
}
fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

#[derive(Debug, Clone)]
struct SymbolState {
    closes: Vec<f64>,
    last_bar_idx: u32,
    last_eval: Option<RecommendationEval>,
    last_of: Option<OfInspection>,
    last_price: Option<f64>,
}

/// Push evaluator: ≤20 watch symbols + recommendation state updates from events.
#[derive(Debug)]
pub struct PushEvalEngine {
    watch_symbols: Vec<String>,
    states: BTreeMap<String, SymbolState>,
    /// token → symbol
    token_map: BTreeMap<u32, String>,
    feed_stale: bool,
    stale_reason: Option<StaleReason>,
}

impl PushEvalEngine {
    pub fn new(watch_symbols: Vec<String>) -> Self {
        let mut watch_symbols = watch_symbols;
        watch_symbols.truncate(WATCHLIST_TOP_N);
        watch_symbols.sort();
        let mut states = BTreeMap::new();
        for s in &watch_symbols {
            states.insert(
                s.clone(),
                SymbolState {
                    closes: Vec::new(),
                    last_bar_idx: 0,
                    last_eval: None,
                    last_of: None,
                    last_price: None,
                },
            );
        }
        Self {
            watch_symbols,
            states,
            token_map: BTreeMap::new(),
            feed_stale: false,
            stale_reason: None,
        }
    }

    pub fn watch_symbols(&self) -> &[String] {
        &self.watch_symbols
    }

    pub fn bind_token(&mut self, token: u32, symbol: &str) {
        self.token_map.insert(token, symbol.to_string());
    }

    pub fn on_disconnect(&mut self) {
        self.feed_stale = true;
        self.stale_reason = Some(StaleReason::Disconnect);
        self.suppress_current();
    }

    pub fn begin_reconcile(&mut self) {
        self.feed_stale = true;
        self.stale_reason = Some(StaleReason::Disconnect);
        self.suppress_current();
    }

    pub fn complete_reconcile(&mut self) {
        self.feed_stale = false;
        self.stale_reason = None;
        // Current restored only by subsequent live events.
        self.suppress_current();
    }

    fn suppress_current(&mut self) {
        for st in self.states.values_mut() {
            if let Some(ev) = st.last_eval.as_mut() {
                ev.is_current = false;
                ev.suppressed_reason = self.stale_reason;
            }
        }
    }

    /// Bar-completion event: append close, recompute recommendation.
    pub fn on_bar_complete(
        &mut self,
        symbol: &str,
        bar_idx: u32,
        close: f64,
        has_valid_fresh_quote: bool,
        quote_flags: QuoteFlags,
    ) -> Option<RecommendationEval> {
        let st = self.states.get_mut(symbol)?;
        st.closes.push(close);
        if st.closes.len() > 60 {
            st.closes.remove(0);
        }
        st.last_bar_idx = bar_idx;
        st.last_price = Some(close);
        let mut eval = evaluate_recommendation(
            symbol,
            bar_idx,
            &st.closes,
            has_valid_fresh_quote && !self.feed_stale,
            quote_flags.missing,
            quote_flags.stale || self.feed_stale,
            quote_flags.bad_prices,
            quote_flags.crossed,
            false,
            None,
        );
        eval.order_flow = st.last_of.clone();
        if self.feed_stale {
            eval.is_current = false;
            eval.suppressed_reason = self.stale_reason;
        }
        st.last_eval = Some(eval.clone());
        Some(eval)
    }

    /// Market / OF event: update price + OF badge; attach event identity.
    pub fn on_market_of_event(
        &mut self,
        symbol: &str,
        ev: &MarketEvent,
        of_mark: &OfMarkRecord,
    ) -> Option<RecommendationEval> {
        let st = self.states.get_mut(symbol)?;
        st.last_price = Some(ev.last_price);
        st.last_of = Some(of_mark.inspection.clone());
        if let Some(mut eval) = st.last_eval.clone() {
            eval.ingest_seq = Some(ev.ingest_seq);
            eval.content_fp = Some(ev.content_fp);
            eval.order_flow = Some(of_mark.inspection.clone());
            let current = !self.feed_stale && of_mark.is_current;
            eval.is_current = current;
            eval.suppressed_reason = if current {
                None
            } else {
                self.stale_reason.or(of_mark.suppressed_reason)
            };
            st.last_eval = Some(eval.clone());
            Some(eval)
        } else {
            None
        }
    }

    pub fn current_recommendation(&self, symbol: &str) -> Option<&RecommendationEval> {
        if self.feed_stale {
            return None;
        }
        self.states
            .get(symbol)
            .and_then(|s| s.last_eval.as_ref())
            .filter(|e| e.is_current)
    }

    pub fn all_recommendations(&self) -> Vec<RecommendationEval> {
        self.watch_symbols
            .iter()
            .filter_map(|s| self.states.get(s).and_then(|st| st.last_eval.clone()))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct QuoteFlags {
    pub missing: bool,
    pub stale: bool,
    pub bad_prices: bool,
    pub crossed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::of_mark::ORDER_FLOW_IMBALANCE_THRESHOLD;

    #[test]
    fn traj_threshold_frozen() {
        assert!((TRAJ15_ENTRY_THRESHOLD - 0.05).abs() < 1e-12);
        assert_eq!(WATCHLIST_TOP_N, 20);
        assert!((ORDER_FLOW_IMBALANCE_THRESHOLD - 0.30).abs() < 1e-12);
    }
}
