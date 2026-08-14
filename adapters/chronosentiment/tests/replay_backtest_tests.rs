use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::backtest::{
    run_replay_backtest, DecisionLedger, ReplayTick,
};
use chronosentiment_adapter::decision_support::policy::{BaselineTrendMappingPolicy, DecisionPolicy};
use chronosentiment_adapter::decision_support::replay::{
    decide_from_inputs, DecideAt, ReplayAssessment, ReplayError, ReplayInputs, ReplayObservation,
    UNFROZEN_ENGINE_VERSION,
};
use chronosentiment_adapter::metrics::concepts::Concept;
use chronosentiment_adapter::reasoning::assessment::AssessmentEngine;
use coralys_moga::runtime::optimization::metric::{MetricReport, MetricValue};
use uuid::Uuid;

fn t(day: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2021, 10, day, 15, 30, 0).unwrap()
}

fn lake(instrument_id: Uuid) -> InMemoryLake {
    let mut metrics = MetricReport::default();
    metrics
        .metrics
        .insert("ma_20".to_string(), MetricValue::Float(2100.0));
    metrics
        .metrics
        .insert("ma_50".to_string(), MetricValue::Float(2050.0));
    let mut profile = AssessmentEngine.assess_at(&metrics, &[Concept::Trend], t(31), Some(instrument_id));
    let id = Uuid::from_u128(1);
    profile.metadata.artifact_id = id;
    InMemoryLake {
        assessments: vec![ReplayAssessment {
            id,
            evaluation_timestamp: t(31),
            signature_hash: profile.to_hash(),
            profile,
        }],
    }
}

struct InMemoryLake {
    assessments: Vec<ReplayAssessment>,
}

#[async_trait::async_trait]
impl DecideAt for InMemoryLake {
    async fn decide_at(
        &self,
        as_of: chrono::DateTime<Utc>,
        instrument_id: Uuid,
        engine_version: &str,
        policy: &dyn DecisionPolicy,
    ) -> Result<chronosentiment_adapter::decision_support::TradingDecision, ReplayError> {
        decide_from_inputs(
            ReplayInputs {
            instrument_id,
            as_of,
            engine_version: engine_version.to_string(),
            produced_by: chronosentiment_adapter::decision_support::replay::REPLAY_PRODUCER
                .to_string(),
            assessments: self.assessments.clone(),
            lake_decisions: vec![],
            observations: vec![ReplayObservation {
                id: Uuid::from_u128(11),
                effective_from: t(31),
            }],
        },
            policy,
        )
    }
}

#[tokio::test]
async fn iterates_ticks_into_append_only_ledger() {
    let instrument_id = Uuid::from_u128(7);
    let adapter = lake(instrument_id);
    let ticks = [
        ReplayTick {
            as_of: t(31),
            instrument_id,
        },
        ReplayTick {
            as_of: t(31) + chrono::Duration::days(1),
            instrument_id,
        },
        ReplayTick {
            as_of: t(31) + chrono::Duration::days(2),
            instrument_id,
        },
    ];
    let a = run_replay_backtest(&adapter, &ticks, UNFROZEN_ENGINE_VERSION, &BaselineTrendMappingPolicy)
        .await
        .unwrap();
    let b = run_replay_backtest(&adapter, &ticks, UNFROZEN_ENGINE_VERSION, &BaselineTrendMappingPolicy)
        .await
        .unwrap();

    assert_eq!(a.records.len(), 3);
    assert_eq!(a.identity_hash(), b.identity_hash());
    assert_eq!(a.records[0].decision_id, b.records[0].decision_id);
    assert_eq!(a.records[0].sequence, 1);
    assert_eq!(a.records[0].as_of_timestamp, a.records[0].decision_timestamp);
    assert_eq!(a.engine_version, UNFROZEN_ENGINE_VERSION);
    assert!(serde_json::to_value(&a).unwrap().get("outcome_return").is_none());
    assert!(serde_json::to_value(&a).unwrap()["records"][0]
        .get("outcome_return")
        .is_none());
}

#[tokio::test]
async fn later_ticks_do_not_mutate_earlier_records() {
    let instrument_id = Uuid::from_u128(7);
    let adapter = lake(instrument_id);
    let first_tick = [ReplayTick {
        as_of: t(31),
        instrument_id,
    }];
    let prefix = run_replay_backtest(&adapter, &first_tick, UNFROZEN_ENGINE_VERSION, &BaselineTrendMappingPolicy)
        .await
        .unwrap();
    let all = run_replay_backtest(
        &adapter,
        &[
            ReplayTick {
                as_of: t(31),
                instrument_id,
            },
            ReplayTick {
                as_of: t(31) + chrono::Duration::days(1),
                instrument_id,
            },
        ],
        UNFROZEN_ENGINE_VERSION,
        &BaselineTrendMappingPolicy,
    )
    .await
    .unwrap();

    assert_eq!(prefix.records[0].decision_id, all.records[0].decision_id);
    assert_eq!(prefix.records[0].content_hash, all.records[0].content_hash);
    assert_eq!(all.records.len(), 2);
}

#[tokio::test]
async fn ledger_is_append_only_sequence() {
    let instrument_id = Uuid::from_u128(7);
    let adapter = lake(instrument_id);
    let decision = adapter
        .decide_at(t(31), instrument_id, UNFROZEN_ENGINE_VERSION, &BaselineTrendMappingPolicy)
        .await
        .unwrap();
    let mut ledger = DecisionLedger::new(UNFROZEN_ENGINE_VERSION);
    ledger.append(decision.clone());
    ledger.append(decision);
    assert_eq!(ledger.records[0].sequence, 1);
    assert_eq!(ledger.records[1].sequence, 2);
}
