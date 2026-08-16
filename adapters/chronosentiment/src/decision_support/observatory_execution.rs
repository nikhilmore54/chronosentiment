//! CS-P-006-P.E Targeted Decision Execution.
//!
//! C3-002 still chooses LONG / SHORT / NO_TRADE. This module seals a target at T
//! and detects the first exit on later OHLC. It does not retune C3-002.
//! It does not path-optimize the target. It does not start C.3-G.

use std::collections::BTreeMap;

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::ingestion::yahoo::YahooHistoricalBar;

use super::csp006_protocol::{RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_UNIVERSE};
use super::observatory_historical::{
    generate_historical_replay_decision, parse_replay_clocks, DEFAULT_REPLAY_CLOCKS,
};
use super::observatory_maturity::nth_market_session_after;
use super::observatory_slice::{action_label, SealedDecisionRecord, OBSERVATORY_HORIZON_DAYS};
use super::policy_artifact::PolicyArtifact;
use super::DecisionAction;

pub const TARGETED_EXECUTION_STARTED: bool = true;
pub const TARGETED_EXECUTION_V0_FROZEN: bool = true;
pub const TARGET_PATH_OPTIMIZATION_AUTHORIZED: bool = false;
pub const STOP_EXIT_AUTHORIZED: bool = false;
pub const SEARCH_THREE_AUTHORIZED: bool = false;
pub const C3G_EXPERIMENT_AUTHORIZED: bool = false;
pub const PROSPECTIVE_COHORT_MUTATION_AUTHORIZED: bool = false;
pub const EXECUTION_CONTRACT_ID: &str = "targeted_execution_v0_fixed_5pct_20_sessions";
pub const EXECUTION_CONTRACT_LABEL: &str = "Execution Contract v0";
pub const EXECUTION_TARGET_SOURCE: &str = "deterministic_policy_parameter";
pub const EXECUTION_TARGET_PCT: f64 = 0.05;
pub const EXECUTION_PATH_KIND: &str = "targeted_execution_replay";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExitReason {
    Target,
    Stop,
    Horizon,
    Ambiguous,
    NoTrade,
    Observing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TriggerType {
    HighReached,
    LowReached,
    GapThrough,
    SessionClose,
    Ambiguous,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SealedExecutionIntent {
    pub decision_id: String,
    pub instrument: String,
    pub decision_time: String,
    pub action: String,
    pub entry_price: f64,
    pub target_pct: f64,
    pub target_price: f64,
    pub stop_pct: Option<f64>,
    pub stop_price: Option<f64>,
    pub max_holding_sessions: u32,
    pub target_source: String,
    pub execution_contract: String,
    pub sealed_at_t: bool,
    pub intent_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionExit {
    pub decision_id: String,
    pub target_hit: bool,
    pub target_hit_session: Option<u32>,
    pub exit_price: Option<f64>,
    pub exit_reason: ExitReason,
    pub holding_sessions: Option<u32>,
    pub exit_time: Option<String>,
    pub decision_value: Option<f64>,
    pub trigger_type: Option<TriggerType>,
    pub trigger_session: Option<u32>,
    pub trigger_timestamp: Option<String>,
    pub trigger_price: Option<f64>,
    pub execution_price: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TargetedExecutionTick {
    pub instrument: String,
    pub requested_clock: String,
    pub decision_time: String,
    pub decision_id: String,
    pub direction: String,
    pub entry_price: f64,
    pub target_pct: f64,
    pub target_price: f64,
    pub target_hit: bool,
    pub target_hit_session: Option<u32>,
    pub exit_price: Option<f64>,
    pub exit_reason: ExitReason,
    pub holding_sessions: Option<u32>,
    pub decision_value: Option<f64>,
    pub peeked_returns_at_seal: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TargetedExecutionReport {
    pub path_kind: String,
    pub execution_contract: String,
    pub target_source: String,
    pub target_pct: f64,
    pub max_holding_sessions: u32,
    pub stop_exit_authorized: bool,
    pub target_path_optimization_authorized: bool,
    pub n_decisions: usize,
    pub n_exits: usize,
    pub n_target: usize,
    pub n_horizon: usize,
    pub n_no_trade: usize,
    pub peeked_returns_at_seal: bool,
    pub prospective_cohort_mutated: bool,
    pub statistical_backtest: bool,
    pub ticks: Vec<TargetedExecutionTick>,
}

pub fn refuse_protected_output(path: &str) -> Result<(), String> {
    for forbidden in [
        "observatory/prospective",
        "historical_replay_v0",
        "historical_replay_v1",
        "selected_policy.json",
    ] {
        if path.contains(forbidden) {
            return Err(format!("targeted execution refuses to write {forbidden}"));
        }
    }
    if TARGETED_EXECUTION_V0_FROZEN && path.contains("targeted_execution_v0") {
        return Err("P.E.1 sidecar targeted_execution_v0 is frozen".into());
    }
    if path.contains("observatory/historical_replay") && !path.contains("targeted_execution") {
        return Err("targeted execution refuses to overwrite Historical Replay ledgers".into());
    }
    Ok(())
}

pub fn target_price(action: DecisionAction, entry: f64, target_pct: f64) -> Result<f64, String> {
    if !(entry > 0.0 && entry.is_finite() && target_pct.is_finite() && target_pct > 0.0) {
        return Err("entry and target_pct must be finite and positive".into());
    }
    match action {
        DecisionAction::Long => Ok(entry * (1.0 + target_pct)),
        DecisionAction::Short => Ok(entry * (1.0 - target_pct)),
        DecisionAction::NoTrade => Err("NO_TRADE has no target price".into()),
    }
}

pub fn seal_execution_intent(
    decision: &SealedDecisionRecord,
    entry_price: f64,
    target_pct: f64,
) -> Result<SealedExecutionIntent, String> {
    if TARGET_PATH_OPTIMIZATION_AUTHORIZED {
        return Err("refusing a path-optimized target".into());
    }
    if decision.policy_artifact_sha256 != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("execution identity-gates C3-002".into());
    }
    let (target, stop_pct, stop_price) = match decision.action {
        DecisionAction::NoTrade => (entry_price, None, None),
        action => {
            if STOP_EXIT_AUTHORIZED {
                return Err("stops are not authorized on execution contract v0".into());
            }
            (target_price(action, entry_price, target_pct)?, None, None)
        }
    };
    let identity = serde_json::json!({
        "action": decision.action,
        "decision_id": decision.decision_id,
        "entry_price": format!("{entry_price:.8}"),
        "execution_contract": EXECUTION_CONTRACT_ID,
        "max_holding_sessions": OBSERVATORY_HORIZON_DAYS,
        "target_pct": format!("{target_pct:.8}"),
        "target_price": format!("{target:.8}"),
        "target_source": EXECUTION_TARGET_SOURCE,
    });
    Ok(SealedExecutionIntent {
        decision_id: decision.decision_id.clone(),
        instrument: decision.instrument.clone(),
        decision_time: decision.decision_time.clone(),
        action: action_label(decision.action).to_string(),
        entry_price,
        target_pct,
        target_price: target,
        stop_pct,
        stop_price,
        max_holding_sessions: OBSERVATORY_HORIZON_DAYS,
        target_source: EXECUTION_TARGET_SOURCE.to_string(),
        execution_contract: EXECUTION_CONTRACT_ID.to_string(),
        sealed_at_t: true,
        intent_hash: format!("{:x}", Sha256::digest(identity.to_string().as_bytes())),
    })
}

pub fn first_exit(
    decision: &SealedDecisionRecord,
    intent: &SealedExecutionIntent,
    bars: &[YahooHistoricalBar],
) -> Result<ExecutionExit, String> {
    if intent.decision_id != decision.decision_id {
        return Err("execution intent does not belong to this decision".into());
    }
    if decision.action == DecisionAction::NoTrade {
        return Ok(idle_exit(
            &decision.decision_id,
            ExitReason::NoTrade,
            Some(0.0),
        ));
    }
    let t = super::observatory_maturity::parse_decision_time(&decision.decision_time)?;
    let mut future: Vec<&YahooHistoricalBar> = bars
        .iter()
        .filter(|b| bar_time(b).is_some_and(|ts| ts > t))
        .collect();
    future.sort_by_key(|b| bar_time(b));
    future.dedup_by_key(|b| bar_time(b));
    let horizon_t = nth_market_session_after(bars, t, intent.max_holding_sessions);
    let mut session: u32 = 0;
    for bar in future {
        session += 1;
        if session > intent.max_holding_sessions {
            break;
        }
        let Some(ts) = bar_time(bar) else {
            continue;
        };
        if let Some(due) = horizon_t {
            if ts > due {
                break;
            }
        }
        let (open, high, low, close) = adj_ohlc(bar);
        let hit_target = match decision.action {
            DecisionAction::Long => high >= intent.target_price,
            DecisionAction::Short => low <= intent.target_price,
            DecisionAction::NoTrade => false,
        };
        let hit_stop = match (STOP_EXIT_AUTHORIZED, intent.stop_price, decision.action) {
            (true, Some(stop), DecisionAction::Long) => low <= stop,
            (true, Some(stop), DecisionAction::Short) => high >= stop,
            _ => false,
        };
        if hit_target && hit_stop {
            return Ok(exit_record(
                decision,
                intent,
                ExitReason::Ambiguous,
                session,
                close,
                ts,
                false,
                Some(TriggerType::Ambiguous),
                Some(close),
            ));
        }
        if hit_target {
            let (trigger, trigger_price, fill) =
                target_trigger(decision.action, open, high, low, intent.target_price);
            return Ok(exit_record(
                decision,
                intent,
                ExitReason::Target,
                session,
                fill,
                ts,
                true,
                Some(trigger),
                Some(trigger_price),
            ));
        }
        if hit_stop {
            return Ok(exit_record(
                decision,
                intent,
                ExitReason::Stop,
                session,
                intent.stop_price.unwrap_or(close),
                ts,
                false,
                None,
                None,
            ));
        }
        if Some(ts) == horizon_t || session == intent.max_holding_sessions {
            return Ok(exit_record(
                decision,
                intent,
                ExitReason::Horizon,
                session,
                close,
                ts,
                false,
                Some(TriggerType::SessionClose),
                Some(close),
            ));
        }
    }
    Ok(idle_exit(
        &decision.decision_id,
        ExitReason::Observing,
        None,
    ))
}

/// Same-bar target and stop when a stop is supplied. Used by tests; v0 replay
/// does not authorize stops.
pub fn first_exit_with_optional_stop(
    decision: &SealedDecisionRecord,
    intent: &SealedExecutionIntent,
    bars: &[YahooHistoricalBar],
    stop_price: Option<f64>,
    stop_authorized: bool,
) -> Result<ExecutionExit, String> {
    if !stop_authorized || stop_price.is_none() {
        return first_exit(decision, intent, bars);
    }
    let t = super::observatory_maturity::parse_decision_time(&decision.decision_time)?;
    let mut future: Vec<&YahooHistoricalBar> = bars
        .iter()
        .filter(|b| bar_time(b).is_some_and(|ts| ts > t))
        .collect();
    future.sort_by_key(|b| bar_time(b));
    let stop = stop_price.unwrap();
    let mut session = 0u32;
    for bar in future {
        session += 1;
        if session > intent.max_holding_sessions {
            break;
        }
        let Some(ts) = bar_time(bar) else {
            continue;
        };
        let (open, high, low, close) = adj_ohlc(bar);
        let hit_target = match decision.action {
            DecisionAction::Long => high >= intent.target_price,
            DecisionAction::Short => low <= intent.target_price,
            DecisionAction::NoTrade => false,
        };
        let hit_stop = match decision.action {
            DecisionAction::Long => low <= stop,
            DecisionAction::Short => high >= stop,
            DecisionAction::NoTrade => false,
        };
        if hit_target && hit_stop {
            return Ok(exit_record(
                decision,
                intent,
                ExitReason::Ambiguous,
                session,
                close,
                ts,
                false,
                Some(TriggerType::Ambiguous),
                Some(close),
            ));
        }
        if hit_target {
            let (trigger, trigger_price, fill) =
                target_trigger(decision.action, open, high, low, intent.target_price);
            return Ok(exit_record(
                decision,
                intent,
                ExitReason::Target,
                session,
                fill,
                ts,
                true,
                Some(trigger),
                Some(trigger_price),
            ));
        }
        if hit_stop {
            return Ok(exit_record(
                decision,
                intent,
                ExitReason::Stop,
                session,
                stop,
                ts,
                false,
                None,
                None,
            ));
        }
    }
    first_exit(decision, intent, bars)
}

fn idle_exit(decision_id: &str, reason: ExitReason, decision_value: Option<f64>) -> ExecutionExit {
    ExecutionExit {
        decision_id: decision_id.to_string(),
        target_hit: false,
        target_hit_session: None,
        exit_price: None,
        exit_reason: reason,
        holding_sessions: None,
        exit_time: None,
        decision_value,
        trigger_type: None,
        trigger_session: None,
        trigger_timestamp: None,
        trigger_price: None,
        execution_price: None,
    }
}

fn target_trigger(
    action: DecisionAction,
    open: f64,
    high: f64,
    low: f64,
    target_price: f64,
) -> (TriggerType, f64, f64) {
    match action {
        DecisionAction::Long if open >= target_price => (TriggerType::GapThrough, open, open),
        DecisionAction::Short if open <= target_price => (TriggerType::GapThrough, open, open),
        DecisionAction::Long => (TriggerType::HighReached, high, target_price),
        DecisionAction::Short => (TriggerType::LowReached, low, target_price),
        DecisionAction::NoTrade => (TriggerType::Ambiguous, open, open),
    }
}

fn exit_record(
    decision: &SealedDecisionRecord,
    intent: &SealedExecutionIntent,
    reason: ExitReason,
    session: u32,
    exit_price: f64,
    ts: DateTime<Utc>,
    target_hit: bool,
    trigger_type: Option<TriggerType>,
    trigger_price: Option<f64>,
) -> ExecutionExit {
    let v = signed_value(decision.action, intent.entry_price, exit_price);
    let trigger_session = match reason {
        ExitReason::Target | ExitReason::Horizon | ExitReason::Ambiguous => Some(session),
        _ => None,
    };
    ExecutionExit {
        decision_id: decision.decision_id.clone(),
        target_hit,
        target_hit_session: if target_hit { Some(session) } else { None },
        exit_price: Some(exit_price),
        exit_reason: reason,
        holding_sessions: Some(session),
        exit_time: Some(ts.to_rfc3339()),
        decision_value: v,
        trigger_type,
        trigger_session,
        trigger_timestamp: Some(ts.to_rfc3339()),
        trigger_price,
        execution_price: Some(exit_price),
    }
}

fn signed_value(action: DecisionAction, entry: f64, exit: f64) -> Option<f64> {
    if !(entry > 0.0 && entry.is_finite() && exit.is_finite()) {
        return None;
    }
    match action {
        DecisionAction::Long => Some((exit - entry) / entry),
        DecisionAction::Short => Some((entry - exit) / entry),
        DecisionAction::NoTrade => Some(0.0),
    }
}

fn adj_ohlc(bar: &YahooHistoricalBar) -> (f64, f64, f64, f64) {
    let ratio = if bar.close > 0.0 && bar.close.is_finite() {
        bar.adj_close / bar.close
    } else {
        1.0
    };
    (
        bar.open * ratio,
        bar.high * ratio,
        bar.low * ratio,
        bar.adj_close,
    )
}

fn bar_time(bar: &YahooHistoricalBar) -> Option<DateTime<Utc>> {
    Utc.timestamp_opt(bar.timestamp, 0).single()
}

pub(crate) fn entry_close(bars: &[YahooHistoricalBar], t: DateTime<Utc>) -> Option<f64> {
    bars.iter()
        .filter_map(|b| {
            let ts = bar_time(b)?;
            if ts <= t && b.adj_close.is_finite() && b.adj_close > 0.0 {
                Some((ts, b.adj_close))
            } else {
                None
            }
        })
        .max_by_key(|(ts, _)| *ts)
        .map(|(_, c)| c)
}

pub fn replay_targeted_execution(
    artifact: &PolicyArtifact,
    cache: &BTreeMap<String, Vec<YahooHistoricalBar>>,
    clocks: &[DateTime<Utc>],
    now: DateTime<Utc>,
) -> Result<(Vec<SealedExecutionIntent>, TargetedExecutionReport), String> {
    if PROSPECTIVE_COHORT_MUTATION_AUTHORIZED
        || TARGET_PATH_OPTIMIZATION_AUTHORIZED
        || STOP_EXIT_AUTHORIZED
        || SEARCH_THREE_AUTHORIZED
        || C3G_EXPERIMENT_AUTHORIZED
    {
        return Err("refusing an execution replay that opens research or mutates prospective".into());
    }
    let _ = now;
    let mut intents = Vec::new();
    let mut ticks = Vec::new();
    for &clock in clocks {
        for instrument in RESEARCH_UNIVERSE {
            let bars = cache
                .get(instrument)
                .ok_or_else(|| format!("yahoo cache missing {instrument}"))?;
            let decision = generate_historical_replay_decision(artifact, instrument, bars, clock)?;
            let t = super::observatory_maturity::parse_decision_time(&decision.decision_time)?;
            let entry = entry_close(bars, t).ok_or_else(|| {
                format!("no entry close at {} for {instrument}", decision.decision_time)
            })?;
            let intent = seal_execution_intent(&decision, entry, EXECUTION_TARGET_PCT)?;
            let exit = first_exit(&decision, &intent, bars)?;
            ticks.push(TargetedExecutionTick {
                instrument: instrument.to_string(),
                requested_clock: clock.to_rfc3339(),
                decision_time: decision.decision_time.clone(),
                decision_id: decision.decision_id.clone(),
                direction: intent.action.clone(),
                entry_price: intent.entry_price,
                target_pct: intent.target_pct,
                target_price: intent.target_price,
                target_hit: exit.target_hit,
                target_hit_session: exit.target_hit_session,
                exit_price: exit.exit_price,
                exit_reason: exit.exit_reason,
                holding_sessions: exit.holding_sessions,
                decision_value: exit.decision_value,
                peeked_returns_at_seal: false,
            });
            intents.push(intent);
        }
    }
    let n_target = ticks
        .iter()
        .filter(|t| t.exit_reason == ExitReason::Target)
        .count();
    let n_horizon = ticks
        .iter()
        .filter(|t| t.exit_reason == ExitReason::Horizon)
        .count();
    let n_no_trade = ticks
        .iter()
        .filter(|t| t.exit_reason == ExitReason::NoTrade)
        .count();
    let n_exits = ticks
        .iter()
        .filter(|t| !matches!(t.exit_reason, ExitReason::Observing))
        .count();
    Ok((
        intents,
        TargetedExecutionReport {
            path_kind: EXECUTION_PATH_KIND.to_string(),
            execution_contract: EXECUTION_CONTRACT_ID.to_string(),
            target_source: EXECUTION_TARGET_SOURCE.to_string(),
            target_pct: EXECUTION_TARGET_PCT,
            max_holding_sessions: OBSERVATORY_HORIZON_DAYS,
            stop_exit_authorized: STOP_EXIT_AUTHORIZED,
            target_path_optimization_authorized: TARGET_PATH_OPTIMIZATION_AUTHORIZED,
            n_decisions: ticks.len(),
            n_exits,
            n_target,
            n_horizon,
            n_no_trade,
            peeked_returns_at_seal: false,
            prospective_cohort_mutated: false,
            statistical_backtest: false,
            ticks,
        },
    ))
}

pub fn default_execution_clocks() -> Result<Vec<DateTime<Utc>>, String> {
    parse_replay_clocks(&DEFAULT_REPLAY_CLOCKS)
}

pub fn render_execution_report(report: &TargetedExecutionReport) -> String {
    let mut md = String::new();
    md.push_str("# Targeted Decision Execution Report\n\n");
    md.push_str("**Document type:** Product validation evidence  \n");
    md.push_str("**Parent:** CS-P-006-P.E  \n");
    md.push_str("**Does not:** start C.3-G, run Search #3, retune C3-002, path-optimize the target, mutate the 14 August cohort  \n\n");
    md.push_str("`.cursor/rules/chronosentiment-core.mdc`: the target is sealed at T; future OHLC never chooses the target.\n\n");
    md.push_str("C3-002 chooses direction only. Execution Contract v0 owns `target_pct = 5.0%` and the 20-market-session maximum hold. Historical replay is a backtesting mechanism. This replay is not yet a statistical strategy backtest. Replay integrity is not strategy validation.\n\n");
    md.push_str("## Layers\n\n");
    md.push_str("| Layer | Question |\n|---|---|\n");
    md.push_str("| Decision | Was LONG / SHORT / NO_TRADE selected from the certified state? |\n");
    md.push_str("| Execution | Did the predefined target get reached before the maximum holding period? |\n");
    md.push_str("| Evidence | What was the realized value after that exit? |\n\n");
    md.push_str("TARGET and HORIZON exits are both evidence. Neither is hidden.\n\n");
    md.push_str("## Integrity\n\n");
    md.push_str(&format!("- product label: {}\n", EXECUTION_CONTRACT_LABEL));
    md.push_str(&format!("- execution contract: `{}`\n", report.execution_contract));
    md.push_str(&format!("- target source: `{}`\n", report.target_source));
    md.push_str(&format!("- target_pct: {:.1}%\n", report.target_pct * 100.0));
    md.push_str(&format!("- max holding sessions: {}\n", report.max_holding_sessions));
    md.push_str(&format!("- stop authorized: {}\n", report.stop_exit_authorized));
    md.push_str(&format!(
        "- target path-optimization authorized: {}\n",
        report.target_path_optimization_authorized
    ));
    md.push_str(&format!("- peeked_returns_at_seal: {}\n", report.peeked_returns_at_seal));
    md.push_str(&format!(
        "- prospective cohort mutated: {}\n",
        report.prospective_cohort_mutated
    ));
    md.push_str(&format!(
        "- statistical strategy backtest: {}\n\n",
        if report.statistical_backtest {
            "DONE"
        } else {
            "not done"
        }
    ));
    md.push_str("Replay v0/v1 close-to-close observations are not reinterpreted here.\n\n");
    md.push_str("## Counts\n\n");
    md.push_str(&format!("- decisions: {}\n", report.n_decisions));
    md.push_str(&format!("- exits: {}\n", report.n_exits));
    md.push_str(&format!("- TARGET: {}\n", report.n_target));
    md.push_str(&format!("- HORIZON: {}\n", report.n_horizon));
    md.push_str(&format!("- NO_TRADE: {}\n\n", report.n_no_trade));
    md.push_str("## Ticks\n\n");
    md.push_str("| Instrument | Requested clock | Decision time | Direction | Entry | Target | Hit | Hit session | Exit | Reason | Hold | V |\n");
    md.push_str("|---|---|---|---|---:|---:|---|---:|---:|---|---:|---:|\n");
    for tick in &report.ticks {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {:.2} | {:.2} | {} | {} | {} | {} | {} | {} |\n",
            tick.instrument,
            tick.requested_clock,
            tick.decision_time,
            tick.direction,
            tick.entry_price,
            tick.target_price,
            tick.target_hit,
            tick.target_hit_session
                .map(|n| n.to_string())
                .unwrap_or_else(|| "—".into()),
            tick.exit_price
                .map(|p| format!("{p:.2}"))
                .unwrap_or_else(|| "—".into()),
            exit_label(tick.exit_reason),
            tick.holding_sessions
                .map(|n| n.to_string())
                .unwrap_or_else(|| "—".into()),
            tick.decision_value.map(pct).unwrap_or_else(|| "—".into()),
        ));
    }
    md.push_str("\nExit reason TARGET means the high (LONG) or low (SHORT) reached the sealed target. HORIZON means the 20th market session closed without a hit. Both are evidence. Aggregates are not a homepage metric. C3-002 does not have a 5% target.\n");
    md
}

pub fn render_execution_html(report: &TargetedExecutionReport) -> String {
    let mut cards = String::new();
    for tick in &report.ticks {
        let hold = tick
            .holding_sessions
            .map(|n| format!("{n} sessions"))
            .unwrap_or_else(|| "—".into());
        let v = tick.decision_value.map(pct).unwrap_or_else(|| "—".into());
        let reason = exit_label(tick.exit_reason);
        let tone = match tick.exit_reason {
            ExitReason::Target => "target",
            ExitReason::Horizon => "horizon",
            _ => "other",
        };
        cards.push_str(&format!(
            r#"<article class="card {tone}">
<p class="meta">{instrument} · {date}</p>
<dl>
<div><dt>Decision</dt><dd>{direction}</dd></div>
<div><dt>Target</dt><dd>{target}</dd></div>
<div><dt>Maximum hold</dt><dd>20 sessions</dd></div>
<div><dt>Exit</dt><dd>{reason}</dd></div>
<div><dt>Holding period</dt><dd>{hold}</dd></div>
<div><dt>Realized decision value</dt><dd class="v">{value}</dd></div>
</dl>
</article>"#,
            instrument = escape(tick.instrument.as_str()),
            date = escape(&tick.decision_time),
            direction = escape(&tick.direction),
            target = format!("{:+.1}%", tick.target_pct * 100.0),
            reason = reason,
            hold = escape(&hold),
            value = escape(&v),
        ));
    }
    if cards.is_empty() {
        cards = "<p class=\"note\">No execution ticks in this report.</p>".into();
    }
    format!(
        r##"<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>Execution Contract v0 — ChronoSentiment</title>
<style>
:root {{ --ink:#141414; --muted:#5c5c5c; --line:#d8d8d4; --paper:#f4f3ef; --card:#fff; }}
body {{ margin:0; color:var(--ink); background:var(--paper); font-family:ui-sans-serif,system-ui,sans-serif; }}
main {{ max-width:880px; margin:0 auto; padding:28px 24px 64px; }}
.brand {{ font-size:12px; letter-spacing:0.14em; text-transform:uppercase; color:var(--muted); }}
h1 {{ font-size:26px; font-weight:600; margin:6px 0 12px; }}
.note {{ color:var(--muted); max-width:640px; line-height:1.5; }}
.layers {{ width:100%; border-collapse:collapse; background:var(--card); margin:16px 0 24px; }}
.layers th,.layers td {{ border-bottom:1px solid var(--line); text-align:left; padding:8px 10px; font-size:14px; }}
.feed {{ display:grid; gap:12px; }}
.card {{ background:var(--card); border:1px solid var(--line); padding:16px 18px; }}
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
<h1>Execution Contract v0</h1>
<p class="note">C3-002 chooses direction. Execution Contract v0 owns target_pct = 5.0% and a 20-session maximum hold. TARGET and HORIZON exits are both evidence. This is not a statistical strategy backtest. C.3-G is untouched. Search #3 is not authorized.</p>
<table class="layers">
<thead><tr><th>Layer</th><th>Question</th></tr></thead>
<tbody>
<tr><td>Decision</td><td>Was LONG / SHORT / NO_TRADE selected from the certified state?</td></tr>
<tr><td>Execution</td><td>Did the predefined target get reached before the maximum holding period?</td></tr>
<tr><td>Evidence</td><td>What was the realized value after that exit?</td></tr>
</tbody>
</table>
<p class="note">{n_dec} decisions · {n_target} TARGET · {n_horizon} HORIZON · IDEA and MAHABANK remain. Mean / median / total V are not homepage metrics.</p>
<div class="feed">{cards}</div>
</main>
</body></html>"##,
        n_dec = report.n_decisions,
        n_target = report.n_target,
        n_horizon = report.n_horizon,
        cards = cards
    )
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
