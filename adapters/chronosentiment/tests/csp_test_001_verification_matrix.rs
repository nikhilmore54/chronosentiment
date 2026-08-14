//! CS-P-TEST-001 — Decision Intelligence Verification Matrix.
//!
//! Product-vision contracts. Not a performance experiment. Not G-GATE. Not v1.0.

use std::collections::BTreeMap;

use chrono::{TimeZone, Utc};
use coralys_moga::runtime::optimization::metric::{MetricEngine, MetricReport, MetricValue};
use chronosentiment_adapter::decision_support::backtest::{
    run_replay_backtest, DecisionLedger, ReplayTick,
};
use chronosentiment_adapter::decision_support::enrichment_certify::assess_from_bars_at_t;
use chronosentiment_adapter::decision_support::outcome::{
    measure_record, DecisionOutcomeBundle, LakeOutcomeRow, OutcomeReport, HORIZON_DAYS,
};
use chronosentiment_adapter::decision_support::performance::measure_performance;
use chronosentiment_adapter::decision_support::policy::{DecisionPolicy, TrendMappingPolicy};
use chronosentiment_adapter::decision_support::replay::{
    decide_from_inputs, DecideAt, ReplayAssessment, ReplayError, ReplayInputs, ReplayLakeDecision,
    ReplayObservation, UNFROZEN_ENGINE_VERSION,
};
use chronosentiment_adapter::decision_support::{
    ConfidenceStatus, DecisionAction, DecisionLineage, TradingDecision,
};
use chronosentiment_adapter::ingestion::yahoo::YahooHistoricalBar;
use chronosentiment_adapter::metrics::concepts::Concept;
use chronosentiment_adapter::metrics::instrument::{
    AverageTrueRangeMetric, InstrumentMetricEngine, RateOfChangeMetric, SimpleMovingAverageMetric,
};
use chronosentiment_adapter::observation::ValidatedObservation;
use chronosentiment_adapter::reasoning::assessment::{
    AssessmentEngine, AssessmentProfile, Direction, ENRICHMENT_CONCEPTS, FactorAvailability,
};
use chronosentiment_adapter::validation::context::InstrumentEvaluationContext;
use uuid::Uuid;

fn t() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2021, 10, 31, 15, 30, 0).unwrap()
}

fn inst() -> Uuid {
    Uuid::from_u128(7)
}

fn trend_metrics(bullish: bool) -> MetricReport {
    let mut metrics = MetricReport::default();
    if bullish {
        metrics.metrics.insert("ma_20".to_string(), MetricValue::Float(2100.0));
        metrics.metrics.insert("ma_50".to_string(), MetricValue::Float(2050.0));
    } else {
        metrics.metrics.insert("ma_20".to_string(), MetricValue::Float(1900.0));
        metrics.metrics.insert("ma_50".to_string(), MetricValue::Float(2050.0));
    }
    metrics
}

fn full_metrics(bullish: bool) -> MetricReport {
    let mut metrics = trend_metrics(bullish);
    metrics.metrics.insert("roc_20".to_string(), MetricValue::Float(8.3));
    metrics.metrics.insert("atr_14".to_string(), MetricValue::Float(12.0));
    metrics
}

fn profile_at(metrics: &MetricReport, dt: chrono::DateTime<Utc>) -> AssessmentProfile {
    AssessmentEngine.assess_at(metrics, &ENRICHMENT_CONCEPTS, dt, Some(inst()))
}

fn inputs_from(mut profile: AssessmentProfile) -> ReplayInputs {
    let id = Uuid::from_u128(1);
    profile.metadata.artifact_id = id;
    ReplayInputs {
        instrument_id: inst(),
        as_of: t(),
        engine_version: UNFROZEN_ENGINE_VERSION.to_string(),
        produced_by: chronosentiment_adapter::decision_support::replay::REPLAY_PRODUCER.to_string(),
        assessments: vec![ReplayAssessment {
            id,
            evaluation_timestamp: profile.metadata.evaluation_timestamp,
            signature_hash: profile.to_hash(),
            profile,
        }],
        lake_decisions: vec![],
        observations: vec![ReplayObservation {
            id: Uuid::from_u128(11),
            effective_from: t(),
        }],
    }
}

