//! CS-P-006-P.H Historical Observatory Replay.
//!
//! The production Observatory running against a historical clock.
//! Same C3-002 policy. Same generate_decision / generate_prospective_decision
//! path. Future bars are stripped before the decide path can see them.
//! Outcomes are attached only after the contractual close.
//!
//! Not C.3-G. Not Search #3. Historical replay is a backtesting mechanism;
//! this replay is not yet a statistical strategy backtest.
//! Does not mutate the 14 August prospective cohort.

use std::collections::BTreeMap;

use chrono::{DateTime, TimeZone, Utc};
use serde::Serialize;

use crate::ingestion::yahoo::YahooHistoricalBar;

use super::csp006_protocol::{RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_UNIVERSE};
use super::enrichment_certify::bars_at_or_before;
use super::observatory_maturity::{
    append_matured_observation_with_bars, format_observation_close_with_bars,
    observation_due_at_with_bars, require_window_closed_with_bars, ui_lifecycle_status_with_bars,
    HISTORICAL_REPLAY_V1_CONTRACT, HORIZON_CALENDAR_BASIS, HORIZON_UNIT, SESSION_RESOLUTION_RULE,
    TRADING_SESSION_HORIZON_AUTHORIZED, UI_STATUS_OUTCOME_DUE,
};
use super::observatory_prospective::generate_prospective_decision;
use super::observatory_slice::{
    action_label, empty_ledger, measure_decision_value, observe_outcome,
    render_observatory_html_with_clocks, seal_into_ledger, ObservatoryLedger, OutcomeObservation,
    SealedDecisionRecord, OBSERVATORY_HORIZON_DAYS,
};
use super::policy_artifact::PolicyArtifact;

pub const HISTORICAL_REPLAY_PATH_KIND: &str = "historical_observatory_replay";
pub const HISTORICAL_REPLAY_STARTED: bool = true;
pub const PEEKED_RETURNS_AT_SEAL: bool = false;
pub const LOOKAHEAD_BACKTEST_AUTHORIZED: bool = false;
pub const PROSPECTIVE_COHORT_MUTATION_AUTHORIZED: bool = false;
pub const C3G_EXPERIMENT_AUTHORIZED: bool = false;
pub const SEARCH_THREE_AUTHORIZED: bool = false;

/// Two closed-window clock requests. 14 June 2026 is not a session;
/// the engine uses the latest session ≤ that timestamp.
pub const DEFAULT_REPLAY_CLOCKS: [&str; 2] =
    ["2026-05-15T03:45:00+00:00", "2026-06-14T03:45:00+00:00"];

#[derive(Debug, Clone, Serialize)]
pub struct ReplayTickReport {
    pub instrument: String,
    pub requested_clock: String,
    pub decision_time: String,
    pub state_hash: String,
    pub policy_id: String,
    pub action: String,
    pub decision_id: String,
    pub observation_closes: String,
    pub evidence_status: String,
    pub outcome: Option<f64>,
    pub decision_value: Option<f64>,
    pub peeked_returns: bool,
    pub determinism_pass: bool,
    pub lookahead_clean: bool,
    pub session_resolved_from_request: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct HistoricalReplayReport {
    pub path_kind: String,
    pub policy_id: String,
    pub policy_artifact_sha256: String,
    pub n_decisions: usize,
    pub n_observed: usize,
    pub peeked_returns: bool,
    pub determinism_pass: bool,
    pub lookahead_clean: bool,
    pub prospective_cohort_mutated: bool,
    pub replay_contract: String,
    pub horizon_duration_days: u32,
    pub horizon_unit: String,
    pub horizon_calendar_basis: String,
    pub session_resolution_rule: String,
    pub trading_session_horizon_authorized: bool,
    pub statistical_backtest: bool,
    pub ticks: Vec<ReplayTickReport>,
}

pub fn empty_replay_ledger() -> ObservatoryLedger {
    let mut ledger = empty_ledger();
    ledger.path_kind = HISTORICAL_REPLAY_PATH_KIND.to_string();
    ledger
}

pub fn parse_replay_clocks(raw: &[&str]) -> Result<Vec<DateTime<Utc>>, String> {
    raw.iter()
        .map(|s| {
            DateTime::parse_from_rfc3339(s)
                .map(|t| t.with_timezone(&Utc))
                .map_err(|e| format!("replay clock is not RFC3339 ({s}): {e}"))
        })
        .collect()
}

/// Bars the decide path is allowed to see. Future sessions are not present.
pub fn decision_time_bars(
    bars: &[YahooHistoricalBar],
    t: DateTime<Utc>,
) -> Vec<YahooHistoricalBar> {
    bars_at_or_before(bars, t)
}

/// Same production decide path as the prospective Observatory, after the
/// future has been removed from the bar slice.
pub fn generate_historical_replay_decision(
    artifact: &PolicyArtifact,
    instrument: &str,
    bars: &[YahooHistoricalBar],
    historical_now: DateTime<Utc>,
) -> Result<SealedDecisionRecord, String> {
    if artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("historical replay identity-gates C3-002 to Search #2".into());
    }
    if LOOKAHEAD_BACKTEST_AUTHORIZED || PEEKED_RETURNS_AT_SEAL {
        return Err("historical replay refuses a look-ahead configuration".into());
    }
    let known = decision_time_bars(bars, historical_now);
    if known
        .iter()
        .any(|b| bar_time(b).is_some_and(|ts| ts > historical_now))
    {
        return Err(format!(
            "{instrument} decision-time slice contained a bar after {historical_now}"
        ));
    }
    generate_prospective_decision(artifact, instrument, &known, historical_now)
}

