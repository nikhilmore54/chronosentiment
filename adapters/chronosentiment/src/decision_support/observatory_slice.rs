//! CS-P-006-P.3–P.7 — sealed-then-measured observatory path and product screens.
//!
//! Decide from C3-002, seal an immutable record, append an observation later,
//! then measure decision value. Outcomes never enter the decision object.
//! P.7 renders Decision as the product object. It does not evolve or go live.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::observatory_maturity::{
    days_remaining, format_observation_close, horizon_label, maturity_line, ui_lifecycle_status,
    UI_STATUS_OUTCOME_DUE,
};

use super::csp006_protocol::RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH;
use super::decision_value_landscape::action_value;
use super::observatory_registry::{candidate_c3_002, CANDIDATE_C3_002};
use super::policy_artifact::{
    first_match_action_from_tmv, PolicyArtifact, CERTIFIED_INPUT_CONCEPTS,
};
use super::DecisionAction;

pub const OBSERVATORY_SLICE_CONTRACT_ID: &str = "csp006p.observatory_slice.1";
pub const OBSERVATORY_ENGINE_VERSION: &str = "unfrozen-dev";
pub const OBSERVATORY_HORIZON_DAYS: u32 = 20;
pub const OBSERVATORY_P7_STARTED: bool = true;
pub const OBSERVATORY_PROSPECTIVE_STARTED: bool = true;
pub const HISTORICAL_PATH_DEMONSTRATION: &str = "historical_path_demonstration";
pub const UI_STATUS_SEALED: &str = "SEALED";
pub const UI_STATUS_OBSERVING: &str = "OBSERVING";
pub const UI_STATUS_OBSERVED: &str = "OBSERVED";
pub const OBSERVATION_STATUS_COMPLETED: &str = "COMPLETED";
pub const OBSERVATORY_MATURITY_UI: bool = true;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertifiedTmvState {
    pub trend: String,
    pub momentum: String,
    pub volatility: String,
    pub input_schema: Vec<String>,
    pub state_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealedDecisionRecord {
    pub decision_id: String,
    pub instrument: String,
    pub decision_time: String,
    pub state: CertifiedTmvState,
    pub action: DecisionAction,
    pub policy_id: String,
    pub policy_artifact_sha256: String,
    pub engine_version: String,
    pub horizon_days: u32,
    pub sealed_status: String,
    pub paper_only: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutcomeObservation {
    pub decision_id: String,
    pub observation_time: String,
    pub observation_status: String,
    pub realized_return: f64,
    pub value_long: f64,
    pub value_short: f64,
    pub value_no_trade: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionValueMeasure {
    pub decision_id: String,
    pub recommended_value: f64,
    pub best_available_value: f64,
    pub regret: f64,
    pub advantage_vs_long: f64,
    pub advantage_vs_short: f64,
    pub advantage_vs_no_trade: f64,
    pub decided_before_outcome: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservatoryLedger {
    pub contract_id: String,
    pub policy_id: String,
    pub policy_artifact_sha256: String,
    pub paper_only: bool,
    pub path_kind: String,
    pub search_three_authorized: bool,
    pub regime_persistence_experiment_authorized: bool,
    pub decisions: Vec<SealedDecisionRecord>,
    pub observations: Vec<OutcomeObservation>,
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn certified_tmv_state(trend: &str, momentum: &str, volatility: &str) -> CertifiedTmvState {
    let input_schema: Vec<String> = CERTIFIED_INPUT_CONCEPTS
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let canonical = serde_json::json!({
        "input_schema": input_schema,
        "momentum": momentum,
        "trend": trend,
        "volatility": volatility,
    });
    CertifiedTmvState {
        trend: trend.to_string(),
        momentum: momentum.to_string(),
        volatility: volatility.to_string(),
        input_schema,
        state_hash: sha256_hex(canonical.to_string().as_bytes()),
    }
}

fn require_c3_002(artifact: &PolicyArtifact) -> Result<(), String> {
    if artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("observatory slice identity-gates C3-002 to Search #2".into());
    }
    Ok(())
}

pub fn generate_decision(
    artifact: &PolicyArtifact,
    instrument: &str,
    decision_time: &str,
    trend: &str,
    momentum: &str,
    volatility: &str,
) -> Result<SealedDecisionRecord, String> {
    require_c3_002(artifact)?;
    if instrument.trim().is_empty() || decision_time.trim().is_empty() {
        return Err("instrument and decision_time are required".into());
    }
    let registry = candidate_c3_002();
    let state = certified_tmv_state(trend, momentum, volatility);
    let action = first_match_action_from_tmv(artifact, trend, momentum, volatility);
    let identity = serde_json::json!({
        "action": action,
        "decision_time": decision_time,
        "engine_version": OBSERVATORY_ENGINE_VERSION,
        "horizon_days": OBSERVATORY_HORIZON_DAYS,
        "instrument": instrument,
        "policy_artifact_sha256": registry.artifact_hash,
        "policy_id": CANDIDATE_C3_002,
        "state_hash": state.state_hash,
    });
    Ok(SealedDecisionRecord {
        decision_id: sha256_hex(identity.to_string().as_bytes()),
        instrument: instrument.to_string(),
        decision_time: decision_time.to_string(),
        state,
        action,
        policy_id: CANDIDATE_C3_002.to_string(),
        policy_artifact_sha256: registry.artifact_hash,
        engine_version: OBSERVATORY_ENGINE_VERSION.to_string(),
        horizon_days: OBSERVATORY_HORIZON_DAYS,
        sealed_status: "OPEN".into(),
        paper_only: true,
    })
}

pub fn observe_outcome(
    decision: &SealedDecisionRecord,
    observation_time: &str,
    realized_return: f64,
) -> Result<OutcomeObservation, String> {
    if !realized_return.is_finite() {
        return Err("realized_return must be finite".into());
    }
    if observation_time.trim().is_empty() {
        return Err("observation_time is required".into());
    }
    Ok(OutcomeObservation {
        decision_id: decision.decision_id.clone(),
        observation_time: observation_time.to_string(),
        observation_status: OBSERVATION_STATUS_COMPLETED.to_string(),
        realized_return,
        value_long: action_value(DecisionAction::Long, realized_return),
        value_short: action_value(DecisionAction::Short, realized_return),
        value_no_trade: action_value(DecisionAction::NoTrade, realized_return),
    })
}

pub fn measure_decision_value(
    decision: &SealedDecisionRecord,
    observation: &OutcomeObservation,
) -> Result<DecisionValueMeasure, String> {
    if decision.decision_id != observation.decision_id {
        return Err("observation does not belong to this decision".into());
    }
    let recommended = action_value(decision.action, observation.realized_return);
    let best = observation
        .value_long
        .max(observation.value_short)
        .max(observation.value_no_trade);
    Ok(DecisionValueMeasure {
        decision_id: decision.decision_id.clone(),
        recommended_value: recommended,
        best_available_value: best,
        regret: best - recommended,
        advantage_vs_long: recommended - observation.value_long,
        advantage_vs_short: recommended - observation.value_short,
        advantage_vs_no_trade: recommended - observation.value_no_trade,
        decided_before_outcome: true,
    })
}

pub fn empty_ledger() -> ObservatoryLedger {
    let registry = candidate_c3_002();
    ObservatoryLedger {
        contract_id: OBSERVATORY_SLICE_CONTRACT_ID.to_string(),
        policy_id: registry.registry_id,
        policy_artifact_sha256: registry.artifact_hash,
        paper_only: true,
        path_kind: HISTORICAL_PATH_DEMONSTRATION.to_string(),
        search_three_authorized: registry.search_three_authorized,
        regime_persistence_experiment_authorized: registry.regime_persistence_experiment_authorized,
        decisions: Vec::new(),
        observations: Vec::new(),
    }
}

pub fn seal_into_ledger(
    ledger: &mut ObservatoryLedger,
    decision: SealedDecisionRecord,
) -> Result<(), String> {
    if ledger
        .decisions
        .iter()
        .any(|d| d.decision_id == decision.decision_id)
    {
        return Err("decision is already sealed".into());
    }
    ledger.decisions.push(decision);
    Ok(())
}

pub fn append_observation(
    ledger: &mut ObservatoryLedger,
    observation: OutcomeObservation,
) -> Result<(), String> {
    if !ledger
        .decisions
        .iter()
        .any(|d| d.decision_id == observation.decision_id)
    {
        return Err("cannot observe an unsealed decision".into());
    }
    if ledger
        .observations
        .iter()
        .any(|o| o.decision_id == observation.decision_id)
    {
        return Err("observation is append-once; the decision record is not rewritten".into());
    }
    ledger.observations.push(observation);
    Ok(())
}

pub fn derived_status(ledger: &ObservatoryLedger, decision_id: &str) -> &'static str {
    if ledger
        .observations
        .iter()
        .any(|o| o.decision_id == decision_id)
    {
        "COMPLETED"
    } else {
        "OPEN"
    }
}

pub fn observation_status_of<'a>(
    ledger: &'a ObservatoryLedger,
    decision_id: &str,
) -> Option<&'a str> {
    ledger
        .observations
        .iter()
        .find(|o| o.decision_id == decision_id)
        .map(|o| o.observation_status.as_str())
}

pub fn ui_decision_status(ledger: &ObservatoryLedger, decision_id: &str) -> &'static str {
    if observation_status_of(ledger, decision_id).is_some() {
        UI_STATUS_OBSERVED
    } else if ledger
        .decisions
        .iter()
        .any(|d| d.decision_id == decision_id)
    {
        UI_STATUS_OBSERVING
    } else {
        UI_STATUS_SEALED
    }
}

