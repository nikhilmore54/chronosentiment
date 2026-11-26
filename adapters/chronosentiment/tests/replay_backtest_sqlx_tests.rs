use chronosentiment_adapter::decision_support::backtest::populate_ledger_from_assessment_schedule;
use chronosentiment_adapter::decision_support::policy::BaselineTrendMappingPolicy;
use chronosentiment_adapter::decision_support::replay::{ReplayAdapter, UNFROZEN_ENGINE_VERSION};

fn forbidden_db(name: &str) -> bool {
    matches!(name, "chrono_b3_test" | "chrono_b4_test")
}

#[tokio::test]
async fn b4_schedule_populates_deterministic_append_only_ledger(
) -> Result<(), Box<dyn std::error::Error>> {
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

    let adapter = ReplayAdapter::new(pool);
    let first = populate_ledger_from_assessment_schedule(
        &adapter,
        UNFROZEN_ENGINE_VERSION,
        &BaselineTrendMappingPolicy,
    )
    .await?;
    let second = populate_ledger_from_assessment_schedule(
        &adapter,
        UNFROZEN_ENGINE_VERSION,
        &BaselineTrendMappingPolicy,
    )
    .await?;

    assert!(!first.records.is_empty(), "B4 schedule must not be empty");
    assert_eq!(first.records.len(), second.records.len());
    assert_eq!(first.identity_hash(), second.identity_hash());
    assert_eq!(first.engine_version, UNFROZEN_ENGINE_VERSION);

    for i in 1..first.records.len() {
        let prev = &first.records[i - 1];
        let cur = &first.records[i];
        assert!(
            prev.as_of_timestamp < cur.as_of_timestamp
                || (prev.as_of_timestamp == cur.as_of_timestamp
                    && prev.instrument_id <= cur.instrument_id)
        );
        assert_eq!(cur.sequence, (i as u32) + 1);
        assert_eq!(cur.decision_timestamp, cur.as_of_timestamp);
        assert_eq!(cur.engine_version, UNFROZEN_ENGINE_VERSION);
        assert!(!cur.input_set_hash.is_empty());
        assert!(
            !cur.lineage.consumed_artifact_ids.is_empty() || cur.lineage.assessment_id.is_some()
        );
    }

    let json = serde_json::to_value(&first)?;
    assert!(json.get("outcome_return").is_none());
    assert!(json["records"][0].get("outcome_return").is_none());
    Ok(())
}
