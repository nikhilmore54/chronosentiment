//! CS-P-006-P.E.3.H Historical replay under coralys-exec-v0.
//!
//! Executes the frozen coralys-exec-v0 artifact against every eligible historical
//! timestamp T in the RESEARCH_UNIVERSE. Produces a separate P.E.3 historical ledger.
//!
//! ## Architecture
//!
//! ```text
//! Historical bars ≤ T
//!        ↓
//! generate_historical_replay_decision()   ← same as P.E.2
//!        ↓
//! seal_coralys_execution_intent()         ← same 9-arg call as live P.E.3
//!        ↓
//! coralys_intent_to_sealed()              ← same conversion as live P.E.3
//!        ↓
//! first_exit()                            ← same exit logic as P.E.2 / live P.E.3
//!        ↓
//! HistoricalPe3Ledger                     ← separate from P.E.2 ledger
//! ```
//!
//! ## What this module does NOT do
//!
//! - Does NOT modify C3-002 direction.
//! - Does NOT modify coralys-exec-v0 multipliers (frozen artifact 3876ffa2...).
//! - Does NOT fall back to +5% when ATR=0 (Invalid positions excluded from P.E.3 sample).
//! - Does NOT touch the P.E.2 historical ledger.
//! - Does NOT touch the P.E.2 live execution path.
//! - Does NOT start C.3-G, Search #3, or stop-exit research.
//!
//! ## Immutability
//!
//! The coralys artifact hash is frozen:
//!   3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f
//!
//! This module will refuse to run if the artifact hash does not match.

use std::collections::BTreeMap;

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::ingestion::yahoo::YahooHistoricalBar;

use super::coralys_execution_model::{
    seal_coralys_execution_intent, CoralysExecutionResult, CORALYS_EXEC_ARTIFACT_HASH,
    CORALYS_EXEC_MODEL_ID, CORALYS_EXEC_MODEL_VERSION, MAXIMUM_HOLD_SESSIONS,
};
use super::csp006_protocol::{RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_UNIVERSE};
use super::observatory_execution::{
    first_exit, ExecutionExit, ExitReason, SealedExecutionIntent,
    C3G_EXPERIMENT_AUTHORIZED, TARGET_PATH_OPTIMIZATION_AUTHORIZED,
};
use super::observatory_historical::{
    decision_time_bars, generate_historical_replay_decision, poison_future_bars,
};
use super::observatory_live_execution_pe3::{
    atr_14_at_t, coralys_intent_to_sealed, placeholder_no_trade_intent,
    PE3_EXECUTION_CONTRACT_ID,
};
use super::observatory_maturity::nth_market_session_after;
use super::observatory_prospective::latest_session_at_or_before;
use super::observatory_slice::{action_label, SealedDecisionRecord, OBSERVATORY_HORIZON_DAYS};
use super::policy_artifact::PolicyArtifact;
use super::DecisionAction;

// ─── Constants ────────────────────────────────────────────────────────────────

pub const HISTORICAL_PE3_PATH_KIND: &str = "historical_pe3_replay";
pub const HISTORICAL_PE3_EXECUTION_CONTRACT: &str = "coralys_exec_v0_atr_tmv_20_sessions";
pub const HISTORICAL_PE3_EXECUTION_CONTRACT_LABEL: &str =
    "coralys-exec-v0 (ATR/TMV, 20 sessions) — P.E.3 Historical";

/// Same requested clock as P.E.2 — same historical timestamp, different execution contract.
pub const HISTORICAL_PE3_REQUESTED_CLOCK: &str = "2026-07-15T03:45:00+00:00";

pub const REQUIRED_SUBSEQUENT_SESSIONS: u32 = 20;

pub fn historical_pe3_requested_clock() -> Result<DateTime<Utc>, String> {
    DateTime::parse_from_rfc3339(HISTORICAL_PE3_REQUESTED_CLOCK)
        .map(|t| t.with_timezone(&Utc))
        .map_err(|e| format!("historical P.E.3 clock is not RFC3339: {e}"))
}

// ─── Output guard ─────────────────────────────────────────────────────────────

/// Refuse to write to any path that belongs to P.E.2 or other protected artifacts.
pub fn refuse_historical_pe3_output(path: &str) -> Result<(), String> {
    for forbidden in [
        "observatory/prospective",
        "historical_replay_v0",
        "historical_replay_v1",
        "targeted_execution_v0",
        "prospective_execution_v0",
        "prospective_execution_pe3_v0",
        "historical_pe2_replay",
        "selected_policy.json",
    ] {
        if path.contains(forbidden) {
            return Err(format!(
                "historical P.E.3 refuses to write to protected path: {forbidden}"
            ));
        }
    }
    Ok(())
}

