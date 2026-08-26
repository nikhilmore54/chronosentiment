//! CS-P-006-P.E.3 Live Execution Path — coralys-exec-v0 wired into the live execution path.
//!
//! This module replaces the fixed +5% `seal_execution_intent()` call in the live path
//! with `seal_coralys_execution_intent()` from coralys-exec-v0.
//!
//! Architecture:
//!   C3-002 direction (sealed at T)
//!     ↓
//!   coralys-exec-v0 (ATR/TMV → target%, risk%, sealed at E)
//!     ↓
//!   SealedExecutionIntent (consumed by first_exit())
//!     ↓
//!   ExecutionExit → ExecutionFeedbackRecord
//!
//! What this module does NOT do:
//!   - Does not modify C3-002 direction
//!   - Does not modify coralys-exec-v0 multipliers (frozen artifact 3876ffa2...)
//!   - Does not fall back to +5% when ATR=0 (Invalid positions excluded from P.E.3 sample)
//!   - Does not touch the P.E.2 live execution path (observatory_live_execution.rs)
//!   - Does not start C.3-G, Search #3, or stop-exit research
//!
//! P.E.3 treatment positions are those where coralys-exec-v0 produces a valid intent.
//! Positions where ATR=0 or unavailable are excluded (CoralysExecutionResult::Invalid).
//! They are NOT replaced with the P.E.2 +5% control — that would contaminate the comparison.

use std::collections::BTreeMap;

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ingestion::yahoo::YahooHistoricalBar;

use super::coralys_execution_model::{
    seal_coralys_execution_intent, CoralysExecutionResult, CORALYS_EXEC_ARTIFACT_HASH,
    CORALYS_EXEC_MODEL_ID, CORALYS_EXEC_MODEL_VERSION, MAXIMUM_HOLD_SESSIONS,
};
use super::csp006_protocol::{RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_UNIVERSE};
use super::enrichment_certify::{bars_at_or_before, metrics_from_bars_at_t};
use super::observatory_execution::{
    first_exit, ExecutionExit, ExitReason, SealedExecutionIntent, TriggerType,
    C3G_EXPERIMENT_AUTHORIZED, STOP_EXIT_AUTHORIZED, TARGET_PATH_OPTIMIZATION_AUTHORIZED,
};
use super::observatory_live_execution::{
    is_protected_direction_only_clock, latest_universe_session,
};
use super::observatory_prospective::{generate_prospective_decision, latest_session_at_or_before};
use super::observatory_slice::{action_label, SealedDecisionRecord};
use super::policy_artifact::PolicyArtifact;
use super::DecisionAction;

// ─── P.E.3 live execution gate flags ─────────────────────────────────────────

/// P.E.3 live execution is authorized (coralys-exec-v0 is frozen and live).
pub const PE3_LIVE_EXECUTION_AUTHORIZED: bool = true;

/// P.E.3 live execution path kind — distinct from P.E.2 "prospective_execution_v0".
pub const PE3_LIVE_EXECUTION_PATH_KIND: &str = "prospective_execution_pe3_v0";

/// P.E.3 execution contract ID — distinct from P.E.2 "targeted_execution_v0_fixed_5pct_20_sessions".
pub const PE3_EXECUTION_CONTRACT_ID: &str = "coralys_exec_v0_atr_tmv_20_sessions";

/// P.E.3 execution contract label.
pub const PE3_EXECUTION_CONTRACT_LABEL: &str = "coralys-exec-v0 (ATR/TMV, 20 sessions)";

pub const PE3_STATUS_AWAITING: &str = "AWAITING_NEXT_SESSION";
pub const PE3_STATUS_OBSERVING: &str = "OBSERVING";

// ─── Output types ─────────────────────────────────────────────────────────────

