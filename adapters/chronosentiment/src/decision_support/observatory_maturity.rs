//! CS-P-006-P Observatory maturity path.
//!
//! OBSERVING → countdown → OUTCOME DUE → append observation → decision value.
//! Horizon is 20 market sessions after the decision session.
//! Does not peek at returns before the window closes. Does not retune C3-002.
//! Does not expand the universe. Does not start C.3-G.

use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc, Weekday};

use crate::ingestion::yahoo::YahooHistoricalBar;

use super::csp006_protocol::RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH;
use super::observatory_slice::{
    append_observation, observation_status_of, ObservatoryLedger, OutcomeObservation,
    SealedDecisionRecord, UI_STATUS_OBSERVED, UI_STATUS_OBSERVING, UI_STATUS_SEALED,
};

pub const OBSERVATORY_MATURITY_STARTED: bool = true;
pub const INTERMEDIATE_INTERPRETATION_AUTHORIZED: bool = false;
pub const POLICY_RETUNE_FROM_PROSPECTIVE_AUTHORIZED: bool = false;
pub const UNIVERSE_EXPANSION_AUTHORIZED: bool = false;
pub const UI_STATUS_OUTCOME_DUE: &str = "OUTCOME DUE";
pub const HORIZON_UNIT: &str = "MARKET_SESSIONS";
pub const HORIZON_CALENDAR_BASIS: &str = "TRADING_DAYS";
pub const TRADING_SESSION_HORIZON_AUTHORIZED: bool = true;
pub const HISTORICAL_REPLAY_V0_CONTRACT: &str = "historical_replay_v0_20_calendar_days";
pub const HISTORICAL_REPLAY_V1_CONTRACT: &str = "historical_replay_v1_20_market_sessions";
pub const SESSION_RESOLUTION_RULE: &str = "latest_certified_session_at_or_before_requested_clock";

pub fn horizon_label(sessions: u32) -> String {
    if sessions == 1 {
        "1 market session".into()
    } else {
        format!("{sessions} market sessions")
    }
}

pub fn parse_decision_time(iso: &str) -> Result<DateTime<Utc>, String> {
    DateTime::parse_from_rfc3339(iso)
        .map(|t| t.with_timezone(&Utc))
        .map_err(|e| format!("decision_time is not RFC3339: {e}"))
}

fn bar_time(bar: &YahooHistoricalBar) -> Option<DateTime<Utc>> {
    Utc.timestamp_opt(bar.timestamp, 0).single()
}

/// Sessions strictly after T, in chronological order. Session 0 is T itself.
pub fn market_sessions_after(bars: &[YahooHistoricalBar], t: DateTime<Utc>) -> Vec<DateTime<Utc>> {
    let mut sessions: Vec<DateTime<Utc>> = bars
        .iter()
        .filter_map(bar_time)
        .filter(|ts| *ts > t)
        .collect();
    sessions.sort();
    sessions.dedup();
    sessions
}

pub fn nth_market_session_after(
    bars: &[YahooHistoricalBar],
    t: DateTime<Utc>,
    n: u32,
) -> Option<DateTime<Utc>> {
    if n == 0 {
        return Some(t);
    }
    market_sessions_after(bars, t)
        .into_iter()
        .nth((n as usize).saturating_sub(1))
}

/// Weekday projection when the exchange series does not yet contain 20 sessions.
/// Holidays may extend the true close; the window does not close on this date alone.
pub fn projected_nth_weekday_after(t: DateTime<Utc>, n: u32) -> DateTime<Utc> {
    let mut cursor = t;
    let mut counted = 0u32;
    while counted < n {
        cursor += Duration::days(1);
        if !matches!(cursor.weekday(), Weekday::Sat | Weekday::Sun) {
            counted += 1;
        }
    }
    cursor
}

pub fn observation_due_at(decision: &SealedDecisionRecord) -> Result<DateTime<Utc>, String> {
    observation_due_at_with_bars(decision, None)
}

pub fn observation_due_at_with_bars(
    decision: &SealedDecisionRecord,
    bars: Option<&[YahooHistoricalBar]>,
) -> Result<DateTime<Utc>, String> {
    let t = parse_decision_time(&decision.decision_time)?;
    if let Some(bars) = bars {
        if let Some(due) = nth_market_session_after(bars, t, decision.horizon_days) {
            return Ok(due);
        }
    }
    Ok(projected_nth_weekday_after(t, decision.horizon_days))
}

pub fn sessions_remaining(
    decision: &SealedDecisionRecord,
    now: DateTime<Utc>,
    bars: Option<&[YahooHistoricalBar]>,
) -> Result<i64, String> {
    let t = parse_decision_time(&decision.decision_time)?;
    let elapsed = if let Some(bars) = bars {
        market_sessions_after(bars, t)
            .into_iter()
            .filter(|ts| *ts <= now)
            .count() as i64
    } else {
        weekday_sessions_elapsed(t, now)
    };
    Ok(decision.horizon_days as i64 - elapsed)
}

pub fn days_remaining(decision: &SealedDecisionRecord, now: DateTime<Utc>) -> Result<i64, String> {
    sessions_remaining(decision, now, None)
}

fn weekday_sessions_elapsed(t: DateTime<Utc>, now: DateTime<Utc>) -> i64 {
    if now <= t {
        return 0;
    }
    let mut cursor = t;
    let mut counted = 0i64;
    while cursor < now {
        cursor += Duration::days(1);
        if !matches!(cursor.weekday(), Weekday::Sat | Weekday::Sun) && cursor <= now {
            counted += 1;
        }
    }
    counted
}