pub fn realized_return_after_close(
    bars: &[YahooHistoricalBar],
    decision: &SealedDecisionRecord,
    now: DateTime<Utc>,
) -> Result<f64, String> {
    require_window_closed_with_bars(decision, now, Some(bars))?;
    let t = super::observatory_maturity::parse_decision_time(&decision.decision_time)?;
    let due = observation_due_at_with_bars(decision, Some(bars))?;
    let entry = close_at_or_before(bars, t)
        .ok_or_else(|| format!("no entry close at {} for {}", t, decision.instrument))?;
    let exit = close_at_or_after(bars, due)
        .ok_or_else(|| format!("no exit close at {due} for {}", decision.instrument))?;
    if entry <= 0.0 || !exit.is_finite() {
        return Err(format!("invalid close pair for {}", decision.instrument));
    }
    Ok((exit - entry) / entry)
}

pub fn observe_if_due(
    ledger: &mut ObservatoryLedger,
    decision: &SealedDecisionRecord,
    bars: &[YahooHistoricalBar],
    now: DateTime<Utc>,
) -> Result<Option<OutcomeObservation>, String> {
    if ledger.path_kind != HISTORICAL_REPLAY_PATH_KIND {
        return Err("historical replay observations belong on the replay ledger".into());
    }
    if ui_lifecycle_status_with_bars(ledger, &decision.decision_id, now, Some(bars))
        != UI_STATUS_OUTCOME_DUE
    {
        return Ok(None);
    }
    let realized = realized_return_after_close(bars, decision, now)?;
    let due = observation_due_at_with_bars(decision, Some(bars))?;
    let observation = observe_outcome(decision, &due.to_rfc3339(), realized)?;
    append_matured_observation_with_bars(ledger, decision, observation.clone(), now, Some(bars))?;
    Ok(Some(observation))
}

pub fn replay_cohort(
    artifact: &PolicyArtifact,
    cache: &BTreeMap<String, Vec<YahooHistoricalBar>>,
    clocks: &[DateTime<Utc>],
    now: DateTime<Utc>,
) -> Result<(ObservatoryLedger, HistoricalReplayReport), String> {
    replay_selected(artifact, cache, clocks, &RESEARCH_UNIVERSE, now)
}