/// A single P.E.3 live execution record.
/// Carries the coralys intent alongside the standard execution exit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pe3LiveRecord {
    pub instrument: String,
    pub decision: SealedDecisionRecord,
    /// The coralys-exec-v0 intent that produced the SealedExecutionIntent.
    /// None if ATR was unavailable (position excluded from P.E.3 sample).
    pub coralys_intent_hash: Option<String>,
    pub coralys_target_pct: Option<f64>,
    pub coralys_risk_pct: Option<f64>,
    pub coralys_atr_14_at_t: Option<f64>,
    pub coralys_tmv_state: Option<String>,
    /// The execution intent consumed by first_exit().
    pub intent: SealedExecutionIntent,
    pub exit: ExecutionExit,
    /// true = coralys-exec-v0 produced a valid intent.
    /// false = ATR unavailable; position excluded from P.E.3 treatment sample.
    pub pe3_eligible: bool,
    pub exclusion_reason: Option<String>,
}

/// The P.E.3 live execution ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pe3LiveLedger {
    pub path_kind: String,
    pub execution_contract: String,
    pub execution_contract_label: String,
    pub coralys_model_id: String,
    pub coralys_model_version: String,
    pub coralys_artifact_hash: String,
    pub seal_status: String,
    pub certified_t: Option<String>,
    pub peeked_returns_at_seal: bool,
    pub statistical_backtest: bool,
    pub n_decisions: usize,
    pub n_pe3_eligible: usize,
    pub n_excluded_no_atr: usize,
    pub n_observing: usize,
    pub n_target: usize,
    pub n_risk: usize,
    pub n_horizon: usize,
    pub records: Vec<Pe3LiveRecord>,
}

pub fn empty_pe3_ledger(status: &str, certified_t: Option<String>) -> Pe3LiveLedger {
    Pe3LiveLedger {
        path_kind: PE3_LIVE_EXECUTION_PATH_KIND.to_string(),
        execution_contract: PE3_EXECUTION_CONTRACT_ID.to_string(),
        execution_contract_label: PE3_EXECUTION_CONTRACT_LABEL.to_string(),
        coralys_model_id: CORALYS_EXEC_MODEL_ID.to_string(),
        coralys_model_version: CORALYS_EXEC_MODEL_VERSION.to_string(),
        coralys_artifact_hash: CORALYS_EXEC_ARTIFACT_HASH.to_string(),
        seal_status: status.to_string(),
        certified_t,
        peeked_returns_at_seal: false,
        statistical_backtest: false,
        n_decisions: 0,
        n_pe3_eligible: 0,
        n_excluded_no_atr: 0,
        n_observing: 0,
        n_target: 0,
        n_risk: 0,
        n_horizon: 0,
        records: Vec::new(),
    }
}

// ─── ATR extraction ───────────────────────────────────────────────────────────

/// Extract ATR(14) from bars ≤ T.
///
/// Uses the same `metrics_from_bars_at_t` pipeline as the rest of the system.
/// Returns None if ATR is unavailable or zero — the caller must treat this as
/// `CoralysExecutionResult::Invalid` (excluded from P.E.3 sample, not a +5% fallback).
pub fn atr_14_at_t(bars: &[YahooHistoricalBar], t: DateTime<Utc>) -> Option<f64> {
    // Use a stable instrument_id for metric computation (not stored, just for the engine).
    let instrument_id = Uuid::nil();
    let report = metrics_from_bars_at_t(bars, t, instrument_id);
    let atr = report.get_float("atr_14")?;
    if atr > 0.0 {
        Some(atr)
    } else {
        None
    }
}

// ─── Intent conversion ────────────────────────────────────────────────────────