// ─── Record types ─────────────────────────────────────────────────────────────

/// A single P.E.3 historical execution record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPe3Record {
    pub instrument: String,
    pub requested_clock: String,
    pub certified_t: String,
    pub decision: SealedDecisionRecord,

    // Coralys execution parameters
    pub coralys_model_id: String,
    pub coralys_model_version: String,
    pub coralys_artifact_hash: String,
    pub coralys_intent_hash: Option<String>,
    pub atr_14_at_t: Option<f64>,
    pub coralys_target_pct: Option<f64>,
    pub coralys_risk_pct: Option<f64>,
    pub coralys_tmv_state: Option<String>,

    pub intent: SealedExecutionIntent,
    pub exit: ExecutionExit,

    /// true = coralys-exec-v0 produced a valid intent.
    /// false = NO_TRADE or ATR unavailable; excluded from P.E.3 treatment sample.
    pub pe3_eligible: bool,
    pub exclusion_reason: Option<String>,

    // Integrity checks
    pub determinism_pass: bool,
    pub lookahead_clean: bool,
    pub poison_test_pass: bool,

    // Feedback ledger fields (retrospective characterization)
    pub learning_scope: String,
    pub retrospective_characterization: bool,
}

/// The P.E.3 historical execution ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPe3Ledger {
    pub path_kind: String,
    pub execution_contract: String,
    pub execution_contract_label: String,
    pub coralys_model_id: String,
    pub coralys_model_version: String,
    pub coralys_artifact_hash: String,
    pub requested_clock: String,
    pub certified_t: String,
    pub max_holding_sessions: u32,
    pub n_decisions: usize,
    pub n_pe3_eligible: usize,
    pub n_excluded_no_atr: usize,
    pub n_target: usize,
    pub n_risk: usize,
    pub n_horizon: usize,
    pub n_no_trade: usize,
    pub n_ambiguous: usize,
    pub determinism_pass: bool,
    pub lookahead_clean: bool,
    pub poison_test_pass: bool,
    pub peeked_returns_at_seal: bool,
    pub statistical_backtest: bool,
    pub retrospective_characterization: bool,
    pub lifecycle_validation: String,
    pub records: Vec<HistoricalPe3Record>,
}

// ─── Replay ───────────────────────────────────────────────────────────────────

