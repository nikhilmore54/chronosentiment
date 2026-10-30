//! RP-608 regime / breakout / dynamic-boundary primitives (research-only).
//!
//! Rust port of:
//! - `scripts/market_regime_classifier.py`
//! - `scripts/nse_market_context.py`
//! - `scripts/breakout_lifecycle.py`
//! - `scripts/dynamic_trade_boundaries.py`
//!
//! Deterministic · live_order_authority = 0.0 · no order placement.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OhlcBar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

fn round_n(x: f64, n: i32) -> f64 {
    let p = 10f64.powi(n);
    (x * p).round() / p
}

fn clamp(x: f64, lo: f64, hi: f64) -> f64 {
    x.max(lo).min(hi)
}

fn valid_hlc(h: f64, l: f64, c: f64) -> bool {
    h.is_finite() && l.is_finite() && c.is_finite() && h > 0.0 && l > 0.0 && c > 0.0 && l <= h
}

// ─── market_regime_classifier ───────────────────────────────────────────────

pub fn kaufman_efficiency_ratio(closes: &[f64], n: usize) -> Option<f64> {
    if closes.len() < n + 1 || n < 1 {
        return None;
    }
    let slice = &closes[closes.len() - (n + 1)..];
    for &c in slice {
        if !c.is_finite() || c <= 0.0 {
            return None;
        }
    }
    let net = (closes[closes.len() - 1] - closes[closes.len() - 1 - n]).abs();
    let mut path = 0.0;
    for i in 0..n {
        path += (closes[closes.len() - 1 - i] - closes[closes.len() - 2 - i]).abs();
    }
    if path == 0.0 {
        return Some(0.0);
    }
    let er = (net / path).clamp(0.0, 1.0);
    Some(round_n(er, 4))
}

pub fn calculate_atr_series(bars: &[OhlcBar], period: usize) -> Option<Vec<f64>> {
    if bars.len() < period + 1 || period < 1 {
        return None;
    }
    for b in bars {
        if !valid_hlc(b.high, b.low, b.close) {
            return None;
        }
    }
    let mut tr = Vec::with_capacity(bars.len());
    tr.push(bars[0].high - bars[0].low);
    for i in 1..bars.len() {
        let curr = &bars[i];
        let prev_c = bars[i - 1].close;
        tr.push(
            (curr.high - curr.low)
                .max((curr.high - prev_c).abs())
                .max((curr.low - prev_c).abs()),
        );
    }
    let mut atr_series = vec![None; bars.len()];
    let initial: f64 = tr[1..=period].iter().sum::<f64>() / period as f64;
    atr_series[period] = Some(initial);
    let mut running = initial;
    for i in (period + 1)..bars.len() {
        running = (running * (period as f64 - 1.0) + tr[i]) / period as f64;
        atr_series[i] = Some(running);
    }
    Some(atr_series.into_iter().flatten().collect())
}

pub fn relative_volatility_ratio(
    bars: &[OhlcBar],
    n_trailing: usize,
    atr_period: usize,
) -> Option<f64> {
    let required = atr_period + n_trailing;
    if bars.len() < required {
        return None;
    }
    let atr_vals = calculate_atr_series(bars, atr_period)?;
    if atr_vals.len() < n_trailing {
        return None;
    }
    let window = &atr_vals[atr_vals.len() - n_trailing..];
    let current = *window.last()?;
    let mut sorted = window.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = sorted.len() / 2;
    let median = if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    };
    if median <= 0.0 {
        return None;
    }
    Some(round_n(current / median, 4))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdxResult {
    pub adx: f64,
    pub plus_di: f64,
    pub minus_di: f64,
}