/// Convert a `CoralysExecutionIntent` into the `SealedExecutionIntent` that
/// `first_exit()` consumes.
///
/// The coralys risk_boundary maps to `stop_price` / `stop_pct` in the sealed intent.
/// Note: `STOP_EXIT_AUTHORIZED = false` in the P.E.2 path, so `first_exit()` will
/// not act on the stop fields. They are carried for observability only.
/// P.E.3 exit tracking uses TARGET / HORIZON / OBSERVING (same as P.E.2 control).
pub fn coralys_intent_to_sealed(
    coralys: &super::coralys_execution_model::CoralysExecutionIntent,
) -> SealedExecutionIntent {
    use sha2::{Digest, Sha256};
    let identity = serde_json::json!({
        "action": coralys.direction,
        "coralys_artifact_hash": coralys.coralys_artifact_hash,
        "coralys_model_id": coralys.coralys_model_id,
        "decision_id": coralys.decision_time,
        "entry_price": format!("{:.8}", coralys.entry_price),
        "execution_contract": PE3_EXECUTION_CONTRACT_ID,
        "instrument": coralys.instrument,
        "max_holding_sessions": coralys.maximum_hold_sessions,
        "risk_pct": format!("{:.8}", coralys.risk_pct),
        "target_pct": format!("{:.8}", coralys.target_pct),
        "target_price": format!("{:.8}", coralys.target_price),
    });
    let intent_hash = format!("{:x}", Sha256::digest(identity.to_string().as_bytes()));

    // Map coralys direction string to action label used by SealedExecutionIntent.
    let action = coralys.direction.to_string(); // "LONG" / "SHORT"

    SealedExecutionIntent {
        // decision_id is used by first_exit() to match intent to decision.
        // We use the coralys intent_hash as a stable identifier here;
        // the caller must ensure decision.decision_id == intent.decision_id.
        decision_id: String::new(), // filled by caller
        instrument: coralys.instrument.clone(),
        decision_time: coralys.decision_time.clone(),
        action,
        entry_price: coralys.entry_price,
        target_pct: coralys.target_pct,
        target_price: coralys.target_price,
        // Carry risk boundary as stop for observability; first_exit() won't act on it
        // because STOP_EXIT_AUTHORIZED = false.
        stop_pct: Some(coralys.risk_pct),
        stop_price: Some(coralys.risk_boundary),
        max_holding_sessions: coralys.maximum_hold_sessions,
        target_source: format!(
            "coralys-exec-v0 atr_tmv (atr={:.4}, tmv={})",
            coralys.atr_14_at_t, coralys.tmv_state
        ),
        execution_contract: PE3_EXECUTION_CONTRACT_ID.to_string(),
        sealed_at_t: false, // P.E.3 seals at E (entry session open), not at T
        intent_hash,
    }
}

// ─── Recount ──────────────────────────────────────────────────────────────────

fn recount(ledger: &mut Pe3LiveLedger) {
    ledger.n_decisions = ledger.records.len();
    ledger.n_pe3_eligible = ledger.records.iter().filter(|r| r.pe3_eligible).count();
    ledger.n_excluded_no_atr = ledger.records.iter().filter(|r| !r.pe3_eligible).count();
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
    // Risk exits: Stop reason (coralys risk boundary hit).
    // Note: STOP_EXIT_AUTHORIZED = false in first_exit(), so this will be 0 until
    // a P.E.3-specific exit function is added. Tracked for future use.
    ledger.n_risk = ledger
        .records
        .iter()
        .filter(|r| r.exit.exit_reason == ExitReason::Stop)
        .count();
    if ledger.records.is_empty() {
        ledger.seal_status = PE3_STATUS_AWAITING.to_string();
    } else if ledger.n_observing == ledger.n_decisions {
        ledger.seal_status = PE3_STATUS_OBSERVING.to_string();
    }
}

fn exit_is_terminal(reason: ExitReason) -> bool {
    matches!(
        reason,
        ExitReason::Target
            | ExitReason::Horizon
            | ExitReason::Stop
            | ExitReason::NoTrade
            | ExitReason::Ambiguous
    )
}

// ─── Observe existing records ─────────────────────────────────────────────────

pub fn observe_pe3_records(
    ledger: &mut Pe3LiveLedger,
    cache: &BTreeMap<String, Vec<YahooHistoricalBar>>,
) -> Result<usize, String> {
    let mut updated = 0usize;
    for record in &mut ledger.records {
        if exit_is_terminal(record.exit.exit_reason) {
            continue;
        }
        let bars = cache
            .get(&record.instrument)
            .ok_or_else(|| format!("yahoo cache missing {}", record.instrument))?;
        let next = first_exit(&record.decision, &record.intent, bars)?;
        if next.exit_reason != record.exit.exit_reason
            || next.trigger_type != record.exit.trigger_type
        {
            record.exit = next;
            updated += 1;
        }
    }
    recount(ledger);
    Ok(updated)
}

// ─── Main entry point ─────────────────────────────────────────────────────────

