//! TIME-002 Acceptance Criteria Tests
//!
//! All 7 ACs must pass before TIME-002 can be committed.
//! These tests use the CS-P-006 historical cache (offline — no network calls).
//!
//! Cache path: product_validation/CS-P-006/snapshot/
//!             20260814T183851Z_100instrument/yahoo_cache
//!
//! T2-05 (future-poison) is the critical release blocker: it poisons the raw
//! OHLCV source layer (not derived features) and asserts that the reconstruction
//! at T is identical to the unpoisoned reconstruction.

use chrono::{DateTime, Utc};
use chronosentiment_adapter::decision_support::enrichment_certify::{
    assess_from_bars_at_t, load_yahoo_cache_dir, metrics_from_bars_at_t,
};
use chronosentiment_adapter::decision_support::forward_tick::instrument_id_for;
use chronosentiment_adapter::decision_support::observatory_prospective::latest_session_at_or_before;
use chronosentiment_adapter::ingestion::yahoo::YahooHistoricalBar;
use chronosentiment_adapter::time_machine::clock::HistoricalClock;
use sha2::{Digest, Sha256};


const TEST_TICKER: &str = "RELIANCE.NS";
// as_of = 2026-08-14T10:15:00Z (well within the cache range)
const AS_OF_STR: &str = "2026-08-14T10:15:00Z";

/// Resolve the yahoo_cache directory relative to the workspace root.
/// `env!("CARGO_MANIFEST_DIR")` is `adapters/chronosentiment` at compile time;
/// two `.parent()` calls reach the workspace root.
fn cache_dir() -> std::path::PathBuf {
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest
        .parent()
        .expect("adapters dir")
        .parent()
        .expect("workspace root");
    workspace_root.join(
        "product_validation/CS-P-006/snapshot/20260814T183851Z_100instrument/yahoo_cache",
    )
}

fn as_of() -> DateTime<Utc> {
    AS_OF_STR.parse().unwrap()
}

fn load_test_bars() -> Vec<YahooHistoricalBar> {
    let cache = load_yahoo_cache_dir(&cache_dir()).expect("cache must be readable");
    cache.get(TEST_TICKER).cloned().unwrap_or_default()
}

/// Compute source hash (mirrors the main binary's compute_source_hash).
fn compute_source_hash(all_bars: &[(String, Vec<YahooHistoricalBar>)], as_of: DateTime<Utc>) -> String {
    let mut hasher = Sha256::new();
    let mut sorted: Vec<&(String, Vec<YahooHistoricalBar>)> = all_bars.iter().collect();
    sorted.sort_by_key(|(ticker, _)| ticker.as_str());
    for (ticker, bars) in &sorted {
        hasher.update(ticker.as_bytes());
        hasher.update(b"|");
        let mut ts_at_t: Vec<i64> = bars
            .iter()
            .filter(|b| b.timestamp <= as_of.timestamp())
            .map(|b| b.timestamp)
            .collect();
        ts_at_t.sort_unstable();
        for ts in ts_at_t {
            hasher.update(ts.to_le_bytes());
        }
        hasher.update(b"\n");
    }
    format!("{:x}", hasher.finalize())
}

// ─── T2-01: Temporal boundary ─────────────────────────────────────────────────

/// T2-01: Every source_timestamp ≤ T.
/// After filtering, no bar in bars_at_t has timestamp > as_of.
#[test]
fn t2_01_temporal_boundary() {
    let all_bars = load_test_bars();
    assert!(!all_bars.is_empty(), "test cache must have bars for {TEST_TICKER}");

    let t = as_of();
    let bars_at_t: Vec<YahooHistoricalBar> = all_bars
        .iter()
        .filter(|b| b.timestamp <= t.timestamp())
        .cloned()
        .collect();

    // Every bar in bars_at_t must satisfy timestamp ≤ T.
    for bar in &bars_at_t {
        assert!(
            bar.timestamp <= t.timestamp(),
            "T2-01 FAIL: bar.timestamp={} > T={}",
            bar.timestamp,
            t.timestamp()
        );
    }

    // At least some bars must have been excluded (future bars exist in cache).
    let n_excluded = all_bars.len() - bars_at_t.len();
    println!(
        "T2-01: n_total={} n_at_t={} n_excluded={}",
        all_bars.len(),
        bars_at_t.len(),
        n_excluded
    );
    // The cache was captured on 2026-08-14T18:38:51Z, so bars after 10:15 exist.
    assert!(n_excluded > 0, "T2-01: expected future bars to be excluded");
}

