use chrono::{DateTime, TimeZone, Utc};
use serde_json::json;
use std::error::Error;
use uuid::Uuid;

use chronosentiment_adapter::observation::ValidatedObservation;
use chronosentiment_adapter::reasoning::strategy::{Horizon, OpportunityStrategy, PriceRange};
use chronosentiment_adapter::repository::knowledge::{
    ArtifactMetadata, ArtifactRepository, ArtifactType,
};
use chronosentiment_adapter::repository::postgres_knowledge::PostgresKnowledgeRepository;
use chronosentiment_adapter::validation::outcome::{OutcomeEngine, OutcomeRecord};
use sqlx::PgPool;

fn get_mock_strategy(decision_id: Uuid) -> OpportunityStrategy {
    use chronosentiment_adapter::repository::knowledge::ArtifactMetadata;
    OpportunityStrategy {
        metadata: ArtifactMetadata::mock(),
        decision_id,
        expected_horizon: Horizon::Swing,
        expected_holding_period_days: (5, 20),
        entry_zone: PriceRange {
            min: 98.0,
            max: 102.0,
        },
        target_zone: PriceRange {
            min: 110.0,
            max: 120.0,
        },
        stop_loss_zone: PriceRange {
            min: 80.0,
            max: 90.0,
        },
        expected_return: 0.1,
        expected_drawdown: 0.05,
        expected_volatility: 0.05,
        risk_reward_ratio: 2.0,
        confidence: 0.8,
    }
}

