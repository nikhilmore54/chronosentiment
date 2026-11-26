use chrono::{TimeZone, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use chronosentiment_adapter::reasoning::strategy::Horizon;
use chronosentiment_adapter::research::dataset::{ArtifactPopulation, DateRange, ResearchDataset};
use chronosentiment_adapter::research::experiment::ResearchExperiment;
use chronosentiment_adapter::research::predictive_value::PredictiveValueExperiment;

#[sqlx::test]
async fn test_phase_g_predictive_value_experiment(
    pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Run migrations to ensure schema
    sqlx::migrate!("./migrations").run(&pool).await?;

    // We can insert some mock data into the database to verify the SQL logic.
    let instrument_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO instruments (id, exchange, display_symbol) VALUES ($1, 'TEST', 'PHASE_G')",
    )
    .bind(instrument_id)
    .execute(&pool)
    .await?;

    let assessment_id = Uuid::new_v4();
    let decision_id = Uuid::new_v4();
    let strategy_id = Uuid::new_v4();
    let outcome_id = Uuid::new_v4();

    let eval_ts = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();

    let signature = "Trend=Bullish;Strength=Strong;Phase=Early";
    let signature_hash = "hash_xyz";

    // 1. Insert Assessment
    sqlx::query(
        r#"
        INSERT INTO knowledge_assessments (
            id, instrument_id, evaluation_timestamp,
            signature, signature_hash, metadata_json, profile_json
        ) VALUES ($1, $2, $3, $4, $5, '{}', '{}')
        "#,
    )
    .bind(assessment_id)
    .bind(instrument_id)
    .bind(eval_ts)
    .bind(serde_json::Value::String(signature.to_string()))
    .bind(signature_hash)
    .execute(&pool)
    .await?;

    // 2. Insert Decision with proper lineage
    let decision_metadata = serde_json::json!({
        "lineage": {
            "parent_artifacts": [assessment_id.to_string()]
        }
    });
    sqlx::query(
        r#"
        INSERT INTO knowledge_decisions (
            id, instrument_id, evaluation_timestamp, opportunity, metadata_json, decision_json
        ) VALUES ($1, $2, $3, 'Positive', $4, '{}')
        "#,
    )
    .bind(decision_id)
    .bind(instrument_id)
    .bind(eval_ts)
    .bind(decision_metadata)
    .execute(&pool)
    .await?;

    // 3. Insert Strategy with proper lineage
    let strategy_metadata = serde_json::json!({
        "lineage": {
            "parent_artifacts": [decision_id.to_string()]
        }
    });
    sqlx::query(
        r#"
        INSERT INTO knowledge_strategies (
            id, decision_id, expected_horizon, metadata_json, strategy_json
        ) VALUES ($1, $2, '5D', $3, '{}')
        "#,
    )
    .bind(strategy_id)
    .bind(decision_id)
    .bind(strategy_metadata)
    .execute(&pool)
    .await?;

    // 4. Insert Outcome
    sqlx::query(
        r#"
        INSERT INTO knowledge_outcomes (
            id, decision_id, strategy_id, instrument_id,
            evaluation_timestamp, horizon, horizon_expiry_timestamp, observation_end_timestamp,
            entry_reached, target_hit, stop_hit, exit_reason,
            outcome_return, mfe, mae, drawdown,
            metadata_json, outcome_json
        ) VALUES ($1, $2, $3, $4, $5, '5D', $5, $5, true, true, false, 'Target', 0.05, 0.06, -0.01, -0.01, '{}', '{}')
        "#
    )
    .bind(outcome_id)
    .bind(decision_id)
    .bind(strategy_id)
    .bind(instrument_id)
    .bind(eval_ts)
    .execute(&pool)
    .await?;

    // Run the Phase G Experiment
    let experiment = PredictiveValueExperiment::new(pool.clone());

    let dataset = ResearchDataset::new(
        "Phase G Dataset".to_string(),
        "1.0".to_string(),
        serde_json::json!("Nifty50"),
        DateRange {
            start: eval_ts,
            end: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
        },
        vec![Horizon::Swing],
        serde_json::json!([]),
        serde_json::json!([]),
        ArtifactPopulation {
            artifact_types: vec!["Outcome".to_string()],
            population_rules: serde_json::json!({}),
        },
    );

    let measurements = experiment.execute(&dataset).await.unwrap();

    // Verify findings structure
    assert_eq!(measurements.findings.len(), 3);

    let aggregate_matrix = &measurements.findings[0]["data"];
    assert!(aggregate_matrix.is_array());
    let agg = &aggregate_matrix[0];
    assert_eq!(agg["N"], 1);
    assert_eq!(agg["signature"], signature);
    assert_eq!(agg["horizon"], "5D");
    assert_eq!(agg["target_pct"], 1.0); // 1 out of 1 targets hit

    let raw_ledger = &measurements.findings[1]["data"];
    assert!(raw_ledger.is_array());
    let raw = &raw_ledger[0];
    assert_eq!(raw["assessment_id"], assessment_id.to_string());
    assert_eq!(raw["decision_id"], decision_id.to_string());
    assert_eq!(raw["strategy_id"], strategy_id.to_string());
    assert_eq!(raw["outcome_id"], outcome_id.to_string());
    assert_eq!(raw["outcome_return"], 0.05);

    let pop_acc = &measurements.findings[2]["data"];
    assert!(pop_acc.is_array());
    let pop = &pop_acc[0];
    assert_eq!(pop["5D_N"], 1);
    assert_eq!(pop["5D_Entry"], 1);

    Ok(())
}
