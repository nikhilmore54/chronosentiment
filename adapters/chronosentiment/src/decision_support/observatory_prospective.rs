//! CS-P-006-P prospective C3-002 paper clock.
//!
//! Same sealed policy, certified TMV at latest session ≤ now, then seal.
//! Outcomes are not known at T and are not attached. Not CS-P-003 validation.
//! Does not evolve. Does not start C.3-G.

use chrono::{DateTime, TimeZone, Utc};

use crate::ingestion::yahoo::YahooHistoricalBar;
use crate::reasoning::assessment::AssessmentProfile;

use super::csp006_protocol::RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH;
use super::enrichment_certify::assess_from_bars_at_t;
use super::forward_tick::instrument_id_for;
use super::observatory_slice::{
    empty_ledger, generate_decision, ObservatoryLedger, SealedDecisionRecord,
};
use super::policy_artifact::PolicyArtifact;
use super::recommendation_outcome::tmv_labels;

pub const PROSPECTIVE_PATH_KIND: &str = "prospective_paper_clock";
pub const PROSPECTIVE_NOT_CSP003_VALIDATION: bool = true;

pub fn empty_prospective_ledger() -> ObservatoryLedger {
    let mut ledger = empty_ledger();
    ledger.path_kind = PROSPECTIVE_PATH_KIND.to_string();
    ledger
}

pub fn latest_session_at_or_before(
    bars: &[YahooHistoricalBar],
    now: DateTime<Utc>,
) -> Option<DateTime<Utc>> {
    bars.iter()
        .filter_map(|b| Utc.timestamp_opt(b.timestamp, 0).single())
        .filter(|t| *t <= now)
        .max()
}

pub fn certified_tmv_from_profile(profile: &AssessmentProfile) -> (String, String, String) {
    tmv_labels(profile)
}

pub fn generate_prospective_decision(
    artifact: &PolicyArtifact,
    instrument: &str,
    bars: &[YahooHistoricalBar],
    now: DateTime<Utc>,
) -> Result<SealedDecisionRecord, String> {
    if artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("prospective clock identity-gates C3-002 to Search #2".into());
    }
    let t = latest_session_at_or_before(bars, now)
        .ok_or_else(|| format!("no session ≤ now for {instrument}"))?;
    let instrument_id = instrument_id_for(instrument);
    let (profile, _n, max_from) = assess_from_bars_at_t(bars, t, instrument_id);
    if let Some(max_from) = max_from {
        if max_from > t {
            return Err(format!(
                "{instrument} reconstruction included a bar after T ({max_from} > {t})"
            ));
        }
    }
    let (trend, momentum, volatility) = certified_tmv_from_profile(&profile);
    let decision = generate_decision(
        artifact,
        instrument,
        &t.to_rfc3339(),
        &trend,
        &momentum,
        &volatility,
    )?;
    let json = serde_json::to_string(&decision).map_err(|e| e.to_string())?;
    for forbidden in [
        "future_return",
        "\"outcome\"",
        "regret",
        "evaluation_score",
        "confidence",
        "realized_return",
    ] {
        if json.contains(forbidden) {
            return Err(format!("{forbidden} leaked onto the prospective decision"));
        }
    }
    Ok(decision)
}

pub fn seal_prospective(
    ledger: &mut ObservatoryLedger,
    decision: SealedDecisionRecord,
) -> Result<bool, String> {
    if ledger.path_kind != PROSPECTIVE_PATH_KIND {
        return Err("prospective decisions belong on the prospective ledger".into());
    }
    if !ledger.observations.is_empty() {
        return Err("prospective seal refuses a ledger that already has observations".into());
    }
    if ledger
        .decisions
        .iter()
        .any(|d| d.decision_id == decision.decision_id)
    {
        return Ok(false);
    }
    super::observatory_slice::seal_into_ledger(ledger, decision)?;
    Ok(true)
}