/// Run the full P.E.3 historical replay.
///
/// Uses the same historical market data and C3-002 decisions as P.E.2,
/// but applies coralys-exec-v0 (ATR/TMV) instead of the fixed +5% contract.
///
/// The P.E.2 ledger is not touched. The output is a separate P.E.3 ledger.
///
/// The calling pattern for `seal_coralys_execution_intent` is identical to
/// `run_pe3_live_execution()` in `observatory_live_execution_pe3.rs`.
/// The only difference is the source of the decision (historical vs prospective).
pub fn replay_historical_pe3(
    artifact: &PolicyArtifact,
    cache: &BTreeMap<String, Vec<YahooHistoricalBar>>,
) -> Result<HistoricalPe3Ledger, String> {
    // ── Gate flags ────────────────────────────────────────────────────────────
    if TARGET_PATH_OPTIMIZATION_AUTHORIZED || C3G_EXPERIMENT_AUTHORIZED {
        return Err("refusing a historical P.E.3 run that opens research".into());
    }

    // ── Identity gates ────────────────────────────────────────────────────────
    if artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("historical P.E.3 identity-gates C3-002".into());
    }
    if CORALYS_EXEC_ARTIFACT_HASH
        != "3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f"
    {
        return Err(format!(
            "historical P.E.3 coralys artifact hash mismatch: got {CORALYS_EXEC_ARTIFACT_HASH}"
        ));
    }

    let requested = historical_pe3_requested_clock()?;
    let mut certified: Option<DateTime<Utc>> = None;
    let mut records = Vec::new();
    let mut determinism_pass = true;
    let mut lookahead_clean = true;
    let mut poison_test_pass = true;
    let mut n_pe3_eligible = 0usize;
    let mut n_excluded_no_atr = 0usize;
    let mut n_target = 0usize;
    let mut n_risk = 0usize;
    let mut n_horizon = 0usize;
    let mut n_no_trade = 0usize;
    let mut n_ambiguous = 0usize;

    for instrument in RESEARCH_UNIVERSE {
        let bars = cache
            .get(instrument)
            .ok_or_else(|| format!("yahoo cache missing {instrument}"))?;

        // ── Certified T ───────────────────────────────────────────────────────
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

        // Enforce cohort T consistency across universe.
        match certified {
            None => certified = Some(t),
            Some(cur) if cur != t => {
                return Err(format!(
                    "{instrument} certified T {t} differs from cohort certified T {cur}"
                ));
            }
            Some(_) => {}
        }

        // ── C3-002 decision (determinism + lookahead checks) ──────────────────
        let known = decision_time_bars(bars, t);
        let decision = generate_historical_replay_decision(artifact, instrument, bars, t)?;
        let again = generate_historical_replay_decision(artifact, instrument, bars, t)?;
        let from_known = generate_historical_replay_decision(artifact, instrument, &known, t)?;
        let poisoned = poison_future_bars(bars, t);
        let from_poisoned =
            generate_historical_replay_decision(artifact, instrument, &poisoned, t)?;

        let tick_det = decision == again;
        let tick_lookahead = decision == from_known;
        if !tick_det {
            determinism_pass = false;
        }
        if !tick_lookahead {
            lookahead_clean = false;
        }

        // ── Direction string ──────────────────────────────────────────────────
        let direction_str = match decision.action {
            DecisionAction::Long => "LONG",
            DecisionAction::Short => "SHORT",
            DecisionAction::NoTrade => "NO_TRADE",
        };

        // ── Entry price = last adj_close at or before T ───────────────────────
        let entry_opt = known
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
            .map(|(_, c)| c);

        // ── ATR(14) at T ──────────────────────────────────────────────────────
        let atr = atr_14_at_t(&known, t);

        // ── Entry time = T (same approximation as live path) ─────────────────
        let entry_time = decision.decision_time.clone();

        // ── coralys-exec-v0 intent — identical call pattern to live module ────
        let (pe3_eligible, exclusion_reason, coralys_intent_hash,
             coralys_target_pct, coralys_risk_pct, coralys_atr_14_at_t,
             coralys_tmv_state, mut sealed_intent) =
            if direction_str == "NO_TRADE" {
                n_no_trade += 1;
                let placeholder = placeholder_no_trade_intent(&decision);
                (false, Some("NO_TRADE: no execution intent".to_string()),
                 None, None, None, None, None, placeholder)
            } else {
                let entry = entry_opt.ok_or_else(|| {
                    format!("no entry close at T for {instrument}")
                })?;
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
                        n_pe3_eligible += 1;
                        (true, None,
                         Some(ci.intent_hash.clone()),
                         Some(ci.target_pct),
                         Some(ci.risk_pct),
                         Some(ci.atr_14_at_t),
                         Some(ci.tmv_state.clone()),
                         sealed)
                    }
                    CoralysExecutionResult::Invalid { reason, .. } => {
                        // ATR unavailable — excluded from P.E.3 sample.
                        // Do NOT fall back to +5%. That would contaminate the P.E.2 comparison.
                        n_excluded_no_atr += 1;
                        let placeholder = placeholder_no_trade_intent(&decision);
                        (false, Some(reason), None, None, None, None, None, placeholder)
                    }
                }
            };

        // Patch decision_id into the sealed intent so first_exit() can match it.
        sealed_intent.decision_id = decision.decision_id.clone();

        // ── Poison test (only for eligible positions) ─────────────────────────
        let tick_poison = if pe3_eligible {
            let poison_known = decision_time_bars(&poisoned, t);
            let poison_atr = atr_14_at_t(&poison_known, t);
            from_poisoned == decision && poison_atr == atr
        } else {
            true // not applicable for excluded positions
        };
        if !tick_poison {
            poison_test_pass = false;
        }

        // ── Exit replay ───────────────────────────────────────────────────────
        let exit = first_exit(&decision, &sealed_intent, bars)?;

        // Tally exit reasons.
        match exit.exit_reason {
            ExitReason::Target => n_target += 1,
            ExitReason::Stop => n_risk += 1,
            ExitReason::Horizon => n_horizon += 1,
            ExitReason::Ambiguous => n_ambiguous += 1,
            ExitReason::NoTrade | ExitReason::Observing => {}
        }

        // ── Contamination guard ───────────────────────────────────────────────
        if pe3_eligible
            && sealed_intent.execution_contract
                == "targeted_execution_v0_fixed_5pct_20_sessions"
        {
            return Err(format!(
                "{instrument} P.E.3 intent carries P.E.2 execution_contract — contamination detected"
            ));
        }

        records.push(HistoricalPe3Record {
            instrument: instrument.to_string(),
            requested_clock: HISTORICAL_PE3_REQUESTED_CLOCK.to_string(),
            certified_t: t.to_rfc3339(),
            decision,
            coralys_model_id: CORALYS_EXEC_MODEL_ID.to_string(),
            coralys_model_version: CORALYS_EXEC_MODEL_VERSION.to_string(),
            coralys_artifact_hash: CORALYS_EXEC_ARTIFACT_HASH.to_string(),
            coralys_intent_hash,
            atr_14_at_t: coralys_atr_14_at_t,
            coralys_target_pct,
            coralys_risk_pct,
            coralys_tmv_state,
            intent: sealed_intent,
            exit,
            pe3_eligible,
            exclusion_reason,
            determinism_pass: tick_det,
            lookahead_clean: tick_lookahead,
            poison_test_pass: tick_poison,
            learning_scope: "ExecutionOnly".to_string(),
            retrospective_characterization: true,
        });
    }

    let certified_t = certified
        .ok_or("no certified T resolved")?
        .to_rfc3339();

    let n_decisions = records.len();

    Ok(HistoricalPe3Ledger {
        path_kind: HISTORICAL_PE3_PATH_KIND.to_string(),
        execution_contract: HISTORICAL_PE3_EXECUTION_CONTRACT.to_string(),
        execution_contract_label: HISTORICAL_PE3_EXECUTION_CONTRACT_LABEL.to_string(),
        coralys_model_id: CORALYS_EXEC_MODEL_ID.to_string(),
        coralys_model_version: CORALYS_EXEC_MODEL_VERSION.to_string(),
        coralys_artifact_hash: CORALYS_EXEC_ARTIFACT_HASH.to_string(),
        requested_clock: HISTORICAL_PE3_REQUESTED_CLOCK.to_string(),
        certified_t,
        max_holding_sessions: MAXIMUM_HOLD_SESSIONS,
        n_decisions,
        n_pe3_eligible,
        n_excluded_no_atr,
        n_target,
        n_risk,
        n_horizon,
        n_no_trade,
        n_ambiguous,
        determinism_pass,
        lookahead_clean,
        poison_test_pass,
        peeked_returns_at_seal: false,
        statistical_backtest: false,
        retrospective_characterization: true,
        lifecycle_validation: format!(
            "P.E.3 historical replay: {n_decisions} decisions, {n_pe3_eligible} eligible, \
             {n_excluded_no_atr} excluded (no ATR), {n_no_trade} NO_TRADE. \
             Artifact: {CORALYS_EXEC_ARTIFACT_HASH}. \
             determinism={determinism_pass} lookahead={lookahead_clean} poison={poison_test_pass}"
        ),
        records,
    })
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pe3_path_kind_is_distinct_from_pe2() {
        assert_ne!(HISTORICAL_PE3_PATH_KIND, "historical_pe2_replay");
        assert_ne!(HISTORICAL_PE3_PATH_KIND, "prospective_execution_v0");
        assert_ne!(HISTORICAL_PE3_PATH_KIND, "prospective_execution_pe3_v0");
        assert_eq!(HISTORICAL_PE3_PATH_KIND, "historical_pe3_replay");
    }

    #[test]
    fn pe3_execution_contract_is_distinct_from_pe2() {
        assert_ne!(
            HISTORICAL_PE3_EXECUTION_CONTRACT,
            "targeted_execution_v0_fixed_5pct_20_sessions"
        );
        assert_eq!(
            HISTORICAL_PE3_EXECUTION_CONTRACT,
            "coralys_exec_v0_atr_tmv_20_sessions"
        );
    }

    #[test]
    fn pe3_artifact_hash_is_frozen() {
        assert_eq!(
            CORALYS_EXEC_ARTIFACT_HASH,
            "3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f"
        );
    }

    #[test]
    fn pe3_requested_clock_parses() {
        assert!(historical_pe3_requested_clock().is_ok());
    }

    #[test]
    fn refuse_pe3_output_blocks_pe2_paths() {
        assert!(refuse_historical_pe3_output("historical_pe2_replay/ledger.json").is_err());
        assert!(refuse_historical_pe3_output("prospective_execution_v0/ledger.json").is_err());
        assert!(refuse_historical_pe3_output("historical_replay_v0/ledger.json").is_err());
        assert!(refuse_historical_pe3_output("selected_policy.json").is_err());
    }

    #[test]
    fn refuse_pe3_output_allows_pe3_paths() {
        assert!(refuse_historical_pe3_output("historical_pe3_replay/ledger.json").is_ok());
        assert!(refuse_historical_pe3_output(
            "historical_runs/pe3_coralys_v0_2026-08-16/execution_ledger/pe3_historical_ledger.json"
        )
        .is_ok());
    }
}