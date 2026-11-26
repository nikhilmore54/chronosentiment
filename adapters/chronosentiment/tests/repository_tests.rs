use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use chronosentiment_adapter::instrument::Instrument;
use chronosentiment_adapter::observation::ValidatedObservation;
use chronosentiment_adapter::repository::observation_repository::{
    InstrumentRepository, PostgresRepository, ValidatedObservationRepository,
};

// To run this test, ensure you have a valid DATABASE_URL in your environment.
// For example:
// DATABASE_URL=postgres://user:pass@localhost:5432/chronosentiment cargo test

#[sqlx::test]
async fn test_instrument_and_observation_repository(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let repo = PostgresRepository::new(pool);

    // 1. Create and Store Instrument
    let mut inst = Instrument::new("NSE".to_string(), "RELIANCE".to_string());
    inst.add_provider_id("kite_token", "738561");

    repo.store_instrument(&inst).await?;

    // Verify retrieval
    let fetched_inst = repo
        .get_by_symbol("NSE", "RELIANCE")
        .await?
        .expect("Instrument should exist");
    assert_eq!(fetched_inst.display_symbol, "RELIANCE");
    assert_eq!(
        fetched_inst.provider_ids.get("kite_token").unwrap(),
        "738561"
    );

    // 2. Create and Store Observation
    let raw_payload = json!({"status": "success", "data": {"candles": []}});
    let normalized_payload = json!({"interval": "minute", "candle_count": 0});
    let now = Utc::now();

    let obs = ValidatedObservation {
        id: Uuid::new_v4(),
        research_session_id: None,
        instrument_id: Some(inst.id),
        observation_type: "MarketPrice".to_string(),
        source: "Kite".to_string(),
        source_identifier: None,
        observed_at: now,
        effective_from: now,
        effective_to: None,
        recorded_at: now,
        raw_payload,
        normalized_payload,
        confidence: 1.0,
        freshness: 0.0,
        coverage: "Complete".to_string(),
        consistency: Some(1.0),
        quality_score: 0.95,
        provenance_hash: "test-hash".to_string(),
        schema_version: 1,
    };

    repo.store_observation(&obs).await?;

    // 3. Test Time-Travel Query
    let future_time = now + chrono::Duration::days(1);
    let past_time = now - chrono::Duration::days(1);

    // Should find it when querying future_time (as it's effective now)
    let as_of_future = repo.get_observations_as_of(inst.id, future_time).await?;
    assert_eq!(
        as_of_future.len(),
        1,
        "Should find the observation in the future query"
    );

    // Should NOT find it when querying past_time (as it wasn't effective yet)
    let as_of_past = repo.get_observations_as_of(inst.id, past_time).await?;
    assert_eq!(
        as_of_past.len(),
        0,
        "Should NOT find the observation in the past query"
    );

    Ok(())
}