pub fn action_label(action: DecisionAction) -> &'static str {
    match action {
        DecisionAction::Long => "LONG",
        DecisionAction::Short => "SHORT",
        DecisionAction::NoTrade => "NO TRADE",
    }
}

pub fn render_observatory_html_with_clocks(
    ledger: &ObservatoryLedger,
    now: DateTime<Utc>,
    requested_clocks: &BTreeMap<String, String>,
) -> String {
    let note = format!(
        "Historical path demonstration. {}/{} decisions completed the seal → observe lifecycle. That is a lifecycle PASS, not a profitability claim. Historical replay is a backtesting mechanism. This replay is not yet a statistical strategy backtest. Replay integrity is not strategy validation. Winners and losers stay visible. Horizon is 20 market sessions. C.3-G is untouched. Search #3 is not authorized.",
        ledger.observations.len(),
        ledger.decisions.len()
    );
    render_observatory_pages(ledger, &note, now, requested_clocks)
}

pub fn render_product_html(
    historical: &ObservatoryLedger,
    prospective: Option<&ObservatoryLedger>,
    now: DateTime<Utc>,
) -> String {
    let mut combined = historical.clone();
    let n_prospective = prospective.map(|p| p.decisions.len()).unwrap_or(0);
    if let Some(p) = prospective {
        combined.decisions.extend(p.decisions.iter().cloned());
        combined.observations.extend(p.observations.iter().cloned());
    }
    let note = format!(
        "Historical path: {h_dec} sealed / {h_obs} observed — lifecycle PASS, not a profitability claim. Historical replay is a backtesting mechanism. This replay is not yet a statistical strategy backtest. Replay integrity is not strategy validation. Horizon is 20 market sessions. Prospective C3-002: {p_dec} sealed, status OBSERVING. Outcomes are not known at T and are not attached. Not CS-P-003 validation. C.3-G is untouched. Search #3 is not authorized.",
        h_dec = historical.decisions.len(),
        h_obs = historical.observations.len(),
        p_dec = n_prospective
    );
    render_observatory_pages(&combined, &note, now, &BTreeMap::new())
}