/// Run the P.E.3 live execution path.
///
/// This is the product execution path. It replaces the fixed +5% `seal_execution_intent()`
/// with `seal_coralys_execution_intent()` from coralys-exec-v0.
///
/// Positions where ATR is unavailable are excluded from the P.E.3 treatment sample
/// (pe3_eligible = false). They are NOT replaced with the P.E.2 +5% control.
///
/// The P.E.2 live ledger (observatory_live_execution.rs) is not touched.
pub fn run_pe3_live_execution(
    artifact: &PolicyArtifact,
    cache: &BTreeMap<String, Vec<YahooHistoricalBar>>,
    now: DateTime<Utc>,
    existing: Option<Pe3LiveLedger>,
) -> Result<Pe3LiveLedger, String> {
    // Gate: refuse if any research flag is open.
    if TARGET_PATH_OPTIMIZATION_AUTHORIZED || STOP_EXIT_AUTHORIZED || C3G_EXPERIMENT_AUTHORIZED {
        return Err("P.E.3 live execution refuses to run with research gates open".into());
    }
    if !PE3_LIVE_EXECUTION_AUTHORIZED {
        return Err("PE3_LIVE_EXECUTION_AUTHORIZED is false".into());
    }
    // Gate: identity-check C3-002.
    if artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("P.E.3 live execution identity-gates C3-002".into());
    }

    let mut ledger = existing.unwrap_or_else(|| empty_pe3_ledger(PE3_STATUS_AWAITING, None));
    if ledger.path_kind != PE3_LIVE_EXECUTION_PATH_KIND {
        return Err(
            "P.E.3 live execution belongs on the prospective_execution_pe3_v0 ledger".into(),
        );
    }
    if ledger.coralys_artifact_hash != CORALYS_EXEC_ARTIFACT_HASH {
        return Err(format!(
            "P.E.3 ledger coralys_artifact_hash mismatch: expected {}, got {}",
            CORALYS_EXEC_ARTIFACT_HASH, ledger.coralys_artifact_hash
        ));
    }

    let certified_t = latest_universe_session(cache, now)?;
    ledger.certified_t = Some(certified_t.to_rfc3339());

    // Respect the 14-August direction-only clock boundary.
    if is_protected_direction_only_clock(certified_t) {
        if !ledger.records.is_empty() {
            observe_pe3_records(&mut ledger, cache)?;
            return Ok(ledger);
        }
        return Ok(empty_pe3_ledger(
            PE3_STATUS_AWAITING,
            Some(certified_t.to_rfc3339()),
        ));
    }

    // Seal new records if the ledger is empty (first run after T).
    if ledger.records.is_empty() {
        for instrument in RESEARCH_UNIVERSE {
            let bars = cache
                .get(instrument)
                .ok_or_else(|| format!("yahoo cache missing {instrument}"))?;
            let instrument_t = latest_session_at_or_before(bars, now)
                .ok_or_else(|| format!("no session ≤ now for {instrument}"))?;
            if is_protected_direction_only_clock(instrument_t) {
                return Err(format!(
                    "P.E.3 refusing to attach coralys-exec-v0 to the 14 August clock for {instrument}"
                ));
            }

            // Generate C3-002 direction decision.
            let decision = generate_prospective_decision(artifact, instrument, bars, now)?;
            let t = super::observatory_maturity::parse_decision_time(&decision.decision_time)?;
            if is_protected_direction_only_clock(t) {
                return Err(
                    "P.E.3 refusing to seal coralys-exec-v0 on the 14 August cohort".into(),
                );
            }

            // Entry price = last adj_close at or before T (next session open approximation).
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
                .ok_or_else(|| {
                    format!(
                        "no entry close at {} for {instrument}",
                        decision.decision_time
                    )
                })?;

            // ATR(14) from bars ≤ T.
            let atr = atr_14_at_t(bars, t);

            // Direction string for coralys.
            let direction_str = match decision.action {
                DecisionAction::Long => "LONG",
                DecisionAction::Short => "SHORT",
                DecisionAction::NoTrade => "NO_TRADE",
            };

            // Entry time = T (we use T as the entry time approximation;
            // the actual next session open is the true E but we don't have it yet).
            let entry_time = decision.decision_time.clone();

            let (
                pe3_eligible,
                exclusion_reason,
                coralys_intent_hash,
                coralys_target_pct,
                coralys_risk_pct,
                coralys_atr_14_at_t,
                coralys_tmv_state,
                mut sealed_intent,
            ) = if direction_str == "NO_TRADE" {
                // NO_TRADE: excluded from P.E.3 treatment (no execution intent needed).
                // Use a placeholder intent so first_exit() can return NoTrade.
                let placeholder = placeholder_no_trade_intent(&decision);
                (
                    false,
                    Some("NO_TRADE: no execution intent".to_string()),
                    None,
                    None,
                    None,
                    None,
                    None,
                    placeholder,
                )
            } else {
                match seal_coralys_execution_intent(
                    instrument,
                    &decision.decision_time,
                    &entry_time,
                    direction_str,
                    entry,
                    atr,
                    &decision.state.trend,
                    &decision.state.momentum,
                    &decision.state.state_hash,
                )? {
                    CoralysExecutionResult::Intent(ci) => {
                        let sealed = coralys_intent_to_sealed(&ci);
                        (
                            true,
                            None,
                            Some(ci.intent_hash.clone()),
                            Some(ci.target_pct),
                            Some(ci.risk_pct),
                            Some(ci.atr_14_at_t),
                            Some(ci.tmv_state.clone()),
                            sealed,
                        )
                    }
                    CoralysExecutionResult::Invalid { reason, .. } => {
                        // ATR unavailable — excluded from P.E.3 sample.
                        // Do NOT fall back to +5%. That would contaminate the P.E.2 comparison.
                        let placeholder = placeholder_no_trade_intent(&decision);
                        (
                            false,
                            Some(reason),
                            None,
                            None,
                            None,
                            None,
                            None,
                            placeholder,
                        )
                    }
                }
            };

            // Patch decision_id into the sealed intent so first_exit() can match it.
            sealed_intent.decision_id = decision.decision_id.clone();

            let exit = first_exit(&decision, &sealed_intent, bars)?;

            ledger.records.push(Pe3LiveRecord {
                instrument: instrument.to_string(),
                decision,
                coralys_intent_hash,
                coralys_target_pct,
                coralys_risk_pct,
                coralys_atr_14_at_t,
                coralys_tmv_state,
                intent: sealed_intent,
                exit,
                pe3_eligible,
                exclusion_reason,
            });
        }
        ledger.peeked_returns_at_seal = false;
    }

    observe_pe3_records(&mut ledger, cache)?;
    Ok(ledger)
}

