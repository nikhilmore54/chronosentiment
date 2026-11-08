//! Structured association discovery across predeclared feature families.

use super::data::wl_stats;
use super::lineage::lineage_for_feature;
use super::types::{CandidateAssociation, Contrast, SessionEffect, TradeRow};

fn key_of<'a>(r: &'a TradeRow, feature_id: &str) -> &'a str {
    match feature_id {
        "direction" => r.direction.as_str(),
        "of_supported" => r.of_supported.as_str(),
        "dir_x_of" => r.dir_x_of.as_str(),
        "imb_bucket" => r.imb_bucket.as_str(),
        "atr_bucket" => r.atr_bucket.as_str(),
        "net_rr_bucket" => r.net_rr_bucket.as_str(),
        "hour_ist" => r.hour_ist.as_str(),
        _ => "?",
    }
}

fn contrast_keys(
    rows: &[&TradeRow],
    feature_id: &str,
    high: &str,
    low: &str,
    book_n: usize,
) -> Contrast {
    let hi: Vec<&TradeRow> = rows
        .iter()
        .copied()
        .filter(|r| key_of(r, feature_id) == high)
        .collect();
    let lo: Vec<&TradeRow> = rows
        .iter()
        .copied()
        .filter(|r| key_of(r, feature_id) == low)
        .collect();
    let high_s = wl_stats(&hi, Some(book_n));
    let low_s = wl_stats(&lo, Some(book_n));
    let gap = match (high_s.win_rate_pct, low_s.win_rate_pct) {
        (Some(a), Some(b)) => Some(a - b),
        _ => None,
    };
    Contrast {
        high: high_s,
        low: low_s,
        wr_gap_pp: gap,
    }
}

fn session_effects(
    rows: &[&TradeRow],
    feature_id: &str,
    high: &str,
    low: &str,
) -> Vec<SessionEffect> {
    let mut days: Vec<String> = rows.iter().map(|r| r.day_ist.clone()).collect();
    days.sort();
    days.dedup();
    let mut out = Vec::new();
    for day in days {
        let day_rows: Vec<&TradeRow> = rows.iter().copied().filter(|r| r.day_ist == day).collect();
        let hi: Vec<&TradeRow> = day_rows
            .iter()
            .copied()
            .filter(|r| key_of(r, feature_id) == high)
            .collect();
        let lo: Vec<&TradeRow> = day_rows
            .iter()
            .copied()
            .filter(|r| key_of(r, feature_id) == low)
            .collect();
        let hs = wl_stats(&hi, None);
        let ls = wl_stats(&lo, None);
        let gap = match (hs.win_rate_pct, ls.win_rate_pct) {
            (Some(a), Some(b)) if hi.len() >= 3 && lo.len() >= 3 => Some(a - b),
            _ => None,
        };
        let sign = match gap {
            Some(g) if g > 1.0 => "POSITIVE".to_string(),
            Some(g) if g < -1.0 => "NEGATIVE".to_string(),
            Some(_) => "NEAR_ZERO".to_string(),
            None => "UNEVALUABLE".to_string(),
        };
        out.push(SessionEffect {
            day,
            high_n: hi.len(),
            high_wr: hs.win_rate_pct,
            low_n: lo.len(),
            low_wr: ls.win_rate_pct,
            gap_pp: gap,
            sign,
        });
    }
    out
}

struct Spec {
    family: &'static str,
    feature_id: &'static str,
    high: &'static str,
    low: &'static str,
    of_era_only: bool,
    exclude_unavailable: bool,
}

/// Predeclared family contrasts — not exhaustive slice mining.
const SPECS: &[Spec] = &[
    Spec {
        family: "directional_state",
        feature_id: "direction",
        high: "LONG",
        low: "SHORT",
        of_era_only: false,
        exclude_unavailable: false,
    },
    Spec {
        family: "of_support_state",
        feature_id: "of_supported",
        high: "OF_SUP=True",
        low: "OF_SUP=False",
        of_era_only: true,
        exclude_unavailable: true,
    },
    Spec {
        family: "direction_x_of",
        feature_id: "dir_x_of",
        high: "LONG|OF_SUP=True",
        low: "SHORT|OF_SUP=True",
        of_era_only: true,
        exclude_unavailable: true,
    },
    Spec {
        family: "market_state",
        feature_id: "imb_bucket",
        high: "ABS_IMB_≥0.70",
        low: "ABS_IMB_<0.30",
        of_era_only: true,
        exclude_unavailable: true,
    },
    Spec {
        family: "market_state",
        feature_id: "atr_bucket",
        high: "ATR_REL_<20bps",
        low: "ATR_REL_40_80bps",
        of_era_only: false,
        exclude_unavailable: true,
    },
    Spec {
        family: "market_state",
        feature_id: "net_rr_bucket",
        high: "NET_RR_1_1.5",
        low: "NET_RR_<1",
        of_era_only: false,
        exclude_unavailable: true,
    },
    // Temporal: reported diagnostically; hour mining of all hours is intentionally avoided.
    Spec {
        family: "temporal_descriptors",
        feature_id: "hour_ist",
        high: "06",
        low: "20",
        of_era_only: false,
        exclude_unavailable: false,
    },
];

pub fn discover_associations(discovery: &[&TradeRow]) -> Vec<CandidateAssociation> {
    let book_n = discovery.len();
    let mut out = Vec::new();
    for spec in SPECS {
        let mut rows: Vec<&TradeRow> = discovery.to_vec();
        if spec.of_era_only {
            rows.retain(|r| r.of_era);
        }
        if spec.exclude_unavailable {
            rows.retain(|r| {
                let k = key_of(r, spec.feature_id);
                !k.contains("UNAVAILABLE")
            });
        }
        let contrast = contrast_keys(&rows, spec.feature_id, spec.high, spec.low, book_n);
        let sessions = session_effects(&rows, spec.feature_id, spec.high, spec.low);
        let mut flags = Vec::new();
        if lineage_for_feature(spec.feature_id).is_some() {
            flags.push("V1_LINEAGE_REGISTERED".into());
        }
        if spec.family == "temporal_descriptors" {
            flags.push("TEMPORAL_DIAGNOSTIC_ONLY".into());
        }
        if contrast.high.n < 15 || contrast.low.n < 15 {
            flags.push("LOW_COUNT_COMPARISON".into());
        }
        out.push(CandidateAssociation {
            candidate_id: format!("CAND-{}-{}-vs-{}", spec.feature_id, spec.high, spec.low),
            family: spec.family.into(),
            feature_id: spec.feature_id.into(),
            high_key: spec.high.into(),
            low_key: spec.low.into(),
            discovery: contrast,
            session_effects: sessions,
            flags,
            v1_lineage_id: lineage_for_feature(spec.feature_id).map(|s| s.to_string()),
        });
    }
    out
}

pub fn holdout_contrast(
    holdout: &[&TradeRow],
    feature_id: &str,
    high: &str,
    low: &str,
    of_era_only: bool,
    exclude_unavailable: bool,
) -> Contrast {
    let book_n = holdout.len();
    let mut rows: Vec<&TradeRow> = holdout.to_vec();
    if of_era_only {
        rows.retain(|r| r.of_era);
    }
    if exclude_unavailable {
        rows.retain(|r| !key_of(r, feature_id).contains("UNAVAILABLE"));
    }
    contrast_keys(&rows, feature_id, high, low, book_n)
}

pub fn spec_meta(feature_id: &str) -> Option<(bool, bool)> {
    SPECS
        .iter()
        .find(|s| s.feature_id == feature_id)
        .map(|s| (s.of_era_only, s.exclude_unavailable))
}
