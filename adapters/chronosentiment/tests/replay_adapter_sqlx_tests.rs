//! Integration test against a disposable B4 restore.
//!
//! Do not set DATABASE_URL to `chrono_b3_test` or `chrono_b4_test`.
//! Official runner: `./run_replay_b4_validate.sh`

use chrono::Duration;
use chronosentiment_adapter::decision_support::replay::{
    ReplayAdapter, UNFROZEN_ENGINE_VERSION,
};
use chronosentiment_adapter::metrics::concepts::Concept;
use chronosentiment_adapter::reasoning::assessment::AssessmentEngine;
use chronosentiment_adapter::repository::knowledge::KnowledgeArtifact;
use coralys_moga::runtime::optimization::metric::{MetricReport, MetricValue};
use sqlx::Row;
use uuid::Uuid;

fn forbidden_db(name: &str) -> bool {
    matches!(name, "chrono_b3_test" | "chrono_b4_test")
}

#[tokio::test]
async fn decide_at_is_read_only_deterministic_and_ignores_future_and_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    let url = match std::env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(_) => {
            if std::env::var("REPLAY_REQUIRE_B4").ok().as_deref() == Some("1") {
                panic!("REPLAY_REQUIRE_B4=1 but DATABASE_URL is not set");
            }
            eprintln!("SKIP: DATABASE_URL not set (use ./run_replay_b4_validate.sh)");
            return Ok(());
        }
    };

    let pool = sqlx::PgPool::connect(&url).await?;
    let dbname: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&pool)
        .await?;
    assert!(
        !forbidden_db(&dbname),
        "refusing to run against certified database {dbname}"
    );

    let row = sqlx::query(
        r#"
        SELECT instrument_id, evaluation_timestamp
        FROM knowledge_assessments
        WHERE instrument_id IS NOT NULL
        ORDER BY evaluation_timestamp ASC, id ASC
        LIMIT 1
        "#,
    )
    .fetch_one(&pool)
    .await?;
    let instrument_id: Uuid = row.try_get("instrument_id")?;
    let t: chrono::DateTime<chrono::Utc> = row.try_get("evaluation_timestamp")?;

    let adapter = ReplayAdapter::new(pool.clone());
    let first = adapter
        .decide_at(t, instrument_id, UNFROZEN_ENGINE_VERSION)
        .await?;
    let second = adapter
        .decide_at(t, instrument_id, UNFROZEN_ENGINE_VERSION)
        .await?;

    assert_eq!(first.as_of_timestamp, t);
    assert_eq!(first.engine_version, UNFROZEN_ENGINE_VERSION);
    assert_eq!(first.decision_id, second.decision_id);
    assert_eq!(first.provenance.content_hash, second.provenance.content_hash);
    assert!(first.lineage.assessment_id.is_some());
    assert!(!first.lineage.input_set_hash.is_empty());
    assert!(matches!(
        first.action,
        chronosentiment_adapter::decision_support::DecisionAction::Long
            | chronosentiment_adapter::decision_support::DecisionAction::Short
            | chronosentiment_adapter::decision_support::DecisionAction::NoTrade
    ));

    let future_dt = t + Duration::days(365);
    let mut future_metrics = MetricReport::default();
    future_metrics
        .metrics
        .insert("ma_20".to_string(), MetricValue::Float(1800.0));
    future_metrics
        .metrics
        .insert("ma_50".to_string(), MetricValue::Float(2050.0));
    let future = AssessmentEngine.assess_at(
        &future_metrics,
        &[Concept::Trend],
        future_dt,
        Some(instrument_id),
    );
    let future_assessment_id = future.metadata().artifact_id;

    sqlx::query(
        r#"
        INSERT INTO knowledge_assessments (
            id, instrument_id, evaluation_timestamp,
            signature, signature_hash, metadata_json, profile_json
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(future_assessment_id)
    .bind(instrument_id)
    .bind(future_dt)
    .bind(serde_json::Value::String(future.to_signature()))
    .bind(future.to_hash())
    .bind(serde_json::to_value(future.metadata())?)
    .bind(serde_json::to_value(&future)?)
    .execute(&pool)
    .await?;

    let future_decision_id = Uuid::from_u128(0xB4F07_0000_0002);
    sqlx::query(
        r#"
        INSERT INTO knowledge_decisions (
            id, instrument_id, evaluation_timestamp, opportunity,
            metadata_json, decision_json, assessment_id
        ) VALUES ($1, $2, $3, 'Positive', '{}'::jsonb, '{}'::jsonb, $4)
        "#,
    )
    .bind(future_decision_id)
    .bind(instrument_id)
    .bind(future_dt)
    .bind(future_assessment_id)
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO knowledge_outcomes (
            id, decision_id, strategy_id, instrument_id,
            evaluation_timestamp, horizon, horizon_expiry_timestamp, observation_end_timestamp,
            entry_reached, target_hit, stop_hit, exit_reason,
            outcome_return, mfe, mae, drawdown,
            metadata_json, outcome_json
        ) VALUES (
            $1, $2, $3, $4, $5, '5D', $6, $6,
            true, true, false, 'Target',
            0.42, 0.5, -0.1, 0.05,
            '{}'::jsonb, '{"outcome_return":0.42}'::jsonb
        )
        "#,
    )
    .bind(Uuid::from_u128(0xB4F07_0000_0004))
    .bind(future_decision_id)
    .bind(Uuid::from_u128(0xB4F07_0000_0003))
    .bind(instrument_id)
    .bind(future_dt)
    .bind(future_dt + Duration::days(5))
    .execute(&pool)
    .await?;

    let after = adapter
        .decide_at(t, instrument_id, UNFROZEN_ENGINE_VERSION)
        .await?;
    assert_eq!(first.decision_id, after.decision_id);
    assert_eq!(first.provenance.content_hash, after.provenance.content_hash);
    assert_eq!(first.action, after.action);
    assert!(!after
        .lineage
        .consumed_artifact_ids
        .contains(&future_assessment_id));
    assert!(!after
        .lineage
        .consumed_artifact_ids
        .contains(&future_decision_id));
    Ok(())
}
