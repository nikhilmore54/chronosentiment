//! Observation-path outcomes for forward/paper (CS-P-003).
//!
//! Does not replace lake-attaching `outcome::OutcomeEngine`. Does not modify B4.
//! Measures 5/10/20/60D close-to-close returns from caller-supplied prices after T.
//! LONG and SHORT are both measurable. `now` is caller-supplied (not wall clock).

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::backtest::{DecisionLedger, LedgerRecord};
use super::outcome::{DecisionOutcomeBundle, HorizonOutcome, OutcomeReport, HORIZON_DAYS};
use super::DecisionAction;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceBar {
    pub effective_from: DateTime<Utc>,
    pub close: f64,
    #[serde(default)]
    pub instrument_id: Option<Uuid>,
}

pub fn measure_ledger_from_prices(
    ledger: &DecisionLedger,
    prices: &[PriceBar],
    now: DateTime<Utc>,
) -> OutcomeReport {
    OutcomeReport {
        bundles: ledger
            .records
            .iter()
            .map(|r| measure_record_from_prices(r, prices, now))
            .collect(),
    }
}

pub fn measure_record_from_prices(
    record: &LedgerRecord,
    prices: &[PriceBar],
    now: DateTime<Utc>,
) -> DecisionOutcomeBundle {
    let mut known: Vec<&PriceBar> = prices
        .iter()
        .filter(|p| p.effective_from <= now && p.close.is_finite() && p.close > 0.0)
        .filter(|p| p.instrument_id.is_none() || p.instrument_id == Some(record.instrument_id))
        .collect();
    known.sort_by(|a, b| {
        a.effective_from.cmp(&b.effective_from).then(
            a.close
                .partial_cmp(&b.close)
                .unwrap_or(std::cmp::Ordering::Equal),
        )
    });

    let entry = known
        .iter()
        .rev()
        .find(|p| p.effective_from <= record.as_of_timestamp)
        .map(|p| p.close);

    let mut horizons = Vec::with_capacity(HORIZON_DAYS.len());
    for days in HORIZON_DAYS {
        let expiry = record.as_of_timestamp + Duration::days(days as i64);
        let exit = known
            .iter()
            .find(|p| p.effective_from >= expiry)
            .map(|p| (p.close, p.effective_from));
        let available = entry.is_some() && exit.is_some() && expiry <= now;
        let outcome_return = match (available, entry, exit) {
            (true, Some(p0), Some((ph, _))) => Some(signed_return(record.action, p0, ph)),
            _ => None,
        };
        horizons.push(HorizonOutcome {
            horizon_days: days,
            available: outcome_return.is_some(),
            lake_outcome_id: None,
            lake_decision_id: None,
            outcome_return,
            entry_reached: outcome_return.map(|_| true),
            target_hit: None,
            stop_hit: None,
            exit_reason: outcome_return.map(|_| "HorizonClose".to_string()),
            mfe: None,
            mae: None,
            drawdown: None,
            horizon_expiry_timestamp: if outcome_return.is_some() {
                Some(expiry)
            } else {
                None
            },
        });
    }
    let content_hash = bundle_hash(record.decision_id, &horizons);
    DecisionOutcomeBundle {
        ledger_decision_id: record.decision_id,
        instrument_id: record.instrument_id,
        as_of_timestamp: record.as_of_timestamp,
        action: record.action,
        horizons,
        content_hash,
    }
}

fn signed_return(action: DecisionAction, entry: f64, exit: f64) -> f64 {
    let instrument = (exit - entry) / entry;
    match action {
        DecisionAction::Long => instrument,
        DecisionAction::Short => -instrument,
        DecisionAction::NoTrade => instrument,
    }
}

fn bundle_hash(decision_id: Uuid, horizons: &[HorizonOutcome]) -> String {
    #[derive(Serialize)]
    struct Payload<'a> {
        decision_id: Uuid,
        source: &'static str,
        horizons: &'a [HorizonOutcome],
    }
    let bytes = serde_json::to_vec(&Payload {
        decision_id,
        source: "csp003.price_path",
        horizons,
    })
    .expect("observation outcome serializes");
    format!("{:x}", Sha256::digest(&bytes))
}
