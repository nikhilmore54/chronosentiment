//! Immutable v1 hypothesis lineage — prevent circular rediscovery-as-evidence.

use super::types::LineageEntry;

pub fn v1_lineage_registry() -> Vec<LineageEntry> {
    vec![
        LineageEntry {
            lineage_id: "V1-H_DIR".into(),
            original_feature: "direction LONG vs SHORT".into(),
            discovery_summary: "Discovery gap ~+21pp LONG>SHORT (Nov3–5 journal)".into(),
            holdout_result: "FAILS_REPLICATION".into(),
            failure_reason: "Holdout gap flipped (~−1.3pp on ex-1029)".into(),
            related_variants: vec![
                "LONG-only filter".into(),
                "SHORT suppress".into(),
                "direction×hour".into(),
            ],
            future_analysis_rule: "Do not present LONG/SHORT WR slices as independent new evidence unless a genuinely new decision-time feature family is added.".into(),
        },
        LineageEntry {
            lineage_id: "V1-H_OF".into(),
            original_feature: "OF_SUP=True vs False".into(),
            discovery_summary: "Discovery gap ~+13pp".into(),
            holdout_result: "FAILS_REPLICATION".into(),
            failure_reason: "Holdout gap flipped (~−19pp); OF_SUP=False stronger in holdout OF-era sample".into(),
            related_variants: vec!["OF latch require".into(), "OF label strata".into()],
            future_analysis_rule: "Not an OF ±0.30 proposal. Rediscovery of OF_SUP slices is lineage-blocked as Gate2 evidence.".into(),
        },
        LineageEntry {
            lineage_id: "V1-H_DIR_OF".into(),
            original_feature: "LONG|OF_SUP=True vs SHORT|OF_SUP=True".into(),
            discovery_summary: "Discovery gap ~+20pp".into(),
            holdout_result: "INCONCLUSIVE_UNDERPOWERED".into(),
            failure_reason: "Holdout SHORT|OF_SUP side n too small".into(),
            related_variants: vec!["direction×OF".into()],
            future_analysis_rule: "May re-test only with predeclared min-n after more OF-era sessions; not by mining new OF thresholds.".into(),
        },
        LineageEntry {
            lineage_id: "V1-H_IMB".into(),
            original_feature: "ABS_IMB_≥0.70 vs <0.30".into(),
            discovery_summary: "Discovery gap ~+16pp".into(),
            holdout_result: "FAILS_REPLICATION".into(),
            failure_reason: "Holdout gap flipped".into(),
            related_variants: vec!["imbalance tertiles".into(), "imbalance×direction".into()],
            future_analysis_rule: "Bucket re-cuts of imbalance alone are re-expressions, not new evidence.".into(),
        },
        LineageEntry {
            lineage_id: "V1-H_ATR".into(),
            original_feature: "ATR_REL_<20bps vs 40–80bps".into(),
            discovery_summary: "Discovery gap ~+16pp".into(),
            holdout_result: "FAILS_REPLICATION".into(),
            failure_reason: "Holdout gap flipped".into(),
            related_variants: vec!["ATR quintiles".into(), "ATR×direction".into()],
            future_analysis_rule: "ATR re-bucketing alone is lineage-blocked as independent Gate2 evidence.".into(),
        },
        LineageEntry {
            lineage_id: "V1-HOUR".into(),
            original_feature: "entry hour IST".into(),
            discovery_summary: "Descriptive; several hours single-session dominant".into(),
            holdout_result: "NOT_PROMOTED".into(),
            failure_reason: "Session-mix / single-session domination risk".into(),
            related_variants: vec!["hour bans".into(), "session TOD filters".into()],
            future_analysis_rule: "Keep as diagnostic only; never promote single-session hour leads.".into(),
        },
    ]
}

pub fn lineage_for_feature(feature_id: &str) -> Option<&'static str> {
    match feature_id {
        "direction" => Some("V1-H_DIR"),
        "of_supported" => Some("V1-H_OF"),
        "dir_x_of" => Some("V1-H_DIR_OF"),
        "imb_bucket" => Some("V1-H_IMB"),
        "atr_bucket" => Some("V1-H_ATR"),
        "hour_ist" => Some("V1-HOUR"),
        _ => None,
    }
}