pub fn observation_window_closed(
    decision: &SealedDecisionRecord,
    now: DateTime<Utc>,
) -> Result<bool, String> {
    observation_window_closed_with_bars(decision, now, None)
}

pub fn observation_window_closed_with_bars(
    decision: &SealedDecisionRecord,
    now: DateTime<Utc>,
    bars: Option<&[YahooHistoricalBar]>,
) -> Result<bool, String> {
    let t = parse_decision_time(&decision.decision_time)?;
    if let Some(bars) = bars {
        return match nth_market_session_after(bars, t, decision.horizon_days) {
            Some(due) => Ok(now >= due),
            None => Ok(false),
        };
    }
    Ok(now >= projected_nth_weekday_after(t, decision.horizon_days))
}

pub fn require_window_closed(
    decision: &SealedDecisionRecord,
    now: DateTime<Utc>,
) -> Result<(), String> {
    require_window_closed_with_bars(decision, now, None)
}

pub fn require_window_closed_with_bars(
    decision: &SealedDecisionRecord,
    now: DateTime<Utc>,
    bars: Option<&[YahooHistoricalBar]>,
) -> Result<(), String> {
    if observation_window_closed_with_bars(decision, now, bars)? {
        Ok(())
    } else {
        let due = observation_due_at_with_bars(decision, bars)?;
        Err(format!(
            "observation window has not closed for {} (due {due})",
            decision.instrument
        ))
    }
}

pub fn ui_lifecycle_status(
    ledger: &ObservatoryLedger,
    decision_id: &str,
    now: DateTime<Utc>,
) -> &'static str {
    ui_lifecycle_status_with_bars(ledger, decision_id, now, None)
}

pub fn ui_lifecycle_status_with_bars(
    ledger: &ObservatoryLedger,
    decision_id: &str,
    now: DateTime<Utc>,
    bars: Option<&[YahooHistoricalBar]>,
) -> &'static str {
    if observation_status_of(ledger, decision_id).is_some() {
        return UI_STATUS_OBSERVED;
    }
    let Some(decision) = ledger
        .decisions
        .iter()
        .find(|d| d.decision_id == decision_id)
    else {
        return UI_STATUS_SEALED;
    };
    match observation_window_closed_with_bars(decision, now, bars) {
        Ok(true) => UI_STATUS_OUTCOME_DUE,
        _ => UI_STATUS_OBSERVING,
    }
}

pub fn format_close_time(due: DateTime<Utc>) -> String {
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let mon = months.get(due.month0() as usize).copied().unwrap_or("???");
    format!(
        "{} {} {}, {:02}:{:02} UTC",
        due.day(),
        mon,
        due.year(),
        due.hour(),
        due.minute()
    )
}

pub fn format_observation_close(decision: &SealedDecisionRecord) -> String {
    format_observation_close_with_bars(decision, None)
}

pub fn format_observation_close_with_bars(
    decision: &SealedDecisionRecord,
    bars: Option<&[YahooHistoricalBar]>,
) -> String {
    match observation_due_at_with_bars(decision, bars) {
        Ok(due) => {
            let known = bars
                .and_then(|b| {
                    parse_decision_time(&decision.decision_time)
                        .ok()
                        .and_then(|t| nth_market_session_after(b, t, decision.horizon_days))
                })
                .is_some();
            if known {
                format!("Observation closes {}", format_close_time(due))
            } else {
                format!(
                    "Observation closes after {} (projected {})",
                    horizon_label(decision.horizon_days),
                    format_close_time(due)
                )
            }
        }
        Err(_) => "Observation close time unavailable".into(),
    }
}

pub fn maturity_line(decision: &SealedDecisionRecord, now: DateTime<Utc>) -> String {
    maturity_line_with_bars(decision, now, None)
}

pub fn maturity_line_with_bars(
    decision: &SealedDecisionRecord,
    now: DateTime<Utc>,
    bars: Option<&[YahooHistoricalBar]>,
) -> String {
    let close = format_observation_close_with_bars(decision, bars);
    match sessions_remaining(decision, now, bars) {
        Ok(n) if n > 1 => format!("OBSERVING · {n} market sessions remaining · {close}"),
        Ok(1) => format!("OBSERVING · 1 market session remaining · {close}"),
        Ok(0) => format!("OUTCOME DUE · {close}"),
        Ok(_) => format!("OUTCOME DUE · {close}"),
        Err(_) => "OBSERVING".into(),
    }
}

pub fn append_matured_observation(
    ledger: &mut ObservatoryLedger,
    decision: &SealedDecisionRecord,
    observation: OutcomeObservation,
    now: DateTime<Utc>,
) -> Result<(), String> {
    append_matured_observation_with_bars(ledger, decision, observation, now, None)
}

pub fn append_matured_observation_with_bars(
    ledger: &mut ObservatoryLedger,
    decision: &SealedDecisionRecord,
    observation: OutcomeObservation,
    now: DateTime<Utc>,
    bars: Option<&[YahooHistoricalBar]>,
) -> Result<(), String> {
    if decision.policy_artifact_sha256 != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("maturity path identity-gates C3-002; the policy is not rewritten".into());
    }
    require_window_closed_with_bars(decision, now, bars)?;
    append_observation(ledger, observation)
}
