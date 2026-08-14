use chrono::Duration;
use chronosentiment_adapter::decision_support::backtest::populate_ledger_from_assessment_schedule;
use chronosentiment_adapter::decision_support::outcome::OutcomeEngine;
use chronosentiment_adapter::decision_support::replay::{ReplayAdapter, UNFROZEN_ENGINE_VERSION};
use uuid::Uuid;

fn forbidden_db(name: &str) -> bool {
    matches!(name, "chrono_b3_test" | "chrono_b4_test")
}

#[tokio::test]
async fn b4_outcomes_are_deterministic_and_ignore_future_rows()
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
    assert!(!forbidden_db(&dbname), "refusing certified database {dbname}");

    let adapter = ReplayAdapter::new(pool.clone());
    let ledger =
        populate_ledger_from_assessment_schedule(&adapter, UNFROZEN_ENGINE_VERSION).await?;
    let engine = OutcomeEngine::new(pool.clone());
    let first = engine.measure_ledger(&ledger).await?;
    let second = engine.measure_ledger(&ledger).await?;
    assert_eq!(first.bundles.len(), ledger.records.len());
    assert_eq!(first.identity_hash(), second.identity_hash());
    assert_eq!(first.bundles[0].ledger_decision_id, ledger.records[0].decision_id);
    assert!(first.bundles.iter().all(|b| b.horizons.len() == 4));
    assert!(
        first
            .bundles
            .iter()
            .any(|b| b.horizons.iter().any(|h| h.available)),
        "B4 restore must yield at least one attached 5/10/20/60D outcome"
    );
    let ledger_hash_before = ledger.identity_hash();

    let rec = &ledger.records[0];
    let last_as_of = ledger
        .records
        .iter()
        .map(|r| r.as_of_timestamp)
        .max()
        .expect("ledger is non-empty");
    let future_dt = last_as_of + Duration::days(365);
    let future_decision = Uuid::from_u128(0x0A7_0000_0001);
    let future_assessment = Uuid::from_u128(0x0A7_0000_0002);
    sqlx::query(
        r#"
        INSERT INTO knowledge_assessments (
            id, instrument_id, evaluation_timestamp,
            signature, signature_hash, metadata_json, profile_json
        ) VALUES ($1, $2, $3, '"x"', 'x', '{}'::jsonb, '{}'::jsonb)
        "#,
    )
    .bind(future_assessment)
    .bind(rec.instrument_id)
    .bind(future_dt)
    .execute(&pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO knowledge_decisions (
            id, instrument_id, evaluation_timestamp, opportunity,
            metadata_json, decision_json, assessment_id
        ) VALUES ($1, $2, $3, 'Positive', '{}'::jsonb, '{}'::jsonb, $4)
        "#,
    )
    .bind(future_decision)
    .bind(rec.instrument_id)
    .bind(future_dt)
    .bind(future_assessment)
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
            9.99, 1.0, -1.0, 1.0,
            '{}'::jsonb, '{}'::jsonb
        )
        "#,
    )
    .bind(Uuid::from_u128(0x0A7_0000_0004))
    .bind(future_decision)
    .bind(Uuid::from_u128(0x0A7_0000_0003))
    .bind(rec.instrument_id)
    .bind(future_dt)
    .bind(future_dt + Duration::days(5))
    .execute(&pool)
    .await?;

    let after = engine.measure_ledger(&ledger).await?;
    assert_eq!(first.identity_hash(), after.identity_hash());
    assert_eq!(ledger.identity_hash(), ledger_hash_before);
    assert!(!after.bundles[0]
        .horizons
        .iter()
        .any(|h| h.outcome_return == Some(9.99)));
    Ok(())
}
