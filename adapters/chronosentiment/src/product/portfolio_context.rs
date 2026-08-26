//! PortfolioContext — point-in-time snapshot of the user's portfolio state.
//!
//! This is the information the Allocation Engine needs to make sizing decisions.
//! It has no knowledge of C3-002 directions or Coralys execution parameters.
//!
//! The allocation engine reads this to answer:
//!   "Given what the user already holds, how much more should they allocate?"
//!
//! Separation of concerns:
//!   C3-002        → direction (LONG / SHORT / NO_TRADE)
//!   Coralys v0    → execution parameters (target_pct, risk_pct, entry_price)
//!   PortfolioContext → existing exposure, available cash
//!   AllocationEngine → sizing decision (how much INR to allocate)

use serde::{Deserialize, Serialize};

/// A single position the user currently holds.
///
/// `instrument` is the canonical instrument symbol (e.g. "INFY.NS").
/// `quantity` is the number of shares/units held (positive = long, negative = short).
/// `average_cost_inr` is the average cost per unit in INR.
/// `current_value_inr` is the current market value of the position in INR (optional;
///   may be absent if the market is closed or the price feed is unavailable).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortfolioPosition {
    /// Canonical instrument symbol, e.g. "INFY.NS".
    pub instrument: String,
    /// Number of units held. Positive = long, negative = short.
    pub quantity: f64,
    /// Average cost per unit in INR.
    pub average_cost_inr: f64,
    /// Current market value of the full position in INR.
    /// None if price feed is unavailable.
    pub current_value_inr: Option<f64>,
}

impl PortfolioPosition {
    /// Cost basis of the position in INR (quantity × average_cost_inr).
    pub fn cost_basis_inr(&self) -> f64 {
        self.quantity * self.average_cost_inr
    }

    /// Unrealised P&L in INR, if current value is available.
    pub fn unrealised_pnl_inr(&self) -> Option<f64> {
        self.current_value_inr.map(|v| v - self.cost_basis_inr())
    }
}

/// Point-in-time snapshot of the user's portfolio state.
///
/// `as_of` is an ISO-8601 timestamp string indicating when this snapshot was taken.
/// The allocation engine must not use data from after `as_of`.
///
/// `available_cash_inr` is the cash available for new investments.
/// `holdings` is the list of current positions.
/// `existing_exposure_inr` is a pre-computed map of instrument → current exposure in INR.
///   This allows the allocation engine to check overexposure without iterating holdings.
///   It must be consistent with `holdings` at the time of snapshot creation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortfolioContext {
    /// ISO-8601 timestamp of when this snapshot was taken.
    pub as_of: String,
    /// Cash available for new investments, in INR.
    pub available_cash_inr: f64,
    /// Current holdings.
    pub holdings: Vec<PortfolioPosition>,
    /// Pre-computed exposure per instrument in INR (absolute value of position value).
    /// Key: instrument symbol. Value: exposure in INR.
    /// Must be consistent with `holdings`.
    pub existing_exposure_inr: std::collections::HashMap<String, f64>,
}

impl PortfolioContext {
    /// Total portfolio value in INR (sum of all position current values + cash).
    /// Returns None if any position is missing a current value.
    pub fn total_value_inr(&self) -> Option<f64> {
        let mut total = self.available_cash_inr;
        for pos in &self.holdings {
            total += pos.current_value_inr?;
        }
        Some(total)
    }

    /// Existing exposure for a specific instrument in INR.
    /// Returns 0.0 if the instrument is not in the portfolio.
    pub fn exposure_for(&self, instrument: &str) -> f64 {
        self.existing_exposure_inr
            .get(instrument)
            .copied()
            .unwrap_or(0.0)
    }

