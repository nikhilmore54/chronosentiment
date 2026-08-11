use async_trait::async_trait;
use uuid::Uuid;
use std::error::Error;
use crate::reasoning::decision::Decision;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait DecisionRepository {
    async fn store_decision(&self, decision: &Decision) -> Result<(), Box<dyn Error>>;
    
    async fn get_decisions_for_universe(
        &self,
        universe: &str,
        evaluation_timestamp: DateTime<Utc>,
    ) -> Result<Vec<Decision>, Box<dyn Error>>;
    
    async fn get_decision_history(
        &self,
        instrument_id: Uuid,
    ) -> Result<Vec<Decision>, Box<dyn Error>>;
}
