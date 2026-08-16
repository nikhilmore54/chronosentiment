//! CS-P-006-P.E.2.H Historical time-machine of the frozen P.E.2 control.
//!
//! Executes Execution Contract v0 against a historical certified T.
//! Does not modify the P.E.2 specification. Does not seal the 14 August cohort.
//! Does not rewrite P.E.1, Replay v0/v1, or live prospective_execution_v0.

use std::collections::BTreeMap;

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::ingestion::yahoo::YahooHistoricalBar;

use super::csp006_protocol::{RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_UNIVERSE};
use super::observatory_execution::{
    entry_close, first_exit, seal_execution_intent, ExecutionExit, ExitReason,
    SealedExecutionIntent, TriggerType, C3G_EXPERIMENT_AUTHORIZED, EXECUTION_CONTRACT_ID,
    EXECUTION_CONTRACT_LABEL, EXECUTION_TARGET_PCT, SEARCH_THREE_AUTHORIZED, STOP_EXIT_AUTHORIZED,
    TARGET_PATH_OPTIMIZATION_AUTHORIZED,
};
use super::observatory_historical::{
    decision_time_bars, generate_historical_replay_decision, poison_future_bars,
};
use super::observatory_maturity::nth_market_session_after;
use super::observatory_prospective::latest_session_at_or_before;
use super::observatory_slice::{SealedDecisionRecord, OBSERVATORY_HORIZON_DAYS};
use super::policy_artifact::PolicyArtifact;
use super::DecisionAction;

pub const HISTORICAL_PE2_PATH_KIND: &str = "historical_pe2_replay";
pub const HISTORICAL_PE2_REQUESTED_CLOCK: &str = "2026-07-15T03:45:00+00:00";
pub const REQUIRED_SUBSEQUENT_SESSIONS: u32 = 20;

pub fn historical_pe2_requested_clock() -> Result<DateTime<Utc>, String> {
    DateTime::parse_from_rfc3339(HISTORICAL_PE2_REQUESTED_CLOCK)
        .map(|t| t.with_timezone(&Utc))
        .map_err(|e| format!("historical P.E.2 clock is not RFC3339: {e}"))
}

