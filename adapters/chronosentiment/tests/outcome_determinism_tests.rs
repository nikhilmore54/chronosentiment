use chrono::{DateTime, Utc, TimeZone};
use uuid::Uuid;
use serde_json::json;

use chronosentiment_adapter::observation::ValidatedObservation;
use chronosentiment_adapter::reasoning::strategy::{OpportunityStrategy, Horizon, PriceRange};
use chronosentiment_adapter::validation::outcome::{OutcomeEngine, OutcomeRecord};
use chronosentiment_adapter::repository::knowledge::{ArtifactMetadata, ArtifactType};

fn create_mock_obs(time: DateTime<Utc>, inst_id: Uuid, close: f64, high: f64, low: f64) -> ValidatedObservation {
    ValidatedObservation {
        id: Uuid::new_v4(),
        research_session_id: None,
        instrument_id: Some(inst_id),
        observation_type: "MarketPrice".to_string(),
        source: "Test".to_string(),
        source_identifier: None,
        observed_at: time,
        effective_from: time,
        effective_to: None,
        recorded_at: Utc::now(),
        raw_payload: json!({}),
        normalized_payload: json!({
            "close": close,
            "high": high,
            "low": low
        }),
        confidence: 1.0,
        freshness: 0.0,
        coverage: "".to_string(),
        consistency: None,
        quality_score: 1.0,
        provenance_hash: "".to_string(),
        schema_version: 1,
    }
}

pub fn compare_outcomes(out1: &OutcomeRecord, out2: &OutcomeRecord) -> Result<(), String> {
    if out1.decision_id != out2.decision_id { return Err("decision_id mismatch".to_string()); }
    if out1.evaluation_timestamp != out2.evaluation_timestamp { return Err("evaluation_timestamp mismatch".to_string()); }
    if out1.observation_end_timestamp != out2.observation_end_timestamp { return Err("observation_end_timestamp mismatch".to_string()); }
    if out1.horizon != out2.horizon { return Err("horizon mismatch".to_string()); }
    if out1.holding_period_days != out2.holding_period_days { return Err("holding_period_days mismatch".to_string()); }
    if out1.exit_reason != out2.exit_reason { return Err("exit_reason mismatch".to_string()); }
    if out1.outcome_return != out2.outcome_return { return Err("outcome_return mismatch".to_string()); }
    if out1.mfe != out2.mfe { return Err("mfe mismatch".to_string()); }
    if out1.mae != out2.mae { return Err("mae mismatch".to_string()); }
    if out1.maximum_drawdown != out2.maximum_drawdown { return Err("maximum_drawdown mismatch".to_string()); }
    if out1.realized_volatility != out2.realized_volatility { return Err("realized_volatility mismatch".to_string()); }
    Ok(())
}

#[test]
fn test_c2_outcome_determinism() {
    let t = Utc.timestamp_opt(1705312800, 0).unwrap(); // 2024-01-15T10:00:00Z
    let inst_id = Uuid::new_v4();
    let decision_id = Uuid::new_v4();
    
    let strategy = OpportunityStrategy {
        metadata: ArtifactMetadata::mock(),
        decision_id,
        expected_horizon: Horizon::Swing,
        expected_holding_period_days: (10, 20),
        entry_zone: PriceRange { min: 99.0, max: 101.0 },
        target_zone: PriceRange { min: 110.0, max: 120.0 },
        stop_loss_zone: PriceRange { min: 80.0, max: 90.0 },
        expected_return: 0.1,
        expected_drawdown: 0.1,
        expected_volatility: 0.05,
        risk_reward_ratio: 1.0,
        confidence: 0.5,
    };
    
    let mut strat_meta = ArtifactMetadata::mock();
    strat_meta.artifact_id = Uuid::new_v4();
    strat_meta.artifact_type = ArtifactType::Decision;
    
    let obs1 = create_mock_obs(t + chrono::Duration::days(1), inst_id, 100.0, 100.0, 100.0);
    let obs2 = create_mock_obs(t + chrono::Duration::days(2), inst_id, 105.0, 106.0, 104.0);
    let obs3 = create_mock_obs(t + chrono::Duration::days(3), inst_id, 115.0, 116.0, 114.0); // Target hit!
    
    let future_observations = vec![obs1.clone(), obs2.clone(), obs3.clone()];
    
    let engine = OutcomeEngine;
    
    let out1 = engine.measure_outcome(decision_id, &strategy, &strat_meta, &future_observations, t, 20, Some(inst_id));
    let out2 = engine.measure_outcome(decision_id, &strategy, &strat_meta, &future_observations, t, 20, Some(inst_id));
    
    compare_outcomes(&out1, &out2).expect("C2 Outcome Determinism FAILED");
    
    assert_eq!(out1.exit_reason, "Target Hit");
    assert_eq!(out1.holding_period_days, 2); // days after entry
}

#[test]
fn test_c2_negative_determinism_fault_injection() {
    let t = Utc.timestamp_opt(1705312800, 0).unwrap();
    let inst_id = Uuid::new_v4();
    let decision_id = Uuid::new_v4();
    
    let strategy = OpportunityStrategy {
        metadata: ArtifactMetadata::mock(),
        decision_id,
        expected_horizon: Horizon::Swing,
        expected_holding_period_days: (10, 20),
        entry_zone: PriceRange { min: 99.0, max: 101.0 },
        target_zone: PriceRange { min: 110.0, max: 120.0 },
        stop_loss_zone: PriceRange { min: 80.0, max: 90.0 },
        expected_return: 0.1,
        expected_drawdown: 0.1,
        expected_volatility: 0.05,
        risk_reward_ratio: 1.0,
        confidence: 0.5,
    };
    
    let mut strat_meta = ArtifactMetadata::mock();
    strat_meta.artifact_id = Uuid::new_v4();
    strat_meta.artifact_type = ArtifactType::Decision;
    
    let obs1 = create_mock_obs(t + chrono::Duration::days(1), inst_id, 100.0, 100.0, 100.0);
    let obs2 = create_mock_obs(t + chrono::Duration::days(2), inst_id, 105.0, 106.0, 104.0);
    let obs3 = create_mock_obs(t + chrono::Duration::days(3), inst_id, 115.0, 116.0, 114.0); // Target hit!
    
    let future_obs_run1 = vec![obs1.clone(), obs2.clone(), obs3.clone()];
    
    // Fault injection: modify the high of obs3 in run 2 so target is NOT hit
    let mut obs3_mutated = obs3.clone();
    obs3_mutated.normalized_payload = json!({
        "close": 105.0,
        "high": 109.0, // Missed target of 110!
        "low": 104.0
    });
    let future_obs_run2 = vec![obs1.clone(), obs2.clone(), obs3_mutated];
    
    let engine = OutcomeEngine;
    
    let out1 = engine.measure_outcome(decision_id, &strategy, &strat_meta, &future_obs_run1, t, 20, Some(inst_id));
    let out2 = engine.measure_outcome(decision_id, &strategy, &strat_meta, &future_obs_run2, t, 20, Some(inst_id));
    
    // This comparison should fail because outcomes are different due to different future realities
    let result = compare_outcomes(&out1, &out2);
    assert!(result.is_err(), "Test should have caught the mismatch in outcome due to mutated future reality!");
}