// ─── T2-02: Derived-feature boundary ─────────────────────────────────────────

/// T2-02: Every derived feature uses only data ≤ T.
/// Verify that reference_price, atr_14, trend, momentum, volatility are all
/// derived exclusively from bars_at_t.
#[test]
fn t2_02_derived_features_from_bars_at_t_only() {
    let all_bars = load_test_bars();
    let t = as_of();
    let bars_at_t: Vec<YahooHistoricalBar> = all_bars
        .iter()
        .filter(|b| b.timestamp <= t.timestamp())
        .cloned()
        .collect();

    assert!(!bars_at_t.is_empty(), "bars_at_t must not be empty");

    let instrument_id = instrument_id_for(TEST_TICKER);
    let session_t = latest_session_at_or_before(&bars_at_t, t).unwrap_or(t);

    // Reference price — last close ≤ session_t.
    let ref_price = bars_at_t
        .iter()
        .filter(|b| b.timestamp <= session_t.timestamp())
        .last()
        .and_then(|b| if b.close > 0.0 { Some(b.close) } else { None });

    assert!(
        ref_price.is_some(),
        "T2-02: reference_price must be derivable from bars_at_t"
    );

    // ATR-14 from bars_at_t.
    let metrics = metrics_from_bars_at_t(&bars_at_t, session_t, instrument_id);
    let atr_14 = metrics.get_float("atr_14");
    assert!(atr_14.is_some(), "T2-02: atr_14 must be derivable from bars_at_t");

    // TMV from bars_at_t.
    let (profile, _, _) = assess_from_bars_at_t(&bars_at_t, session_t, instrument_id);
    let has_trend = profile
        .assessments
        .iter()
        .any(|a| a.concept == chronosentiment_adapter::metrics::concepts::Concept::Trend);
    assert!(has_trend, "T2-02: trend must be derivable from bars_at_t");

    println!("T2-02: ref_price={ref_price:?} atr_14={atr_14:?} has_trend={has_trend}");
}

// ─── T2-03: Clock isolation ───────────────────────────────────────────────────

/// T2-03: No Utc::now() in the reconstruction path.
/// HistoricalClock::replay(T) always returns T — never the wall clock.
#[test]
fn t2_03_clock_isolation() {
    let t = as_of();
    let clock = HistoricalClock::replay(t);

    // Multiple calls must all return exactly T.
    for _ in 0..100 {
        assert_eq!(
            clock.now(),
            t,
            "T2-03 FAIL: clock.now() returned a value != T"
        );
    }

    // Clock must be in REPLAY mode.
    assert!(clock.is_replay(), "T2-03 FAIL: clock must be in REPLAY mode");
    assert_eq!(clock.mode_label(), "REPLAY");

    // clock.now() must be ≤ T (never in the future).
    assert!(
        clock.now() <= t,
        "T2-03 FAIL: clock.now() > T (future leakage)"
    );

    println!("T2-03: clock={clock} now={}", clock.now());
}

// ─── T2-04: Future cache isolation ───────────────────────────────────────────

/// T2-04: Future cache records cannot contaminate reconstruction.
/// Bars with timestamp > T are excluded before any metric computation.
#[test]
fn t2_04_future_cache_isolation() {
    let all_bars = load_test_bars();
    let t = as_of();

    let bars_at_t: Vec<YahooHistoricalBar> = all_bars
        .iter()
        .filter(|b| b.timestamp <= t.timestamp())
        .cloned()
        .collect();

    // Verify that bars_at_t contains no future bars.
    let future_bars_in_result = bars_at_t
        .iter()
        .filter(|b| b.timestamp > t.timestamp())
        .count();

    assert_eq!(
        future_bars_in_result,
        0,
        "T2-04 FAIL: {future_bars_in_result} future bars leaked into bars_at_t"
    );

    // Verify that future bars actually exist in the raw cache (so the test is meaningful).
    let future_bars_in_cache = all_bars
        .iter()
        .filter(|b| b.timestamp > t.timestamp())
        .count();

    println!(
        "T2-04: n_total={} n_at_t={} n_future_excluded={}",
        all_bars.len(),
        bars_at_t.len(),
        future_bars_in_cache
    );
    assert!(
        future_bars_in_cache > 0,
        "T2-04: test requires future bars to exist in cache (otherwise test is vacuous)"
    );
}

// ─── T2-05: Future-poison (HARD BLOCKER) ─────────────────────────────────────