pub fn refuse_historical_pe2_output(path: &str) -> Result<(), String> {
    for forbidden in [
        "observatory/prospective",
        "historical_replay_v0",
        "historical_replay_v1",
        "targeted_execution_v0",
        "selected_policy.json",
    ] {
        if path.contains(forbidden) && !path.contains("historical_pe2") {
            return Err(format!("historical P.E.2 refuses to write {forbidden}"));
        }
    }
    if path.contains("prospective_execution_v0") {
        return Err("historical P.E.2 refuses to write live P.E.2 prospective_execution_v0".into());
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPe2Record {
    pub instrument: String,
    pub requested_clock: String,
    pub certified_t: String,
    pub decision: SealedDecisionRecord,
    pub intent: SealedExecutionIntent,
    pub exit: ExecutionExit,
    pub determinism_pass: bool,
    pub lookahead_clean: bool,
    pub poison_test_pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPe2Ledger {
    pub path_kind: String,
    pub execution_contract: String,
    pub execution_contract_label: String,
    pub requested_clock: String,
    pub certified_t: String,
    pub target_pct: f64,
    pub max_holding_sessions: u32,
    pub n_decisions: usize,
    pub n_execution_intents: usize,
    pub n_target: usize,
    pub n_horizon: usize,
    pub n_gap_through: usize,
    pub n_high_reached: usize,
    pub n_low_reached: usize,
    pub n_session_close: usize,
    pub determinism_pass: bool,
    pub lookahead_clean: bool,
    pub poison_test_pass: bool,
    pub peeked_returns_at_seal: bool,
    pub prospective_cohort_mutated: bool,
    pub protected_artifacts_mutated: bool,
    pub statistical_backtest: bool,
    pub lifecycle_validation: String,
    pub records: Vec<HistoricalPe2Record>,
}

pub fn replay_historical_pe2(
    artifact: &PolicyArtifact,
    cache: &BTreeMap<String, Vec<YahooHistoricalBar>>,
) -> Result<HistoricalPe2Ledger, String> {
    if TARGET_PATH_OPTIMIZATION_AUTHORIZED
        || STOP_EXIT_AUTHORIZED
        || SEARCH_THREE_AUTHORIZED
        || C3G_EXPERIMENT_AUTHORIZED
    {
        return Err("refusing a historical P.E.2 run that opens research".into());
    }
    if artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("historical P.E.2 identity-gates C3-002".into());
    }
    let requested = historical_pe2_requested_clock()?;
    let mut certified: Option<DateTime<Utc>> = None;
    let mut records = Vec::new();
    let mut determinism_pass = true;
    let mut lookahead_clean = true;
    let mut poison_test_pass = true;

    for instrument in RESEARCH_UNIVERSE {
        let bars = cache
            .get(instrument)
            .ok_or_else(|| format!("yahoo cache missing {instrument}"))?;
        let t = latest_session_at_or_before(bars, requested)
            .ok_or_else(|| format!("no certified session ≤ requested T for {instrument}"))?;
        let subsequent = bars
            .iter()
            .filter(|b| Utc.timestamp_opt(b.timestamp, 0).single().is_some_and(|ts| ts > t))
            .count();
        if subsequent < REQUIRED_SUBSEQUENT_SESSIONS as usize {
            return Err(format!(
                "{instrument} has {subsequent} sessions after {t}, need {REQUIRED_SUBSEQUENT_SESSIONS}"
            ));
        }
        if nth_market_session_after(bars, t, REQUIRED_SUBSEQUENT_SESSIONS).is_none() {
            return Err(format!(
                "{instrument} cannot resolve the 20th market session after {t}"
            ));
        }
        match certified {
            None => certified = Some(t),
            Some(cur) if cur != t => {
                return Err(format!(
                    "{instrument} certified T {t} differs from cohort certified T {cur}"
                ));
            }
            Some(_) => {}
        }

        let known = decision_time_bars(bars, t);
        let decision = generate_historical_replay_decision(artifact, instrument, bars, t)?;
        let again = generate_historical_replay_decision(artifact, instrument, bars, t)?;
        let from_known = generate_historical_replay_decision(artifact, instrument, &known, t)?;
        let poisoned = poison_future_bars(bars, t);
        let from_poisoned = generate_historical_replay_decision(artifact, instrument, &poisoned, t)?;

        let tick_det = decision == again;
        let tick_lookahead = decision == from_known;
        if !tick_det {
            determinism_pass = false;
        }
        if !tick_lookahead {
            lookahead_clean = false;
        }

        let entry = entry_close(&known, t).ok_or_else(|| {
            format!("no entry close at {} for {instrument}", decision.decision_time)
        })?;
        let intent = seal_execution_intent(&decision, entry, EXECUTION_TARGET_PCT)?;
        if (intent.target_pct - EXECUTION_TARGET_PCT).abs() > 1e-12 {
            return Err("historical P.E.2 target_pct is not the frozen 5%".into());
        }
        if intent.max_holding_sessions != OBSERVATORY_HORIZON_DAYS {
            return Err("historical P.E.2 max hold is not 20 market sessions".into());
        }
        if !intent.sealed_at_t {
            return Err("execution intent was not sealed at T".into());
        }

        let poison_known = decision_time_bars(&poisoned, t);
        let poison_entry = entry_close(&poison_known, t)
            .ok_or_else(|| format!("no poisoned-path entry close for {instrument}"))?;
        let poison_intent =
            seal_execution_intent(&from_poisoned, poison_entry, EXECUTION_TARGET_PCT)?;
        let tick_poison = from_poisoned == decision && poison_intent == intent;
        if !tick_poison {
            poison_test_pass = false;
        }

        let exit = first_exit(&decision, &intent, bars)?;
        if matches!(exit.exit_reason, ExitReason::Observing) {
            return Err(format!(
                "{instrument} still OBSERVING after 20 subsequent sessions"
            ));
        }
        if decision.action == DecisionAction::Long
            && exit.exit_reason == ExitReason::Target
            && !matches!(
                exit.trigger_type,
                Some(TriggerType::HighReached) | Some(TriggerType::GapThrough)
            )
        {
            return Err(format!(
                "{instrument} LONG TARGET missing HIGH_REACHED/GAP_THROUGH"
            ));
        }
        if decision.action == DecisionAction::Short
            && exit.exit_reason == ExitReason::Target
            && !matches!(
                exit.trigger_type,
                Some(TriggerType::LowReached) | Some(TriggerType::GapThrough)
            )
        {
            return Err(format!(
                "{instrument} SHORT TARGET missing LOW_REACHED/GAP_THROUGH"
            ));
        }
        if exit.exit_reason == ExitReason::Horizon
            && exit.trigger_type != Some(TriggerType::SessionClose)
        {
            return Err(format!("{instrument} HORIZON missing SESSION_CLOSE"));
        }

        records.push(HistoricalPe2Record {
            instrument: instrument.to_string(),
            requested_clock: requested.to_rfc3339(),
            certified_t: t.to_rfc3339(),
            decision,
            intent,
            exit,
            determinism_pass: tick_det,
            lookahead_clean: tick_lookahead,
            poison_test_pass: tick_poison,
        });
    }

    let n_target = count_reason(&records, ExitReason::Target);
    let n_horizon = count_reason(&records, ExitReason::Horizon);
    let lifecycle_ok = determinism_pass
        && lookahead_clean
        && poison_test_pass
        && records.len() == RESEARCH_UNIVERSE.len()
        && records
            .iter()
            .all(|r| (r.intent.target_pct - EXECUTION_TARGET_PCT).abs() < 1e-12)
        && n_target + n_horizon == records.len();

    Ok(HistoricalPe2Ledger {
        path_kind: HISTORICAL_PE2_PATH_KIND.to_string(),
        execution_contract: EXECUTION_CONTRACT_ID.to_string(),
        execution_contract_label: EXECUTION_CONTRACT_LABEL.to_string(),
        requested_clock: requested.to_rfc3339(),
        certified_t: certified
            .ok_or_else(|| "no certified T".to_string())?
            .to_rfc3339(),
        target_pct: EXECUTION_TARGET_PCT,
        max_holding_sessions: OBSERVATORY_HORIZON_DAYS,
        n_decisions: records.len(),
        n_execution_intents: records.len(),
        n_target,
        n_horizon,
        n_gap_through: count_trigger(&records, TriggerType::GapThrough),
        n_high_reached: count_trigger(&records, TriggerType::HighReached),
        n_low_reached: count_trigger(&records, TriggerType::LowReached),
        n_session_close: count_trigger(&records, TriggerType::SessionClose),
        determinism_pass,
        lookahead_clean,
        poison_test_pass,
        peeked_returns_at_seal: false,
        prospective_cohort_mutated: false,
        protected_artifacts_mutated: false,
        statistical_backtest: false,
        lifecycle_validation: if lifecycle_ok {
            "PASS".into()
        } else {
            "FAIL".into()
        },
        records,
    })
}

fn count_reason(records: &[HistoricalPe2Record], reason: ExitReason) -> usize {
    records.iter().filter(|r| r.exit.exit_reason == reason).count()
}

fn count_trigger(records: &[HistoricalPe2Record], trigger: TriggerType) -> usize {
    records
        .iter()
        .filter(|r| r.exit.trigger_type == Some(trigger))
        .count()
}

pub fn render_historical_pe2_report(ledger: &HistoricalPe2Ledger) -> String {
    let mut md = String::new();
    md.push_str("# Historical P.E.2 Lifecycle Validation Report\n\n");
    md.push_str("**Document type:** Product validation evidence  \n");
    md.push_str("**Parent:** CS-P-006-P.E.2.H  \n");
    md.push_str("**Does not:** modify the P.E.2 specification, mutate the 14 August cohort, rewrite P.E.1 / Replay v0 / Replay v1 / live P.E.2, start P.E.3  \n\n");
    md.push_str("`.cursor/rules/chronosentiment-core.mdc`: the Decision and Execution Intent are sealed at T; future OHLC never chooses the target.\n\n");
    md.push_str(&format!(
        "Historical P.E.2 lifecycle validation: **{}**  \n",
        ledger.lifecycle_validation
    ));
    md.push_str("Statistical strategy backtest: **NOT PERFORMED**\n\n");
    md.push_str("This is a time-machine of the frozen P.E.2 control. Live P.E.2 remains `AWAITING_NEXT_SESSION` with 0 seals. The 14-August cohort stays decision-only.\n\n");
    md.push_str("## Clock\n\n");
    md.push_str(&format!("- requested T: `{}`\n", ledger.requested_clock));
    md.push_str(&format!("- certified T: `{}`\n", ledger.certified_t));
    md.push_str(&format!("- path kind: `{}`\n", ledger.path_kind));
    md.push_str(&format!("- product label: {}\n", ledger.execution_contract_label));
    md.push_str(&format!("- execution contract: `{}`\n", ledger.execution_contract));
    md.push_str(&format!("- target_pct: {:.1}%\n", ledger.target_pct * 100.0));
    md.push_str(&format!(
        "- max holding sessions: {}\n\n",
        ledger.max_holding_sessions
    ));
    md.push_str("## Integrity\n\n");
    md.push_str(&format!("- determinism: {}\n", pass_fail(ledger.determinism_pass)));
    md.push_str(&format!("- no-lookahead: {}\n", pass_fail(ledger.lookahead_clean)));
    md.push_str(&format!("- poison test: {}\n", pass_fail(ledger.poison_test_pass)));
    md.push_str(&format!(
        "- peeked_returns_at_seal: {}\n",
        ledger.peeked_returns_at_seal
    ));
    md.push_str(&format!(
        "- prospective cohort mutated: {}\n",
        ledger.prospective_cohort_mutated
    ));
    md.push_str(&format!(
        "- protected artifacts mutated: {}\n\n",
        ledger.protected_artifacts_mutated
    ));
    md.push_str("## Counts\n\n");
    md.push_str(&format!("- intents: {}\n", ledger.n_decisions));
    md.push_str(&format!("- execution intents: {}\n", ledger.n_execution_intents));
    md.push_str(&format!("- TARGET exits: {}\n", ledger.n_target));
    md.push_str(&format!("- HORIZON exits: {}\n", ledger.n_horizon));
    md.push_str(&format!("- GAP_THROUGH: {}\n", ledger.n_gap_through));
    md.push_str(&format!("- HIGH_REACHED: {}\n", ledger.n_high_reached));
    md.push_str(&format!("- LOW_REACHED: {}\n", ledger.n_low_reached));
    md.push_str(&format!("- SESSION_CLOSE: {}\n\n", ledger.n_session_close));
    md.push_str("TARGET and HORIZON are both evidence. Trigger type records why the exit fired. Mean / median / total V, Sharpe, CAGR, and win rate are not product claims.\n\n");
    md.push_str("## Ticks\n\n");
    md.push_str("| Instrument | Certified T | Decision | Target | Exit | Trigger | Session | Execution price |\n");
    md.push_str("|---|---|---|---:|---|---|---:|---:|\n");
    for record in &ledger.records {
        md.push_str(&format!(
            "| {} | {} | {} | {:.2} | {} | {} | {} | {} |\n",
            record.instrument,
            record.certified_t,
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
        ));
    }
    md.push_str("\nP.E.1, Replay v0/v1, the 14-August prospective ledger, and live `prospective_execution_v0` were not written. C.3-G is untouched. Search #3 is not authorized. P.E.3 is not started.\n");
    md
}

pub fn render_historical_pe2_html(ledger: &HistoricalPe2Ledger) -> String {
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
</dl>
</article>"#,
            instrument = escape(&record.instrument),
            date = escape(&record.certified_t),
            direction = escape(&record.intent.action),
            target = format!("{:+.1}%", record.intent.target_pct * 100.0),
            reason = reason,
            trigger = trigger_label(record.exit.trigger_type),
            hold = escape(&hold),
        ));
    }
    format!(
        r##"<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>Historical P.E.2 — ChronoSentiment</title>
<style>
:root {{ --ink:#141414; --muted:#5c5c5c; --line:#d8d8d4; --paper:#f4f3ef; --card:#fff; }}
body {{ margin:0; color:var(--ink); background:var(--paper); font-family:ui-sans-serif,system-ui,sans-serif; }}
main {{ max-width:880px; margin:0 auto; padding:28px 24px 64px; }}
.brand {{ font-size:12px; letter-spacing:0.14em; text-transform:uppercase; color:var(--muted); }}
h1 {{ font-size:26px; font-weight:600; margin:6px 0 12px; }}
.note {{ color:var(--muted); max-width:640px; line-height:1.5; }}
.cohorts {{ background:var(--card); border:1px solid var(--line); padding:14px 16px; max-width:480px; line-height:1.45; font-size:13px; }}
.card {{ background:var(--card); border:1px solid var(--line); padding:16px 18px; margin:12px 0; }}
.card.target {{ border-left:3px solid #1b5e3b; }}
.card.horizon {{ border-left:3px solid #5c5c5c; }}
.card dl {{ display:grid; gap:6px; margin:8px 0 0; }}
.card div {{ display:grid; grid-template-columns:200px 1fr; gap:8px; }}
dt {{ color:var(--muted); font-size:13px; }}
dd {{ margin:0; }}
.meta {{ color:var(--muted); font-size:13px; margin:0 0 8px; }}
</style></head><body>
<main>
<p class="brand">ChronoSentiment</p>
<h1>Historical P.E.2 lifecycle validation</h1>
<p class="note">Time-machine of the frozen P.E.2 control. Certified T {certified}. Execution Contract v0 owns target_pct = 5.0%. Historical P.E.2 lifecycle validation: {status}. Statistical strategy backtest: NOT PERFORMED. Live P.E.2 remains AWAITING_NEXT_SESSION. The 14-August cohort stays decision-only. C.3-G is untouched. Search #3 is not authorized.</p>
<pre class="cohorts">14-Aug live cohort
Decision only
UNTOUCHED

Live P.E.2
AWAITING_NEXT_SESSION
0 seals

This sidecar
15 Jul 2026 historical T
Decision + Execution Intent v0
lifecycle validation</pre>
<div class="feed">{cards}</div>
</main>
</body></html>"##,
        certified = escape(&ledger.certified_t),
        status = escape(&ledger.lifecycle_validation),
        cards = cards
    )
}

pub fn historical_pe2_contract_text() -> &'static str {
    "Historical P.E.2 lifecycle validation\n\
P.E.2 specification remains CLOSED.\n\
Live P.E.2 remains AWAITING_NEXT_SESSION with 0 seals.\n\
Requested T: 2026-07-15T03:45:00+00:00\n\
Execution Contract v0: target_pct = 5.0%, max hold = 20 market sessions.\n\
C3-002 chooses direction only.\n\
The 14-August cohort stays decision-only.\n\
P.E.1 targeted_execution_v0 is frozen.\n\
Statistical strategy backtest: NOT PERFORMED\n"
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

fn pass_fail(ok: bool) -> &'static str {
    if ok {
        "PASS"
    } else {
        "FAIL"
    }
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