pub fn adx_indicator(bars: &[OhlcBar], period: usize) -> Option<AdxResult> {
    let required = 2 * period + 1;
    if bars.len() < required || period < 2 {
        return None;
    }
    for b in bars {
        if !valid_hlc(b.high, b.low, b.close) {
            return None;
        }
    }
    let mut tr_list = Vec::new();
    let mut plus_dm = Vec::new();
    let mut minus_dm = Vec::new();
    for i in 1..bars.len() {
        let curr = &bars[i];
        let prev = &bars[i - 1];
        tr_list.push(
            (curr.high - curr.low)
                .max((curr.high - prev.close).abs())
                .max((curr.low - prev.close).abs()),
        );
        let up = curr.high - prev.high;
        let down = prev.low - curr.low;
        plus_dm.push(if up > down && up > 0.0 { up } else { 0.0 });
        minus_dm.push(if down > up && down > 0.0 { down } else { 0.0 });
    }
    let mut sm_tr: f64 = tr_list[..period].iter().sum();
    let mut sm_pdm: f64 = plus_dm[..period].iter().sum();
    let mut sm_mdm: f64 = minus_dm[..period].iter().sum();

    let calc_dx = |p_dm: f64, m_dm: f64, t_r: f64| -> f64 {
        if t_r <= 0.0 {
            return 0.0;
        }
        let p_di = 100.0 * (p_dm / t_r);
        let m_di = 100.0 * (m_dm / t_r);
        let di_sum = p_di + m_di;
        if di_sum <= 0.0 {
            return 0.0;
        }
        100.0 * ((p_di - m_di).abs() / di_sum)
    };

    let mut dx_list = vec![calc_dx(sm_pdm, sm_mdm, sm_tr)];
    for i in period..tr_list.len() {
        sm_tr = sm_tr - (sm_tr / period as f64) + tr_list[i];
        sm_pdm = sm_pdm - (sm_pdm / period as f64) + plus_dm[i];
        sm_mdm = sm_mdm - (sm_mdm / period as f64) + minus_dm[i];
        dx_list.push(calc_dx(sm_pdm, sm_mdm, sm_tr));
    }
    if dx_list.len() < period {
        return None;
    }
    let mut adx: f64 = dx_list[..period].iter().sum::<f64>() / period as f64;
    for i in period..dx_list.len() {
        adx = (adx * (period as f64 - 1.0) + dx_list[i]) / period as f64;
    }
    let p_di = if sm_tr > 0.0 {
        100.0 * (sm_pdm / sm_tr)
    } else {
        0.0
    };
    let m_di = if sm_tr > 0.0 {
        100.0 * (sm_mdm / sm_tr)
    } else {
        0.0
    };
    Some(AdxResult {
        adx: round_n(adx, 2),
        plus_di: round_n(p_di, 2),
        minus_di: round_n(m_di, 2),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegimeClassification {
    pub regime: String,
    pub reason: String,
    pub er: Option<f64>,
    pub vol_ratio: Option<f64>,
    pub adx: Option<f64>,
    pub plus_di: Option<f64>,
    pub minus_di: Option<f64>,
}

pub fn classify_market_regime(bars: &[OhlcBar]) -> RegimeClassification {
    if bars.is_empty() {
        return RegimeClassification {
            regime: "UNAVAILABLE".into(),
            reason: "EMPTY_BAR_HISTORY".into(),
            er: None,
            vol_ratio: None,
            adx: None,
            plus_di: None,
            minus_di: None,
        };
    }
    let mut closes = Vec::with_capacity(bars.len());
    for b in bars {
        if !valid_hlc(b.high, b.low, b.close) {
            return RegimeClassification {
                regime: "UNAVAILABLE".into(),
                reason: "CORRUPT_BAR_DATA".into(),
                er: None,
                vol_ratio: None,
                adx: None,
                plus_di: None,
                minus_di: None,
            };
        }
        closes.push(b.close);
    }
    let er = kaufman_efficiency_ratio(&closes, 16);
    let vol_ratio = relative_volatility_ratio(bars, 48, 14);
    let adx_data = adx_indicator(bars, 14);
    if er.is_none() || vol_ratio.is_none() || adx_data.is_none() {
        let mut missing = Vec::new();
        if er.is_none() {
            missing.push("ER_16(need 17)");
        }
        if vol_ratio.is_none() {
            missing.push("VolRatio_48(need 62)");
        }
        if adx_data.is_none() {
            missing.push("ADX_14(need 29)");
        }
        return RegimeClassification {
            regime: "UNAVAILABLE".into(),
            reason: format!(
                "INSUFFICIENT_WARMUP:{}:bars={}",
                missing.join(","),
                bars.len()
            ),
            er,
            vol_ratio,
            adx: adx_data.as_ref().map(|a| a.adx),
            plus_di: adx_data.as_ref().map(|a| a.plus_di),
            minus_di: adx_data.as_ref().map(|a| a.minus_di),
        };
    }
    let er = er.unwrap();
    let vol_ratio = vol_ratio.unwrap();
    let adx_data = adx_data.unwrap();
    let adx = adx_data.adx;
    let plus_di = adx_data.plus_di;
    let minus_di = adx_data.minus_di;

    const MAX_ER_SIDEWAYS: f64 = 0.25;
    const MAX_ADX_SIDEWAYS: f64 = 20.0;
    const MIN_VOL: f64 = 0.80;
    const MIN_ER_DIR: f64 = 0.40;
    const MIN_ADX_DIR: f64 = 25.0;
    const MAX_VOL: f64 = 1.80;

    if er < MAX_ER_SIDEWAYS || adx < MAX_ADX_SIDEWAYS || vol_ratio < MIN_VOL {
        let mut reasons = Vec::new();
        if er < MAX_ER_SIDEWAYS {
            reasons.push(format!("LOW_EFFICIENCY({:.4}<{})", er, MAX_ER_SIDEWAYS));
        }
        if adx < MAX_ADX_SIDEWAYS {
            reasons.push(format!("WEAK_ADX({:.1}<{})", adx, MAX_ADX_SIDEWAYS));
        }
        if vol_ratio < MIN_VOL {
            reasons.push(format!("VOL_COMPRESSION({:.4}<{})", vol_ratio, MIN_VOL));
        }
        return RegimeClassification {
            regime: "SIDEWAYS".into(),
            reason: reasons.join(";"),
            er: Some(er),
            vol_ratio: Some(vol_ratio),
            adx: Some(adx),
            plus_di: Some(plus_di),
            minus_di: Some(minus_di),
        };
    }

    if er >= MIN_ER_DIR && adx >= MIN_ADX_DIR && (MIN_VOL..=MAX_VOL).contains(&vol_ratio) {
        let trend = if plus_di >= minus_di {
            "BULLISH"
        } else {
            "BEARISH"
        };
        return RegimeClassification {
            regime: "DIRECTIONAL".into(),
            reason: format!(
                "CONFIRMED_TREND:{}(ER={:.4},ADX={:.1},VR={:.4})",
                trend, er, adx, vol_ratio
            ),
            er: Some(er),
            vol_ratio: Some(vol_ratio),
            adx: Some(adx),
            plus_di: Some(plus_di),
            minus_di: Some(minus_di),
        };
    }

    let mut trans = Vec::new();
    if (MAX_ER_SIDEWAYS..MIN_ER_DIR).contains(&er) {
        trans.push(format!("MODERATE_EFFICIENCY({:.4})", er));
    }
    if (MAX_ADX_SIDEWAYS..MIN_ADX_DIR).contains(&adx) {
        trans.push(format!("DEVELOPING_ADX({:.1})", adx));
    }
    if vol_ratio > MAX_VOL {
        trans.push(format!("VOL_CLIMAX({:.4}>{})", vol_ratio, MAX_VOL));
    }
    if trans.is_empty() {
        trans.push(format!(
            "MIXED_SIGNALS(ER={:.4},ADX={:.1},VR={:.4})",
            er, adx, vol_ratio
        ));
    }
    RegimeClassification {
        regime: "TRANSITIONING".into(),
        reason: trans.join(";"),
        er: Some(er),
        vol_ratio: Some(vol_ratio),
        adx: Some(adx),
        plus_di: Some(plus_di),
        minus_di: Some(minus_di),
    }
}

// ─── nse_market_context ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketContext {
    pub session_breadth: Option<f64>,
    pub market_return_bps: Option<f64>,
    pub market_dispersion_bps: Option<f64>,
    pub n_active: usize,
    pub market_chop: bool,
    pub reason: String,
}

pub fn compute_market_context(symbol_returns_bps: &[f64]) -> MarketContext {
    let rets: Vec<f64> = symbol_returns_bps
        .iter()
        .copied()
        .filter(|r| r.is_finite())
        .collect();
    let n = rets.len();
    if n == 0 {
        return MarketContext {
            session_breadth: None,
            market_return_bps: None,
            market_dispersion_bps: None,
            n_active: 0,
            market_chop: false,
            reason: "NO_ACTIVE_SYMBOLS".into(),
        };
    }
    let advancing = rets.iter().filter(|&&r| r > 0.0).count();
    let breadth = advancing as f64 / n as f64;
    let mean_r = rets.iter().sum::<f64>() / n as f64;
    let var = rets.iter().map(|r| (r - mean_r).powi(2)).sum::<f64>() / n as f64;
    let disp = var.sqrt();
    let chop = (0.40..=0.60).contains(&breadth) && mean_r.abs() < 10.0;
    MarketContext {
        session_breadth: Some(round_n(breadth, 4)),
        market_return_bps: Some(round_n(mean_r, 4)),
        market_dispersion_bps: Some(round_n(disp, 4)),
        n_active: n,
        market_chop: chop,
        reason: if chop {
            "MARKET_CHOP".into()
        } else {
            "MARKET_CLEAR".into()
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChopAdjustedRegime {
    pub base: RegimeClassification,
    pub market_chop: bool,
    pub session_breadth: Option<f64>,
    pub market_return_bps: Option<f64>,
    pub market_dispersion_bps: Option<f64>,
    pub directional_er_hurdle: f64,
}

pub fn apply_market_chop_to_regime(
    stock: &RegimeClassification,
    market: &MarketContext,
    chop_er_hurdle: f64,
    base_er_hurdle: f64,
) -> ChopAdjustedRegime {
    let mut out = ChopAdjustedRegime {
        base: stock.clone(),
        market_chop: market.market_chop,
        session_breadth: market.session_breadth,
        market_return_bps: market.market_return_bps,
        market_dispersion_bps: market.market_dispersion_bps,
        directional_er_hurdle: if market.market_chop {
            chop_er_hurdle
        } else {
            base_er_hurdle
        },
    };
    if matches!(out.base.regime.as_str(), "UNAVAILABLE" | "SIDEWAYS") || !market.market_chop {
        return out;
    }
    if out.base.regime == "DIRECTIONAL" {
        if let Some(er) = out.base.er {
            if er < chop_er_hurdle {
                out.base.regime = "TRANSITIONING".into();
                out.base.reason = format!(
                    "MARKET_CHOP_ER_HURDLE({:.4}<{});{}",
                    er, chop_er_hurdle, out.base.reason
                );
            }
        }
    }
    out
}

// ─── breakout_lifecycle ─────────────────────────────────────────────────────

pub fn donchian_20(bars: &[OhlcBar], exclude_forming: bool) -> Option<(f64, f64)> {
    let hist: &[OhlcBar] = if exclude_forming && !bars.is_empty() {
        &bars[..bars.len() - 1]
    } else {
        bars
    };
    if hist.len() < 20 {
        return None;
    }
    let window = &hist[hist.len() - 20..];
    let mut hi = f64::NEG_INFINITY;
    let mut lo = f64::INFINITY;
    for b in window {
        if !valid_hlc(b.high, b.low, b.close) {
            return None;
        }
        hi = hi.max(b.high);
        lo = lo.min(b.low);
    }
    Some((hi, lo))
}

pub fn half_spread_bps(ask: f64, bid: f64) -> Option<f64> {
    if ask <= 0.0 || bid <= 0.0 || ask <= bid {
        return None;
    }
    let mid = (ask + bid) / 2.0;
    Some(((ask - bid) / (2.0 * mid)) * 10_000.0)
}

pub fn delta_buffer(mid: f64, ask: f64, bid: f64) -> f64 {
    let full_spread = ask - bid;
    let bps_component = mid * (2.0 / 10_000.0);
    bps_component.max(0.5 * full_spread)
}

pub fn dedupe_key(session_date: &str, symbol: &str, boundary_price: f64, direction: &str) -> String {
    let raw = format!("{session_date}|{symbol}|{boundary_price:.8}|{direction}");
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn net_reward_risk(target_distance: f64, stop_distance: f64, friction: f64) -> f64 {
    let denom = stop_distance + friction;
    if denom <= 0.0 {
        0.0
    } else {
        (target_distance - friction) / denom
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LifecycleState {
    pub state: String,
    pub direction: Option<String>,
    pub boundary_price: Option<f64>,
    pub reason: Option<String>,
    pub dedupe: Option<String>,
    pub net_rr: Option<f64>,
}

impl Default for LifecycleState {
    fn default() -> Self {
        Self {
            state: "IDLE".into(),
            direction: None,
            boundary_price: None,
            reason: None,
            dedupe: None,
            net_rr: None,
        }
    }
}

fn is_terminal(s: &str) -> bool {
    matches!(s, "TRADE_QUALIFIED" | "INVALIDATED" | "EXPIRED")
}
fn is_active(s: &str) -> bool {
    matches!(s, "WATCH" | "BREAKOUT_DETECTED")
}

pub struct QuoteEvalArgs<'a> {
    pub state: &'a LifecycleState,
    pub regime: &'a str,
    pub bars: &'a [OhlcBar],
    pub ask: f64,
    pub bid: f64,
    pub quote_age_ms: f64,
    pub session_date: &'a str,
    pub symbol: &'a str,
    pub half_spread_cap_bps: f64,
    pub target_distance: f64,
    pub stop_distance: f64,
    pub friction: f64,
    pub session_expired: bool,
    pub approach_pct: f64,
}

pub fn evaluate_quote(args: QuoteEvalArgs<'_>) -> LifecycleState {
    let mut out = args.state.clone();
    if is_terminal(&out.state) {
        return out;
    }
    if args.session_expired {
        if is_active(&out.state) {
            out.state = "EXPIRED".into();
            out.reason = Some("session_or_bar_expiry".into());
        }
        return out;
    }
    if args.quote_age_ms > 500.0 {
        if is_active(&out.state) {
            out.state = "INVALIDATED".into();
            out.reason = Some("stale".into());
        }
        return out;
    }
    let hs = half_spread_bps(args.ask, args.bid);
    if hs.is_none() {
        if is_active(&out.state) {
            out.state = "INVALIDATED".into();
            out.reason = Some("invalid_book".into());
        }
        return out;
    }
    if hs.unwrap() > args.half_spread_cap_bps {
        if is_active(&out.state) {
            out.state = "INVALIDATED".into();
            out.reason = Some("spread".into());
        }
        return out;
    }
    let Some((upper, lower)) = donchian_20(args.bars, true) else {
        return out;
    };
    let mid = (args.ask + args.bid) / 2.0;
    let buf = delta_buffer(mid, args.ask, args.bid);
    let long_cross = args.ask > upper + buf;
    let short_cross = args.bid < lower - buf;
    if long_cross || short_cross {
        let direction = if long_cross { "LONG" } else { "SHORT" };
        let boundary = if long_cross { upper } else { lower };
        out.direction = Some(direction.into());
        out.boundary_price = Some(boundary);
        out.dedupe = Some(dedupe_key(
            args.session_date,
            args.symbol,
            boundary,
            direction,
        ));
        let rr = net_reward_risk(args.target_distance, args.stop_distance, args.friction);
        out.net_rr = Some(round_n(rr, 4));
        if rr >= 1.0 {
            out.state = "TRADE_QUALIFIED".into();
            out.reason = Some("net_rr_ok".into());
        } else {
            out.state = "INVALIDATED".into();
            out.reason = Some("insufficient_rr".into());
        }
        return out;
    }
    let approach_long = upper > 0.0 && (args.ask - upper).abs() / upper <= args.approach_pct;
    let approach_short = lower > 0.0 && (args.bid - lower).abs() / lower <= args.approach_pct;
    if (approach_long || approach_short)
        && matches!(args.regime, "TRANSITIONING" | "DIRECTIONAL")
    {
        out.state = "WATCH".into();
        out.direction = Some(if approach_long { "LONG" } else { "SHORT" }.into());
        out.boundary_price = Some(if approach_long { upper } else { lower });
        out.reason = Some("approach".into());
        out.dedupe = Some(dedupe_key(
            args.session_date,
            args.symbol,
            out.boundary_price.unwrap(),
            out.direction.as_deref().unwrap(),
        ));
        return out;
    }
    if is_active(&out.state) {
        out.state = "INVALIDATED".into();
        out.reason = Some("retreat".into());
    }
    out
}

// ─── dynamic_trade_boundaries ───────────────────────────────────────────────

const ATR_STOP_MULT_MIN: f64 = 0.75;
const ATR_STOP_MULT_MAX: f64 = 2.50;
const ATR_TARGET_MULT_MIN: f64 = 1.00;
const ATR_TARGET_MULT_MAX: f64 = 4.00;
const STOP_BPS_FLOOR: f64 = 40.0;
const STOP_BPS_CEIL: f64 = 250.0;
const TARGET_BPS_FLOOR: f64 = 60.0;
const TARGET_BPS_CEIL: f64 = 400.0;
const MIN_NET_RR: f64 = 1.0;

pub fn recent_swing_pivot(bars: &[OhlcBar], side: &str, lookback: usize, pivot: usize) -> Option<f64> {
    if bars.len() < (2 * pivot + 1) {
        return None;
    }
    let start = if bars.len() > lookback + pivot {
        bars.len() - (lookback + pivot)
    } else {
        0
    };
    let window = &bars[start..];
    let mut extremes = Vec::new();
    for i in pivot..(window.len() - pivot) {
        let h = window[i].high;
        let l = window[i].low;
        if !valid_hlc(h, l, window[i].close) {
            continue;
        }
        if side == "LONG" {
            let ok = (1..=pivot).all(|k| window[i - k].low >= l && window[i + k].low >= l);
            if ok {
                extremes.push(l);
            }
        } else {
            let ok = (1..=pivot).all(|k| window[i - k].high <= h && window[i + k].high <= h);
            if ok {
                extremes.push(h);
            }
        }
    }
    extremes.last().copied()
}

pub fn trend_scale_from_er_adx(er: Option<f64>, adx: Option<f64>) -> (&'static str, f64, f64) {
    let er_v = er.unwrap_or(0.0);
    let adx_v = adx.unwrap_or(0.0);
    if er_v >= 0.40 && adx_v >= 25.0 {
        ("STRONG_TREND", 1.25, 2.50)
    } else if er_v >= 0.25 && adx_v >= 20.0 {
        ("DEVELOPING_TREND", 1.50, 2.00)
    } else {
        ("WEAK_OR_CHOP", 1.00, 1.25)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DynamicBoundaries {
    pub ok: bool,
    pub reason: String,
    pub side: String,
    pub entry_price: f64,
    pub stop_price: f64,
    pub target_price: f64,
    pub stop_bps: f64,
    pub target_bps: f64,
    pub net_rr: f64,
    pub atr_14: f64,
    pub er_16: Option<f64>,
    pub adx_14: Option<f64>,
    pub regime: Option<String>,
    pub trend_label: String,
    pub policy_id: String,
}

pub fn compute_dynamic_boundaries(
    bars: &[OhlcBar],
    side: &str,
    entry_price: f64,
    regime: Option<&str>,
    friction_bps: f64,
) -> DynamicBoundaries {
    assert!(side == "LONG" || side == "SHORT");
    if entry_price <= 0.0 || !entry_price.is_finite() {
        return DynamicBoundaries {
            ok: false,
            reason: "INVALID_ENTRY_PRICE".into(),
            side: side.into(),
            entry_price: 0.0,
            stop_price: 0.0,
            target_price: 0.0,
            stop_bps: 0.0,
            target_bps: 0.0,
            net_rr: 0.0,
            atr_14: 0.0,
            er_16: None,
            adx_14: None,
            regime: None,
            trend_label: "".into(),
            policy_id: "RP608_DYNAMIC_OHLCV_STRUCTURE_V1".into(),
        };
    }
    let Some(atr_series) = calculate_atr_series(bars, 14) else {
        return DynamicBoundaries {
            ok: false,
            reason: "INSUFFICIENT_ATR_HISTORY".into(),
            side: side.into(),
            entry_price,
            stop_price: 0.0,
            target_price: 0.0,
            stop_bps: 0.0,
            target_bps: 0.0,
            net_rr: 0.0,
            atr_14: 0.0,
            er_16: None,
            adx_14: None,
            regime: None,
            trend_label: "".into(),
            policy_id: "RP608_DYNAMIC_OHLCV_STRUCTURE_V1".into(),
        };
    };
    let atr = *atr_series.last().unwrap_or(&0.0);
    if atr <= 0.0 || !atr.is_finite() {
        return DynamicBoundaries {
            ok: false,
            reason: "INVALID_ATR".into(),
            side: side.into(),
            entry_price,
            stop_price: 0.0,
            target_price: 0.0,
            stop_bps: 0.0,
            target_bps: 0.0,
            net_rr: 0.0,
            atr_14: atr,
            er_16: None,
            adx_14: None,
            regime: None,
            trend_label: "".into(),
            policy_id: "RP608_DYNAMIC_OHLCV_STRUCTURE_V1".into(),
        };
    }
    let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();
    let er = kaufman_efficiency_ratio(&closes, 16);
    let adx_data = adx_indicator(bars, 14);
    let adx = adx_data.as_ref().map(|a| a.adx);
    let regime_s = regime
        .map(|s| s.to_string())
        .unwrap_or_else(|| classify_market_regime(bars).regime);
    let (label, mut stop_mult, mut target_mult) = trend_scale_from_er_adx(er, adx);
    stop_mult = clamp(stop_mult, ATR_STOP_MULT_MIN, ATR_STOP_MULT_MAX);
    target_mult = clamp(target_mult, ATR_TARGET_MULT_MIN, ATR_TARGET_MULT_MAX);
    if regime_s == "SIDEWAYS" {
        target_mult = target_mult.min(1.25);
        stop_mult = stop_mult.min(1.00);
    }
    let atr_stop_dist = atr * stop_mult;
    let atr_target_dist = atr * target_mult;
    let swing = recent_swing_pivot(bars, side, 20, 2);
    let mut stop_px;
    let mut target_px;
    if side == "LONG" {
        let atr_stop_px = entry_price - atr_stop_dist;
        let structure_stop = swing.filter(|&s| s < entry_price).map(|s| s - 0.1 * atr);
        stop_px = structure_stop.map(|s| atr_stop_px.max(s)).unwrap_or(atr_stop_px);
        target_px = entry_price + atr_target_dist;
        if let Some(prior_high) = recent_swing_pivot(bars, "SHORT", 30, 2) {
            if prior_high > target_px {
                target_px = prior_high.min(entry_price * (1.0 + TARGET_BPS_CEIL / 10_000.0));
            }
        }
    } else {
        let atr_stop_px = entry_price + atr_stop_dist;
        let structure_stop = swing.filter(|&s| s > entry_price).map(|s| s + 0.1 * atr);
        stop_px = structure_stop.map(|s| atr_stop_px.min(s)).unwrap_or(atr_stop_px);
        target_px = entry_price - atr_target_dist;
        if let Some(prior_low) = recent_swing_pivot(bars, "LONG", 30, 2) {
            if prior_low < target_px {
                target_px = prior_low.max(entry_price * (1.0 - TARGET_BPS_CEIL / 10_000.0));
            }
        }
    }
    let mut stop_bps = (entry_price - stop_px).abs() / entry_price * 10_000.0;
    let mut target_bps = (target_px - entry_price).abs() / entry_price * 10_000.0;
    stop_bps = clamp(stop_bps, STOP_BPS_FLOOR, STOP_BPS_CEIL);
    target_bps = clamp(target_bps, TARGET_BPS_FLOOR, TARGET_BPS_CEIL);
    if side == "LONG" {
        stop_px = entry_price * (1.0 - stop_bps / 10_000.0);
        target_px = entry_price * (1.0 + target_bps / 10_000.0);
    } else {
        stop_px = entry_price * (1.0 + stop_bps / 10_000.0);
        target_px = entry_price * (1.0 - target_bps / 10_000.0);
    }
    let net_rr = (target_bps - friction_bps) / (stop_bps + friction_bps + 1e-9);
    let ok = net_rr >= MIN_NET_RR && atr > 0.0;
    DynamicBoundaries {
        ok,
        reason: if ok {
            "OK".into()
        } else if net_rr < MIN_NET_RR {
            "INSUFFICIENT_NET_RR".into()
        } else {
            "INVALID_ATR".into()
        },
        side: side.into(),
        entry_price: round_n(entry_price, 6),
        stop_price: round_n(stop_px, 6),
        target_price: round_n(target_px, 6),
        stop_bps: round_n(stop_bps, 4),
        target_bps: round_n(target_bps, 4),
        net_rr: round_n(net_rr, 4),
        atr_14: round_n(atr, 6),
        er_16: er.map(|v| round_n(v, 4)),
        adx_14: adx.map(|v| round_n(v, 2)),
        regime: Some(regime_s),
        trend_label: label.into(),
        policy_id: "RP608_DYNAMIC_OHLCV_STRUCTURE_V1".into(),
    }
}
