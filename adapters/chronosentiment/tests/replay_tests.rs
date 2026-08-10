use chrono::{TimeZone, Utc};
use chronosentiment_adapter::observation::Observation;
use chronosentiment_adapter::validation::replay::{ReplayEngine, ReplayRequest};
use chronosentiment_adapter::repository::ObservationRepository;
use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;

// A simple mock repository to test the replay logic without Postgres
struct MockRepository {
    observations: Vec<Observation>,
}

#[async_trait]
impl ObservationRepository for MockRepository {
    async fn store_observation(&self, _observation: &Observation) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    async fn get_observations_as_of(
        &self,
        instrument_id: Uuid,
        evaluation_timestamp: chrono::DateTime<Utc>,
    ) -> Result<Vec<Observation>, Box<dyn Error>> {
        let filtered = self.observations.iter()
            .filter(|o| o.instrument_id == Some(instrument_id))
            .filter(|o| o.effective_from <= evaluation_timestamp)
            .cloned()
            .collect();
        Ok(filtered)
    }

    async fn get_complete_history(&self, instrument_id: Uuid) -> Result<Vec<Observation>, Box<dyn Error>> {
        let filtered = self.observations.iter()
            .filter(|o| o.instrument_id == Some(instrument_id))
            .cloned()
            .collect();
        Ok(filtered)
    }
}

#[tokio::test]
async fn test_replay_engine_strictly_bounds_time() {
    let instrument_id = Uuid::new_v4();
    
    // Create observations on Jan 10, Jan 15, Jan 20
    let t10 = Utc.with_ymd_and_hms(2025, 1, 10, 0, 0, 0).unwrap();
    let t15 = Utc.with_ymd_and_hms(2025, 1, 15, 0, 0, 0).unwrap();
    let t20 = Utc.with_ymd_and_hms(2025, 1, 20, 0, 0, 0).unwrap();

    let mut o10 = Observation::new("Price".to_string(), "Kite".to_string(), t10, t10, serde_json::json!({}), serde_json::json!({}));
    o10.instrument_id = Some(instrument_id);
    let mut o15 = Observation::new("Price".to_string(), "Kite".to_string(), t15, t15, serde_json::json!({}), serde_json::json!({}));
    o15.instrument_id = Some(instrument_id);
    let mut o20 = Observation::new("Price".to_string(), "Kite".to_string(), t20, t20, serde_json::json!({}), serde_json::json!({}));
    o20.instrument_id = Some(instrument_id);

    let repo = MockRepository {
        observations: vec![o10, o15, o20],
    };

    let engine = ReplayEngine::new(&repo);

    // Replay on Jan 16 (Should see 10, 15. MUST NOT see 20)
    let t16 = Utc.with_ymd_and_hms(2025, 1, 16, 0, 0, 0).unwrap();
    let request = ReplayRequest {
        research_session_id: "rs-test".to_string(),
        evaluation_timestamp: t16,
        portfolio_snapshot: None,
        policy_snapshot: None,
        target_instrument_id: instrument_id,
    };

    let context = engine.generate_context(request).await.unwrap();

    assert_eq!(context.observations.len(), 2);
    assert_eq!(context.observations[0].effective_from, t10);
    assert_eq!(context.observations[1].effective_from, t15);
}