    /// Validate that the context is internally consistent.
    pub fn validate(&self) -> Result<(), PortfolioContextError> {
        if self.as_of.trim().is_empty() {
            return Err(PortfolioContextError::EmptyAsOf);
        }
        if !self.available_cash_inr.is_finite() || self.available_cash_inr < 0.0 {
            return Err(PortfolioContextError::InvalidCash);
        }
        for pos in &self.holdings {
            if pos.instrument.trim().is_empty() {
                return Err(PortfolioContextError::EmptyInstrument);
            }
            if !pos.average_cost_inr.is_finite() || pos.average_cost_inr < 0.0 {
                return Err(PortfolioContextError::InvalidAverageCost(
                    pos.instrument.clone(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortfolioContextError {
    EmptyAsOf,
    InvalidCash,
    EmptyInstrument,
    InvalidAverageCost(String),
}

impl std::fmt::Display for PortfolioContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortfolioContextError::EmptyAsOf => write!(f, "as_of must be non-empty"),
            PortfolioContextError::InvalidCash => {
                write!(f, "available_cash_inr must be a non-negative finite number")
            }
            PortfolioContextError::EmptyInstrument => {
                write!(f, "all holdings must have a non-empty instrument symbol")
            }
            PortfolioContextError::InvalidAverageCost(sym) => {
                write!(
                    f,
                    "average_cost_inr for {sym} must be a non-negative finite number"
                )
            }
        }
    }
}

impl std::error::Error for PortfolioContextError {}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn infy_position() -> PortfolioPosition {
        PortfolioPosition {
            instrument: "INFY.NS".into(),
            quantity: 20.0,
            average_cost_inr: 1250.0,
            current_value_inr: Some(26000.0),
        }
    }

    fn empty_context() -> PortfolioContext {
        PortfolioContext {
            as_of: "2026-08-16T09:15:00+05:30".into(),
            available_cash_inr: 10000.0,
            holdings: vec![],
            existing_exposure_inr: HashMap::new(),
        }
    }

    fn context_with_infy() -> PortfolioContext {
        let pos = infy_position();
        let mut exposure = HashMap::new();
        exposure.insert("INFY.NS".into(), 26000.0);
        PortfolioContext {
            as_of: "2026-08-16T09:15:00+05:30".into(),
            available_cash_inr: 5000.0,
            holdings: vec![pos],
            existing_exposure_inr: exposure,
        }
    }

    #[test]
    fn empty_context_validates() {
        assert!(empty_context().validate().is_ok());
    }

    #[test]
    fn context_with_holdings_validates() {
        assert!(context_with_infy().validate().is_ok());
    }

    #[test]
    fn empty_as_of_fails_validation() {
        let mut ctx = empty_context();
        ctx.as_of = "".into();
        assert_eq!(ctx.validate(), Err(PortfolioContextError::EmptyAsOf));
    }

    #[test]
    fn negative_cash_fails_validation() {
        let mut ctx = empty_context();
        ctx.available_cash_inr = -1.0;
        assert_eq!(ctx.validate(), Err(PortfolioContextError::InvalidCash));
    }

    #[test]
    fn zero_cash_is_valid() {
        let mut ctx = empty_context();
        ctx.available_cash_inr = 0.0;
        assert!(ctx.validate().is_ok());
    }

    #[test]
    fn cost_basis_is_quantity_times_average_cost() {
        let pos = infy_position(); // 20 × 1250 = 25000
        assert!((pos.cost_basis_inr() - 25000.0).abs() < 1e-6);
    }

    #[test]
    fn unrealised_pnl_computed_correctly() {
        let pos = infy_position(); // current=26000, cost=25000 → pnl=1000
        assert!((pos.unrealised_pnl_inr().unwrap() - 1000.0).abs() < 1e-6);
    }

    #[test]
    fn unrealised_pnl_none_when_no_current_value() {
        let mut pos = infy_position();
        pos.current_value_inr = None;
        assert!(pos.unrealised_pnl_inr().is_none());
    }

    #[test]
    fn exposure_for_known_instrument_returns_value() {
        let ctx = context_with_infy();
        assert!((ctx.exposure_for("INFY.NS") - 26000.0).abs() < 1e-6);
    }

    #[test]
    fn exposure_for_unknown_instrument_returns_zero() {
        let ctx = context_with_infy();
        assert!((ctx.exposure_for("TCS.NS") - 0.0).abs() < 1e-9);
    }

    #[test]
    fn total_value_includes_cash_and_positions() {
        let ctx = context_with_infy(); // cash=5000, INFY current=26000 → total=31000
        assert!((ctx.total_value_inr().unwrap() - 31000.0).abs() < 1e-6);
    }

    #[test]
    fn total_value_none_when_any_position_missing_current_value() {
        let mut ctx = context_with_infy();
        ctx.holdings[0].current_value_inr = None;
        assert!(ctx.total_value_inr().is_none());
    }
}