// ─── Placeholder intent for NO_TRADE / Invalid ────────────────────────────────

/// A placeholder `SealedExecutionIntent` for NO_TRADE or ATR-invalid positions.
/// `first_exit()` will return `ExitReason::NoTrade` for NO_TRADE actions.
/// `pub` so the historical replay module can reuse the same placeholder logic.
pub fn placeholder_no_trade_intent(decision: &SealedDecisionRecord) -> SealedExecutionIntent {
    SealedExecutionIntent {
        decision_id: decision.decision_id.clone(),
        instrument: decision.instrument.clone(),
        decision_time: decision.decision_time.clone(),
        action: action_label(decision.action).to_string(),
        entry_price: 0.0,
        target_pct: 0.0,
        target_price: 0.0,
        stop_pct: None,
        stop_price: None,
        max_holding_sessions: MAXIMUM_HOLD_SESSIONS,
        target_source: "NO_CORALYS_EXECUTION".to_string(),
        execution_contract: PE3_EXECUTION_CONTRACT_ID.to_string(),
        sealed_at_t: false,
        intent_hash: String::new(),
    }
}

// ─── Report rendering ─────────────────────────────────────────────────────────

pub fn render_pe3_live_report(ledger: &Pe3LiveLedger) -> String {
    let mut md = String::new();
    md.push_str("# P.E.3 Live Execution Report — coralys-exec-v0\n\n");
    md.push_str("**Document type:** Product validation evidence  \n");
    md.push_str("**Parent:** CS-P-006-P.E.3  \n");
    md.push_str("**Execution model:** coralys-exec-v0 (ATR-anchored, TMV-scaled)  \n");
    md.push_str("**Does not:** modify C3-002, modify coralys-exec-v0 multipliers, fall back to +5%, touch P.E.2 ledger  \n\n");
    md.push_str(&format!(
        "- coralys artifact hash: `{}`\n",
        ledger.coralys_artifact_hash
    ));
    md.push_str(&format!("- path kind: `{}`\n", ledger.path_kind));
    md.push_str(&format!("- seal status: `{}`\n", ledger.seal_status));
    md.push_str(&format!(
        "- certified T: {}\n",
        ledger.certified_t.as_deref().unwrap_or("—")
    ));
    md.push_str(&format!(
        "- peeked_returns_at_seal: {}\n",
        ledger.peeked_returns_at_seal
    ));
    md.push_str(&format!(
        "- statistical strategy backtest: {}\n\n",
        if ledger.statistical_backtest {
            "DONE"
        } else {
            "not done"
        }
    ));
    md.push_str(&format!("- decisions: {}\n", ledger.n_decisions));
    md.push_str(&format!(
        "- P.E.3 eligible (ATR available): {}\n",
        ledger.n_pe3_eligible
    ));
    md.push_str(&format!(
        "- excluded (ATR unavailable): {}\n",
        ledger.n_excluded_no_atr
    ));
    md.push_str(&format!("- OBSERVING: {}\n", ledger.n_observing));
    md.push_str(&format!("- TARGET: {}\n", ledger.n_target));
    md.push_str(&format!("- HORIZON: {}\n", ledger.n_horizon));
    md.push_str(&format!("- RISK (stop): {}\n\n", ledger.n_risk));

    if ledger.seal_status == PE3_STATUS_AWAITING {
        md.push_str(
            "AWAITING_NEXT_SESSION — no eligible session after 2026-08-14T03:45:00Z yet.\n",
        );
        return md;
    }

    md.push_str("| Instrument | P.E.3 eligible | Direction | ATR(14) | TMV | Target% | Risk% | Exit | Hold | V |\n");
    md.push_str("|---|---|---|---:|---|---:|---:|---|---:|---:|\n");
    for record in &ledger.records {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            record.instrument,
            if record.pe3_eligible { "YES" } else { "NO" },
            record.intent.action,
            record
                .coralys_atr_14_at_t
                .map(|v| format!("{v:.2}"))
                .unwrap_or_else(|| "—".into()),
            record.coralys_tmv_state.as_deref().unwrap_or("—"),
            record
                .coralys_target_pct
                .map(|v| format!("{:+.1}%", v * 100.0))
                .unwrap_or_else(|| "—".into()),
            record
                .coralys_risk_pct
                .map(|v| format!("{:.1}%", v * 100.0))
                .unwrap_or_else(|| "—".into()),
            exit_label(record.exit.exit_reason),
            record
                .exit
                .holding_sessions
                .map(|n| n.to_string())
                .unwrap_or_else(|| "—".into()),
            record
                .exit
                .decision_value
                .map(pct)
                .unwrap_or_else(|| "—".into()),
        ));
    }
    md.push_str("\nTARGET and HORIZON are both evidence. Excluded positions (ATR unavailable) are not replaced with +5% — that would contaminate the P.E.2 comparison. This is not a statistical strategy backtest.\n");
    md
}

