//! Feature availability + session composition audits.

use super::data::{filter_days, per_day_stats, wl_stats, DISCOVERY_DAYS, HOLDOUT_EX_ANOMALY};
use super::types::{FeatureAvailability, SessionComposition, TradeRow};

fn present_missing(rows: &[&TradeRow], pred: impl Fn(&TradeRow) -> bool) -> (usize, usize) {
    let present = rows.iter().filter(|r| pred(r)).count();
    (present, rows.len().saturating_sub(present))
}

fn distinct(rows: &[&TradeRow], f: impl Fn(&TradeRow) -> &str) -> usize {
    let mut s = std::collections::BTreeSet::new();
    for r in rows {
        s.insert(f(r).to_string());
    }
    s.len()
}

pub fn feature_availability_matrix(
    discovery: &[&TradeRow],
    holdout: &[&TradeRow],
) -> Vec<FeatureAvailability> {
    let mut out = Vec::new();

    let mut push = |feature_id: &str,
                    family: &str,
                    status: &str,
                    sources: &[&str],
                    pred: &dyn Fn(&TradeRow) -> bool,
                    distinct_f: &dyn Fn(&TradeRow) -> &str,
                    notes: &str,
                    consistent: bool| {
        let (dp, dm) = present_missing(discovery, pred);
        let (hp, hm) = present_missing(holdout, pred);
        out.push(FeatureAvailability {
            feature_id: feature_id.into(),
            family: family.into(),
            status: status.into(),
            source_fields: sources.iter().map(|s| (*s).into()).collect(),
            decision_time: status != "UNAVAILABLE",
            existed_before_entry: if status == "UNAVAILABLE" {
                "NOT_RECORDED".into()
            } else {
                "YES_AT_OR_BEFORE_ENTRY_LATCH".into()
            },
            discovery_present_n: dp,
            discovery_missing_n: dm,
            holdout_present_n: hp,
            holdout_missing_n: hm,
            n_distinct_discovery: if status == "UNAVAILABLE" {
                0
            } else {
                distinct(discovery, distinct_f)
            },
            definition_consistent_across_windows: consistent,
            notes: notes.into(),
        });
    };

    push(
        "direction",
        "directional_state",
        "AVAILABLE",
        &["direction"],
        &|r| r.direction == "LONG" || r.direction == "SHORT",
        &|r| r.direction.as_str(),
        "Binary LONG/SHORT at plan open.",
        true,
    );
    push(
        "of_supported",
        "of_support_state",
        "AVAILABLE",
        &["order_flow_supported"],
        &|r| r.of_supported == "OF_SUP=True" || r.of_supported == "OF_SUP=False",
        &|r| r.of_supported.as_str(),
        "Missing entirely pre-OF-era → OF_SUP_UNAVAILABLE (not coerced to False).",
        true,
    );
    push(
        "dir_x_of",
        "direction_x_of",
        "AVAILABLE",
        &["direction", "order_flow_supported"],
        &|r| !r.dir_x_of.contains("UNAVAILABLE"),
        &|r| r.dir_x_of.as_str(),
        "Interaction only where OF support is recorded.",
        true,
    );
    push(
        "hour_ist",
        "temporal_descriptors",
        "AVAILABLE",
        &["entry_time_ms"],
        &|r| r.hour_ist != "??",
        &|r| r.hour_ist.as_str(),
        "Previously single-session dominant in v1; retained as diagnostic family only.",
        true,
    );
    push(
        "imb_bucket",
        "market_state",
        "AVAILABLE",
        &["order_flow_imbalance"],
        &|r| r.imb_bucket != "IMB_UNAVAILABLE",
        &|r| r.imb_bucket.as_str(),
        "v1 replication failed; same bucket defs preserved for lineage (not re-mined as new evidence).",
        true,
    );
    push(
        "atr_bucket",
        "market_state",
        "AVAILABLE",
        &["atr_14", "entry_price"],
        &|r| r.atr_bucket != "ATR_REL_UNAVAILABLE",
        &|r| r.atr_bucket.as_str(),
        "v1 replication failed; volatility proxy only.",
        true,
    );
    push(
        "net_rr_bucket",
        "market_state",
        "AVAILABLE",
        &["net_rr_ratio"],
        &|r| r.net_rr_bucket != "NET_RR_UNAVAILABLE",
        &|r| r.net_rr_bucket.as_str(),
        "Plan geometry at entry (not path).",
        true,
    );
    push(
        "trend",
        "market_state",
        "UNAVAILABLE",
        &[],
        &|_| false,
        &|_| "",
        "No look-ahead-safe bar join; not imputed from later prices.",
        true,
    );
    push(
        "momentum",
        "market_state",
        "UNAVAILABLE",
        &[],
        &|_| false,
        &|_| "",
        "No look-ahead-safe bar join; not imputed.",
        true,
    );
    push(
        "price_location",
        "market_state",
        "UNAVAILABLE",
        &[],
        &|_| false,
        &|_| "",
        "No look-ahead-safe structure join; not imputed.",
        true,
    );
    push(
        "signal_persistence",
        "of_support_state",
        "UNAVAILABLE",
        &[],
        &|_| false,
        &|_| "",
        "Multi-bar persistence not recorded as decision-time series.",
        true,
    );

    let _ = discovery;
    let _ = holdout;
    out
}