fn decide(metrics: &MetricReport) -> TradingDecision {
    decide_from_inputs(inputs_from(profile_at(metrics, t()))).unwrap()
}

fn synthetic_bars() -> Vec<YahooHistoricalBar> {
    let start = Utc.with_ymd_and_hms(2021, 8, 1, 0, 0, 0).unwrap().timestamp();
    (0..80)
        .map(|i| {
            let close = 100.0 + i as f64;
            YahooHistoricalBar {
                timestamp: start + i * 86400,
                open: close,
                high: close + 1.0,
                low: close - 1.0,
                close,
                adj_close: close,
                volume: 1_000.0,
            }
        })
        .collect()
}

fn price_obs(id: u128, day_offset: i64, close: f64, high_low: Option<(f64, f64)>) -> ValidatedObservation {
    let ts = t() + chrono::Duration::days(day_offset);
    let (high, low) = high_low.unwrap_or((close + 1.0, close - 1.0));
    let payload = serde_json::json!({
        "open": close,
        "high": high,
        "low": low,
        "close": close,
        "volume": 1000.0
    });
    ValidatedObservation {
        id: Uuid::from_u128(id),
        research_session_id: None,
        instrument_id: Some(inst()),
        observation_type: "MarketPrice".to_string(),
        source: "test".to_string(),
        source_identifier: None,
        observed_at: ts,
        effective_from: ts,
        effective_to: None,
        recorded_at: ts,
        raw_payload: payload.clone(),
        normalized_payload: payload,
        confidence: 1.0,
        freshness: 0.0,
        coverage: "Full".to_string(),
        consistency: Some(1.0),
        quality_score: 1.0,
        provenance_hash: "hash".to_string(),
        schema_version: 1,
    }
}

fn engine() -> InstrumentMetricEngine {
    let mut engine = InstrumentMetricEngine::new();
    engine.add_model(Box::new(SimpleMovingAverageMetric::new(20)));
    engine.add_model(Box::new(SimpleMovingAverageMetric::new(50)));
    engine.add_model(Box::new(RateOfChangeMetric::new(20)));
    engine.add_model(Box::new(AverageTrueRangeMetric::new(14)));
    engine
}

fn identity_tuple(d: &TradingDecision) -> (Uuid, String, String, DecisionAction, Option<Uuid>) {
    (
        d.decision_id,
        d.provenance.content_hash.clone(),
        d.lineage.produced_by.clone(),
        d.action,
        d.lineage.assessment_id,
    )
}

#[test]
fn temp_001_future_observation_leaves_decision_bit_identical() {
    let clean = decide(&full_metrics(true));
    let mut dirty = inputs_from(profile_at(&full_metrics(true), t()));
    dirty.observations.push(ReplayObservation {
        id: Uuid::from_u128(99),
        effective_from: t() + chrono::Duration::days(1),
    });
    let attacked = decide_from_inputs(dirty).unwrap();
    assert_eq!(identity_tuple(&clean), identity_tuple(&attacked));
    assert_eq!(clean.action, DecisionAction::Long);
}

#[test]
fn temp_002_future_bar_does_not_enter_factor_or_decision() {
    let bars = synthetic_bars();
    let mut attacked = bars.clone();
    attacked.push(YahooHistoricalBar {
        timestamp: t().timestamp() + 86400,
        open: 9_999.0,
        high: 9_999.0,
        low: 9_999.0,
        close: 9_999.0,
        adj_close: 9_999.0,
        volume: 1.0,
    });
    let (clean_p, _, _) = assess_from_bars_at_t(&bars, t(), inst());
    let (dirty_p, _, max_from) = assess_from_bars_at_t(&attacked, t(), inst());
    assert!(max_from.unwrap() <= t());
    assert_eq!(clean_p.factor_status, dirty_p.factor_status);
    let a = decide_from_inputs(inputs_from(clean_p)).unwrap();
    let b = decide_from_inputs(inputs_from(dirty_p)).unwrap();
    assert_eq!(a.action, b.action);
    assert_eq!(a.decision_id, b.decision_id);
}

