//! CS-P-004 Historical Research Laboratory v0.1.
//!
//! Consumes an already-reconstructed `DecisionLedger` and `OutcomeReport`.
//! Never calls `decide_at`. Never tunes thresholds. Never mutates B4.
//! Walk-forward is temporal slicing of the same `unfrozen-dev` policy, not training.

use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Datelike, TimeZone, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::backtest::DecisionLedger;
use super::outcome::OutcomeReport;
use super::performance::{measure_performance, PerformanceReport, ReturnStats};
use super::DecisionAction;

pub const SCHEMA_VERSION: &str = "csp004.lab.0";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DecisionContext {
    pub decision_id: Uuid,
    pub instrument_label: String,
    pub trend: Option<String>,
    pub trend_strength: Option<String>,
    pub momentum: Option<String>,
    pub momentum_strength: Option<String>,
    pub volatility: Option<String>,
    pub confidence_status: Option<String>,
    pub mapping_rule: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ActionCounts {
    pub long: u32,
    pub short: u32,
    pub no_trade: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TransitionCounts {
    pub long_to_short: u32,
    pub short_to_long: u32,
    pub long_to_long: u32,
    pub short_to_short: u32,
    pub involving_no_trade: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DecisionBehaviour {
    pub n_records: u32,
    pub counts: ActionCounts,
    pub by_instrument: BTreeMap<String, ActionCounts>,
    pub by_year: BTreeMap<i32, ActionCounts>,
    pub confidence_counts: BTreeMap<String, u32>,
    pub transitions: TransitionCounts,
    pub streak_lengths: BTreeMap<u32, u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RegimeMix {
    pub dimension: String,
    pub value: String,
    pub counts: ActionCounts,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SlicePerformance {
    pub dimension: String,
    pub value: String,
    pub n_records: u32,
    pub performance: PerformanceReport,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WalkForwardFold {
    pub name: String,
    pub train_end: DateTime<Utc>,
    pub test_start: DateTime<Utc>,
    pub test_end: DateTime<Utc>,
    pub train_n: u32,
    pub test_n: u32,
    pub train_counts: ActionCounts,
    pub test_performance: PerformanceReport,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RobustnessCell {
    pub dimension: String,
    pub value: String,
    pub horizon_days: u32,
    pub n_observed: u32,
    pub n_unavailable: u32,
    pub mean: Option<f64>,
    pub mean_sign: Option<i8>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VsBaselineRow {
    pub action: DecisionAction,
    pub horizon_days: u32,
    pub n_decisions: u32,
    pub n_observed: u32,
    pub n_unavailable: u32,
    pub mean: Option<f64>,
    /// Descriptive: observed mean compared with standing aside (return 0). None if unobserved.
    pub mean_vs_stand_aside: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CoverageNotes {
    pub n_records: u32,
    pub n_with_any_lake_outcome: u32,
    pub n_short: u32,
    pub n_short_with_outcome: u32,
    pub n_no_trade: u32,
    pub n_volatility_labeled: u32,
    pub short_unevaluated: bool,
    pub no_trade_unevaluable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LaboratoryReport {
    pub schema_version: String,
    pub decision_engine_version: String,
    pub ledger_identity_hash: String,
    pub outcome_identity_hash: String,
    pub behavior: DecisionBehaviour,
    pub regime_mix: Vec<RegimeMix>,
    pub stratification: Vec<SlicePerformance>,
    pub walk_forward: Vec<WalkForwardFold>,
    pub robustness: Vec<RobustnessCell>,
    pub vs_baseline: Vec<VsBaselineRow>,
    pub coverage: CoverageNotes,
    pub content_hash: String,
}

pub struct LaboratoryInput<'a> {
    pub ledger: &'a DecisionLedger,
    pub outcomes: &'a OutcomeReport,
    pub context: &'a [DecisionContext],
}

/// Pure analysis. Same ledger + outcomes + context → same `content_hash`.
pub fn run_laboratory(input: LaboratoryInput<'_>) -> LaboratoryReport {
    let ctx = index_context(input.context);
    let behavior = behaviour(input.ledger, &ctx);
    let regime_mix = regime_mix(input.ledger, &ctx);
    let stratification = stratify(input.ledger, input.outcomes, &ctx);
    let walk_forward = walk_forward(input.ledger, input.outcomes);
    let robustness = robustness_cells(&stratification);
    let vs_baseline = vs_baseline(input.ledger, input.outcomes);
    let coverage = coverage(input.ledger, input.outcomes, &ctx);

    let mut report = LaboratoryReport {
        schema_version: SCHEMA_VERSION.to_string(),
        decision_engine_version: input.ledger.engine_version.clone(),
        ledger_identity_hash: input.ledger.identity_hash(),
        outcome_identity_hash: input.outcomes.identity_hash(),
        behavior,
        regime_mix,
        stratification,
        walk_forward,
        robustness,
        vs_baseline,
        coverage,
        content_hash: String::new(),
    };
    report.content_hash = report_hash(&report);
    report
}

fn index_context(rows: &[DecisionContext]) -> HashMap<Uuid, &DecisionContext> {
    let mut map = HashMap::new();
    for row in rows {
        map.insert(row.decision_id, row);
    }
    map
}

fn action_name(action: DecisionAction) -> &'static str {
    match action {
        DecisionAction::Long => "LONG",
        DecisionAction::Short => "SHORT",
        DecisionAction::NoTrade => "NO_TRADE",
    }
}

fn empty_counts() -> ActionCounts {
    ActionCounts {
        long: 0,
        short: 0,
        no_trade: 0,
    }
}

fn bump(counts: &mut ActionCounts, action: DecisionAction) {
    match action {
        DecisionAction::Long => counts.long += 1,
        DecisionAction::Short => counts.short += 1,
        DecisionAction::NoTrade => counts.no_trade += 1,
    }
}

fn instrument_label<'a>(
    rec: &super::backtest::LedgerRecord,
    ctx: &HashMap<Uuid, &'a DecisionContext>,
) -> String {
    ctx.get(&rec.decision_id)
        .map(|c| c.instrument_label.clone())
        .unwrap_or_else(|| rec.instrument_id.to_string())
}

fn behaviour(ledger: &DecisionLedger, ctx: &HashMap<Uuid, &DecisionContext>) -> DecisionBehaviour {
    let mut counts = empty_counts();
    let mut by_instrument: BTreeMap<String, ActionCounts> = BTreeMap::new();
    let mut by_year: BTreeMap<i32, ActionCounts> = BTreeMap::new();
    let mut confidence_counts: BTreeMap<String, u32> = BTreeMap::new();

    for rec in &ledger.records {
        bump(&mut counts, rec.action);
        bump(
            by_instrument
                .entry(instrument_label(rec, ctx))
                .or_insert_with(empty_counts),
            rec.action,
        );
        bump(
            by_year
                .entry(rec.as_of_timestamp.year())
                .or_insert_with(empty_counts),
            rec.action,
        );
        let key = match rec.confidence_status {
            super::ConfidenceStatus::Unavailable => "unavailable".to_string(),
            super::ConfidenceStatus::Available => rec
                .confidence
                .map(|x| format!("{x:.4}"))
                .unwrap_or_else(|| "available_missing".to_string()),
        };
        *confidence_counts.entry(key).or_insert(0) += 1;
    }

    DecisionBehaviour {
        n_records: ledger.records.len() as u32,
        counts,
        by_instrument,
        by_year,
        confidence_counts,
        transitions: transitions(ledger),
        streak_lengths: streaks(ledger),
    }
}

fn transitions(ledger: &DecisionLedger) -> TransitionCounts {
    let mut grouped: BTreeMap<Uuid, Vec<DecisionAction>> = BTreeMap::new();
    for rec in &ledger.records {
        grouped
            .entry(rec.instrument_id)
            .or_default()
            .push(rec.action);
    }
    let mut t = TransitionCounts {
        long_to_short: 0,
        short_to_long: 0,
        long_to_long: 0,
        short_to_short: 0,
        involving_no_trade: 0,
    };
    for actions in grouped.values() {
        for pair in actions.windows(2) {
            match (pair[0], pair[1]) {
                (DecisionAction::Long, DecisionAction::Short) => t.long_to_short += 1,
                (DecisionAction::Short, DecisionAction::Long) => t.short_to_long += 1,
                (DecisionAction::Long, DecisionAction::Long) => t.long_to_long += 1,
                (DecisionAction::Short, DecisionAction::Short) => t.short_to_short += 1,
                _ => t.involving_no_trade += 1,
            }
        }
    }
    t
}

fn streaks(ledger: &DecisionLedger) -> BTreeMap<u32, u32> {
    let mut grouped: BTreeMap<Uuid, Vec<DecisionAction>> = BTreeMap::new();
    for rec in &ledger.records {
        grouped
            .entry(rec.instrument_id)
            .or_default()
            .push(rec.action);
    }
    let mut lengths: BTreeMap<u32, u32> = BTreeMap::new();
    for actions in grouped.values() {
        if actions.is_empty() {
            continue;
        }
        let mut len = 1u32;
        for i in 1..actions.len() {
            if actions[i] == actions[i - 1] {
                len += 1;
            } else {
                *lengths.entry(len).or_insert(0) += 1;
                len = 1;
            }
        }
        *lengths.entry(len).or_insert(0) += 1;
    }
    lengths
}

fn regime_mix(ledger: &DecisionLedger, ctx: &HashMap<Uuid, &DecisionContext>) -> Vec<RegimeMix> {
    let mut buckets: BTreeMap<(String, String), ActionCounts> = BTreeMap::new();
    for rec in &ledger.records {
        let labels = regime_labels(rec.decision_id, ctx);
        for (dim, value) in labels {
            bump(
                buckets.entry((dim, value)).or_insert_with(empty_counts),
                rec.action,
            );
        }
    }
    buckets
        .into_iter()
        .map(|((dimension, value), counts)| RegimeMix {
            dimension,
            value,
            counts,
        })
        .collect()
}

fn regime_labels(
    decision_id: Uuid,
    ctx: &HashMap<Uuid, &DecisionContext>,
) -> Vec<(String, String)> {
    let some = ctx.get(&decision_id);
    let unlabeled = "unlabeled".to_string();
    vec![
        (
            "trend".to_string(),
            some.and_then(|c| c.trend.clone())
                .unwrap_or_else(|| unlabeled.clone()),
        ),
        (
            "momentum".to_string(),
            some.and_then(|c| c.momentum.clone())
                .unwrap_or_else(|| unlabeled.clone()),
        ),
        (
            "volatility".to_string(),
            some.and_then(|c| c.volatility.clone())
                .unwrap_or_else(|| unlabeled.clone()),
        ),
        (
            "trend_strength".to_string(),
            some.and_then(|c| c.trend_strength.clone())
                .unwrap_or_else(|| unlabeled.clone()),
        ),
        (
            "momentum_strength".to_string(),
            some.and_then(|c| c.momentum_strength.clone())
                .unwrap_or_else(|| unlabeled.clone()),
        ),
        (
            "confidence_status".to_string(),
            some.and_then(|c| c.confidence_status.clone())
                .unwrap_or_else(|| unlabeled.clone()),
        ),
        (
            "mapping_rule".to_string(),
            some.and_then(|c| c.mapping_rule.clone())
                .filter(|s| !s.is_empty())
                .unwrap_or(unlabeled),
        ),
    ]
}

fn stratify(
    ledger: &DecisionLedger,
    outcomes: &OutcomeReport,
    ctx: &HashMap<Uuid, &DecisionContext>,
) -> Vec<SlicePerformance> {
    let mut out = Vec::new();
    out.extend(slices_for(
        ledger,
        outcomes,
        "action",
        |rec, _| Some(action_name(rec.action).to_string()),
        ctx,
    ));
    out.extend(slices_for(
        ledger,
        outcomes,
        "year",
        |rec, _| Some(rec.as_of_timestamp.year().to_string()),
        ctx,
    ));
    out.extend(slices_for(
        ledger,
        outcomes,
        "instrument",
        |rec, ctx| Some(instrument_label(rec, ctx)),
        ctx,
    ));
    out.extend(slices_for(
        ledger,
        outcomes,
        "trend",
        |rec, ctx| {
            Some(
                ctx.get(&rec.decision_id)
                    .and_then(|c| c.trend.clone())
                    .unwrap_or_else(|| "unlabeled".to_string()),
            )
        },
        ctx,
    ));
    out.extend(slices_for(
        ledger,
        outcomes,
        "momentum",
        |rec, ctx| {
            Some(
                ctx.get(&rec.decision_id)
                    .and_then(|c| c.momentum.clone())
                    .unwrap_or_else(|| "unlabeled".to_string()),
            )
        },
        ctx,
    ));
    out.extend(slices_for(
        ledger,
        outcomes,
        "action+trend",
        |rec, ctx| {
            let trend = ctx
                .get(&rec.decision_id)
                .and_then(|c| c.trend.clone())
                .unwrap_or_else(|| "unlabeled".to_string());
            Some(format!("{}+{trend}", action_name(rec.action)))
        },
        ctx,
    ));
    out
}

fn slices_for(
    ledger: &DecisionLedger,
    outcomes: &OutcomeReport,
    dimension: &str,
    label: impl Fn(&super::backtest::LedgerRecord, &HashMap<Uuid, &DecisionContext>) -> Option<String>,
    ctx: &HashMap<Uuid, &DecisionContext>,
) -> Vec<SlicePerformance> {
    let mut groups: BTreeMap<String, Vec<Uuid>> = BTreeMap::new();
    for rec in &ledger.records {
        if let Some(value) = label(rec, ctx) {
            groups.entry(value).or_default().push(rec.decision_id);
        }
    }
    groups
        .into_iter()
        .map(|(value, ids)| {
            let (sub_ledger, sub_outcomes) = subset(ledger, outcomes, &ids);
            let n_records = sub_ledger.records.len() as u32;
            SlicePerformance {
                dimension: dimension.to_string(),
                value,
                n_records,
                performance: measure_performance(&sub_ledger, &sub_outcomes),
            }
        })
        .collect()
}

pub fn subset(
    ledger: &DecisionLedger,
    outcomes: &OutcomeReport,
    keep: &[Uuid],
) -> (DecisionLedger, OutcomeReport) {
    let set: std::collections::HashSet<Uuid> = keep.iter().copied().collect();
    let mut sub = DecisionLedger::new(ledger.engine_version.clone());
    sub.records = ledger
        .records
        .iter()
        .filter(|r| set.contains(&r.decision_id))
        .cloned()
        .collect();
    let bundles = outcomes
        .bundles
        .iter()
        .filter(|b| set.contains(&b.ledger_decision_id))
        .cloned()
        .collect();
    (sub, OutcomeReport { bundles })
}

pub fn calendar_year_folds(
    first: DateTime<Utc>,
    last: DateTime<Utc>,
) -> Vec<(String, DateTime<Utc>, DateTime<Utc>, DateTime<Utc>)> {
    let start_year = first.year() + 1;
    let end_year = last.year();
    let mut folds = Vec::new();
    for year in start_year..=end_year {
        let train_end = Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0).unwrap();
        let test_start = train_end;
        let test_end = Utc.with_ymd_and_hms(year + 1, 1, 1, 0, 0, 0).unwrap();
        folds.push((format!("test_{year}"), train_end, test_start, test_end));
    }
    folds
}

fn walk_forward(ledger: &DecisionLedger, outcomes: &OutcomeReport) -> Vec<WalkForwardFold> {
    let Some(first) = ledger.records.first().map(|r| r.as_of_timestamp) else {
        return Vec::new();
    };
    let last = ledger
        .records
        .last()
        .map(|r| r.as_of_timestamp)
        .unwrap_or(first);
    calendar_year_folds(first, last)
        .into_iter()
        .map(|(name, train_end, test_start, test_end)| {
            let train_ids: Vec<Uuid> = ledger
                .records
                .iter()
                .filter(|r| r.as_of_timestamp < train_end)
                .map(|r| r.decision_id)
                .collect();
            let test_ids: Vec<Uuid> = ledger
                .records
                .iter()
                .filter(|r| r.as_of_timestamp >= test_start && r.as_of_timestamp < test_end)
                .map(|r| r.decision_id)
                .collect();
            let (train_ledger, _) = subset(ledger, outcomes, &train_ids);
            let (test_ledger, test_outcomes) = subset(ledger, outcomes, &test_ids);
            let mut train_counts = empty_counts();
            for rec in &train_ledger.records {
                bump(&mut train_counts, rec.action);
            }
            WalkForwardFold {
                name,
                train_end,
                test_start,
                test_end,
                train_n: train_ledger.records.len() as u32,
                test_n: test_ledger.records.len() as u32,
                train_counts,
                test_performance: measure_performance(&test_ledger, &test_outcomes),
            }
        })
        .collect()
}

fn robustness_cells(slices: &[SlicePerformance]) -> Vec<RobustnessCell> {
    let mut cells = Vec::new();
    for slice in slices {
        if !matches!(
            slice.dimension.as_str(),
            "year" | "instrument" | "trend" | "momentum"
        ) {
            continue;
        }
        for h in &slice.performance.horizons {
            let r = &h.trading.returns;
            cells.push(RobustnessCell {
                dimension: slice.dimension.clone(),
                value: slice.value.clone(),
                horizon_days: h.horizon_days,
                n_observed: r.n_observed,
                n_unavailable: r.n_unavailable,
                mean: r.mean,
                mean_sign: r.mean.map(sign),
            });
        }
    }
    cells
}

fn sign(x: f64) -> i8 {
    if x > 0.0 {
        1
    } else if x < 0.0 {
        -1
    } else {
        0
    }
}

fn vs_baseline(ledger: &DecisionLedger, outcomes: &OutcomeReport) -> Vec<VsBaselineRow> {
    let full = measure_performance(ledger, outcomes);
    let mut rows = Vec::new();
    for h in &full.horizons {
        rows.push(baseline_row(
            DecisionAction::Long,
            h.horizon_days,
            &h.by_action.long,
        ));
        rows.push(baseline_row(
            DecisionAction::Short,
            h.horizon_days,
            &h.by_action.short,
        ));
        rows.push(baseline_row(
            DecisionAction::NoTrade,
            h.horizon_days,
            &h.by_action.no_trade,
        ));
    }
    rows
}

fn baseline_row(action: DecisionAction, horizon_days: u32, stats: &ReturnStats) -> VsBaselineRow {
    VsBaselineRow {
        action,
        horizon_days,
        n_decisions: stats.n_decisions,
        n_observed: stats.n_observed,
        n_unavailable: stats.n_unavailable,
        mean: stats.mean,
        mean_vs_stand_aside: stats.mean,
    }
}

fn coverage(
    ledger: &DecisionLedger,
    outcomes: &OutcomeReport,
    ctx: &HashMap<Uuid, &DecisionContext>,
) -> CoverageNotes {
    let n_short = ledger
        .records
        .iter()
        .filter(|r| r.action == DecisionAction::Short)
        .count() as u32;
    let n_no_trade = ledger
        .records
        .iter()
        .filter(|r| r.action == DecisionAction::NoTrade)
        .count() as u32;
    let mut n_with_any = 0u32;
    let mut n_short_with = 0u32;
    for rec in &ledger.records {
        let some = outcomes
            .bundles
            .iter()
            .find(|b| b.ledger_decision_id == rec.decision_id)
            .map(|b| {
                b.horizons
                    .iter()
                    .any(|h| h.available && h.outcome_return.is_some())
            })
            .unwrap_or(false);
        if some {
            n_with_any += 1;
            if rec.action == DecisionAction::Short {
                n_short_with += 1;
            }
        }
    }
    let n_vol = ctx.values().filter(|c| c.volatility.is_some()).count() as u32;
    CoverageNotes {
        n_records: ledger.records.len() as u32,
        n_with_any_lake_outcome: n_with_any,
        n_short,
        n_short_with_outcome: n_short_with,
        n_no_trade,
        n_volatility_labeled: n_vol,
        short_unevaluated: n_short > 0 && n_short_with == 0,
        no_trade_unevaluable: n_no_trade == 0,
    }
}

fn report_hash(report: &LaboratoryReport) -> String {
    #[derive(Serialize)]
    struct Payload<'a> {
        schema_version: &'a str,
        decision_engine_version: &'a str,
        ledger_identity_hash: &'a str,
        outcome_identity_hash: &'a str,
        behavior: &'a DecisionBehaviour,
        regime_mix: &'a [RegimeMix],
        stratification: &'a [SlicePerformance],
        walk_forward: &'a [WalkForwardFold],
        robustness: &'a [RobustnessCell],
        vs_baseline: &'a [VsBaselineRow],
        coverage: &'a CoverageNotes,
    }
    let bytes = serde_json::to_vec(&Payload {
        schema_version: &report.schema_version,
        decision_engine_version: &report.decision_engine_version,
        ledger_identity_hash: &report.ledger_identity_hash,
        outcome_identity_hash: &report.outcome_identity_hash,
        behavior: &report.behavior,
        regime_mix: &report.regime_mix,
        stratification: &report.stratification,
        walk_forward: &report.walk_forward,
        robustness: &report.robustness,
        vs_baseline: &report.vs_baseline,
        coverage: &report.coverage,
    })
    .expect("laboratory report serializes");
    format!("{:x}", Sha256::digest(&bytes))
}

pub fn render_reports(report: &LaboratoryReport) -> Vec<(String, String)> {
    vec![
        (
            "DECISION_BEHAVIOUR.md".to_string(),
            render_behaviour(report),
        ),
        ("REGIME_CONTEXT.md".to_string(), render_regime(report)),
        (
            "OUTCOME_STRATIFICATION.md".to_string(),
            render_stratification(report),
        ),
        ("WALK_FORWARD.md".to_string(), render_walk_forward(report)),
        ("ROBUSTNESS.md".to_string(), render_robustness(report)),
        (
            "DECISION_VS_BASELINE.md".to_string(),
            render_baseline(report),
        ),
        (
            "HISTORICAL_RESEARCH_SUMMARY.md".to_string(),
            render_summary(report),
        ),
    ]
}

fn header() -> &'static str {
    "**Engine version: `unfrozen-dev`.** Not G-GATE. Not a v1.0 freeze. Not a strategy score. Not parameter tuning.\n\n`.cursor/rules/chronosentiment-core.mdc`: same input → same output; no invented methodology.\n\n"
}

fn opt_f(v: Option<f64>, digits: usize) -> String {
    match v {
        Some(x) if x.is_finite() => format!("{x:.digits$}"),
        _ => "n/a".to_string(),
    }
}

fn render_behaviour(report: &LaboratoryReport) -> String {
    let b = &report.behavior;
    let mut md = String::from("# Decision Behaviour Report\n\n");
    md.push_str(header());
    md.push_str("| Field | Value |\n|---|---|\n");
    md.push_str(&format!("| Decisions | `{}` |\n", b.n_records));
    md.push_str(&format!("| LONG | `{}` |\n", b.counts.long));
    md.push_str(&format!("| SHORT | `{}` |\n", b.counts.short));
    md.push_str(&format!("| NO_TRADE | `{}` |\n", b.counts.no_trade));
    md.push_str(
        "\n## By instrument\n\n| Instrument | LONG | SHORT | NO_TRADE |\n|---|---:|---:|---:|\n",
    );
    for (k, c) in &b.by_instrument {
        md.push_str(&format!(
            "| {k} | {} | {} | {} |\n",
            c.long, c.short, c.no_trade
        ));
    }
    md.push_str("\n## By year\n\n| Year | LONG | SHORT | NO_TRADE |\n|---|---:|---:|---:|\n");
    for (y, c) in &b.by_year {
        md.push_str(&format!(
            "| {y} | {} | {} | {} |\n",
            c.long, c.short, c.no_trade
        ));
    }
    md.push_str("\n## Confidence\n\n| Confidence | n |\n|---|---:|\n");
    for (k, n) in &b.confidence_counts {
        md.push_str(&format!("| {k} | {n} |\n"));
    }
    let t = &b.transitions;
    md.push_str("\n## Transitions (same instrument, consecutive as-of)\n\n");
    md.push_str("| From → To | n |\n|---|---:|\n");
    md.push_str(&format!("| LONG → SHORT | {} |\n", t.long_to_short));
    md.push_str(&format!("| SHORT → LONG | {} |\n", t.short_to_long));
    md.push_str(&format!("| LONG → LONG | {} |\n", t.long_to_long));
    md.push_str(&format!("| SHORT → SHORT | {} |\n", t.short_to_short));
    md.push_str(&format!(
        "| involving NO_TRADE | {} |\n",
        t.involving_no_trade
    ));
    md.push_str("\n## Streak lengths (consecutive same action, per instrument)\n\n| Length | n streaks |\n|---|---:|\n");
    for (len, n) in &b.streak_lengths {
        md.push_str(&format!("| {len} | {n} |\n"));
    }
    md
}

fn render_regime(report: &LaboratoryReport) -> String {
    let mut md = String::from("# Regime / Context Analysis\n\n");
    md.push_str(header());
    md.push_str(
        "Labels are read from the assessment at T. They are not a second Decision Engine.\n\n",
    );
    md.push_str("| Dimension | Value | LONG | SHORT | NO_TRADE |\n|---|---|---:|---:|---:|\n");
    for row in &report.regime_mix {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            row.dimension, row.value, row.counts.long, row.counts.short, row.counts.no_trade
        ));
    }
    md.push_str(&format!(
        "\nVolatility labels present on {} of {} decisions. Absence is not imputed.\n",
        report.coverage.n_volatility_labeled, report.coverage.n_records
    ));
    md
}

fn h60_mean(p: &PerformanceReport) -> Option<f64> {
    p.horizons
        .iter()
        .find(|h| h.horizon_days == 60)
        .and_then(|h| h.trading.returns.mean)
}

fn h60_nobs(p: &PerformanceReport) -> u32 {
    p.horizons
        .iter()
        .find(|h| h.horizon_days == 60)
        .map(|h| h.trading.returns.n_observed)
        .unwrap_or(0)
}

fn render_stratification(report: &LaboratoryReport) -> String {
    let mut md = String::from("# Outcome Stratification\n\n");
    md.push_str(header());
    md.push_str(
        "Trading means use LONG+SHORT attached lake returns only. Missing SHORT stays missing.\n\n",
    );
    md.push_str("| Dimension | Value | n | 60D n obs | 60D mean |\n|---|---|---:|---:|---:|\n");
    for s in &report.stratification {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            s.dimension,
            s.value,
            s.n_records,
            h60_nobs(&s.performance),
            opt_f(h60_mean(&s.performance), 6)
        ));
    }
    md
}

