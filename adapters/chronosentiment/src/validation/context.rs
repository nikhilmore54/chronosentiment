use crate::observation::ValidatedObservation;
use crate::policy::PolicySnapshot;
use crate::portfolio::PortfolioSnapshot;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

pub struct InstrumentEvaluationContext {
    pub instrument_id: Uuid,
    pub observations: Vec<ValidatedObservation>,
}

/// The canonical input for every downstream reasoning engine.
/// Represents exactly what was known on a specific date for an entire universe.
pub struct MarketEvaluationContext {
    pub evaluation_timestamp: DateTime<Utc>,
    pub universe: String,

    /// Observations that apply to the entire market (e.g., VIX, breadth, macro data)
    pub market_observations: Vec<ValidatedObservation>,

    /// Local contexts for each specific instrument in the universe
    pub instrument_contexts: HashMap<Uuid, InstrumentEvaluationContext>,

    /// The state of the user's portfolio exactly AT the `evaluation_timestamp`
    pub portfolio: Option<PortfolioSnapshot>,

    /// The active policies governing decisions exactly AT the `evaluation_timestamp`
    pub policy: Option<PolicySnapshot>,
}