#[test]
fn temp_003_future_assessment_cannot_flip_action() {
    let mut inputs = inputs_from(profile_at(&full_metrics(true), t()));
    let future = profile_at(&full_metrics(false), t() + chrono::Duration::days(30));
    inputs.assessments.push(ReplayAssessment {
        id: future.metadata.artifact_id,
        evaluation_timestamp: future.metadata.evaluation_timestamp,
        signature_hash: future.to_hash(),
        profile: future,
    });
    let d = decide_from_inputs(inputs).unwrap();
    assert_eq!(d.action, DecisionAction::Long);
}

#[test]
fn temp_004_future_lake_decision_is_not_consumed() {
    let mut inputs = inputs_from(profile_at(&full_metrics(true), t()));
    let future_id = Uuid::from_u128(42);
    inputs.lake_decisions.push(ReplayLakeDecision {
        id: future_id,
        evaluation_timestamp: t() + chrono::Duration::days(1),
    });
    let d = decide_from_inputs(inputs).unwrap();
    assert!(!d.lineage.consumed_artifact_ids.contains(&future_id));
}

#[test]
fn temp_005_and_dec_002_outcome_cannot_enter_decision_identity() {
    let clean = decide(&full_metrics(true));
    let json = serde_json::to_value(&clean).unwrap();
    assert!(json.get("outcome_return").is_none());
    assert!(json.get("knowledge_outcomes").is_none());
    let _poison = LakeOutcomeRow {
        id: Uuid::from_u128(50),
        lake_decision_id: clean.decision_id,
        instrument_id: inst(),
        decision_as_of: t(),
        horizon: "5D".into(),
        outcome_return: 0.50,
        entry_reached: true,
        target_hit: true,
        stop_hit: false,
        exit_reason: "Target".into(),
        mfe: 0.5,
        mae: 0.0,
        drawdown: 0.0,
        horizon_expiry_timestamp: t() + chrono::Duration::days(5),
    };
    let again = decide(&full_metrics(true));
    assert_eq!(clean.decision_id, again.decision_id);
}

#[test]
fn fact_001_missing_roc_20_is_unavailable_not_zero() {
    let p = profile_at(&trend_metrics(true), t());
    let mom = p
        .factor_status
        .iter()
        .find(|s| s.concept == Concept::Momentum)
        .unwrap();
    assert_eq!(mom.availability, FactorAvailability::Unavailable);
    assert!(mom.missing_metrics.contains(&"roc_20".to_string()));
    assert!(!p.assessments.iter().any(|a| a.concept == Concept::Momentum));
}

#[test]
fn fact_002_missing_atr_14_is_unavailable_not_zero() {
    let mut metrics = trend_metrics(true);
    metrics.metrics.insert("roc_20".to_string(), MetricValue::Float(1.0));
    let p = profile_at(&metrics, t());
    let vol = p
        .factor_status
        .iter()
        .find(|s| s.concept == Concept::Volatility)
        .unwrap();
    assert_eq!(vol.availability, FactorAvailability::Unavailable);
    assert!(vol.missing_metrics.contains(&"atr_14".to_string()));
}

#[test]
fn fact_003_zero_roc_is_available_not_unavailable() {
    let mut metrics = trend_metrics(true);
    metrics.metrics.insert("roc_20".to_string(), MetricValue::Float(0.0));
    metrics.metrics.insert("atr_14".to_string(), MetricValue::Float(12.0));
    let p = profile_at(&metrics, t());
    let mom = p
        .factor_status
        .iter()
        .find(|s| s.concept == Concept::Momentum)
        .unwrap();
    assert_eq!(mom.availability, FactorAvailability::Available);
    assert!(mom.missing_metrics.is_empty());
}

#[test]
fn fact_004_volatility_has_no_high_low_direction() {
    let p = profile_at(&full_metrics(true), t());
    let vol = p
        .factor_status
        .iter()
        .find(|s| s.concept == Concept::Volatility)
        .unwrap();
    assert_eq!(vol.availability, FactorAvailability::Available);
    assert!(!p.assessments.iter().any(|a| a.concept == Concept::Volatility));
}