fn get_mock_obs(
    time: DateTime<Utc>,
    inst_id: Uuid,
    close: f64,
    high: f64,
    low: f64,
) -> ValidatedObservation {
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
        recorded_at: time,
        raw_payload: json!({}),
        normalized_payload: json!({
            "close": close,
            "high": high,
            "low": low,
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

#[tokio::test]
async fn test_phase_d_final_acceptance_gate() {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://invalid:invalid@localhost:5432/invalid".to_string());
    let pool_res = PgPool::connect(&db_url).await;
    let has_db = pool_res.is_ok();

    let engine = OutcomeEngine;

    let eval_time = Utc.timestamp_opt(1705312800, 0).unwrap();
    let inst_id = Uuid::new_v4();
    let decision_id = Uuid::new_v4();

    let strategy = get_mock_strategy(decision_id);
    let mut strat_meta = ArtifactMetadata::mock();
    strat_meta.artifact_id = Uuid::new_v4();
    strat_meta.artifact_type = ArtifactType::Decision;
    strat_meta.replay_context_hash = "mock_replay_hash_123".to_string();
    strat_meta.knowledge_lake_version = "v14".to_string();
    strat_meta.engine_versions = json!({"StrategyEngine": "v2", "OutcomeEngine": "v1"});

    // Future observations over 70 days to cover up to 60D horizon
    let mut future_obs = Vec::new();

    // Day 0 (exactly T, should be ignored)
    future_obs.push(get_mock_obs(eval_time, inst_id, 99.0, 99.5, 98.5));

    // Day 2 (entry)
    future_obs.push(get_mock_obs(
        eval_time + chrono::Duration::days(2),
        inst_id,
        100.0,
        101.0,
        99.0,
    ));

    // Day 8 (between 5D and 10D)
    future_obs.push(get_mock_obs(
        eval_time + chrono::Duration::days(8),
        inst_id,
        105.0,
        106.0,
        104.0,
    ));

    // Day 15 (between 10D and 20D)
    future_obs.push(get_mock_obs(
        eval_time + chrono::Duration::days(15),
        inst_id,
        112.0,
        115.0,
        110.0,
    )); // Hits target!

    // Day 25 (between 20D and 60D)
    future_obs.push(get_mock_obs(
        eval_time + chrono::Duration::days(25),
        inst_id,
        85.0,
        90.0,
        80.0,
    )); // Hits stop!

    // Day 70 (past 60D)
    future_obs.push(get_mock_obs(
        eval_time + chrono::Duration::days(70),
        inst_id,
        200.0,
        200.0,
        200.0,
    ));

    // ---------------------------------------------------------
    // HORIZON MATRIX
    // ---------------------------------------------------------
    // Generate outcomes for all 4 horizons from the EXACT same frozen strategy
    let out_5d = engine.measure_outcome(
        decision_id,
        &strategy,
        &strat_meta,
        &future_obs,
        eval_time,
        5,
        Some(inst_id),
    );
    let out_10d = engine.measure_outcome(
        decision_id,
        &strategy,
        &strat_meta,
        &future_obs,
        eval_time,
        10,
        Some(inst_id),
    );
    let out_20d = engine.measure_outcome(
        decision_id,
        &strategy,
        &strat_meta,
        &future_obs,
        eval_time,
        20,
        Some(inst_id),
    );
    let out_60d = engine.measure_outcome(
        decision_id,
        &strategy,
        &strat_meta,
        &future_obs,
        eval_time,
        60,
        Some(inst_id),
    );

    // Verify matrix fields
    assert_eq!(out_5d.horizon, "5D");
    assert_eq!(out_10d.horizon, "10D");
    assert_eq!(out_20d.horizon, "20D");
    assert_eq!(out_60d.horizon, "60D");

    assert_eq!(
        out_5d.strategy_id, strat_meta.artifact_id,
        "Same Strategy -> 5D"
    );
    assert_eq!(
        out_10d.strategy_id, strat_meta.artifact_id,
        "Same Strategy -> 10D"
    );
    assert_eq!(
        out_20d.strategy_id, strat_meta.artifact_id,
        "Same Strategy -> 20D"
    );
    assert_eq!(
        out_60d.strategy_id, strat_meta.artifact_id,
        "Same Strategy -> 60D"
    );

    // ---------------------------------------------------------
    // ARTIFACT & DETERMINISM
    // ---------------------------------------------------------
    // Outcome identity != Decision identity
    assert_ne!(out_5d.metadata.artifact_id, decision_id);
    assert_ne!(out_5d.metadata.artifact_id, strat_meta.artifact_id);

    // Different horizons -> Different content hash
    assert_ne!(out_5d.metadata.content_hash, out_10d.metadata.content_hash);
    assert_ne!(out_10d.metadata.content_hash, out_20d.metadata.content_hash);

    // Same measurement -> Same content hash, but new UUID
    let out_20d_b = engine.measure_outcome(
        decision_id,
        &strategy,
        &strat_meta,
        &future_obs,
        eval_time,
        20,
        Some(inst_id),
    );
    assert_ne!(
        out_20d.metadata.artifact_id, out_20d_b.metadata.artifact_id,
        "New evaluation = New artifact UUID"
    );
    assert_eq!(
        out_20d.metadata.content_hash, out_20d_b.metadata.content_hash,
        "Same inputs = Same content hash"
    );

    assert_eq!(out_5d.metadata.artifact_schema_version, "1.0.0");

    // ---------------------------------------------------------
    // LINEAGE
    // ---------------------------------------------------------
    for out in [&out_5d, &out_10d, &out_20d, &out_60d] {
        assert!(
            out.metadata.lineage.parent_artifacts.contains(&decision_id),
            "Decision lineage"
        );
        assert!(
            out.metadata
                .lineage
                .parent_artifacts
                .contains(&strat_meta.artifact_id),
            "Strategy lineage"
        );
        assert_eq!(
            out.metadata.replay_context_hash, "mock_replay_hash_123",
            "Replay context"
        );
        assert_eq!(
            out.metadata.knowledge_lake_version, "v14",
            "Knowledge Lake version"
        );
        assert_eq!(
            out.metadata.engine_versions["OutcomeEngine"], "v1",
            "Engine version"
        );
    }

    // ---------------------------------------------------------
    // TEMPORAL FIREWALL
    // ---------------------------------------------------------
    // 5D: Observation on Day 0 is ignored. Observations on Day 8, 15, 25, 70 are beyond T+5D.
    // Max observation end must be <= T+5D.
    assert!(out_5d.observation_end_timestamp > eval_time);
    assert!(out_5d.observation_end_timestamp <= eval_time + chrono::Duration::days(5));

    // 10D: Includes Day 2, 8. Excludes Day 15, 25, 70.
    assert!(out_10d.observation_end_timestamp > eval_time);
    assert!(out_10d.observation_end_timestamp <= eval_time + chrono::Duration::days(10));

    // 20D: Includes Day 2, 8, 15 (Target hit!). Stops early.
    assert!(out_20d.observation_end_timestamp > eval_time);
    assert!(out_20d.observation_end_timestamp <= eval_time + chrono::Duration::days(20));
    assert_eq!(out_20d.exit_reason, "Target Hit");

    // 60D: Target already hit on day 15, so observation ends early.
    assert!(out_60d.observation_end_timestamp <= eval_time + chrono::Duration::days(60));
    assert_eq!(out_60d.exit_reason, "Target Hit");

    // ---------------------------------------------------------
    // PERSISTENCE (Integration)
    // ---------------------------------------------------------
    if has_db {
        let repo = PostgresKnowledgeRepository::new(pool_res.unwrap());

        let store_res = repo.store(&out_10d).await;
        assert!(
            store_res.is_ok(),
            "Failed to store outcome: {:?}",
            store_res.err()
        );

        let dup_res = repo.store(&out_10d).await;
        assert!(dup_res.is_err(), "Duplicate UUID must be rejected");

        let fetched: OutcomeRecord = repo
            .get(out_10d.metadata.artifact_id)
            .await
            .unwrap()
            .expect("Outcome not found");
        assert_eq!(
            fetched.metadata.content_hash, out_10d.metadata.content_hash,
            "Round-trip hash"
        );
        assert_eq!(
            fetched.outcome_return, out_10d.outcome_return,
            "Round-trip payload"
        );
        assert_eq!(
            fetched.metadata.knowledge_lake_version, out_10d.metadata.knowledge_lake_version,
            "Round-trip metadata"
        );
    }

    // ---------------------------------------------------------
    // RESEARCH BOUNDARY
    // ---------------------------------------------------------
    // Asserted architecturally: OutcomeEngine operates purely on Production structs.
    // Research will only load `OutcomeRecord` from the database.

    println!("Phase D Final Acceptance Gate PASSED");
}

#[tokio::test]
async fn test_production_integration_replay_to_knowledge_lake() {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://invalid:invalid@localhost:5432/invalid".to_string());
    let pool_res = PgPool::connect(&db_url).await;
    let has_db = pool_res.is_ok();
    if !has_db {
        println!("Skipping DB integration test because DB is not available.");
        return;
    }

    let pool = pool_res.unwrap();
    let repo = PostgresKnowledgeRepository::new(pool);

    // Simulate a Time Machine Replay
    let n_decisions = 2;
    // Let's assume 1 generated a Strategy, 1 did not (was skipped).
    let m_strategies = 1;

    let engine = OutcomeEngine;
    let inst_id = Uuid::new_v4();
    let eval_time = Utc.timestamp_opt(1705312800, 0).unwrap();

    // Mock future data
    let mut future_obs = Vec::new();
    future_obs.push(get_mock_obs(
        eval_time + chrono::Duration::days(1),
        inst_id,
        100.0,
        101.0,
        99.0,
    ));
    future_obs.push(get_mock_obs(
        eval_time + chrono::Duration::days(10),
        inst_id,
        115.0,
        116.0,
        114.0,
    ));

    let mut persisted_outcomes = 0;

    // Simulate loop over decisions
    for i in 0..n_decisions {
        let decision_id = Uuid::new_v4();
        if i == 0 {
            // Did not generate strategy
            continue;
        }

        let strategy = get_mock_strategy(decision_id);
        let mut strat_meta = ArtifactMetadata::mock();
        strat_meta.artifact_id = Uuid::new_v4();
        strat_meta.artifact_type = ArtifactType::Decision;

        for horizon in [5, 10, 20, 60] {
            let outcome = engine.measure_outcome(
                decision_id,
                &strategy,
                &strat_meta,
                &future_obs,
                eval_time,
                horizon,
                Some(inst_id),
            );

            // Persist to knowledge lake
            let store_res = repo.store(&outcome).await;
            if store_res.is_ok() {
                persisted_outcomes += 1;
            }

            // Read back to verify
            let fetched: OutcomeRecord = repo
                .get(outcome.metadata.artifact_id)
                .await
                .unwrap()
                .expect("Outcome not found");
            assert_eq!(fetched.metadata.artifact_id, outcome.metadata.artifact_id);
        }
    }

    assert_eq!(
        persisted_outcomes,
        4 * m_strategies,
        "Persisted outcome artifacts must be 4 x M"
    );
    println!("Replay -> Outcome -> Knowledge Lake Integration PASSED");
}