pub fn session_composition(all: &[TradeRow]) -> Vec<SessionComposition> {
    let windows: Vec<(&str, &[&str])> = vec![
        ("discovery", DISCOVERY_DAYS),
        ("holdout_pre_ex_anomaly", HOLDOUT_EX_ANOMALY),
    ];
    let mut out = Vec::new();
    for (name, days) in windows {
        let rows = filter_days(all, days);
        let n = rows.len();
        let overall = wl_stats(&rows, Some(n));
        let long_n = rows.iter().filter(|r| r.direction == "LONG").count();
        let of_true = rows
            .iter()
            .filter(|r| r.of_supported == "OF_SUP=True")
            .count();
        let of_known = rows
            .iter()
            .filter(|r| r.of_supported == "OF_SUP=True" || r.of_supported == "OF_SUP=False")
            .count();
        let of_unavail = rows
            .iter()
            .filter(|r| r.of_supported == "OF_SUP_UNAVAILABLE")
            .count();
        out.push(SessionComposition {
            window: name.into(),
            n,
            wr_pct: overall.win_rate_pct,
            long_share_pct: if n == 0 {
                0.0
            } else {
                100.0 * long_n as f64 / n as f64
            },
            of_sup_true_share_pct: if of_known == 0 {
                None
            } else {
                Some(100.0 * of_true as f64 / of_known as f64)
            },
            of_unavailable_share_pct: if n == 0 {
                0.0
            } else {
                100.0 * of_unavail as f64 / n as f64
            },
            per_day: per_day_stats(&rows),
        });
    }
    out
}

pub fn integrity_check(all: &[TradeRow]) -> (bool, Vec<String>) {
    let mut notes = Vec::new();
    let disc = filter_days(all, DISCOVERY_DAYS);
    let hold = filter_days(all, HOLDOUT_EX_ANOMALY);
    if disc.is_empty() {
        notes.push("DISCOVERY_EMPTY".into());
    }
    if hold.is_empty() {
        notes.push("HOLDOUT_EMPTY".into());
    }
    let bad_net = all.iter().filter(|r| r.net.is_nan()).count();
    if bad_net > 0 {
        notes.push(format!("NAN_NET_PNL={bad_net}"));
    }
    // Post-entry fields are never loaded into TradeRow predictors — structural pass note
    notes.push("POST_ENTRY_PREDICTORS_EXCLUDED_BY_SCHEMA".into());
    notes.push(format!(
        "discovery_n={} holdout_ex_anomaly_n={}",
        disc.len(),
        hold.len()
    ));
    let pass = disc.len() >= 50 && hold.len() >= 50 && bad_net == 0;
    if !pass {
        notes.push("INTEGRITY_FAIL_MIN_N_OR_NAN".into());
    }
    (pass, notes)
}