#[test]
fn fact_005_missing_trend_is_not_invented_bullish() {
    let mut metrics = MetricReport::default();
    metrics.metrics.insert("roc_20".to_string(), MetricValue::Float(8.3));
    let p = profile_at(&metrics, t());
    let trend = p
        .factor_status
        .iter()
        .find(|s| s.concept == Concept::Trend)
        .unwrap();
    assert_eq!(trend.availability, FactorAvailability::Unavailable);
    assert!(!p.assessments.iter().any(|a| a.concept == Concept::Trend));
    let d = TrendMappingPolicy.decide(&p, t());
    assert_eq!(d.action, DecisionAction::NoTrade);
}

#[test]
fn fact_missing_high_low_does_not_emit_zero_atr() {
    let obs: Vec<_> = (0..20)
        .map(|i| price_obs(100 + i as u128, i as i64 - 20, 100.0 + i as f64, Some((0.0, 0.0))))
        .collect();
    let ctx = InstrumentEvaluationContext {
        instrument_id: inst(),
        observations: obs,
    };
    let report = engine().evaluate(&ctx);
    assert!(report.get_float("atr_14").is_none());
}

#[test]
fn dec_001_repeated_run_is_identical() {
    let a = decide(&full_metrics(true));
    let b = decide(&full_metrics(true));
    assert_eq!(identity_tuple(&a), identity_tuple(&b));
    assert_eq!(a.lineage.input_set_hash, b.lineage.input_set_hash);
}

#[test]
fn dec_003_neutral_and_absent_trend_are_no_trade() {
    let mut p = profile_at(&full_metrics(true), t());
    p.assessments
        .iter_mut()
        .find(|a| a.concept == Concept::Trend)
        .unwrap()
        .direction = Direction::Neutral;
    assert_eq!(TrendMappingPolicy.decide(&p, t()).action, DecisionAction::NoTrade);
    p.assessments.retain(|a| a.concept != Concept::Trend);
    assert_eq!(TrendMappingPolicy.decide(&p, t()).action, DecisionAction::NoTrade);
}

#[test]
fn dec_004_momentum_does_not_secretly_alter_trend_map() {
    let with = decide(&full_metrics(true));
    let without = decide(&trend_metrics(true));
    assert_eq!(with.action, DecisionAction::Long);
    assert_eq!(without.action, DecisionAction::Long);
    assert_eq!(with.decision_id, without.decision_id);
}

#[test]
fn dec_005_atr_change_does_not_change_trend_only_identity() {
    let a = decide(&full_metrics(true));
    let mut other = full_metrics(true);
    other.metrics.insert("atr_14".to_string(), MetricValue::Float(99.0));
    let b = decide(&other);
    assert_eq!(a.action, b.action);
    assert_eq!(a.decision_id, b.decision_id);
    assert_eq!(a.provenance.content_hash, b.provenance.content_hash);
}

#[test]
fn dec_006_replay_adapter_source_never_selects_outcomes() {
    let src = include_str!("../src/decision_support/replay.rs");
    assert!(src.contains("never selects `knowledge_outcomes`"));
    assert!(!src.contains("FROM knowledge_outcomes"));
    assert!(!src.contains("JOIN knowledge_outcomes"));
}

#[test]
fn lin_001_missing_parent_fails_contract() {
    use chronosentiment_adapter::decision_support::{
        DecisionContractError, DecisionDraft, DecisionEvidence, RiskInformation, RiskLevel,
    };
    let draft = DecisionDraft {
        engine_version: UNFROZEN_ENGINE_VERSION.to_string(),
        instrument_id: inst(),
        as_of_timestamp: t(),
        action: DecisionAction::Long,
        confidence: None,
        confidence_status: ConfidenceStatus::Unavailable,
        horizon_trading_days: 5,
        rationale: "x".into(),
        evidence_refs: vec![],
        evidence: DecisionEvidence::default(),
        risk: RiskInformation {
            level: RiskLevel::Medium,
            invalidation: None,
        },
        lineage: DecisionLineage {
            produced_by: "test".into(),
            consumed_artifact_ids: vec![],
            assessment_id: None,
            input_set_hash: "abc".into(),
        },
    };
    assert_eq!(
        TradingDecision::try_from_draft(draft).unwrap_err(),
        DecisionContractError::MissingLineage
    );
}