fn render_walk_forward(report: &LaboratoryReport) -> String {
    let mut md = String::from("# Walk-Forward Historical Analysis\n\n");
    md.push_str(header());
    md.push_str("No fitting. Train describes the same `unfrozen-dev` policy. Test measures it on a later unseen year.\n\n");
    md.push_str("| Fold | Train n | Train LONG/SHORT/NO_TRADE | Test n | Test 60D n obs | Test 60D mean |\n");
    md.push_str("|---|---:|---|---:|---:|---:|\n");
    for f in &report.walk_forward {
        md.push_str(&format!(
            "| {} | {} | {}/{}/{} | {} | {} | {} |\n",
            f.name,
            f.train_n,
            f.train_counts.long,
            f.train_counts.short,
            f.train_counts.no_trade,
            f.test_n,
            h60_nobs(&f.test_performance),
            opt_f(h60_mean(&f.test_performance), 6)
        ));
    }
    md.push_str("\nTest `as_of` is always `>= test_start` and `< test_end`, and `test_start == train_end`.\n");
    md
}

fn render_robustness(report: &LaboratoryReport) -> String {
    let mut md = String::from("# Robustness Report\n\n");
    md.push_str(header());
    md.push_str(
        "Sign of trading mean by slice. This is not an optimizer and not G-GATE inference.\n\n",
    );
    md.push_str("| Dimension | Value | Horizon | n obs | n missing | mean | sign |\n");
    md.push_str("|---|---|---:|---:|---:|---:|---:|\n");
    for c in &report.robustness {
        let s = match c.mean_sign {
            Some(1) => "+",
            Some(-1) => "−",
            Some(0) => "0",
            _ => "n/a",
        };
        md.push_str(&format!(
            "| {} | {} | {}D | {} | {} | {} | {s} |\n",
            c.dimension,
            c.value,
            c.horizon_days,
            c.n_observed,
            c.n_unavailable,
            opt_f(c.mean, 6)
        ));
    }
    md
}

