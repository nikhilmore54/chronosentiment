//! CS-P-002 B4 historical product validation run.
//!
//! Replay → ledger → outcomes → performance. Read-only. No tuning. No G-GATE.
//! Engine version remains `unfrozen-dev`. Decision Engine v1.0 is not frozen.

use std::env;
use std::fs;
use std::path::PathBuf;

use chronosentiment_adapter::decision_support::backtest::{
    populate_ledger_from_assessment_schedule, DecisionLedger,
};
use chronosentiment_adapter::decision_support::policy::BaselineTrendMappingPolicy;
use chronosentiment_adapter::decision_support::outcome::{OutcomeEngine, OutcomeReport};
use chronosentiment_adapter::decision_support::performance::{
    measure_performance, HorizonPerformance, PerformanceReport, ReturnStats, RiskStats,
};
use chronosentiment_adapter::decision_support::replay::{ReplayAdapter, UNFROZEN_ENGINE_VERSION};
use chronosentiment_adapter::decision_support::DecisionAction;
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Serialize)]
struct LineageHorizon {
    horizon_days: u32,
    available: bool,
    lake_outcome_id: Option<Uuid>,
    lake_decision_id: Option<Uuid>,
    outcome_return: Option<f64>,
}

#[derive(Serialize)]
struct LineageRow {
    sequence: u32,
    ledger_decision_id: Uuid,
    instrument_id: Uuid,
    as_of_timestamp: chrono::DateTime<chrono::Utc>,
    action: DecisionAction,
    confidence: Option<f64>,
    engine_version: String,
    input_set_hash: String,
    assessment_id: Option<Uuid>,
    consumed_artifact_ids: Vec<Uuid>,
    decision_content_hash: String,
    outcome_bundle_hash: Option<String>,
    horizons: Vec<LineageHorizon>,
}

#[derive(Serialize)]
struct Provenance {
    kind: &'static str,
    not: &'static [&'static str],
    decision_engine_version: String,
    decision_engine_v1_frozen: bool,
    b4_dump_sha256: String,
    database: String,
    git_head: Option<String>,
    git_dirty: bool,
    ledger_identity_hash: String,
    outcome_identity_hash: String,
    performance_content_hash: String,
    lineage_sha256: String,
    n_decisions: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = parse_output_dir()?;
    let url = env::var("DATABASE_URL")?;
    let b4_hash = env::var("B4_DUMP_SHA256")?;
    let git_head = env::var("GIT_HEAD").ok().filter(|s| !s.is_empty());
    let git_dirty = env::var("GIT_DIRTY").ok().as_deref() == Some("1");

    let pool = sqlx::PgPool::connect(&url).await?;
    let database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&pool)
        .await?;
    if matches!(database.as_str(), "chrono_b3_test" | "chrono_b4_test") {
        return Err(format!("refusing certified database {database}").into());
    }

    let adapter = ReplayAdapter::new(pool.clone());
    let ledger =
        populate_ledger_from_assessment_schedule(
            &adapter,
            UNFROZEN_ENGINE_VERSION,
            &BaselineTrendMappingPolicy,
        ).await?;
    let outcomes = OutcomeEngine::new(pool).measure_ledger(&ledger).await?;
    let performance = measure_performance(&ledger, &outcomes);
    if performance.decision_engine_version != UNFROZEN_ENGINE_VERSION {
        return Err("engine version is not unfrozen-dev".into());
    }

    let lineage = lineage_rows(&ledger, &outcomes);
    let lineage_bytes = serde_json::to_vec_pretty(&lineage)?;
    let lineage_sha256 = format!("{:x}", Sha256::digest(&lineage_bytes));

    let provenance = Provenance {
        kind: "product_validation",
        not: &["g_gate", "v1.0_freeze", "strategy_score", "parameter_tuning"],
        decision_engine_version: UNFROZEN_ENGINE_VERSION.to_string(),
        decision_engine_v1_frozen: false,
        b4_dump_sha256: b4_hash,
        database,
        git_head,
        git_dirty,
        ledger_identity_hash: ledger.identity_hash(),
        outcome_identity_hash: outcomes.identity_hash(),
        performance_content_hash: performance.content_hash.clone(),
        lineage_sha256,
        n_decisions: performance.behavior.n_records,
    };

    fs::create_dir_all(&output)?;
    fs::write(
        output.join("performance.json"),
        serde_json::to_vec_pretty(&performance)?,
    )?;
    fs::write(output.join("lineage.json"), lineage_bytes)?;
    fs::write(
        output.join("provenance.json"),
        serde_json::to_vec_pretty(&provenance)?,
    )?;
    fs::write(
        output.join("HISTORICAL_PERFORMANCE_REPORT.md"),
        render_markdown(&provenance, &performance),
    )?;
    println!("performance_content_hash={}", performance.content_hash);
    println!("ledger_identity_hash={}", provenance.ledger_identity_hash);
    println!("n_decisions={}", provenance.n_decisions);
    Ok(())
}