pub fn replay_selected(
    artifact: &PolicyArtifact,
    cache: &BTreeMap<String, Vec<YahooHistoricalBar>>,
    clocks: &[DateTime<Utc>],
    instruments: &[&str],
    now: DateTime<Utc>,
) -> Result<(ObservatoryLedger, HistoricalReplayReport), String> {
    if PROSPECTIVE_COHORT_MUTATION_AUTHORIZED {
        return Err("historical replay must not mutate the prospective cohort".into());
    }
    let mut ledger = empty_replay_ledger();
    let mut ticks = Vec::new();
    let mut determinism_pass = true;
    let mut lookahead_clean = true;
    for &clock in clocks {
        for ticker in instruments.iter().copied() {
            let bars = cache
                .get(ticker)
                .ok_or_else(|| format!("yahoo cache missing {ticker}"))?;
            let first = generate_historical_replay_decision(artifact, ticker, bars, clock)?;
            let again = generate_historical_replay_decision(artifact, ticker, bars, clock)?;
            if first != again {
                determinism_pass = false;
            }
            let known = decision_time_bars(bars, clock);
            let from_known = generate_historical_replay_decision(artifact, ticker, &known, clock)?;
            let poisoned = poison_future_bars(bars, clock);
            let from_poisoned =
                generate_historical_replay_decision(artifact, ticker, &poisoned, clock)?;
            if first != from_known || first != from_poisoned {
                lookahead_clean = false;
            }
            seal_into_ledger(&mut ledger, first.clone())?;
            let observation = observe_if_due(&mut ledger, &first, bars, now)?;
            let measure = observation
                .as_ref()
                .and_then(|obs| measure_decision_value(&first, obs).ok());
            ticks.push(ReplayTickReport {
                instrument: first.instrument.clone(),
                requested_clock: clock.to_rfc3339(),
                decision_time: first.decision_time.clone(),
                state_hash: first.state.state_hash.clone(),
                policy_id: first.policy_id.clone(),
                action: action_label(first.action).to_string(),
                decision_id: first.decision_id.clone(),
                observation_closes: format_observation_close_with_bars(&first, Some(bars)),
                evidence_status: ui_lifecycle_status_with_bars(
                    &ledger,
                    &first.decision_id,
                    now,
                    Some(bars),
                )
                .to_string(),
                outcome: observation.as_ref().map(|o| o.realized_return),
                decision_value: measure.as_ref().map(|m| m.recommended_value),
                peeked_returns: PEEKED_RETURNS_AT_SEAL,
                determinism_pass: first == again,
                lookahead_clean: first == from_known && first == from_poisoned,
                session_resolved_from_request: first.decision_time != clock.to_rfc3339(),
            });
        }
    }
    let report = HistoricalReplayReport {
        path_kind: HISTORICAL_REPLAY_PATH_KIND.to_string(),
        policy_id: ledger.policy_id.clone(),
        policy_artifact_sha256: ledger.policy_artifact_sha256.clone(),
        n_decisions: ledger.decisions.len(),
        n_observed: ledger.observations.len(),
        peeked_returns: PEEKED_RETURNS_AT_SEAL,
        determinism_pass,
        lookahead_clean,
        prospective_cohort_mutated: false,
        replay_contract: HISTORICAL_REPLAY_V1_CONTRACT.to_string(),
        horizon_duration_days: OBSERVATORY_HORIZON_DAYS,
        horizon_unit: HORIZON_UNIT.to_string(),
        horizon_calendar_basis: HORIZON_CALENDAR_BASIS.to_string(),
        session_resolution_rule: SESSION_RESOLUTION_RULE.to_string(),
        trading_session_horizon_authorized: TRADING_SESSION_HORIZON_AUTHORIZED,
        statistical_backtest: false,
        ticks,
    };
    Ok((ledger, report))
}

pub fn render_replay_report(report: &HistoricalReplayReport) -> String {
    let mut md = String::new();
    md.push_str("# Observatory Historical Replay Report\n\n");
    md.push_str("**Document type:** Product validation evidence  \n");
    md.push_str("**Parent:** CS-P-006-P.H  \n");
    md.push_str("**Does not:** start C.3-G, run Search #3, retune C3-002, mutate the 14 August cohort, build a performance dashboard  \n\n");
    md.push_str("`.cursor/rules/chronosentiment-core.mdc`: same certified state at T + same sealed policy → same decision; outcomes never construct the decision.\n\n");
    md.push_str("This is the production Observatory running against a historical clock. Historical replay is a backtesting mechanism. This replay is not yet a statistical strategy backtest. Replay integrity is not strategy validation.\n\n");
    md.push_str("## Integrity\n\n");
    md.push_str(&format!(
        "- historical replay integrity: {}\n",
        pass_fail(report.determinism_pass && report.lookahead_clean && !report.peeked_returns)
    ));
    md.push_str(&format!(
        "- statistical strategy backtest: {}\n",
        if report.statistical_backtest {
            "DONE"
        } else {
            "not done"
        }
    ));
    md.push_str("- replay integrity ≠ strategy validation\n\n");
    md.push_str("## Contract\n\n");
    md.push_str(&format!(
        "- replay contract: `{}`\n",
        report.replay_contract
    ));
    md.push_str("Replay v0 (20 calendar days) is archived and is not reinterpreted here.\n\n");
    md.push_str("```text\n");
    md.push_str(&format!(
        "horizon:\n    duration = {}\n    unit = {}\n    calendar_basis = {}\n    weekends = excluded\n    market_holidays = excluded\n",
        report.horizon_duration_days, report.horizon_unit, report.horizon_calendar_basis
    ));
    md.push_str("```\n\n");
    md.push_str(&format!(
        "- session rule: `{}`\n",
        report.session_resolution_rule
    ));
    md.push_str(&format!(
        "- trading-session horizon authorized: {}\n",
        report.trading_session_horizon_authorized
    ));
    md.push_str(&format!("- path_kind: `{}`\n", report.path_kind));
    md.push_str(&format!("- policy: {}\n", report.policy_id));
    md.push_str(&format!(
        "- artifact: `{}`\n",
        report.policy_artifact_sha256
    ));
    md.push_str(&format!("- decisions: {}\n", report.n_decisions));
    md.push_str(&format!("- completed evidence: {}\n", report.n_observed));
    md.push_str(&format!("- peeked_returns: {}\n", report.peeked_returns));
    md.push_str(&format!(
        "- determinism: {}\n",
        pass_fail(report.determinism_pass)
    ));
    md.push_str(&format!(
        "- no-lookahead: {}\n",
        pass_fail(report.lookahead_clean)
    ));
    md.push_str(&format!(
        "- prospective cohort mutated: {}\n\n",
        report.prospective_cohort_mutated
    ));
    md.push_str("14 June 2026 is not a session. The certified market timestamp is the latest session ≤ the requested clock (12 Jun 2026, 03:45 UTC).\n\n");
    md.push_str("This decision was generated without access to information after T.\n\n");
    md.push_str("## Ticks\n\n");
    md.push_str("| Instrument | Requested clock | Decision time | Action | State hash | Closes | Status | Outcome | V | peeked | det | lookahead |\n");
    md.push_str("|---|---|---|---|---|---|---|---|---|---|---|---|\n");
    for tick in &report.ticks {
        md.push_str(&format!(
            "| {} | {} | {} | {} | `{}` | {} | {} | {} | {} | {} | {} | {} |\n",
            tick.instrument,
            tick.requested_clock,
            tick.decision_time,
            tick.action,
            short_hash(&tick.state_hash),
            tick.observation_closes,
            tick.evidence_status,
            tick.outcome.map(pct).unwrap_or_else(|| "—".into()),
            tick.decision_value.map(pct).unwrap_or_else(|| "—".into()),
            tick.peeked_returns,
            pass_fail(tick.determinism_pass),
            pass_fail(tick.lookahead_clean),
        ));
    }
    md.push_str("\nOutcome is an evidence field. It is not part of the sealed decision.\n");
    md.push_str("Winners and losers stay visible because their windows have closed. Fourteen observations are not a statistical performance study. Aggregates are not a homepage metric. Historical replay is a backtesting mechanism; replay integrity is not strategy validation.\n");
    md
}