#[test]
fn lin_002_as_of_is_explicit_t_not_wall_clock() {
    let d = decide(&full_metrics(true));
    assert_eq!(d.as_of_timestamp, t());
    assert!(d.as_of_timestamp < Utc::now() - chrono::Duration::days(365));
}

struct Lake {
    assessments: Vec<ReplayAssessment>,
}

#[async_trait::async_trait]
impl DecideAt for Lake {
    async fn decide_at(
        &self,
        as_of: chrono::DateTime<Utc>,
        instrument_id: Uuid,
        engine_version: &str,
    ) -> Result<TradingDecision, ReplayError> {
        decide_from_inputs(ReplayInputs {
            instrument_id,
            as_of,
            engine_version: engine_version.to_string(),
            produced_by: chronosentiment_adapter::decision_support::replay::REPLAY_PRODUCER
                .to_string(),
            assessments: self.assessments.clone(),
            lake_decisions: vec![],
            observations: vec![],
        })
    }
}

#[tokio::test]
async fn led_001_later_tick_does_not_mutate_earlier_row() {
    let p = profile_at(&full_metrics(true), t());
    let lake = Lake {
        assessments: vec![ReplayAssessment {
            id: p.metadata.artifact_id,
            evaluation_timestamp: t(),
            signature_hash: p.to_hash(),
            profile: p,
        }],
    };
    let first = run_replay_backtest(
        &lake,
        &[ReplayTick {
            as_of: t(),
            instrument_id: inst(),
        }],
        UNFROZEN_ENGINE_VERSION,
    )
    .await
    .unwrap();
    let both = run_replay_backtest(
        &lake,
        &[
            ReplayTick {
                as_of: t(),
                instrument_id: inst(),
            },
            ReplayTick {
                as_of: t() + chrono::Duration::days(1),
                instrument_id: inst(),
            },
        ],
        UNFROZEN_ENGINE_VERSION,
    )
    .await
    .unwrap();
    assert_eq!(first.records[0].decision_id, both.records[0].decision_id);
    assert_eq!(first.records[0].content_hash, both.records[0].content_hash);
}

fn rec_from(d: &TradingDecision) -> chronosentiment_adapter::decision_support::backtest::LedgerRecord {
    let mut ledger = DecisionLedger::new(UNFROZEN_ENGINE_VERSION);
    ledger.append(d.clone());
    ledger.records.pop().unwrap()
}

#[test]
fn out_001_and_perf_001_measurement_cannot_write_upward() {
    let decision = decide(&full_metrics(true));
    let before = decision.clone();
    let record = rec_from(&decision);
    let mut ledger = DecisionLedger::new(UNFROZEN_ENGINE_VERSION);
    ledger.append(decision.clone());
    let ledger_before = ledger.clone();
    let rows = [LakeOutcomeRow {
        id: Uuid::from_u128(9),
        lake_decision_id: Uuid::from_u128(8),
        instrument_id: inst(),
        decision_as_of: t(),
        horizon: "5D".into(),
        outcome_return: 0.5,
        entry_reached: true,
        target_hit: true,
        stop_hit: false,
        exit_reason: "Target".into(),
        mfe: 0.5,
        mae: 0.0,
        drawdown: 0.0,
        horizon_expiry_timestamp: t() + chrono::Duration::days(5),
    }];
    let bundle = measure_record(&record, &rows);
    let outcomes = OutcomeReport {
        bundles: vec![DecisionOutcomeBundle {
            ledger_decision_id: decision.decision_id,
            instrument_id: inst(),
            as_of_timestamp: t(),
            action: decision.action,
            horizons: bundle.horizons,
            content_hash: "x".into(),
        }],
    };
    let _perf = measure_performance(&ledger, &outcomes);
    assert_eq!(decision.decision_id, before.decision_id);
    assert_eq!(decision.action, before.action);
    assert_eq!(ledger.records[0].content_hash, ledger_before.records[0].content_hash);
    assert_eq!(ledger.identity_hash(), ledger_before.identity_hash());
}