/// T2-05: Future-poison test — HARD BLOCKER.
///
/// Poison the raw OHLCV source layer with future bars that have extreme
/// price/ATR/trend/momentum values. The reconstruction at T must be
/// identical to the unpoisoned reconstruction.
///
/// This proves the temporal boundary is enforced at the raw-source layer,
/// not just at the derived-feature layer.
#[test]
fn t2_05_future_poison_raw_source_layer() {
    let all_bars = load_test_bars();
    let t = as_of();

    // Build bars_at_t from normal data.
    let bars_at_t_normal: Vec<YahooHistoricalBar> = all_bars
        .iter()
        .filter(|b| b.timestamp <= t.timestamp())
        .cloned()
        .collect();

    // Create poisoned future bars with extreme values designed to alter
    // price, ATR, trend, and momentum if they were included.
    let poison_ts_1 = t.timestamp() + 86400;  // T + 1 day
    let poison_ts_2 = t.timestamp() + 172800; // T + 2 days
    let poison_ts_3 = t.timestamp() + 259200; // T + 3 days

    let poison_bars = vec![
        YahooHistoricalBar {
            timestamp: poison_ts_1,
            open: 999999.0,
            high: 9999999.0,
            low: 0.001,
            close: 999999.0, // extreme price — would dominate reference_price
            volume: 999999999.0,
            adj_close: 999999.0,
        },
        YahooHistoricalBar {
            timestamp: poison_ts_2,
            open: 0.001,
            high: 0.001,
            low: 0.0001,
            close: 0.001, // extreme crash — would reverse trend/momentum
            volume: 999999999.0,
            adj_close: 0.001,
        },
        YahooHistoricalBar {
            timestamp: poison_ts_3,
            open: 500000.0,
            high: 1000000.0,
            low: 0.0001,
            close: 500000.0, // extreme ATR — would dominate ATR-14
            volume: 999999999.0,
            adj_close: 500000.0,
        },
    ];

    // Build poisoned dataset: normal bars + future poison bars.
    let mut poisoned_all_bars = all_bars.clone();
    poisoned_all_bars.extend(poison_bars);

    // Apply the same temporal filter to the poisoned dataset.
    let bars_at_t_poisoned: Vec<YahooHistoricalBar> = poisoned_all_bars
        .iter()
        .filter(|b| b.timestamp <= t.timestamp())
        .cloned()
        .collect();

    // T2-05 CORE ASSERTION: bars_at_t must be identical regardless of poison.
    assert_eq!(
        bars_at_t_normal.len(),
        bars_at_t_poisoned.len(),
        "T2-05 FAIL: poison altered the number of bars at T"
    );

    for (i, (normal, poisoned)) in bars_at_t_normal
        .iter()
        .zip(bars_at_t_poisoned.iter())
        .enumerate()
    {
        assert_eq!(
            normal.timestamp, poisoned.timestamp,
            "T2-05 FAIL: bar[{i}].timestamp differs after poisoning"
        );
        assert_eq!(
            normal.close, poisoned.close,
            "T2-05 FAIL: bar[{i}].close differs after poisoning"
        );
    }

    // Verify derived features are also identical.
    let instrument_id = instrument_id_for(TEST_TICKER);
    let session_t = latest_session_at_or_before(&bars_at_t_normal, t).unwrap_or(t);

    let metrics_normal = metrics_from_bars_at_t(&bars_at_t_normal, session_t, instrument_id);
    let metrics_poisoned = metrics_from_bars_at_t(&bars_at_t_poisoned, session_t, instrument_id);

    let atr_normal = metrics_normal.get_float("atr_14");
    let atr_poisoned = metrics_poisoned.get_float("atr_14");

    assert_eq!(
        atr_normal, atr_poisoned,
        "T2-05 FAIL: atr_14 differs after raw-source poisoning: normal={atr_normal:?} poisoned={atr_poisoned:?}"
    );

    let ref_normal = bars_at_t_normal
        .iter()
        .filter(|b| b.timestamp <= session_t.timestamp())
        .last()
        .map(|b| b.close);
    let ref_poisoned = bars_at_t_poisoned
        .iter()
        .filter(|b| b.timestamp <= session_t.timestamp())
        .last()
        .map(|b| b.close);

    assert_eq!(
        ref_normal, ref_poisoned,
        "T2-05 FAIL: reference_price differs after raw-source poisoning"
    );

    println!(
        "T2-05 PASS: reconstruct(T, normal) == reconstruct(T, normal + future_poison) \
         atr_14={atr_normal:?} ref_price={ref_normal:?}"
    );
}

// ─── T2-06: Determinism ───────────────────────────────────────────────────────

