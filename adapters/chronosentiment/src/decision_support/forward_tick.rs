//! Daily forward tick: latest market session ≤ now → one `TradingDecision` per instrument.
//!
//! Not a B4 replay. Does not iterate historical dates. No brokerage.

use chrono::{DateTime, Utc};
use coralys_moga::runtime::optimization::metric::{MetricReport, MetricValue};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::reasoning::assessment::{AssessmentEngine, ENRICHMENT_CONCEPTS};

use super::forward::{decide_forward, FORWARD_PRODUCER};
use super::observation_outcome::PriceBar;
use super::policy::DecisionPolicy;
use super::replay::{ReplayAssessment, ReplayInputs, ReplayObservation, UNFROZEN_ENGINE_VERSION};
use super::TradingDecision;

/// Yahoo NSE tickers. `IDEA.NS` is Vodafone Idea.
pub const DEFAULT_TICKERS: [&str; 6] = [
    "RELIANCE.NS",
    "TCS.NS",
    "INFY.NS",
    "HDFCBANK.NS",
    "ICICIBANK.NS",
    "IDEA.NS",
];

#[derive(Debug, Clone, PartialEq)]
pub struct DailyBar {
    pub timestamp: DateTime<Utc>,
    pub close: f64,
}

pub fn instrument_id_for(ticker: &str) -> Uuid {
    stable_uuid(&format!("csp003.instrument.{ticker}"))
}

pub fn latest_as_of(bars: &[DailyBar], now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    bars.iter()
        .filter(|b| b.timestamp <= now && b.close.is_finite() && b.close > 0.0)
        .map(|b| b.timestamp)
        .max()
}

/// Decide only at the latest session ≤ now. Lookback bars are for MA20/MA50, not extra decisions.
pub fn decide_latest_session<P: DecisionPolicy + ?Sized>(
    ticker: &str,
    bars: &[DailyBar],
    now: DateTime<Utc>,
    policy: &P,
) -> Result<TradingDecision, super::replay::ReplayError> {
    let t = latest_as_of(bars, now).ok_or(super::replay::ReplayError::NoAssessmentAtT)?;
    let instrument_id = instrument_id_for(ticker);
    let mut known: Vec<&DailyBar> = bars
        .iter()
        .filter(|b| b.timestamp <= t && b.close.is_finite() && b.close > 0.0)
        .collect();
    known.sort_by_key(|b| b.timestamp);
    let closes: Vec<f64> = known.iter().map(|b| b.close).collect();

    let mut metrics = MetricReport::default();
    if let Some(ma) = sma(&closes, 20) {
        metrics
            .metrics
            .insert("ma_20".to_string(), MetricValue::Float(ma));
    }
    if let Some(ma) = sma(&closes, 50) {
        metrics
            .metrics
            .insert("ma_50".to_string(), MetricValue::Float(ma));
    }
    if let Some(roc) = roc(&closes, 20) {
        metrics
            .metrics
            .insert("roc_20".to_string(), MetricValue::Float(roc));
    }

    let mut profile =
        AssessmentEngine.assess_at(&metrics, &ENRICHMENT_CONCEPTS, t, Some(instrument_id));
    let assessment_id = stable_uuid(&format!("csp003.assessment.{instrument_id}.{t}"));
    profile.metadata.artifact_id = assessment_id;
    profile.metadata.created_at = t;
    profile.metadata.evaluation_timestamp = t;

    let observations: Vec<ReplayObservation> = known
        .iter()
        .map(|b| ReplayObservation {
            id: stable_uuid(&format!("csp003.bar.{ticker}.{}", b.timestamp.timestamp())),
            effective_from: b.timestamp,
        })
        .collect();

    decide_forward(
        ReplayInputs {
            instrument_id,
            as_of: t,
            engine_version: UNFROZEN_ENGINE_VERSION.to_string(),
            produced_by: FORWARD_PRODUCER.to_string(),
            assessments: vec![ReplayAssessment {
                id: assessment_id,
                evaluation_timestamp: t,
                signature_hash: profile.to_hash(),
                profile,
            }],
            lake_decisions: vec![],
            observations,
        },
        policy,
    )
}

pub fn price_bars_for(ticker: &str, bars: &[DailyBar], now: DateTime<Utc>) -> Vec<PriceBar> {
    let instrument_id = instrument_id_for(ticker);
    bars.iter()
        .filter(|b| b.timestamp <= now && b.close.is_finite() && b.close > 0.0)
        .map(|b| PriceBar {
            effective_from: b.timestamp,
            close: b.close,
            instrument_id: Some(instrument_id),
        })
        .collect()
}

fn sma(closes: &[f64], window: usize) -> Option<f64> {
    if closes.len() < window {
        None
    } else {
        Some(closes[closes.len() - window..].iter().sum::<f64>() / window as f64)
    }
}

fn roc(closes: &[f64], window: usize) -> Option<f64> {
    if closes.len() < window + 1 {
        return None;
    }
    let current = *closes.last()?;
    let previous = closes[closes.len() - window - 1];
    if previous > 0.0 {
        Some(((current - previous) / previous) * 100.0)
    } else {
        None
    }
}

fn stable_uuid(tag: &str) -> Uuid {
    let digest = Sha256::digest(tag.as_bytes());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    Uuid::from_bytes(bytes)
}