pub fn render_observatory_html(ledger: &ObservatoryLedger, now: DateTime<Utc>) -> String {
    render_observatory_html_with_clocks(ledger, now, &BTreeMap::new())
}

fn render_observatory_pages(
    ledger: &ObservatoryLedger,
    lifecycle_note: &str,
    now: DateTime<Utc>,
    requested_clocks: &BTreeMap<String, String>,
) -> String {
    let observations: BTreeMap<&str, &OutcomeObservation> = ledger
        .observations
        .iter()
        .map(|o| (o.decision_id.as_str(), o))
        .collect();
    let n_dec = ledger.decisions.len();
    let n_obs = ledger.observations.len();
    let n_long = ledger
        .decisions
        .iter()
        .filter(|d| d.action == DecisionAction::Long)
        .count();
    let n_short = ledger
        .decisions
        .iter()
        .filter(|d| d.action == DecisionAction::Short)
        .count();
    let n_no_trade = ledger
        .decisions
        .iter()
        .filter(|d| d.action == DecisionAction::NoTrade)
        .count();
    let mut by_instrument: BTreeMap<&str, (u32, u32, u32, u32, u32, u32)> = BTreeMap::new();
    let mut by_state: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    let mut by_period: BTreeMap<String, (u32, u32, u32, u32)> = BTreeMap::new();
    let mut n_pos_v = 0u32;
    let mut n_neg_v = 0u32;
    for decision in &ledger.decisions {
        let entry = by_instrument
            .entry(decision.instrument.as_str())
            .or_insert((0, 0, 0, 0, 0, 0));
        entry.0 += 1;
        match decision.action {
            DecisionAction::Long => entry.1 += 1,
            DecisionAction::Short => entry.2 += 1,
            DecisionAction::NoTrade => {}
        }
        let state_key = format!(
            "{} / {} / {}",
            display_trend(&decision.state.trend),
            display_momentum(&decision.state.momentum),
            display_volatility(&decision.state.volatility)
        );
        let state_entry = by_state.entry(state_key).or_insert((0, 0));
        state_entry.0 += 1;
        let period_key = split_timestamp(&decision.decision_time).0;
        let period_entry = by_period.entry(period_key).or_insert((0, 0, 0, 0));
        period_entry.0 += 1;
        if let Some(obs) = observations.get(decision.decision_id.as_str()) {
            if let Ok(measure) = measure_decision_value(decision, obs) {
                entry.3 += 1;
                state_entry.1 += 1;
                period_entry.1 += 1;
                if measure.recommended_value > 0.0 {
                    entry.4 += 1;
                    period_entry.2 += 1;
                    n_pos_v += 1;
                } else if measure.recommended_value < 0.0 {
                    entry.5 += 1;
                    period_entry.3 += 1;
                    n_neg_v += 1;
                }
            }
        }
    }
    let mut instrument_rows = String::new();
    for (instrument, (n, long, short, completed, pos, neg)) in &by_instrument {
        instrument_rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td class=\"num pos\">{}</td><td class=\"num neg\">{}</td></tr>",
            escape(instrument),
            n,
            long,
            short,
            completed,
            pos,
            neg
        ));
    }
    let mut state_rows = String::new();
    for (state, (n, completed)) in &by_state {
        state_rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
            escape(state),
            n,
            completed
        ));
    }
    if state_rows.is_empty() {
        state_rows = "<tr><td colspan=\"3\">No certified states in this ledger.</td></tr>".into();
    }
    let mut period_rows = String::new();
    for (period, (n, completed, pos, neg)) in by_period.iter().rev() {
        period_rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td class=\"num pos\">{}</td><td class=\"num neg\">{}</td></tr>",
            escape(period),
            n,
            completed,
            pos,
            neg
        ));
    }
    if period_rows.is_empty() {
        period_rows =
            "<tr><td colspan=\"5\">No decision timestamps in this ledger.</td></tr>".into();
    }

    let mut maturity_rows = String::new();
    let mut open_count = 0u32;
    let mut due_count = 0u32;
    for decision in &ledger.decisions {
        let status = ui_lifecycle_status(ledger, &decision.decision_id, now);
        if status == UI_STATUS_OBSERVED {
            continue;
        }
        if status == UI_STATUS_OUTCOME_DUE {
            due_count += 1;
        } else {
            open_count += 1;
        }
        let remain = days_remaining(decision, now)
            .map(|d| d.to_string())
            .unwrap_or_else(|_| "—".into());
        maturity_rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            escape(&decision.instrument),
            action_label(decision.action),
            escape(&status),
            escape(&format_observation_close(decision)),
            escape(&remain)
        ));
    }
    if maturity_rows.is_empty() {
        maturity_rows = "<tr><td colspan=\"5\">No open observation windows.</td></tr>".into();
    }

    let mut order: Vec<usize> = (0..ledger.decisions.len()).collect();
    order.sort_by(|&a, &b| {
        ledger.decisions[b]
            .decision_time
            .cmp(&ledger.decisions[a].decision_time)
            .then_with(|| {
                ledger.decisions[b]
                    .instrument
                    .cmp(&ledger.decisions[a].instrument)
            })
    });

    let mut feed = String::new();
    let mut details = String::new();
    for idx in order {
        let decision = &ledger.decisions[idx];
        let ui_status = ui_lifecycle_status(ledger, &decision.decision_id, now);
        let (date, time) = split_timestamp(&decision.decision_time);
        let state_line = format!(
            "{} / {} / {}",
            display_trend(&decision.state.trend),
            display_momentum(&decision.state.momentum),
            display_volatility(&decision.state.volatility)
        );
        let action = action_label(decision.action);
        let short_id = short_id(&decision.decision_id);
        let detail_href = format!("#d-{}", decision.decision_id);
        let (value_html, outcome_html, measure_opt) =
            if let Some(obs) = observations.get(decision.decision_id.as_str()) {
                match measure_decision_value(decision, obs) {
                    Ok(measure) => (
                        format!(
                            "<span class=\"{}\">{}</span>",
                            tone_class(measure.recommended_value),
                            pct(measure.recommended_value)
                        ),
                        format!(
                            "<span class=\"{}\">{}</span>",
                            tone_class(obs.realized_return),
                            pct(obs.realized_return)
                        ),
                        Some((obs, measure)),
                    ),
                    Err(_) => ("—".into(), "—".into(), None),
                }
            } else {
                ("—".into(), "—".into(), None)
            };
        feed.push_str(&format!(
            r#"<a class="card" href="{href}">
<p class="meta">{date}</p>
<div class="card-top"><strong>{instrument}</strong><span class="action">{action}</span></div>
<p class="state">{state}</p>
<p class="value">Decision Value {value}</p>
<p class="meta">{status_line}</p>
</a>"#,
            href = detail_href,
            date = escape(&date),
            instrument = escape(&decision.instrument),
            action = action,
            state = escape(&state_line),
            value = value_html,
            status_line = if ui_status == UI_STATUS_OBSERVED {
                format!(
                    "COMPLETED · observed after {}",
                    horizon_label(decision.horizon_days)
                )
            } else if ui_status == UI_STATUS_OUTCOME_DUE {
                "OUTCOME DUE".into()
            } else {
                maturity_line(decision, now)
            }
        ));
        let alternatives = measure_opt
            .as_ref()
            .map(|(obs, _)| {
                format!(
                    "<dl class=\"kv\"><div><dt>LONG alternative</dt><dd class=\"{}\">{}</dd></div><div><dt>SHORT alternative</dt><dd class=\"{}\">{}</dd></div><div><dt>NO TRADE alternative</dt><dd>{}</dd></div></dl>",
                    tone_class(obs.value_long),
                    pct(obs.value_long),
                    tone_class(obs.value_short),
                    pct(obs.value_short),
                    pct(obs.value_no_trade)
                )
            })
            .unwrap_or_default();
        let outcome_block = measure_opt
            .as_ref()
            .map(|(obs, measure)| {
                format!(
                    "<div><dt>Outcome</dt><dd>{}</dd></div><div><dt>Decision Value</dt><dd>{}</dd></div><div><dt>Observed at</dt><dd>{}</dd></div>",
                    outcome_html,
                    format!(
                        "<span class=\"{}\">{}</span>",
                        tone_class(measure.recommended_value),
                        pct(measure.recommended_value)
                    ),
                    escape(&obs.observation_time)
                )
            })
            .unwrap_or_else(|| {
                let remain = days_remaining(decision, now)
                    .map(|d| d.to_string())
                    .unwrap_or_else(|_| "—".into());
                format!(
                    "<div><dt>Outcome</dt><dd>Outcome not yet observed</dd></div><div><dt>Observation window</dt><dd>{}</dd></div><div><dt>Market sessions remaining</dt><dd>{}</dd></div>",
                    escape(&format_observation_close(decision)),
                    escape(&remain)
                )
            });
        let clock_block = match requested_clocks.get(&decision.decision_id) {
            Some(requested) if requested != &decision.decision_time => format!(
                "<div><dt>Requested observation clock</dt><dd>{}</dd></div><div><dt>Certified market timestamp</dt><dd>{date}{time_sep}{time}</dd></div>",
                escape(requested),
                date = escape(&date),
                time_sep = if time.is_empty() { "" } else { " · " },
                time = escape(&time),
            ),
            _ => format!(
                "<div><dt>Certified market timestamp</dt><dd>{date}{time_sep}{time}</dd></div>",
                date = escape(&date),
                time_sep = if time.is_empty() { "" } else { " · " },
                time = escape(&time),
            ),
        };
        details.push_str(&format!(
            r##"<section class="screen" id="d-{id}">
<p class="crumb"><a href="#feed">Decision Feed</a></p>
<p class="meta">Decision {short}</p>
<h2>{instrument}</h2>
<p class="hero-action">{action}</p>
<ul class="state-list">
<li>{trend}</li>
<li>{momentum}</li>
<li>{volatility}</li>
</ul>
<dl class="kv">
<div><dt>Why this decision</dt><dd>{policy}</dd></div>
{clock}
<div><dt>Observation horizon</dt><dd>{horizon}</dd></div>
<div><dt>Engine version</dt><dd class="mono">{engine}</dd></div>
<div><dt>Decision status</dt><dd>{status}</dd></div>
<div><dt>Decision ID</dt><dd class="mono">{id}</dd></div>
<div><dt>Policy artifact</dt><dd class="mono">{hash}</dd></div>
{outcome}
</dl>
{alternatives}
<p class="seal">This decision was generated without access to information after T. Decision sealed before outcome was known.</p>
<ol class="audit">
<li>Decision created</li>
<li>Policy identified ({policy})</li>
<li>Certified TMV state captured</li>
<li>Action sealed — record is immutable</li>
<li>{status_step}</li>
<li>Outcome attached without rewriting the decision</li>
</ol>
</section>"##,
            id = escape(&decision.decision_id),
            short = escape(&short_id),
            instrument = escape(&decision.instrument),
            action = action,
            trend = escape(&display_trend(&decision.state.trend)),
            momentum = escape(&display_momentum(&decision.state.momentum)),
            volatility = escape(&display_volatility(&decision.state.volatility)),
            policy = escape(&decision.policy_id),
            clock = clock_block,
            horizon = escape(&horizon_label(decision.horizon_days)),
            engine = escape(&decision.engine_version),
            status = ui_status,
            hash = escape(&decision.policy_artifact_sha256),
            outcome = outcome_block,
            alternatives = alternatives,
            status_step = if ui_status == UI_STATUS_OBSERVED {
                "Observation completed — append-only"
            } else if ui_status == UI_STATUS_OUTCOME_DUE {
                "Observation window closed — outcome due"
            } else {
                "Observation pending — outcome not yet observed"
            }
        ));
    }
    if feed.is_empty() {
        feed = "<p class=\"note\">No paper decisions in this ledger.</p>".into();
    }

    format!(
        r##"<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>ChronoSentiment Decision Observatory</title>
<style>
:root {{ --ink:#141414; --muted:#5c5c5c; --line:#d8d8d4; --paper:#f4f3ef; --card:#fff; --pos:#1b5e3b; --neg:#8b2e2e; }}
* {{ box-sizing:border-box; }}
body {{ margin:0; color:var(--ink); background:var(--paper); font-family:ui-sans-serif,system-ui,sans-serif; }}
header {{ border-bottom:1px solid var(--line); background:var(--card); padding:20px 24px 0; }}
.brand {{ font-size:12px; letter-spacing:0.14em; text-transform:uppercase; color:var(--muted); }}
h1 {{ font-size:28px; font-weight:600; margin:6px 0 16px; }}
nav {{ display:flex; gap:20px; }}
nav a {{ color:var(--ink); text-decoration:none; padding-bottom:10px; border-bottom:2px solid transparent; font-size:14px; }}
nav a:hover {{ border-bottom-color:var(--ink); }}
main {{ max-width:880px; margin:0 auto; padding:28px 24px 64px; }}
.screen {{ display:none; }}
.screen:target {{ display:block; }}
body:not(:has(.screen:target)) #observatory {{ display:block; }}
.stats {{ font-size:18px; margin:0 0 8px; }}
.note {{ color:var(--muted); max-width:640px; line-height:1.5; }}
.grid {{ display:grid; grid-template-columns:repeat(3,minmax(0,1fr)); gap:12px; margin:20px 0; }}
.stat {{ background:var(--card); border:1px solid var(--line); padding:14px 16px; }}
.stat b {{ display:block; font-size:22px; font-weight:600; }}
.stat span {{ color:var(--muted); font-size:12px; }}
table {{ border-collapse:collapse; width:100%; margin:12px 0 24px; background:var(--card); }}
th,td {{ border-bottom:1px solid var(--line); text-align:left; padding:8px 10px; font-size:14px; }}
.num {{ text-align:right; font-variant-numeric:tabular-nums; }}
.pos {{ color:var(--pos); }}
.neg {{ color:var(--neg); }}
.feed {{ display:grid; gap:10px; }}
.card {{ display:block; background:var(--card); border:1px solid var(--line); padding:16px 18px; text-decoration:none; color:inherit; }}
.card-top {{ display:flex; justify-content:space-between; align-items:baseline; gap:12px; }}
.action {{ letter-spacing:0.08em; font-size:13px; }}
.state, .state-list {{ color:var(--muted); }}
.state-list {{ list-style:none; padding:0; }}
.state-list li {{ margin:4px 0; }}
.value {{ font-size:16px; margin:8px 0 4px; }}
.meta {{ color:var(--muted); font-size:13px; margin:0 0 6px; }}
.hero-action {{ font-size:22px; letter-spacing:0.08em; margin:0 0 16px; }}
.kv {{ display:grid; gap:10px; }}
.kv div {{ display:grid; grid-template-columns:200px 1fr; gap:12px; border-bottom:1px solid var(--line); padding:8px 0; }}
dt {{ color:var(--muted); font-size:13px; }}
dd {{ margin:0; }}
.mono {{ font-family:ui-monospace,monospace; font-size:12px; word-break:break-all; }}
.seal {{ margin:20px 0; padding:12px 0; border-top:1px solid var(--line); }}
.audit {{ color:var(--muted); }}
.crumb a {{ color:var(--ink); }}
h2 {{ font-size:22px; font-weight:600; }}
.principle {{ background:var(--card); border:1px solid var(--line); padding:16px 18px; margin:0 0 20px; max-width:640px; }}
.principle strong {{ display:block; margin-bottom:6px; }}
</style></head><body>
<header>
<p class="brand">ChronoSentiment</p>
<h1>Decision Observatory</h1>
<nav>
<a href="#observatory">Observatory</a>
<a href="#feed">Decision Feed</a>
<a href="#policy">Policy</a>
</nav>
</header>
<main>
<section class="screen" id="observatory">
<p class="principle"><strong>No early peek. No retrospective edits.</strong> Decisions are sealed at decision time. Outcomes become visible only when their observation window closes. ChronoSentiment is not predicting the future on this screen; it is preserving the boundary between a decision and its future evidence.</p>
<p class="principle"><strong>Replay integrity is not strategy validation.</strong> Historical replay is a backtesting mechanism. This replay is not yet a statistical strategy backtest. The observation horizon is part of the decision-observation contract.</p>
<p class="stats">{n_dec} decisions · {n_long} LONG · {n_short} SHORT · {n_no_trade} NO TRADE · {n_obs} completed · {open_count} observing · {due_count} outcome due · {n_pos_v} positive V · {n_neg_v} negative V</p>
<p class="note">Candidate C3-002 · paper only. This is an evidence dashboard, not a performance dashboard. Winners and losers stay visible. IDEA and MAHABANK remain. Mean, median, and total decision value are not homepage metrics. {lifecycle_note}</p>
<div class="grid">
<div class="stat"><b>{n_dec}</b><span>Decisions generated</span></div>
<div class="stat"><b>{open_count}</b><span>Observing</span></div>
<div class="stat"><b>{n_obs}</b><span>Completed evidence</span></div>
</div>
<div class="grid">
<div class="stat"><b>{n_pos_v}</b><span>Positive decision value</span></div>
<div class="stat"><b>{n_neg_v}</b><span>Negative decision value</span></div>
<div class="stat"><b>{n_long}/{n_short}</b><span>LONG / SHORT</span></div>
</div>
<h2>Open decisions</h2>
<p class="note">Outcome not yet observed. OPEN / OBSERVING = evidence has not matured. OUTCOME DUE = window closed, observation not yet appended. COMPLETED = outcome known. Policy and decision stay immutable. Observation is append-only.</p>
<table><thead><tr><th>Instrument</th><th>Action</th><th>Status</th><th>Observation closes</th><th>Market sessions remaining</th></tr></thead><tbody>{maturity_rows}</tbody></table>
<h2>Certified states</h2>
<p class="note">The same certified-state framework produces different outcomes across instruments and across time. A completed observation does not rewrite the sealed state or action.</p>
<table><thead><tr><th>Certified TMV</th><th>Decisions</th><th>Completed</th></tr></thead><tbody>{state_rows}</tbody></table>
<h2>Time periods</h2>
<p class="note">Time-period evidence is grouped by certified decision date. Sign counts are evidence structure, not a performance claim.</p>
<table><thead><tr><th>Decision date</th><th>Decisions</th><th>Completed</th><th class="num">+V</th><th class="num">−V</th></tr></thead><tbody>{period_rows}</tbody></table>
<h2>Instruments</h2>
<p class="note">IDEA and MAHABANK remain in the ledger. The universe is not expanded. Heterogeneous behaviour is part of the product, not a defect to hide. +V / −V are completed-evidence counts, not a mean return.</p>
<table><thead><tr><th>Instrument</th><th>Decisions</th><th>LONG</th><th>SHORT</th><th>Completed</th><th class="num">+V</th><th class="num">−V</th></tr></thead><tbody>{instrument_rows}</tbody></table>
</section>
<section class="screen" id="feed">
<p class="stats">Decision Feed</p>
<p class="note">Live open decisions first in time, then completed history. OBSERVING = outcome not yet observed. OUTCOME DUE = window closed. COMPLETED = outcome known. Winners and losers stay visible. The sealed record is never rewritten.</p>
<div class="feed">{feed}</div>
</section>
<section class="screen" id="policy">
<p class="stats">C3-002</p>
<h2>Candidate Research Policy</h2>
<dl class="kv">
<div><dt>Product label</dt><dd>ChronoSentiment Research Policy — Candidate C3-002</dd></div>
<div><dt>Artifact</dt><dd class="mono">{hash}</dd></div>
<div><dt>Decisions</dt><dd>{n_dec}</dd></div>
<div><dt>Observations</dt><dd>{n_obs}</dd></div>
<div><dt>Capital</dt><dd>Paper only</dd></div>
<div><dt>Promotion</dt><dd>Not a strategy. Not Decision Engine v1.0. Immutable artifact.</dd></div>
</dl>
<h2>Three layers</h2>
<dl class="kv">
<div><dt>Intelligence</dt><dd>Coralys discovers a decision policy. C3-002 is frozen.</dd></div>
<div><dt>Decision integrity</dt><dd>What was known, what was decided, which policy, when. Decision and policy are immutable.</dd></div>
<div><dt>Evidence</dt><dd>What happened afterward, and the resulting decision value. Observation is append-only.</dd></div>
</dl>
<p class="note">ChronoSentiment creates timestamped, auditable decisions from a defined information state and measures the outcome afterward. It does not predict stocks. A strategy claim can emerge only from accumulated prospective evidence. Research (C.3-F frozen, C.3-G a question) stays off the customer path. The seven-name universe is not expanded.</p>
<ol class="audit">
<li>POLICY — immutable artifact {hash}</li>
<li>DECISION — immutable record; sealed_status stays OPEN</li>
<li>OBSERVING — outcome not yet observed; {horizon} countdown</li>
<li>OUTCOME DUE — window closed; observation not yet appended</li>
<li>OBSERVATION — append-only; observation_status COMPLETED</li>
<li>COMPLETED / OBSERVED — outcome known; decision unchanged</li>
</ol>
</section>
{details}
</main>
</body></html>"##,
        lifecycle_note = escape(lifecycle_note),
        n_dec = n_dec,
        n_long = n_long,
        n_short = n_short,
        n_no_trade = n_no_trade,
        n_obs = n_obs,
        open_count = open_count,
        due_count = due_count,
        n_pos_v = n_pos_v,
        n_neg_v = n_neg_v,
        state_rows = state_rows,
        period_rows = period_rows,
        instrument_rows = instrument_rows,
        maturity_rows = maturity_rows,
        feed = feed,
        hash = escape(&ledger.policy_artifact_sha256),
        horizon = escape(&horizon_label(OBSERVATORY_HORIZON_DAYS)),
        details = details
    )
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

fn tone_class(value: f64) -> &'static str {
    if value > 0.0 {
        "pos"
    } else if value < 0.0 {
        "neg"
    } else {
        ""
    }
}

fn short_id(decision_id: &str) -> String {
    decision_id.chars().take(8).collect()
}

fn split_timestamp(iso: &str) -> (String, String) {
    if iso.len() < 10 {
        return (iso.to_string(), String::new());
    }
    let year = iso[0..4].to_string();
    let month: usize = iso[5..7].parse().unwrap_or(0);
    let day: usize = iso[8..10].parse().unwrap_or(0);
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let mon = months
        .get(month.saturating_sub(1))
        .copied()
        .unwrap_or("???");
    let date = format!("{day} {mon} {year}");
    let time = if iso.len() >= 16 {
        format!("{} UTC", &iso[11..16])
    } else {
        String::new()
    };
    (date, time)
}

fn display_trend(trend: &str) -> String {
    trend.to_string()
}

fn display_momentum(momentum: &str) -> String {
    match momentum {
        "Positive" => "Positive Momentum".into(),
        "Negative" => "Negative Momentum".into(),
        "Neutral" => "Neutral Momentum".into(),
        other => other.to_string(),
    }
}

fn display_volatility(volatility: &str) -> String {
    match volatility.to_ascii_lowercase().as_str() {
        "present" => "Volatility Present".into(),
        other => format!("Volatility {other}"),
    }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