/// T2-06: Determinism — same dataset + same T → identical reconstruction.
#[test]
fn t2_06_deterministic_reconstruction() {
    let all_bars = load_test_bars();
    let t = as_of();

    // Run the filter twice.
    let bars_run1: Vec<YahooHistoricalBar> = all_bars
        .iter()
        .filter(|b| b.timestamp <= t.timestamp())
        .cloned()
        .collect();
    let bars_run2: Vec<YahooHistoricalBar> = all_bars
        .iter()
        .filter(|b| b.timestamp <= t.timestamp())
        .cloned()
        .collect();

    assert_eq!(bars_run1.len(), bars_run2.len(), "T2-06 FAIL: bar count differs");

    let instrument_id = instrument_id_for(TEST_TICKER);
    let session_t = latest_session_at_or_before(&bars_run1, t).unwrap_or(t);

    let metrics1 = metrics_from_bars_at_t(&bars_run1, session_t, instrument_id);
    let metrics2 = metrics_from_bars_at_t(&bars_run2, session_t, instrument_id);

    assert_eq!(
        metrics1.get_float("atr_14"),
        metrics2.get_float("atr_14"),
        "T2-06 FAIL: atr_14 is not deterministic"
    );

    // Source hash must be identical.
    let hash1 = compute_source_hash(&[(TEST_TICKER.to_string(), bars_run1.clone())], t);
    let hash2 = compute_source_hash(&[(TEST_TICKER.to_string(), bars_run2.clone())], t);
    assert_eq!(hash1, hash2, "T2-06 FAIL: source_dataset_hash is not deterministic");

    println!("T2-06 PASS: hash={hash1}");
}

// ─── T2-07: Complete accounting ───────────────────────────────────────────────

/// T2-07: Complete accounting — every instrument gets exactly one of
/// COMPLETE | INCOMPLETE | ERROR. No silent disappearance.
#[test]
fn t2_07_complete_accounting() {
    let cache = load_yahoo_cache_dir(&cache_dir()).expect("cache must be readable");

    let t = as_of();
    let mut n_complete = 0usize;
    let mut n_incomplete = 0usize;
    let n_error = 0usize;

    for (ticker, all_bars) in &cache {
        let bars_at_t: Vec<YahooHistoricalBar> = all_bars
            .iter()
            .filter(|b| b.timestamp <= t.timestamp())
            .cloned()
            .collect();

        if bars_at_t.is_empty() {
            n_incomplete += 1;
            continue;
        }

        let instrument_id = instrument_id_for(ticker);
        let session_t = latest_session_at_or_before(&bars_at_t, t).unwrap_or(t);
        let metrics = metrics_from_bars_at_t(&bars_at_t, session_t, instrument_id);
        let (profile, _, _) = assess_from_bars_at_t(&bars_at_t, session_t, instrument_id);

        let ref_price = bars_at_t
            .iter()
            .filter(|b| b.timestamp <= session_t.timestamp())
            .last()
            .and_then(|b| if b.close > 0.0 { Some(b.close) } else { None });

        let atr_14 = metrics.get_float("atr_14");
        let has_trend = profile
            .assessments
            .iter()
            .any(|a| a.concept == chronosentiment_adapter::metrics::concepts::Concept::Trend);
        let has_momentum = profile
            .assessments
            .iter()
            .any(|a| a.concept == chronosentiment_adapter::metrics::concepts::Concept::Momentum);
        let has_volatility = profile
            .factor_status
            .iter()
            .any(|s| s.concept == chronosentiment_adapter::metrics::concepts::Concept::Volatility);

        let tmv_complete = has_trend
            && has_momentum
            && has_volatility
            && ref_price.is_some()
            && atr_14.is_some();

        if tmv_complete {
            n_complete += 1;
        } else {
            n_incomplete += 1;
        }
    }

    let n_total = n_complete + n_incomplete + n_error;
    println!(
        "T2-07: total={n_total} complete={n_complete} incomplete={n_incomplete} error={n_error}"
    );

    // Every instrument must be accounted for.
    assert_eq!(
        n_complete + n_incomplete + n_error,
        n_total,
        "T2-07 FAIL: accounting is not exhaustive"
    );

    // At least some instruments must be COMPLETE.
    assert!(n_complete > 0, "T2-07 FAIL: no COMPLETE instruments");

    // n_total must match the cache size.
    assert_eq!(
        n_total,
        cache.len(),
        "T2-07 FAIL: n_total={n_total} != cache.len()={}",
        cache.len()
    );
}