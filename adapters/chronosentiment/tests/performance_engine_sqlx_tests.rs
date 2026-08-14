use chronosentiment_adapter::decision_support::backtest::populate_ledger_from_assessment_schedule;
use chronosentiment_adapter::decision_support::outcome::OutcomeEngine;
use chronosentiment_adapter::decision_support::performance::{measure_performance, SCHEMA_VERSION};
use chronosentiment_adapter::decision_support::replay::{ReplayAdapter, UNFROZEN_ENGINE_VERSION};

fn forbidden_db(name: &str) -> bool {
    matches!(name, "chrono_b3_test" | "chrono_b4_test")
}

#[tokio::test]
async fn b4_performance_is_deterministic_and_keeps_no_trade_separate()
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
    let outcomes = OutcomeEngine::new(pool).measure_ledger(&ledger).await?;
    let ledger_hash = ledger.identity_hash();
    let outcome_hash = outcomes.identity_hash();

    let first = measure_performance(&ledger, &outcomes);
    let second = measure_performance(&ledger, &outcomes);

    assert_eq!(first.content_hash, second.content_hash);
    assert_eq!(first.schema_version, SCHEMA_VERSION);
    assert_eq!(first.decision_engine_version, UNFROZEN_ENGINE_VERSION);
    assert_eq!(first.ledger_identity_hash, ledger_hash);
    assert_eq!(first.outcome_identity_hash, outcome_hash);
    assert_eq!(ledger.identity_hash(), ledger_hash);
    assert_eq!(outcomes.identity_hash(), outcome_hash);

    let counts = &first.behavior.counts;
    assert_eq!(
        counts.long + counts.short + counts.no_trade,
        first.behavior.n_records
    );
    assert_eq!(first.behavior.n_records, ledger.records.len() as u32);
    assert_eq!(first.horizons.len(), 4);

    for slice in &first.horizons {
        assert_eq!(
            slice.trading.returns.n_decisions,
            counts.long + counts.short
        );
        assert_eq!(slice.opportunity.returns.n_decisions, counts.no_trade);
        assert_eq!(
            slice.trading.returns.n_observed + slice.opportunity.returns.n_observed,
            slice.by_action.long.n_observed
                + slice.by_action.short.n_observed
                + slice.by_action.no_trade.n_observed
        );
        assert_eq!(
            slice.trading.returns.n_observed,
            slice.by_action.long.n_observed + slice.by_action.short.n_observed
        );
    }
    Ok(())
}