fn parse_output_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--output" {
            let path = args.next().ok_or("--output requires a path")?;
            return Ok(PathBuf::from(path));
        }
    }
    Err("usage: csp002_b4_historical_run --output DIR".into())
}

fn lineage_rows(ledger: &DecisionLedger, outcomes: &OutcomeReport) -> Vec<LineageRow> {
    ledger
        .records
        .iter()
        .map(|rec| {
            let bundle = outcomes
                .bundles
                .iter()
                .find(|b| b.ledger_decision_id == rec.decision_id);
            LineageRow {
                sequence: rec.sequence,
                ledger_decision_id: rec.decision_id,
                instrument_id: rec.instrument_id,
                as_of_timestamp: rec.as_of_timestamp,
                action: rec.action,
                confidence: rec.confidence,
                engine_version: rec.engine_version.clone(),
                input_set_hash: rec.input_set_hash.clone(),
                assessment_id: rec.lineage.assessment_id,
                consumed_artifact_ids: rec.lineage.consumed_artifact_ids.clone(),
                decision_content_hash: rec.content_hash.clone(),
                outcome_bundle_hash: bundle.map(|b| b.content_hash.clone()),
                horizons: bundle
                    .map(|b| {
                        b.horizons
                            .iter()
                            .map(|h| LineageHorizon {
                                horizon_days: h.horizon_days,
                                available: h.available,
                                lake_outcome_id: h.lake_outcome_id,
                                lake_decision_id: h.lake_decision_id,
                                outcome_return: h.outcome_return,
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
            }
        })
        .collect()
}

fn render_markdown(prov: &Provenance, report: &PerformanceReport) -> String {
    let b = &report.behavior;
    let mut md = String::new();
    md.push_str("# B4 Historical Product Validation\n\n");
    md.push_str("**Engine version: `unfrozen-dev`.** Decision Engine v1.0 is **not frozen**. ");
    md.push_str("This is a product-validation baseline, not a production trading strategy, ");
    md.push_str("not a strategy score, and not G-GATE v1.1.\n\n");
    md.push_str("Parent: CS-P-002. Does not reopen EV-GOV-003.\n\n");
    md.push_str("`.cursor/rules/chronosentiment-core.mdc`: same input → same output; ");
    md.push_str("no randomness in strategy logic; this run does not change decision rules.\n\n");

    md.push_str("## Identity\n\n");
    md.push_str("| Field | Value |\n|---|---|\n");
    row(&mut md, "Kind", prov.kind);
    row(&mut md, "Decision engine version", &prov.decision_engine_version);
    row(
        &mut md,
        "Decision Engine v1.0 frozen",
        if prov.decision_engine_v1_frozen {
            "yes"
        } else {
            "no"
        },
    );
    row(&mut md, "B4 dump SHA-256", &prov.b4_dump_sha256);
    row(&mut md, "Disposable database", &prov.database);
    row(
        &mut md,
        "Git HEAD",
        prov.git_head.as_deref().unwrap_or("n/a"),
    );
    row(
        &mut md,
        "Working tree dirty",
        if prov.git_dirty { "yes" } else { "no" },
    );
    row(&mut md, "Ledger identity hash", &prov.ledger_identity_hash);
    row(&mut md, "Outcome identity hash", &prov.outcome_identity_hash);
    row(
        &mut md,
        "Performance report hash",
        &prov.performance_content_hash,
    );
    row(&mut md, "Lineage SHA-256", &prov.lineage_sha256);
    row(&mut md, "Performance schema", &report.schema_version);

    md.push_str("\n## Decision behaviour\n\n");
    md.push_str("| Field | Value |\n|---|---|\n");
    row(&mut md, "Historical decisions generated", &b.n_records.to_string());
    row(&mut md, "LONG", &b.counts.long.to_string());
    row(&mut md, "SHORT", &b.counts.short.to_string());
    row(&mut md, "NO_TRADE", &b.counts.no_trade.to_string());
    row(
        &mut md,
        "First as-of",
        &opt_time(b.first_as_of),
    );
    row(&mut md, "Last as-of", &opt_time(b.last_as_of));
    row(
        &mut md,
        "Span (calendar days)",
        &opt_i64(b.span_calendar_days),
    );
    row(
        &mut md,
        "Decisions per calendar day",
        &opt_f(b.decisions_per_calendar_day, 6),
    );

    md.push_str("\n`NO_TRADE` is not treated as a zero-return trade. Trading tables below ");
    md.push_str("use LONG and SHORT only. Opportunity tables use NO_TRADE only.\n");
    md.push_str("\nAttached `outcome_return` is the B4 lake path as stored. ");
    md.push_str("`cumulative_return` is the sum of per-decision simple returns in ledger order ");
    md.push_str("(overlapping horizons are not a portfolio).\n");

    md.push_str("\n## Trading outcomes (LONG + SHORT)\n\n");
    md.push_str("| Horizon | n obs | n missing | mean | median | win | loss | zero | win rate | cumulative sum |\n");
    md.push_str("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    for h in &report.horizons {
        trading_row(&mut md, h);
    }

    md.push_str("\n## Risk (trading path)\n\n");
    md.push_str("| Horizon | max drawdown | volatility | downside vol | worst |\n");
    md.push_str("|---|---:|---:|---:|---:|\n");
    for h in &report.horizons {
        risk_row(&mut md, h.horizon_days, &h.trading.risk);
    }

    md.push_str("\n## LONG only\n\n");
    action_table(&mut md, report, |h| &h.by_action.long);
    md.push_str("\n## SHORT only\n\n");
    action_table(&mut md, report, |h| &h.by_action.short);

    md.push_str("\n## Opportunity cost (NO_TRADE)\n\n");
    md.push_str("What the attached path did after standing aside. Not trading P&L.\n\n");
    md.push_str("| Horizon | n obs | n missing | mean | median | win | loss | zero | cumulative sum | worst |\n");
    md.push_str("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    for h in &report.horizons {
        let r = &h.opportunity.returns;
        let k = &h.opportunity.risk;
        md.push_str(&format!(
            "| {}D | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            h.horizon_days,
            r.n_observed,
            r.n_unavailable,
            opt_f(r.mean, 6),
            opt_f(r.median, 6),
            r.n_win,
            r.n_loss,
            r.n_zero,
            opt_f(r.cumulative_return, 6),
            opt_f(k.worst_outcome, 6),
        ));
    }

    md.push_str("\n## Coverage\n\n");
    md.push_str("| Horizon | trading observed | trading missing | opportunity observed | opportunity missing |\n");
    md.push_str("|---|---:|---:|---:|---:|\n");
    for h in &report.horizons {
        md.push_str(&format!(
            "| {}D | {} | {} | {} | {} |\n",
            h.horizon_days,
            h.trading.returns.n_observed,
            h.trading.returns.n_unavailable,
            h.opportunity.returns.n_observed,
            h.opportunity.returns.n_unavailable,
        ));
    }

    md.push_str("\n## Lineage\n\n");
    md.push_str(&format!(
        "Complete decision → outcome lineage is in `lineage.json` ({} rows). ",
        prov.n_decisions
    ));
    md.push_str("Each row maps ledger `decision_id` / `as_of` / action to lake outcome and decision IDs per 5/10/20/60D horizon. ");
    md.push_str(&format!("Lineage SHA-256: `{}`.\n", prov.lineage_sha256));

    md.push_str("\n## What this is not\n\n");
    md.push_str("- Not G-GATE v1.1, not DETECTED/INCONCLUSIVE under that protocol.\n");
    md.push_str("- Not a freeze of Decision Engine v1.0.\n");
    md.push_str("- Not a ranking of horizons and not an optimizer output.\n");
    md.push_str("- Not a recommendation to trade.\n");
    md
}

fn row(md: &mut String, k: &str, v: &str) {
    md.push_str(&format!("| {k} | `{v}` |\n"));
}

fn trading_row(md: &mut String, h: &HorizonPerformance) {
    let r = &h.trading.returns;
    md.push_str(&format!(
        "| {}D | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
        h.horizon_days,
        r.n_observed,
        r.n_unavailable,
        opt_f(r.mean, 6),
        opt_f(r.median, 6),
        r.n_win,
        r.n_loss,
        r.n_zero,
        opt_f(r.win_rate, 4),
        opt_f(r.cumulative_return, 6),
    ));
}

fn risk_row(md: &mut String, days: u32, k: &RiskStats) {
    md.push_str(&format!(
        "| {days}D | {} | {} | {} | {} |\n",
        opt_f(k.max_drawdown, 6),
        opt_f(k.volatility, 6),
        opt_f(k.downside_volatility, 6),
        opt_f(k.worst_outcome, 6),
    ));
}

fn action_table(md: &mut String, report: &PerformanceReport, pick: fn(&HorizonPerformance) -> &ReturnStats) {
    md.push_str("| Horizon | n obs | n missing | mean | median | win | loss | cumulative sum |\n");
    md.push_str("|---|---:|---:|---:|---:|---:|---:|---:|\n");
    for h in &report.horizons {
        let r = pick(h);
        md.push_str(&format!(
            "| {}D | {} | {} | {} | {} | {} | {} | {} |\n",
            h.horizon_days,
            r.n_observed,
            r.n_unavailable,
            opt_f(r.mean, 6),
            opt_f(r.median, 6),
            r.n_win,
            r.n_loss,
            opt_f(r.cumulative_return, 6),
        ));
    }
}

fn opt_f(v: Option<f64>, digits: usize) -> String {
    match v {
        Some(x) if x.is_finite() => format!("{x:.digits$}"),
        _ => "n/a".to_string(),
    }
}

fn opt_i64(v: Option<i64>) -> String {
    v.map(|x| x.to_string()).unwrap_or_else(|| "n/a".to_string())
}

fn opt_time(v: Option<chrono::DateTime<chrono::Utc>>) -> String {
    v.map(|t| t.to_rfc3339()).unwrap_or_else(|| "n/a".to_string())
}