fn exit_label(reason: ExitReason) -> &'static str {
    match reason {
        ExitReason::Target => "TARGET",
        ExitReason::Stop => "RISK_HIT",
        ExitReason::Horizon => "HORIZON",
        ExitReason::Ambiguous => "AMBIGUOUS",
        ExitReason::NoTrade => "NO_TRADE",
        ExitReason::Observing => "OBSERVING",
    }
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

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decision_support::coralys_execution_model::CORALYS_EXEC_ARTIFACT_HASH;

    #[test]
    fn empty_pe3_ledger_has_correct_identity() {
        let ledger = empty_pe3_ledger(PE3_STATUS_AWAITING, None);
        assert_eq!(ledger.path_kind, PE3_LIVE_EXECUTION_PATH_KIND);
        assert_eq!(ledger.execution_contract, PE3_EXECUTION_CONTRACT_ID);
        assert_eq!(ledger.coralys_artifact_hash, CORALYS_EXEC_ARTIFACT_HASH);
        assert_eq!(ledger.coralys_model_id, CORALYS_EXEC_MODEL_ID);
        assert_eq!(ledger.seal_status, PE3_STATUS_AWAITING);
        assert!(!ledger.peeked_returns_at_seal);
        assert!(!ledger.statistical_backtest);
        assert_eq!(ledger.n_decisions, 0);
    }

    #[test]
    fn pe3_live_execution_authorized() {
        assert!(
            PE3_LIVE_EXECUTION_AUTHORIZED,
            "P.E.3 live execution must be authorized"
        );
    }

    #[test]
    fn pe3_path_kind_distinct_from_pe2() {
        // P.E.3 and P.E.2 must use different path_kind strings to prevent ledger confusion.
        use super::super::observatory_live_execution::LIVE_EXECUTION_PATH_KIND;
        assert_ne!(
            PE3_LIVE_EXECUTION_PATH_KIND, LIVE_EXECUTION_PATH_KIND,
            "P.E.3 path_kind must be distinct from P.E.2 path_kind"
        );
    }

    #[test]
    fn pe3_contract_id_distinct_from_pe2() {
        use super::super::observatory_execution::EXECUTION_CONTRACT_ID;
        assert_ne!(
            PE3_EXECUTION_CONTRACT_ID, EXECUTION_CONTRACT_ID,
            "P.E.3 contract ID must be distinct from P.E.2 contract ID"
        );
    }

    #[test]
    fn coralys_intent_to_sealed_maps_fields_correctly() {
        use super::super::coralys_execution_model::CoralysExecutionIntent;
        use super::super::coralys_execution_model::{
            CORALYS_EXEC_MODEL_ID, CORALYS_EXEC_MODEL_VERSION, ENTRY_SOURCE_NEXT_SESSION_OPEN,
        };
        let ci = CoralysExecutionIntent {
            instrument: "INFY.NS".into(),
            decision_time: "2026-08-16T03:45:00+00:00".into(),
            decision_information_cutoff: "2026-08-16T03:45:00+00:00".into(),
            entry_time: "2026-08-18T09:15:00+05:30".into(),
            execution_information_cutoff: "2026-08-18T09:15:00+05:30".into(),
            entry_source: ENTRY_SOURCE_NEXT_SESSION_OPEN.to_string(),
            direction: "LONG".into(),
            entry_price: 1500.0,
            target_pct: 0.08,
            target_price: 1620.0,
            target_basis: "test".into(),
            risk_pct: 0.04,
            risk_boundary: 1440.0,
            risk_basis: "test".into(),
            atr_14_at_t: 60.0,
            tmv_state: "Bullish / Positive".into(),
            state_hash: "abc123".into(),
            maximum_hold_sessions: 20,
            coralys_model_id: CORALYS_EXEC_MODEL_ID.into(),
            coralys_model_version: CORALYS_EXEC_MODEL_VERSION.into(),
            coralys_artifact_hash: CORALYS_EXEC_ARTIFACT_HASH.into(),
            intent_hash: "placeholder".into(),
            sealed_at_entry: true,
            direction_sealed_at_t: true,
        };
        let sealed = coralys_intent_to_sealed(&ci);
        assert_eq!(sealed.instrument, "INFY.NS");
        assert_eq!(sealed.action, "LONG");
        assert!((sealed.entry_price - 1500.0).abs() < 1e-6);
        assert!((sealed.target_pct - 0.08).abs() < 1e-9);
        assert!((sealed.target_price - 1620.0).abs() < 1e-6);
        assert_eq!(sealed.stop_pct, Some(0.04));
        assert_eq!(sealed.stop_price, Some(1440.0));
        assert_eq!(sealed.max_holding_sessions, 20);
        assert_eq!(sealed.execution_contract, PE3_EXECUTION_CONTRACT_ID);
        assert!(!sealed.sealed_at_t, "P.E.3 seals at E, not at T");
        assert!(!sealed.intent_hash.is_empty());
    }

    #[test]
    fn atr_14_at_t_returns_none_for_empty_bars() {
        let bars: Vec<crate::ingestion::yahoo::YahooHistoricalBar> = vec![];
        let t = chrono::Utc::now();
        let result = atr_14_at_t(&bars, t);
        assert!(result.is_none(), "empty bars must return None for ATR");
    }
}
