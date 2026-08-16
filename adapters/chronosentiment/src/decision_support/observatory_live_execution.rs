//! CS-P-006-P.E.2 Live Execution Observation.
//!
//! Next prospective cohort carries Execution Contract v0 from T.
//! Does not attach a target to the 14 August direction-only seals.
//! Does not rewrite P.E.1. Does not retune C3-002. Does not start C.3-G.

use std::collections::BTreeMap;

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::ingestion::yahoo::YahooHistoricalBar;

use super::csp006_protocol::{RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_UNIVERSE};
use super::observatory_execution::{
    first_exit, seal_execution_intent, ExecutionExit, ExitReason, SealedExecutionIntent,
    TriggerType, C3G_EXPERIMENT_AUTHORIZED, EXECUTION_CONTRACT_ID, EXECUTION_CONTRACT_LABEL,
    EXECUTION_TARGET_PCT, SEARCH_THREE_AUTHORIZED, STOP_EXIT_AUTHORIZED,
    TARGET_PATH_OPTIMIZATION_AUTHORIZED, TARGETED_EXECUTION_V0_FROZEN,
};
use super::observatory_prospective::{generate_prospective_decision, latest_session_at_or_before};
use super::observatory_slice::SealedDecisionRecord;
use super::policy_artifact::PolicyArtifact;

pub const LIVE_EXECUTION_STARTED: bool = true;
pub const FOURTEEN_AUG_COHORT_MUTATION_AUTHORIZED: bool = false;
pub const PE1_SIDECAR_MUTATION_AUTHORIZED: bool = false;
pub const CONTINUOUS_SESSION_SEAL_AUTHORIZED: bool = false;
pub const LIVE_YAHOO_FETCH_AUTHORIZED: bool = false;
pub const LIVE_EXECUTION_PATH_KIND: &str = "prospective_execution_v0";
pub const LIVE_EXECUTION_STATUS_AWAITING: &str = "AWAITING_NEXT_SESSION";
pub const LIVE_EXECUTION_STATUS_OBSERVING: &str = "OBSERVING";

pub fn protected_fourteen_aug_clock() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 14, 3, 45, 0).unwrap()
}

pub fn is_protected_direction_only_clock(t: DateTime<Utc>) -> bool {
    t <= protected_fourteen_aug_clock()
}

