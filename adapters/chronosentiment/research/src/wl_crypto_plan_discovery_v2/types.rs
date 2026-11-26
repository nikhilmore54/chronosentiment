//! Shared types for RP-608 Crypto plan discovery v2.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawPlan {
    #[serde(default)]
    pub plan_id: Option<String>,
    #[serde(default)]
    pub direction: Option<String>,
    #[serde(default)]
    pub entry_time_ms: Option<i64>,
    #[serde(default)]
    pub exit_time_ms: Option<i64>,
    #[serde(default)]
    pub entry_price: Option<f64>,
    #[serde(default)]
    pub reference_price: Option<f64>,
    #[serde(default)]
    pub net_pnl_usdt: Option<f64>,
    #[serde(default)]
    pub atr_14: Option<f64>,
    #[serde(default)]
    pub order_flow_supported: Option<bool>,
    #[serde(default)]
    pub order_flow_imbalance: Option<f64>,
    #[serde(default)]
    pub order_flow_label: Option<String>,
    #[serde(default)]
    pub order_flow_latched_at_open: Option<bool>,
    #[serde(default)]
    pub net_rr_ratio: Option<f64>,
    #[serde(default)]
    pub gross_rr_ratio: Option<f64>,
    #[serde(default)]
    pub horizon_id: Option<String>,
    #[serde(default)]
    pub data_quality_status: Option<String>,
    #[serde(default)]
    pub stop_loss_price: Option<f64>,
    #[serde(default)]
    pub invalidation_price: Option<f64>,
    #[serde(default)]
    pub profit_target_price: Option<f64>,
    #[serde(default)]
    pub symbol: Option<String>,
    // Present in some rows; never used as predictors
    #[serde(default)]
    pub mfe_exec_price: Option<f64>,
    #[serde(default)]
    pub mae_exec_price: Option<f64>,
    #[serde(default)]
    pub exit_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TradeRow {
    pub plan_id: String,
    pub day_ist: String,
    pub window: String,
    pub win: bool,
    pub net: f64,
    pub direction: String,
    pub of_supported: String,
    pub dir_x_of: String,
    pub imb_bucket: String,
    pub atr_bucket: String,
    pub net_rr_bucket: String,
    pub hour_ist: String,
    pub of_era: bool,
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureAvailability {
    pub feature_id: String,
    pub family: String,
    pub status: String,
    pub source_fields: Vec<String>,
    pub decision_time: bool,
    pub existed_before_entry: String,
    pub discovery_present_n: usize,
    pub discovery_missing_n: usize,
    pub holdout_present_n: usize,
    pub holdout_missing_n: usize,
    pub n_distinct_discovery: usize,
    pub definition_consistent_across_windows: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WlStats {
    pub n: usize,
    pub wins: usize,
    pub losses: usize,
    pub win_rate_pct: Option<f64>,
    pub wilson_low_95_pct: Option<f64>,
    pub net_sum: f64,
    pub coverage_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CandidateAssociation {
    pub candidate_id: String,
    pub family: String,
    pub feature_id: String,
    pub high_key: String,
    pub low_key: String,
    pub discovery: Contrast,
    pub session_effects: Vec<SessionEffect>,
    pub flags: Vec<String>,
    pub v1_lineage_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Contrast {
    pub high: WlStats,
    pub low: WlStats,
    pub wr_gap_pp: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionEffect {
    pub day: String,
    pub high_n: usize,
    pub high_wr: Option<f64>,
    pub low_n: usize,
    pub low_wr: Option<f64>,
    pub gap_pp: Option<f64>,
    pub sign: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StabilityAssessment {
    pub candidate_id: String,
    pub association_ok: bool,
    pub stability_ok: bool,
    pub specificity_notes: Vec<String>,
    pub pooled_vs_session_simpson_risk: bool,
    pub single_session_dominant: bool,
    pub low_count_comparison: bool,
    pub large_coverage_drop_if_selected_pct: Option<f64>,
    pub sign_flip_sessions: usize,
    pub sessions_evaluable: usize,
    pub promote_to_compiler: bool,
    pub block_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HypothesisRecord {
    pub hypothesis_id: String,
    pub book: String,
    pub population: String,
    pub feature: String,
    pub comparison: String,
    pub outcome: String,
    pub expected_direction: String,
    pub mechanism: String,
    pub discovery_evidence: Contrast,
    pub replication_evidence: Option<Contrast>,
    pub replication_verdict: String,
    pub known_limitations: Vec<String>,
    pub falsification_condition: String,
    pub gate_status: String,
    pub v1_lineage_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Gate2CriterionResult {
    pub criterion_id: String,
    pub description: String,
    pub satisfied: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Gate2Assessment {
    pub hypothesis_id: String,
    pub criteria: Vec<Gate2CriterionResult>,
    pub all_satisfied: bool,
    pub disposition: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LineageEntry {
    pub lineage_id: String,
    pub original_feature: String,
    pub discovery_summary: String,
    pub holdout_result: String,
    pub failure_reason: String,
    pub related_variants: Vec<String>,
    pub future_analysis_rule: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionComposition {
    pub window: String,
    pub n: usize,
    pub wr_pct: Option<f64>,
    pub long_share_pct: f64,
    pub of_sup_true_share_pct: Option<f64>,
    pub of_unavailable_share_pct: f64,
    pub per_day: BTreeMap<String, WlStats>,
}

#[derive(Debug, Serialize)]
pub struct PipelineReport {
    pub protocol: String,
    pub version: String,
    pub authority: String,
    pub baseline_frozen: bool,
    pub gate2_criteria_changed: bool,
    pub integrity_pass: bool,
    pub integrity_notes: Vec<String>,
    pub feature_availability: Vec<FeatureAvailability>,
    pub session_composition: Vec<SessionComposition>,
    pub lineage_registry: Vec<LineageEntry>,
    pub candidates: Vec<CandidateAssociation>,
    pub stability: Vec<StabilityAssessment>,
    pub hypotheses: Vec<HypothesisRecord>,
    pub research_only_blocked_records: Vec<HypothesisRecord>,
    pub gate2_assessments: Vec<Gate2Assessment>,
    pub gate2_disposition: String,
    pub research_conclusion: String,
}