fn render_baseline(report: &LaboratoryReport) -> String {
    let mut md = String::from("# Decision-vs-Baseline Analysis\n\n");
    md.push_str(header());
    md.push_str(
        "Stand-aside baseline is return 0. `mean_vs_stand_aside` is the attached mean minus 0.\n\n",
    );
    md.push_str(
        "SHORT with no lake rows cannot be judged. NO_TRADE with n=0 cannot be judged.\n\n",
    );
    md.push_str("| Action | Horizon | n decisions | n obs | n missing | mean | vs stand-aside |\n");
    md.push_str("|---|---:|---:|---:|---:|---:|---:|\n");
    for r in &report.vs_baseline {
        md.push_str(&format!(
            "| {:?} | {}D | {} | {} | {} | {} | {} |\n",
            r.action,
            r.horizon_days,
            r.n_decisions,
            r.n_observed,
            r.n_unavailable,
            opt_f(r.mean, 6),
            opt_f(r.mean_vs_stand_aside, 6)
        ));
    }
    md
}

fn render_summary(report: &LaboratoryReport) -> String {
    let mut md = String::from("# Historical Research Summary\n\n");
    md.push_str(header());
    md.push_str(&format!("Laboratory hash: `{}`\n\n", report.content_hash));
    let c = &report.coverage;
    md.push_str("## Coverage\n\n");
    md.push_str(&format!("- Records: {}\n", c.n_records));
    md.push_str(&format!(
        "- Any attached lake outcome: {}\n",
        c.n_with_any_lake_outcome
    ));
    md.push_str(&format!(
        "- SHORT: {} (with outcome: {}). Unevaluated: {}\n",
        c.n_short, c.n_short_with_outcome, c.short_unevaluated
    ));
    md.push_str(&format!(
        "- NO_TRADE: {}. Unevaluable because none occurred: {}\n",
        c.n_no_trade, c.no_trade_unevaluable
    ));
    md.push_str("\n## What the strata cannot tell us yet\n\n");
    md.push_str("- Under `unfrozen-dev`, Trend Bullish maps to LONG and Bearish to SHORT, so trend strata are **not** an independent decision ecology.\n");
    md.push_str("- Momentum and Volatility factors are recorded as **absent** when not on the assessment at T (not imputed).\n");
    md.push_str("- Decision confidence is `UNAVAILABLE` until a confidence model exists. Assessment numeric scores are evidence metadata, not decision confidence.\n");
    md.push_str("- Walk-forward test means are LONG-only wherever SHORT lake rows are missing.\n");
    md.push_str("\n## What this is not\n\n");
    md.push_str("- Not a candidate policy. No thresholds were searched.\n");
    md.push_str("- Not G-GATE v1.1 classification.\n");
    md.push_str("- Not a freeze of Decision Engine v1.0.\n");
    md.push_str("- Not a reason to stop or retune from CS-P-003 Observation #1.\n");
    md.push_str("\nCS-P-003 may continue as a confirmation clock. Promote nothing until these historical questions are answered and a documented successor (if any) is authored.\n");
    md
}
