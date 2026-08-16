//! UserProfile — the minimal user context required by the Portfolio Allocation Engine.
//!
//! Deliberately small. This is not a financial-planning profile.
//! It captures only what the allocation engine needs to personalise a recommendation.
//!
//! The allocation engine must not reach inside this struct to make market intelligence
//! decisions. Market intelligence is C3-002's responsibility. Execution parameters are
//! Coralys's responsibility. This struct only constrains allocation sizing and horizon.

use serde::{Deserialize, Serialize};

/// How much risk the user is willing to accept on a single position.
///
/// This is a user-declared preference, not a computed risk score.
/// The allocation engine uses it to scale position sizing within the
/// Coralys-derived risk boundary — it does NOT override the Coralys risk_pct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskTolerance {
    /// User prefers smaller allocations; allocation engine applies a conservative sizing factor.
    Conservative,
    /// User accepts moderate allocations; allocation engine applies a neutral sizing factor.
    Moderate,
    /// User accepts larger allocations; allocation engine applies a full sizing factor.
    Aggressive,
}

impl RiskTolerance {
    /// Sizing factor applied to the weekly budget when computing allocation_inr.
    ///
    /// Conservative: allocate at most 50% of the weekly budget to any single instrument.
    /// Moderate:     allocate at most 75%.
    /// Aggressive:   allocate up to 100%.
    ///
    /// These are FROZEN DESIGN PARAMETERS for AllocationEngine v0.
    pub fn sizing_factor(&self) -> f64 {
        match self {
            RiskTolerance::Conservative => 0.50,
            RiskTolerance::Moderate => 0.75,
            RiskTolerance::Aggressive => 1.00,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            RiskTolerance::Conservative => "Conservative",
            RiskTolerance::Moderate => "Moderate",
            RiskTolerance::Aggressive => "Aggressive",
        }
    }
}

/// How long the user intends to hold positions.
///
/// Used by the allocation engine to filter recommendations whose
/// `maximum_hold_sessions` exceeds the user's horizon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvestmentHorizon {
    /// Up to 5 sessions (~1 week).
    ShortTerm,
    /// Up to 20 sessions (~1 month).
    MediumTerm,
    /// Up to 60 sessions (~3 months).
    LongTerm,
}

impl InvestmentHorizon {
    /// Maximum sessions the user is willing to hold a position.
    pub fn max_sessions(&self) -> u32 {
        match self {
            InvestmentHorizon::ShortTerm => 5,
            InvestmentHorizon::MediumTerm => 20,
            InvestmentHorizon::LongTerm => 60,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            InvestmentHorizon::ShortTerm => "Short-term (≤5 sessions)",
            InvestmentHorizon::MediumTerm => "Medium-term (≤20 sessions)",
            InvestmentHorizon::LongTerm => "Long-term (≤60 sessions)",
        }
    }
}

/// Minimal user context for the Portfolio Allocation Engine.
///
/// `weekly_investment_inr` is the user's stated weekly budget in Indian Rupees.
/// The allocation engine will never recommend spending more than this in a single week
/// across all instruments combined.
///
/// This struct has no knowledge of C3-002, Coralys, or any market intelligence.
/// It is a pure user preference container.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserProfile {
    /// Stable user identifier. Opaque string; no PII stored here.
    pub user_id: String,
    /// Weekly investment budget in Indian Rupees (INR).
    /// Must be positive. The allocation engine will not exceed this total per week.
    pub weekly_investment_inr: f64,
    /// User's declared risk tolerance. Affects per-instrument sizing factor.
    pub risk_tolerance: RiskTolerance,
    /// User's declared investment horizon. Filters out recommendations that exceed it.
    pub investment_horizon: InvestmentHorizon,
}

impl UserProfile {
    /// Validate that the profile is internally consistent.
    pub fn validate(&self) -> Result<(), UserProfileError> {
        if self.user_id.trim().is_empty() {
            return Err(UserProfileError::EmptyUserId);
        }
        if !self.weekly_investment_inr.is_finite() || self.weekly_investment_inr <= 0.0 {
            return Err(UserProfileError::InvalidWeeklyBudget);
        }
        Ok(())
    }

