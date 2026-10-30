//! Parity tests for RP-608 regime / breakout / market-context Rust port.

use chronosentiment_adapter::research::rp608_regime_breakout::{
    apply_market_chop_to_regime, classify_market_regime, compute_dynamic_boundaries,
    compute_market_context, dedupe_key, donchian_20, evaluate_quote, half_spread_bps,
    kaufman_efficiency_ratio, relative_volatility_ratio, LifecycleState, OhlcBar, QuoteEvalArgs,
    RegimeClassification,
};

fn bar(h: f64, l: f64, c: f64) -> OhlcBar {
    OhlcBar {
        high: h,
        low: l,
        close: c,
    }
}

fn bars_drift(n: usize, start: f64, drift: f64) -> Vec<OhlcBar> {
    let mut out = Vec::new();
    let mut px = start;
    for _ in 0..n {
        let o = px;
        px *= 1.0 + drift;
        out.push(bar(o.max(px) * 1.001, o.min(px) * 0.999, px));
    }
    out
}

#[test]
fn test_er_perfect_trend() {
    let closes: Vec<f64> = (0..20).map(|i| 100.0 + i as f64 * 5.0).collect();
    let er = kaufman_efficiency_ratio(&closes, 16).unwrap();
    assert!((er - 1.0).abs() < 1e-4);
}

#[test]
fn test_er_flatline() {
    let closes = vec![100.0; 25];
    assert_eq!(kaufman_efficiency_ratio(&closes, 16), Some(0.0));
}

#[test]
fn test_er_insufficient() {
    assert!(kaufman_efficiency_ratio(&[100.0, 101.0, 102.0], 16).is_none());
}

#[test]
fn test_vol_ratio_normal() {
    let bars: Vec<_> = (0..70)
        .map(|i| {
            let c = 100.0 + i as f64;
            bar(c + 2.0, c - 2.0, c)
        })
        .collect();
    let vr = relative_volatility_ratio(&bars, 48, 14).unwrap();
    assert!((vr - 1.0).abs() < 0.05);
}

#[test]
fn test_market_chop_detection() {
    let ctx = compute_market_context(&[5.0, -4.0, 3.0, -2.0, 1.0, -1.0, 2.0, -3.0, 4.0, -5.0]);
    assert!(ctx.market_chop);
    let b = ctx.session_breadth.unwrap();
    assert!((0.4..=0.6).contains(&b));
}

#[test]
fn test_clear_market() {
    let ctx = compute_market_context(&[50.0, 40.0, 30.0, 20.0, 10.0, 5.0, 8.0, 12.0, 15.0, 18.0]);
    assert!(!ctx.market_chop);
}

#[test]
fn test_er_hurdle_demotes_directional() {
    let stock = RegimeClassification {
        regime: "DIRECTIONAL".into(),
        reason: "CONFIRMED".into(),
        er: Some(0.45),
        vol_ratio: Some(1.0),
        adx: Some(30.0),
        plus_di: Some(30.0),
        minus_di: Some(10.0),
    };
    let ctx = compute_market_context(&[5.0, -4.0, 3.0, -2.0, 1.0, -1.0, 2.0, -3.0, 4.0, -5.0]);
    let out = apply_market_chop_to_regime(&stock, &ctx, 0.50, 0.40);
    assert_eq!(out.base.regime, "TRANSITIONING");
    assert_eq!(out.directional_er_hurdle, 0.50);
}

#[test]
fn test_donchian_excludes_forming() {
    let mut bars = bars_drift(25, 100.0, 0.001);
    bars.last_mut().unwrap().high = 1e9;
    let (hi, _) = donchian_20(&bars, true).unwrap();
    assert!(hi < 1e8);
}

#[test]
fn test_stale_invalidates_active() {
    let st0 = LifecycleState {
        state: "WATCH".into(),
        direction: Some("LONG".into()),
        boundary_price: Some(100.0),
        ..Default::default()
    };
    let bars = bars_drift(25, 100.0, 0.0);
    let st = evaluate_quote(QuoteEvalArgs {
        state: &st0,
        regime: "DIRECTIONAL",
        bars: &bars,
        ask: 100.1,
        bid: 100.0,
        quote_age_ms: 800.0,
        session_date: "2026-01-01",
        symbol: "X",
        half_spread_cap_bps: 5.0,
        target_distance: 150.0,
        stop_distance: 100.0,
        friction: 5.0,
        session_expired: false,
        approach_pct: 0.0025,
    });
    assert_eq!(st.state, "INVALIDATED");
    assert_eq!(st.reason.as_deref(), Some("stale"));
}

#[test]
fn test_expiry() {
    let st0 = LifecycleState {
        state: "WATCH".into(),
        direction: Some("LONG".into()),
        boundary_price: Some(100.0),
        ..Default::default()
    };
    let bars = bars_drift(25, 100.0, 0.0);
    let st = evaluate_quote(QuoteEvalArgs {
        state: &st0,
        regime: "DIRECTIONAL",
        bars: &bars,
        ask: 100.1,
        bid: 100.0,
        quote_age_ms: 10.0,
        session_date: "2026-01-01",
        symbol: "X",
        half_spread_cap_bps: 5.0,
        target_distance: 150.0,
        stop_distance: 100.0,
        friction: 5.0,
        session_expired: true,
        approach_pct: 0.0025,
    });
    assert_eq!(st.state, "EXPIRED");
}

#[test]
fn test_dedupe_stable() {
    let a = dedupe_key("2026-01-01", "RELIANCE.NS", 100.5, "LONG");
    let b = dedupe_key("2026-01-01", "RELIANCE.NS", 100.5, "LONG");
    assert_eq!(a, b);
}

#[test]
fn test_half_spread() {
    let hs = half_spread_bps(100.2, 100.0).unwrap();
    assert!(hs > 0.0);
}

#[test]
fn test_crossing_not_watch() {
    let bars = bars_drift(25, 100.0, 0.0);
    let (upper, _) = donchian_20(&bars, true).unwrap();
    let ask = upper * 1.01;
    let bid = ask * 0.9995;
    let st = evaluate_quote(QuoteEvalArgs {
        state: &LifecycleState::default(),
        regime: "DIRECTIONAL",
        bars: &bars,
        ask,
        bid,
        quote_age_ms: 50.0,
        session_date: "2026-01-01",
        symbol: "X",
        half_spread_cap_bps: 5.0,
        target_distance: 150.0,
        stop_distance: 100.0,
        friction: 5.0,
        session_expired: false,
        approach_pct: 0.0025,
    });
    assert_ne!(st.state, "WATCH");
    assert!(matches!(
        st.state.as_str(),
        "TRADE_QUALIFIED" | "INVALIDATED" | "BREAKOUT_DETECTED"
    ));
}

#[test]
fn test_dynamic_boundaries_trend_bars() {
    let bars: Vec<_> = (0..80)
        .map(|i| {
            let c = 100.0 + i as f64 * 0.5;
            bar(c + 1.0, c - 1.0, c)
        })
        .collect();
    let out = compute_dynamic_boundaries(&bars, "LONG", 140.0, None, 12.0);
    assert!(out.atr_14 > 0.0);
    assert_eq!(out.policy_id, "RP608_DYNAMIC_OHLCV_STRUCTURE_V1");
}

#[test]
fn test_classify_empty_unavailable() {
    let r = classify_market_regime(&[]);
    assert_eq!(r.regime, "UNAVAILABLE");
}