pub fn requested_clocks_from_report(report: &HistoricalReplayReport) -> BTreeMap<String, String> {
    report
        .ticks
        .iter()
        .map(|t| (t.decision_id.clone(), t.requested_clock.clone()))
        .collect()
}

pub fn render_replay_html(
    ledger: &ObservatoryLedger,
    report: &HistoricalReplayReport,
    now: DateTime<Utc>,
) -> String {
    render_observatory_html_with_clocks(ledger, now, &requested_clocks_from_report(report))
}

pub fn refuse_prospective_output(path: &str) -> Result<(), String> {
    if path.contains("observatory/prospective") {
        return Err("historical replay refuses to write the prospective cohort".into());
    }
    if path.contains("historical_replay_v0") {
        return Err("historical replay refuses to overwrite Replay v0".into());
    }
    Ok(())
}

pub fn poison_future_bars(
    bars: &[YahooHistoricalBar],
    t: DateTime<Utc>,
) -> Vec<YahooHistoricalBar> {
    bars.iter()
        .map(|b| {
            let mut copy = b.clone();
            if bar_time(b).is_some_and(|ts| ts > t) {
                copy.open = 1_000_000.0;
                copy.high = 1_000_000.0;
                copy.low = 1_000_000.0;
                copy.close = 1_000_000.0;
                copy.adj_close = 1_000_000.0;
            }
            copy
        })
        .collect()
}

fn bar_time(bar: &YahooHistoricalBar) -> Option<DateTime<Utc>> {
    Utc.timestamp_opt(bar.timestamp, 0).single()
}

fn close_at_or_before(bars: &[YahooHistoricalBar], t: DateTime<Utc>) -> Option<f64> {
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

fn close_at_or_after(bars: &[YahooHistoricalBar], t: DateTime<Utc>) -> Option<f64> {
    bars.iter()
        .filter_map(|b| {
            let ts = bar_time(b)?;
            if ts >= t && b.adj_close.is_finite() && b.adj_close > 0.0 {
                Some((ts, b.adj_close))
            } else {
                None
            }
        })
        .min_by_key(|(ts, _)| *ts)
        .map(|(_, c)| c)
}

fn short_hash(hash: &str) -> String {
    hash.chars().take(8).collect::<String>() + "…"
}

fn pct(value: f64) -> String {
    format!("{value:+.2}%", value = value * 100.0)
}

fn pass_fail(ok: bool) -> &'static str {
    if ok {
        "PASS"
    } else {
        "FAIL"
    }
}
