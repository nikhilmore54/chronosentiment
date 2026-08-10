use chrono::{DateTime, Utc};
use crate::observation::Observation;
use crate::portfolio::PortfolioSnapshot;
use crate::policy::PolicySnapshot;

/// The canonical input for every downstream reasoning engine.
/// Represents exactly what was known on a specific date, enforcing strict
/// time boundaries and preventing look-ahead bias.
pub struct EvaluationContext {
    pub evaluation_timestamp: DateTime<Utc>,
    pub research_session_id: String,
    
    /// Observations whose `effective_from` is strictly <= `evaluation_timestamp`
    pub observations: Vec<Observation>,
    
    /// The state of the user's portfolio exactly AT the `evaluation_timestamp`
    pub portfolio: Option<PortfolioSnapshot>,
    
    /// The active policies governing decisions exactly AT the `evaluation_timestamp`
    pub policy: Option<PolicySnapshot>,
}
