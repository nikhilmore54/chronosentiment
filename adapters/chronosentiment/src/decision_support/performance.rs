//! Performance Engine v0.1 — measure a recorded ledger against its outcomes.
//!
//! Consumes only `DecisionLedger` and `OutcomeReport`. Never mutates either.
//! Never calls `decide_at`. Never tunes thresholds, picks a “best” horizon, or
//! feeds results back into `TradingDecision`.
//!
//! `NO_TRADE` is not a zero-return trade. Trading stats use LONG/SHORT only.
//! Opportunity-cost stats use NO_TRADE only, on the same attached lake returns.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::backtest::DecisionLedger;
use super::outcome::{OutcomeReport, HORIZON_DAYS};
use super::DecisionAction;

pub const SCHEMA_VERSION: &str = "csp002.performance.0";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReturnStats {
    /// Decisions in this population (action filter), including missing horizons.
    pub n_decisions: u32,
    /// Finite attached `outcome_return` observations.
    pub n_observed: u32,
    pub n_unavailable: u32,
    pub n_win: u32,
    pub n_loss: u32,
    pub n_zero: u32,
    pub mean: Option<f64>,
    pub median: Option<f64>,
    /// Sum of simple attached returns in ledger order. Not a portfolio wealth path.
    pub cumulative_return: Option<f64>,
    /// `n_win / n_observed`. Zeros are neither wins nor losses.
    pub win_rate: Option<f64>,
    pub min: Option<f64>,
    pub p25: Option<f64>,
    pub p75: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskStats {
    /// Peak-to-trough of the cumulative-sum path of attached returns (ledger order).
    pub max_drawdown: Option<f64>,
    /// Sample standard deviation (n−1) of attached returns.
    pub volatility: Option<f64>,
    /// sqrt(mean of min(r, 0)²), MAR = 0, denominator = n_observed.
    pub downside_volatility: Option<f64>,
    pub worst_outcome: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerStats {
    pub returns: ReturnStats,
    pub risk: RiskStats,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionCounts {
    pub long: u32,
    pub short: u32,
    pub no_trade: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionReturnStats {
    pub long: ReturnStats,
    pub short: ReturnStats,
    pub no_trade: ReturnStats,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionBehavior {
    pub n_records: u32,
    pub counts: ActionCounts,
    pub first_as_of: Option<DateTime<Utc>>,
    pub last_as_of: Option<DateTime<Utc>>,
    pub span_calendar_days: Option<i64>,
    pub decisions_per_calendar_day: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HorizonPerformance {
    pub horizon_days: u32,
    /// LONG + SHORT only. NO_TRADE is excluded (not treated as 0).
    pub trading: LayerStats,
    /// NO_TRADE only. Subsequent attached path; not trading P&L.
    pub opportunity: LayerStats,
    pub by_action: ActionReturnStats,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub schema_version: String,
    pub decision_engine_version: String,
    pub ledger_identity_hash: String,
    pub outcome_identity_hash: String,
    pub behavior: DecisionBehavior,
    pub horizons: Vec<HorizonPerformance>,
    pub content_hash: String,
}

/// Pure measurement. Ledger action is the decision; bundles supply attached returns.
pub fn measure_performance(ledger: &DecisionLedger, outcomes: &OutcomeReport) -> PerformanceReport {
    let behavior = behavior(ledger);
    let horizons = HORIZON_DAYS
        .iter()
        .map(|days| horizon_slice(ledger, outcomes, *days))
        .collect();
    let mut report = PerformanceReport {
        schema_version: SCHEMA_VERSION.to_string(),
        decision_engine_version: ledger.engine_version.clone(),
        ledger_identity_hash: ledger.identity_hash(),
        outcome_identity_hash: outcomes.identity_hash(),
        behavior,
        horizons,
        content_hash: String::new(),
    };
    report.content_hash = report_hash(&report);
    report
}

fn behavior(ledger: &DecisionLedger) -> DecisionBehavior {
    let mut counts = ActionCounts {
        long: 0,
        short: 0,
        no_trade: 0,
    };
    for rec in &ledger.records {
        match rec.action {
            DecisionAction::Long => counts.long += 1,
            DecisionAction::Short => counts.short += 1,
            DecisionAction::NoTrade => counts.no_trade += 1,
        }
    }
    let first_as_of = ledger.records.first().map(|r| r.as_of_timestamp);
    let last_as_of = ledger.records.last().map(|r| r.as_of_timestamp);
    let span_calendar_days = match (first_as_of, last_as_of) {
        (Some(a), Some(b)) => Some((b - a).num_days()),
        _ => None,
    };
    let n_records = ledger.records.len() as u32;
    let decisions_per_calendar_day = span_calendar_days.and_then(|d| {
        if d > 0 {
            Some(n_records as f64 / d as f64)
        } else {
            None
        }
    });
    DecisionBehavior {
        n_records,
        counts,
        first_as_of,
        last_as_of,
        span_calendar_days,
        decisions_per_calendar_day,
    }
}

fn horizon_slice(
    ledger: &DecisionLedger,
    outcomes: &OutcomeReport,
    days: u32,
) -> HorizonPerformance {
    let mut long = Vec::new();
    let mut short = Vec::new();
    let mut no_trade = Vec::new();
    let mut trading_rs = Vec::new();
    let mut n_long = 0u32;
    let mut n_short = 0u32;
    let mut n_no_trade = 0u32;

    for rec in &ledger.records {
        match rec.action {
            DecisionAction::Long => n_long += 1,
            DecisionAction::Short => n_short += 1,
            DecisionAction::NoTrade => n_no_trade += 1,
        }
        let Some(bundle) = outcomes
            .bundles
            .iter()
            .find(|b| b.ledger_decision_id == rec.decision_id)
        else {
            continue;
        };
        let Some(r) = attached_return(bundle, days) else {
            continue;
        };
        match rec.action {
            DecisionAction::Long => {
                long.push(r);
                trading_rs.push(r);
            }
            DecisionAction::Short => {
                short.push(r);
                trading_rs.push(r);
            }
            DecisionAction::NoTrade => no_trade.push(r),
        }
    }

    HorizonPerformance {
        horizon_days: days,
        trading: layer_stats(&trading_rs, n_long + n_short),
        opportunity: layer_stats(&no_trade, n_no_trade),
        by_action: ActionReturnStats {
            long: return_stats(&long, n_long),
            short: return_stats(&short, n_short),
            no_trade: return_stats(&no_trade, n_no_trade),
        },
    }
}

fn attached_return(bundle: &super::outcome::DecisionOutcomeBundle, days: u32) -> Option<f64> {
    bundle
        .horizons
        .iter()
        .find(|h| h.horizon_days == days && h.available)
        .and_then(|h| h.outcome_return)
        .filter(|r| r.is_finite())
}

fn layer_stats(returns: &[f64], n_decisions: u32) -> LayerStats {
    LayerStats {
        returns: return_stats(returns, n_decisions),
        risk: risk_stats(returns),
    }
}

fn return_stats(returns: &[f64], n_decisions: u32) -> ReturnStats {
    let n_observed = returns.len() as u32;
    let n_unavailable = n_decisions.saturating_sub(n_observed);
    if returns.is_empty() {
        return ReturnStats {
            n_decisions,
            n_observed: 0,
            n_unavailable,
            n_win: 0,
            n_loss: 0,
            n_zero: 0,
            mean: None,
            median: None,
            cumulative_return: None,
            win_rate: None,
            min: None,
            p25: None,
            p75: None,
            max: None,
        };
    }
    let n_win = returns.iter().filter(|r| **r > 0.0).count() as u32;
    let n_loss = returns.iter().filter(|r| **r < 0.0).count() as u32;
    let n_zero = returns.iter().filter(|r| **r == 0.0).count() as u32;
    let sum: f64 = returns.iter().sum();
    let mean = sum / returns.len() as f64;
    let mut sorted = returns.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    ReturnStats {
        n_decisions,
        n_observed,
        n_unavailable,
        n_win,
        n_loss,
        n_zero,
        mean: Some(mean),
        median: median(&sorted),
        cumulative_return: Some(sum),
        win_rate: Some(n_win as f64 / n_observed as f64),
        min: sorted.first().copied(),
        p25: quantile(&sorted, 0.25),
        p75: quantile(&sorted, 0.75),
        max: sorted.last().copied(),
    }
}

fn risk_stats(returns: &[f64]) -> RiskStats {
    if returns.is_empty() {
        return RiskStats {
            max_drawdown: None,
            volatility: None,
            downside_volatility: None,
            worst_outcome: None,
        };
    }
    let mut equity = 0.0;
    let mut peak = 0.0;
    let mut max_dd = 0.0;
    for r in returns {
        equity += r;
        if equity > peak {
            peak = equity;
        }
        let dd = peak - equity;
        if dd > max_dd {
            max_dd = dd;
        }
    }
    let n = returns.len() as f64;
    let mean: f64 = returns.iter().sum::<f64>() / n;
    let volatility = if returns.len() >= 2 {
        let var: f64 = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
        Some(var.sqrt())
    } else {
        None
    };
    let down: f64 = returns.iter().map(|r| r.min(0.0).powi(2)).sum::<f64>() / n;
    RiskStats {
        max_drawdown: Some(max_dd),
        volatility,
        downside_volatility: Some(down.sqrt()),
        worst_outcome: returns
            .iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)),
    }
}

fn median(sorted: &[f64]) -> Option<f64> {
    let n = sorted.len();
    if n == 0 {
        None
    } else if n % 2 == 1 {
        Some(sorted[n / 2])
    } else {
        Some((sorted[n / 2 - 1] + sorted[n / 2]) / 2.0)
    }
}

fn quantile(sorted: &[f64], p: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let idx = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted.get(idx.min(sorted.len() - 1)).copied()
}

fn report_hash(report: &PerformanceReport) -> String {
    #[derive(Serialize)]
    struct Payload<'a> {
        schema_version: &'a str,
        decision_engine_version: &'a str,
        ledger_identity_hash: &'a str,
        outcome_identity_hash: &'a str,
        behavior: &'a DecisionBehavior,
        horizons: &'a [HorizonPerformance],
    }
    let bytes = serde_json::to_vec(&Payload {
        schema_version: &report.schema_version,
        decision_engine_version: &report.decision_engine_version,
        ledger_identity_hash: &report.ledger_identity_hash,
        outcome_identity_hash: &report.outcome_identity_hash,
        behavior: &report.behavior,
        horizons: &report.horizons,
    })
    .expect("performance report serializes");
    format!("{:x}", Sha256::digest(&bytes))
}