    /// Maximum allocation for a single instrument in a single week, in INR.
    ///
    /// = weekly_investment_inr × risk_tolerance.sizing_factor()
    ///
    /// The allocation engine uses this as the upper bound per instrument.
    pub fn max_single_instrument_inr(&self) -> f64 {
        self.weekly_investment_inr * self.risk_tolerance.sizing_factor()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserProfileError {
    EmptyUserId,
    InvalidWeeklyBudget,
}

impl std::fmt::Display for UserProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserProfileError::EmptyUserId => write!(f, "user_id must be non-empty"),
            UserProfileError::InvalidWeeklyBudget => {
                write!(f, "weekly_investment_inr must be a positive finite number")
            }
        }
    }
}

impl std::error::Error for UserProfileError {}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_profile() -> UserProfile {
        UserProfile {
            user_id: "user-001".into(),
            weekly_investment_inr: 5000.0,
            risk_tolerance: RiskTolerance::Moderate,
            investment_horizon: InvestmentHorizon::MediumTerm,
        }
    }

    #[test]
    fn valid_profile_passes_validation() {
        assert!(valid_profile().validate().is_ok());
    }

    #[test]
    fn empty_user_id_fails_validation() {
        let mut p = valid_profile();
        p.user_id = "  ".into();
        assert_eq!(p.validate(), Err(UserProfileError::EmptyUserId));
    }

    #[test]
    fn zero_budget_fails_validation() {
        let mut p = valid_profile();
        p.weekly_investment_inr = 0.0;
        assert_eq!(p.validate(), Err(UserProfileError::InvalidWeeklyBudget));
    }

    #[test]
    fn negative_budget_fails_validation() {
        let mut p = valid_profile();
        p.weekly_investment_inr = -100.0;
        assert_eq!(p.validate(), Err(UserProfileError::InvalidWeeklyBudget));
    }

    #[test]
    fn nan_budget_fails_validation() {
        let mut p = valid_profile();
        p.weekly_investment_inr = f64::NAN;
        assert_eq!(p.validate(), Err(UserProfileError::InvalidWeeklyBudget));
    }

    #[test]
    fn conservative_sizing_factor_is_half() {
        assert!((RiskTolerance::Conservative.sizing_factor() - 0.50).abs() < 1e-9);
    }

    #[test]
    fn moderate_sizing_factor_is_three_quarters() {
        assert!((RiskTolerance::Moderate.sizing_factor() - 0.75).abs() < 1e-9);
    }

    #[test]
    fn aggressive_sizing_factor_is_one() {
        assert!((RiskTolerance::Aggressive.sizing_factor() - 1.00).abs() < 1e-9);
    }

    #[test]
    fn max_single_instrument_inr_scales_by_risk_tolerance() {
        let mut p = valid_profile(); // 5000 INR, Moderate (0.75)
        assert!((p.max_single_instrument_inr() - 3750.0).abs() < 1e-6);

        p.risk_tolerance = RiskTolerance::Conservative;
        assert!((p.max_single_instrument_inr() - 2500.0).abs() < 1e-6);

        p.risk_tolerance = RiskTolerance::Aggressive;
        assert!((p.max_single_instrument_inr() - 5000.0).abs() < 1e-6);
    }

    #[test]
    fn medium_term_horizon_max_sessions_is_20() {
        assert_eq!(InvestmentHorizon::MediumTerm.max_sessions(), 20);
    }

    #[test]
    fn short_term_horizon_max_sessions_is_5() {
        assert_eq!(InvestmentHorizon::ShortTerm.max_sessions(), 5);
    }

    #[test]
    fn long_term_horizon_max_sessions_is_60() {
        assert_eq!(InvestmentHorizon::LongTerm.max_sessions(), 60);
    }
}