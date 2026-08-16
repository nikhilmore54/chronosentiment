use chrono::{TimeZone, Utc};
use chronosentiment_adapter::observation::ValidatedObservation;
use chronosentiment_adapter::validation::replay::{ReplayEngine, ReplayRequest};
use chronosentiment_adapter::repository::observation_repository::ValidatedObservationRepository;
use async_trait::async_trait;
use serde_json::json;
use std::error::Error;
use uuid::Uuid;

// A simple mock repository to test the replay logic without Postgres
struct MockRepository {
    observations: Vec<ValidatedObservation>,
}

#[async_trait]
impl ValidatedObservationRepository for MockRepository {
    async fn store_observation(
        &self,
        _observation: &ValidatedObservation,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    async fn get_observations_as_of(
        &self,
        instrument_id: Uuid,
        evaluation_timestamp: chrono::DateTime<Utc>,
    ) -> Result<Vec<ValidatedObservation>, Box<dyn Error>> {
        let filtered = self
            .observations
            .iter()
            .filter(|o| o.instrument_id == Some(instrument_id))
            .filter(|o| o.effective_from <= evaluation_timestamp)
            .cloned()
            .collect();
        Ok(filtered)
    }

    async fn get_complete_history(
        &self,
        instrument_id: Uuid,
    ) -> Result<Vec<ValidatedObservation>, Box<dyn Error>> {
        let filtered = self
            .observations
            .iter()
            .filter(|o| o.instrument_id == Some(instrument_id))
            .cloned()
            .collect();
        Ok(filtered)
    }
}

fn make_obs(
    instrument_id: Uuid,
    t: chrono::DateTime<Utc>,
) -> ValidatedObservation {
    ValidatedObservation {
        id: Uuid::new_v4(),
        research_session_id: None,
        instrument_id: Some(instrument_id),
        observation_type: "Price".to_string(),
        source: "Kite".to_string(),
        source_identifier: None,
        observed_at: t,
        effective_from: t,
        effective_to: None,
        recorded_at: t,
        raw_payload: json!({}),
        normalized_payload: json!({}),
        confidence: 1.0,
        freshness: 0.0,
        coverage: "Complete".to_string(),
        consistency: None,
        quality_score: 1.0,
        provenance_hash: "test".to_string(),
        schema_version: 1,
    }
}

#[tokio::test]
async fn test_replay_engine_strictly_bounds_time() {
    let instrument_id = Uuid::new_v4();

    // Create observations on Jan 10, Jan 15, Jan 20
    let t10 = Utc.with_ymd_and_hms(2025, 1, 10, 0, 0, 0).unwrap();
    let t15 = Utc.with_ymd_and_hms(2025, 1, 15, 0, 0, 0).unwrap();
    let t20 = Utc.with_ymd_and_hms(2025, 1, 20, 0, 0, 0).unwrap();

    let o10 = make_obs(instrument_id, t10);
    let o15 = make_obs(instrument_id, t15);
    let o20 = make_obs(instrument_id, t20);

    let repo = MockRepository {
        observations: vec![o10, o15, o20],
    };

    let engine = ReplayEngine::new(&repo);

    // Replay on Jan 16 (Should see 10, 15. MUST NOT see 20)
    let t16 = Utc.with_ymd_and_hms(2025, 1, 16, 0, 0, 0).unwrap();
    let request = ReplayRequest {
        research_session_id: "rs-test".to_string(),
        universe: "TEST".to_string(),
        evaluation_timestamp: t16,
        portfolio_snapshot: None,
        policy_snapshot: None,
        target_instrument_id: instrument_id,
    };

    let context = engine.generate_context(request).await.unwrap();

    assert_eq!(context.market_observations.len(), 2);
    assert_eq!(context.market_observations[0].effective_from, t10);
    assert_eq!(context.market_observations[1].effective_from, t15);
}
