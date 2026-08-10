use chrono::{DateTime, Utc};
use std::error::Error;
use uuid::Uuid;

use crate::policy::PolicySnapshot;
use crate::portfolio::PortfolioSnapshot;
use crate::repository::ObservationRepository;
use crate::validation::context::EvaluationContext;

pub struct ReplayRequest {
    pub research_session_id: String,
    pub evaluation_timestamp: DateTime<Utc>,
    pub portfolio_snapshot: Option<PortfolioSnapshot>,
    pub policy_snapshot: Option<PolicySnapshot>,
    
    // For Phase 1B testing/scaffolding, we can optionally supply instrument_id
    // here so the Postgres query works. In full Phase 2, the ResearchSession
    // will determine which instruments to query.
    pub target_instrument_id: Uuid, 
}

/// The Replay Engine enforces strict chronological reality.
/// It MUST NOT compute metrics, rank evidence, or make decisions.
/// It only reconstructs historical state for downstream engines.
pub struct ReplayEngine<'a> {
    observation_repo: &'a dyn ObservationRepository,
}

impl<'a> ReplayEngine<'a> {
    pub fn new(observation_repo: &'a dyn ObservationRepository) -> Self {
        Self { observation_repo }
    }

    /// Reconstructs the `EvaluationContext` exactly as it would have appeared at the `evaluation_timestamp`.
    pub async fn generate_context(
        &self,
        request: ReplayRequest,
    ) -> Result<EvaluationContext, Box<dyn Error>> {
        // Fetch strictly bounded observations
        let observations = self.observation_repo
            .get_observations_as_of(request.target_instrument_id, request.evaluation_timestamp)
            .await?;

        Ok(EvaluationContext {
            evaluation_timestamp: request.evaluation_timestamp,
            research_session_id: request.research_session_id,
            observations,
            portfolio: request.portfolio_snapshot,
            policy: request.policy_snapshot,
        })
    }
}