#[test]
fn adv_001_created_at_does_not_change_identity() {
    let mut p = profile_at(&full_metrics(true), t());
    let a = decide_from_inputs(inputs_from(p.clone())).unwrap();
    p.metadata.created_at = t() + chrono::Duration::days(400);
    let b = decide_from_inputs(inputs_from(p)).unwrap();
    assert_eq!(a.decision_id, b.decision_id);
}

#[test]
fn adv_002_other_observation_does_not_change_decision_id() {
    let clean = decide(&full_metrics(true));
    let mut dirty = inputs_from(profile_at(&full_metrics(true), t()));
    dirty.observations.push(ReplayObservation {
        id: Uuid::from_u128(77),
        effective_from: t() - chrono::Duration::days(3),
    });
    let attacked = decide_from_inputs(dirty).unwrap();
    assert_eq!(clean.action, attacked.action);
    assert_eq!(clean.decision_id, attacked.decision_id);
}

#[test]
fn adv_003_shuffled_assessments_are_identical() {
    let p0 = profile_at(&full_metrics(true), t());
    let p1 = profile_at(&full_metrics(true), t() - chrono::Duration::days(30));
    let mut a = inputs_from(p0.clone());
    a.assessments.push(ReplayAssessment {
        id: p1.metadata.artifact_id,
        evaluation_timestamp: p1.metadata.evaluation_timestamp,
        signature_hash: p1.to_hash(),
        profile: p1.clone(),
    });
    let mut b = a.clone();
    b.assessments.reverse();
    let da = decide_from_inputs(a).unwrap();
    let db = decide_from_inputs(b).unwrap();
    assert_eq!(da.decision_id, db.decision_id);
}

#[test]
fn adv_004_duplicate_observation_does_not_silently_alter_metrics_or_decision() {
    let mut obs: Vec<_> = (0..60)
        .map(|i| price_obs(200 + i as u128, i as i64 - 60, 100.0 + i as f64, None))
        .collect();
    let ctx = InstrumentEvaluationContext {
        instrument_id: inst(),
        observations: obs.clone(),
    };
    let once = engine().evaluate(&ctx);
    obs.push(obs.last().unwrap().clone());
    let twice = engine().evaluate(&InstrumentEvaluationContext {
        instrument_id: inst(),
        observations: obs,
    });
    assert_eq!(once.get_float("ma_20"), twice.get_float("ma_20"));
    assert_eq!(once.get_float("roc_20"), twice.get_float("roc_20"));
    let clean = decide(&full_metrics(true));
    let mut dirty = inputs_from(profile_at(&full_metrics(true), t()));
    dirty.observations.push(dirty.observations[0].clone());
    let attacked = decide_from_inputs(dirty).unwrap();
    assert_eq!(clean.decision_id, attacked.decision_id);
}

#[test]
fn adv_005_one_at_a_time_identity_changes() {
    let base = decide(&full_metrics(true));
    let mut later = inputs_from(profile_at(&full_metrics(true), t()));
    later.as_of = t() + chrono::Duration::days(1);
    later.assessments[0].evaluation_timestamp = later.as_of;
    later.assessments[0].profile.metadata.evaluation_timestamp = later.as_of;
    assert_ne!(base.decision_id, decide_from_inputs(later).unwrap().decision_id);

    let mut other_inst = inputs_from(profile_at(&full_metrics(true), t()));
    other_inst.instrument_id = Uuid::from_u128(99);
    assert_ne!(base.decision_id, decide_from_inputs(other_inst).unwrap().decision_id);

    let mut other_engine = inputs_from(profile_at(&full_metrics(true), t()));
    other_engine.engine_version = "unfrozen-dev-2".into();
    assert_ne!(
        base.decision_id,
        decide_from_inputs(other_engine).unwrap().decision_id
    );

    let bear = decide(&full_metrics(false));
    assert_ne!(base.decision_id, bear.decision_id);
    assert_eq!(bear.action, DecisionAction::Short);
}

#[test]
fn perf_001_signature_is_read_only() {
    fn assert_sig(_: fn(&DecisionLedger, &OutcomeReport) -> chronosentiment_adapter::decision_support::performance::PerformanceReport) {}
    assert_sig(measure_performance);
    let _ = HORIZON_DAYS;
    let _ = BTreeMap::<String, u32>::new();
}