pub fn refuse_live_execution_output(path: &str) -> Result<(), String> {
    for forbidden in [
        "observatory/prospective",
        "historical_replay_v0",
        "historical_replay_v1",
        "selected_policy.json",
    ] {
        if path.contains(forbidden) && !path.contains("prospective_execution") {
            return Err(format!("live execution refuses to write {forbidden}"));
        }
    }
    if path.contains("targeted_execution_v0") && !PE1_SIDECAR_MUTATION_AUTHORIZED {
        return Err("P.E.1 sidecar targeted_execution_v0 is frozen".into());
    }
    if TARGETED_EXECUTION_V0_FROZEN && path.contains("targeted_execution_v0") {
        return Err("P.E.1 sidecar targeted_execution_v0 is frozen".into());
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveExecutionRecord {
    pub instrument: String,
    pub decision: SealedDecisionRecord,
    pub intent: SealedExecutionIntent,
    pub exit: ExecutionExit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveExecutionLedger {
    pub path_kind: String,
    pub execution_contract: String,
    pub execution_contract_label: String,
    pub target_pct: f64,
    pub seal_status: String,
    pub certified_t: Option<String>,
    pub fourteen_aug_cohort_mutated: bool,
    pub pe1_sidecar_mutated: bool,
    pub peeked_returns_at_seal: bool,
    pub statistical_backtest: bool,
    pub n_decisions: usize,
    pub n_observing: usize,
    pub n_target: usize,
    pub n_horizon: usize,
    pub records: Vec<LiveExecutionRecord>,
}

pub fn empty_live_ledger(status: &str, certified_t: Option<String>) -> LiveExecutionLedger {
    LiveExecutionLedger {
        path_kind: LIVE_EXECUTION_PATH_KIND.to_string(),
        execution_contract: EXECUTION_CONTRACT_ID.to_string(),
        execution_contract_label: EXECUTION_CONTRACT_LABEL.to_string(),
        target_pct: EXECUTION_TARGET_PCT,
        seal_status: status.to_string(),
        certified_t,
        fourteen_aug_cohort_mutated: false,
        pe1_sidecar_mutated: false,
        peeked_returns_at_seal: false,
        statistical_backtest: false,
        n_decisions: 0,
        n_observing: 0,
        n_target: 0,
        n_horizon: 0,
        records: Vec::new(),
    }
}

pub fn latest_universe_session(
    cache: &BTreeMap<String, Vec<YahooHistoricalBar>>,
    now: DateTime<Utc>,
) -> Result<DateTime<Utc>, String> {
    let mut latest: Option<DateTime<Utc>> = None;
    for instrument in RESEARCH_UNIVERSE {
        let bars = cache
            .get(instrument)
            .ok_or_else(|| format!("yahoo cache missing {instrument}"))?;
        let t = latest_session_at_or_before(bars, now)
            .ok_or_else(|| format!("no session ≤ now for {instrument}"))?;
        latest = Some(latest.map_or(t, |cur| cur.max(t)));
    }
    latest.ok_or_else(|| "no certified session in the live universe".into())
}

fn recount(ledger: &mut LiveExecutionLedger) {
    ledger.n_decisions = ledger.records.len();
    ledger.n_observing = ledger
        .records
        .iter()
        .filter(|r| r.exit.exit_reason == ExitReason::Observing)
        .count();
    ledger.n_target = ledger
        .records
        .iter()
        .filter(|r| r.exit.exit_reason == ExitReason::Target)
        .count();
    ledger.n_horizon = ledger
        .records
        .iter()
        .filter(|r| r.exit.exit_reason == ExitReason::Horizon)
        .count();
    if ledger.records.is_empty() {
        ledger.seal_status = LIVE_EXECUTION_STATUS_AWAITING.to_string();
    } else if ledger.n_observing == ledger.n_decisions {
        ledger.seal_status = LIVE_EXECUTION_STATUS_OBSERVING.to_string();
    }
}

fn exit_is_terminal(reason: ExitReason) -> bool {
    matches!(
        reason,
        ExitReason::Target | ExitReason::Horizon | ExitReason::NoTrade | ExitReason::Ambiguous
    )
}

pub fn observe_live_records(
    ledger: &mut LiveExecutionLedger,
    cache: &BTreeMap<String, Vec<YahooHistoricalBar>>,
) -> Result<usize, String> {
    let mut appended = 0usize;
    for record in &mut ledger.records {
        if exit_is_terminal(record.exit.exit_reason) {
            continue;
        }
        let bars = cache
            .get(&record.instrument)
            .ok_or_else(|| format!("yahoo cache missing {}", record.instrument))?;
        let next = first_exit(&record.decision, &record.intent, bars)?;
        if next.exit_reason != record.exit.exit_reason || next.trigger_type != record.exit.trigger_type
        {
            record.exit = next;
            appended += 1;
        }
    }
    recount(ledger);
    Ok(appended)
}

pub fn run_live_execution(
    artifact: &PolicyArtifact,
    cache: &BTreeMap<String, Vec<YahooHistoricalBar>>,
    now: DateTime<Utc>,
    existing: Option<LiveExecutionLedger>,
) -> Result<LiveExecutionLedger, String> {
    if FOURTEEN_AUG_COHORT_MUTATION_AUTHORIZED
        || PE1_SIDECAR_MUTATION_AUTHORIZED
        || CONTINUOUS_SESSION_SEAL_AUTHORIZED
        || LIVE_YAHOO_FETCH_AUTHORIZED
        || TARGET_PATH_OPTIMIZATION_AUTHORIZED
        || STOP_EXIT_AUTHORIZED
        || SEARCH_THREE_AUTHORIZED
        || C3G_EXPERIMENT_AUTHORIZED
    {
        return Err("refusing a live execution run that opens research or mutates protected ledgers".into());
    }
    if artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("live execution identity-gates C3-002".into());
    }

    let mut ledger = existing.unwrap_or_else(|| empty_live_ledger(LIVE_EXECUTION_STATUS_AWAITING, None));
    if ledger.path_kind != LIVE_EXECUTION_PATH_KIND {
        return Err("live execution belongs on the prospective_execution_v0 ledger".into());
    }
    if ledger.fourteen_aug_cohort_mutated || ledger.pe1_sidecar_mutated {
        return Err("refusing a live ledger that claims a protected mutation".into());
    }

    let certified_t = latest_universe_session(cache, now)?;
    ledger.certified_t = Some(certified_t.to_rfc3339());

    if is_protected_direction_only_clock(certified_t) {
        if !ledger.records.is_empty() {
            observe_live_records(&mut ledger, cache)?;
            return Ok(ledger);
        }
        return Ok(empty_live_ledger(
            LIVE_EXECUTION_STATUS_AWAITING,
            Some(certified_t.to_rfc3339()),
        ));
    }

    if ledger.records.is_empty() {
        for instrument in RESEARCH_UNIVERSE {
            let bars = cache
                .get(instrument)
                .ok_or_else(|| format!("yahoo cache missing {instrument}"))?;
            let instrument_t = latest_session_at_or_before(bars, now)
                .ok_or_else(|| format!("no session ≤ now for {instrument}"))?;
            if is_protected_direction_only_clock(instrument_t) {
                return Err(format!(
                    "refusing to attach Execution Contract v0 to the 14 August clock for {instrument}"
                ));
            }
            let decision = generate_prospective_decision(artifact, instrument, bars, now)?;
            let t = super::observatory_maturity::parse_decision_time(&decision.decision_time)?;
            if is_protected_direction_only_clock(t) {
                return Err("refusing to seal Execution Contract v0 on the 14 August cohort".into());
            }
            let entry = bars
                .iter()
                .filter_map(|b| {
                    let ts = Utc.timestamp_opt(b.timestamp, 0).single()?;
                    if ts <= t && b.adj_close.is_finite() && b.adj_close > 0.0 {
                        Some((ts, b.adj_close))
                    } else {
                        None
                    }
                })
                .max_by_key(|(ts, _)| *ts)
                .map(|(_, c)| c)
                .ok_or_else(|| format!("no entry close at {} for {instrument}", decision.decision_time))?;
            let intent = seal_execution_intent(&decision, entry, EXECUTION_TARGET_PCT)?;
            let exit = first_exit(&decision, &intent, bars)?;
            ledger.records.push(LiveExecutionRecord {
                instrument: instrument.to_string(),
                decision,
                intent,
                exit,
            });
        }
        ledger.peeked_returns_at_seal = false;
    } else if CONTINUOUS_SESSION_SEAL_AUTHORIZED {
        return Err("continuous session seal is not authorized in this freeze".into());
    }

    observe_live_records(&mut ledger, cache)?;
    Ok(ledger)
}

pub fn render_live_execution_report(ledger: &LiveExecutionLedger) -> String {
    let mut md = String::new();
    md.push_str("# Live Execution Observation Report\n\n");
    md.push_str("**Document type:** Product validation evidence  \n");
    md.push_str("**Parent:** CS-P-006-P.E.2  \n");
    md.push_str("**Does not:** mutate the 14 August cohort, rewrite P.E.1, retune C3-002, start C.3-G, run Search #3  \n\n");
    md.push_str("`.cursor/rules/chronosentiment-core.mdc`: the target is sealed at T; future OHLC never chooses the target.\n\n");
    md.push_str("C3-002 chooses direction only. Execution Contract v0 owns `target_pct = 5.0%`. The 14-August cohort was sealed without an execution intent and remains untouched. P.E.2 will attach Execution Contract v0 only to the next eligible cohort at T.\n\n");
    md.push_str("```text\n");
    md.push_str("14-Aug cohort\nDecision only\n7 OBSERVING\nNo execution intent\n```\n\n");
    md.push_str("```text\n");
    md.push_str("Next eligible cohort\nDecision + Execution Intent\nP.E.2 control\n```\n\n");
    md.push_str(&format!("- product label: {}\n", ledger.execution_contract_label));
    md.push_str(&format!("- path kind: `{}`\n", ledger.path_kind));
    md.push_str(&format!("- seal status: `{}`\n", ledger.seal_status));
    md.push_str(&format!(
        "- certified T: {}\n",
        ledger.certified_t.as_deref().unwrap_or("—")
    ));
    md.push_str(&format!("- target_pct: {:.1}%\n", ledger.target_pct * 100.0));
    md.push_str(&format!(
        "- 14 August cohort mutated: {}\n",
        ledger.fourteen_aug_cohort_mutated
    ));
    md.push_str(&format!("- P.E.1 sidecar mutated: {}\n", ledger.pe1_sidecar_mutated));
    md.push_str(&format!("- peeked_returns_at_seal: {}\n", ledger.peeked_returns_at_seal));
    md.push_str(&format!(
        "- statistical strategy backtest: {}\n\n",
        if ledger.statistical_backtest {
            "DONE"
        } else {
            "not done"
        }
    ));
    md.push_str(&format!("- decisions: {}\n", ledger.n_decisions));
    md.push_str(&format!("- OBSERVING: {}\n", ledger.n_observing));
    md.push_str(&format!("- TARGET: {}\n", ledger.n_target));
    md.push_str(&format!("- HORIZON: {}\n\n", ledger.n_horizon));
    if ledger.seal_status == LIVE_EXECUTION_STATUS_AWAITING {
        md.push_str("The 14-August cohort was sealed without an execution intent and remains untouched. P.E.2 will attach Execution Contract v0 only to the next eligible cohort at T. AWAITING_NEXT_SESSION until a session strictly after 2026-08-14T03:45:00Z exists. IDEA and MAHABANK remain in the universe.\n");
        return md;
    }
    md.push_str("| Instrument | Decision | Target | Exit | Trigger | Session | Execution price | V |\n");
    md.push_str("|---|---|---:|---|---|---:|---:|---:|\n");
    for record in &ledger.records {
        md.push_str(&format!(
            "| {} | {} | {:.2} | {} | {} | {} | {} | {} |\n",
            record.instrument,
            record.intent.action,
            record.intent.target_price,
            exit_label(record.exit.exit_reason),
            trigger_label(record.exit.trigger_type),
            record
                .exit
                .trigger_session
                .map(|n| n.to_string())
                .unwrap_or_else(|| "—".into()),
            record
                .exit
                .execution_price
                .map(|p| format!("{p:.2}"))
                .unwrap_or_else(|| "—".into()),
            record
                .exit
                .decision_value
                .map(pct)
                .unwrap_or_else(|| "—".into()),
        ));
    }
    md.push_str("\nTARGET and HORIZON are both evidence. Trigger type records why the exit fired. This is not a statistical strategy backtest.\n");
    md
}

pub fn render_live_execution_html(ledger: &LiveExecutionLedger) -> String {
    let mut cards = String::new();
    for record in &ledger.records {
        let reason = exit_label(record.exit.exit_reason);
        let tone = match record.exit.exit_reason {
            ExitReason::Target => "target",
            ExitReason::Horizon => "horizon",
            _ => "other",
        };
        let hold = record
            .exit
            .holding_sessions
            .map(|n| format!("{n} sessions"))
            .unwrap_or_else(|| "—".into());
        let value = record
            .exit
            .decision_value
            .map(pct)
            .unwrap_or_else(|| "—".into());
        cards.push_str(&format!(
            r#"<article class="card {tone}">
<p class="meta">{instrument} · {date}</p>
<dl>
<div><dt>Decision</dt><dd>{direction}</dd></div>
<div><dt>Target</dt><dd>{target}</dd></div>
<div><dt>Maximum hold</dt><dd>20 sessions</dd></div>
<div><dt>Exit</dt><dd>{reason}</dd></div>
<div><dt>Trigger</dt><dd>{trigger}</dd></div>
<div><dt>Holding period</dt><dd>{hold}</dd></div>
<div><dt>Realized decision value</dt><dd class="v">{value}</dd></div>
</dl>
</article>"#,
            instrument = escape(&record.instrument),
            date = escape(&record.decision.decision_time),
            direction = escape(&record.intent.action),
            target = format!("{:+.1}%", record.intent.target_pct * 100.0),
            reason = reason,
            trigger = trigger_label(record.exit.trigger_type),
            hold = escape(&hold),
            value = escape(&value),
        ));
    }
    if cards.is_empty() {
        cards = "<p class=\"note\">The 14-August cohort was sealed without an execution intent and remains untouched. P.E.2 will attach Execution Contract v0 only to the next eligible cohort at T. Status: AWAITING_NEXT_SESSION. IDEA and MAHABANK remain in the universe.</p>".into();
    }
    format!(
        r##"<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>Live Execution Observation — ChronoSentiment</title>
<style>
:root {{ --ink:#141414; --muted:#5c5c5c; --line:#d8d8d4; --paper:#f4f3ef; --card:#fff; }}
body {{ margin:0; color:var(--ink); background:var(--paper); font-family:ui-sans-serif,system-ui,sans-serif; }}
main {{ max-width:880px; margin:0 auto; padding:28px 24px 64px; }}
.brand {{ font-size:12px; letter-spacing:0.14em; text-transform:uppercase; color:var(--muted); }}
h1 {{ font-size:26px; font-weight:600; margin:6px 0 12px; }}
.note {{ color:var(--muted); max-width:640px; line-height:1.5; }}
.cohorts {{ background:var(--card); border:1px solid var(--line); padding:14px 16px; max-width:420px; line-height:1.45; font-size:13px; }}
.card {{ background:var(--card); border:1px solid var(--line); padding:16px 18px; margin:12px 0; }}
.card.target {{ border-left:3px solid #1b5e3b; }}
.card.horizon {{ border-left:3px solid #5c5c5c; }}
.card dl {{ display:grid; gap:6px; margin:8px 0 0; }}
.card div {{ display:grid; grid-template-columns:200px 1fr; gap:8px; }}
dt {{ color:var(--muted); font-size:13px; }}
dd {{ margin:0; }}
.meta {{ color:var(--muted); font-size:13px; margin:0 0 8px; }}
.v {{ font-variant-numeric:tabular-nums; }}
</style></head><body>
<main>
<p class="brand">ChronoSentiment</p>
<h1>Execution Contract v0 — live observation</h1>
<p class="note">C3-002 chooses direction. Execution Contract v0 owns target_pct = 5.0%. The 14-August cohort was sealed without an execution intent and remains untouched. P.E.2 attaches that contract only to the next eligible cohort at T. Status: {status}. This is not a statistical strategy backtest. C.3-G is untouched. Search #3 is not authorized.</p>
<pre class="cohorts">14-Aug cohort
Decision only
7 OBSERVING
No execution intent

Next eligible cohort
Decision + Execution Intent
P.E.2 control</pre>
<div class="feed">{cards}</div>
</main>
</body></html>"##,
        status = escape(&ledger.seal_status),
        cards = cards
    )
}

fn exit_label(reason: ExitReason) -> &'static str {
    match reason {
        ExitReason::Target => "TARGET",
        ExitReason::Stop => "STOP",
        ExitReason::Horizon => "HORIZON",
        ExitReason::Ambiguous => "AMBIGUOUS",
        ExitReason::NoTrade => "NO_TRADE",
        ExitReason::Observing => "OBSERVING",
    }
}

fn trigger_label(trigger: Option<TriggerType>) -> &'static str {
    match trigger {
        Some(TriggerType::HighReached) => "HIGH_REACHED",
        Some(TriggerType::LowReached) => "LOW_REACHED",
        Some(TriggerType::GapThrough) => "GAP_THROUGH",
        Some(TriggerType::SessionClose) => "SESSION_CLOSE",
        Some(TriggerType::Ambiguous) => "AMBIGUOUS",
        None => "—",
    }
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn pct(value: f64) -> String {
    let points = 100.0 * value;
    if points > 0.0 {
        format!("+{points:.2}%")
    } else if points < 0.0 {
        format!("−{:.2}%", points.abs())
    } else {
        "0.00%".into()
    }
}
