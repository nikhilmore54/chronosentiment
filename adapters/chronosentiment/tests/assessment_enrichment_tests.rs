use std::collections::BTreeMap;

use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::factor_availability::report_factor_availability;
use chronosentiment_adapter::decision_support::policy::{
    BaselineTrendMappingPolicy, DecisionPolicy,
};
use chronosentiment_adapter::decision_support::replay::{
    decide_from_inputs, ReplayAssessment, ReplayInputs, TREND_MAPPING_RULE,
};
use chronosentiment_adapter::decision_support::DecisionAction;
use chronosentiment_adapter::metrics::concepts::Concept;
use chronosentiment_adapter::reasoning::assessment::{
    AssessmentEngine, FactorAvailability, ENRICHMENT_CONCEPTS,
};
use coralys_moga::runtime::optimization::metric::{MetricReport, MetricValue};
use uuid::Uuid;

fn t() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2021, 10, 31, 15, 30, 0).unwrap()
}

fn trend_metrics() -> MetricReport {
    let mut metrics = MetricReport::default();
    metrics
        .metrics
        .insert("ma_20".to_string(), MetricValue::Float(2100.0));
    metrics
        .metrics
        .insert("ma_50".to_string(), MetricValue::Float(2050.0));
    metrics
}

fn full_metrics() -> MetricReport {
    let mut metrics = trend_metrics();
    metrics
        .metrics
        .insert("roc_20".to_string(), MetricValue::Float(8.3));
    metrics
        .metrics
        .insert("atr_14".to_string(), MetricValue::Float(12.0));
    metrics
}

#[test]
fn missing_metrics_are_unavailable_not_invented() {
    let profile = AssessmentEngine.assess_at(&trend_metrics(), &ENRICHMENT_CONCEPTS, t(), None);
    assert_eq!(profile.metadata.evaluation_timestamp, t());
    let trend = profile
        .factor_status
        .iter()
        .find(|s| s.concept == Concept::Trend)
        .unwrap();
    assert_eq!(trend.availability, FactorAvailability::Available);
    let mom = profile
        .factor_status
        .iter()
        .find(|s| s.concept == Concept::Momentum)
        .unwrap();
    assert_eq!(mom.availability, FactorAvailability::Unavailable);
    assert!(mom.missing_metrics.contains(&"roc_20".to_string()));
    let vol = profile
        .factor_status
        .iter()
        .find(|s| s.concept == Concept::Volatility)
        .unwrap();
    assert_eq!(vol.availability, FactorAvailability::Unavailable);
    assert!(profile
        .assessments
        .iter()
        .all(|a| a.concept != Concept::Momentum && a.concept != Concept::Volatility));
}

#[test]
fn computable_momentum_and_volatility_are_independent() {
    let profile = AssessmentEngine.assess_at(&full_metrics(), &ENRICHMENT_CONCEPTS, t(), None);
    assert_eq!(
        profile
            .factor_status
            .iter()
            .filter(|s| s.availability == FactorAvailability::Available)
            .count(),
        3
    );
    assert!(profile
        .assessments
        .iter()
        .any(|a| a.concept == Concept::Momentum));
    assert!(profile
        .assessments
        .iter()
        .all(|a| a.concept != Concept::Volatility));
}

#[test]
fn enrichment_hashes_are_deterministic() {
    let a = AssessmentEngine.assess_at(&full_metrics(), &ENRICHMENT_CONCEPTS, t(), None);
    let b = AssessmentEngine.assess_at(&full_metrics(), &ENRICHMENT_CONCEPTS, t(), None);
    assert_eq!(a.to_hash(), b.to_hash());
    assert_eq!(a.metadata.content_hash, b.metadata.content_hash);
    assert_eq!(a.factor_status, b.factor_status);
}

#[test]
fn trend_mapping_policy_is_unchanged_when_momentum_is_present() {
    let instrument_id = Uuid::from_u128(7);
    let mut profile = AssessmentEngine.assess_at(
        &full_metrics(),
        &ENRICHMENT_CONCEPTS,
        t(),
        Some(instrument_id),
    );
    let id = Uuid::from_u128(1);
    profile.metadata.artifact_id = id;
    let decision = BaselineTrendMappingPolicy.decide(&profile, t());
    assert_eq!(decision.action, DecisionAction::Long);
    assert_eq!(decision.mapping_rule, TREND_MAPPING_RULE);
    let vol = decision
        .factors
        .iter()
        .find(|f| f.concept == "Volatility")
        .unwrap();
    assert!(vol.present);
    assert!(vol.direction.is_none());

    let trading = decide_from_inputs(
        ReplayInputs {
            instrument_id,
            as_of: t(),
            engine_version: "unfrozen-dev".to_string(),
            produced_by: "test".to_string(),
            assessments: vec![ReplayAssessment {
                id,
                evaluation_timestamp: t(),
                signature_hash: profile.to_hash(),
                profile,
            }],
            lake_decisions: vec![],
            observations: vec![],
        },
        &BaselineTrendMappingPolicy,
    )
    .unwrap();
    assert_eq!(trading.action, DecisionAction::Long);
    assert!(trading
        .evidence
        .diagnostics
        .contains("Consumed concepts: Trend only"));
}

#[test]
fn factor_availability_report_splits_by_concept() {
    let inst = Uuid::from_u128(9);
    let profile =
        AssessmentEngine.assess_at(&trend_metrics(), &ENRICHMENT_CONCEPTS, t(), Some(inst));
    let mut labels = BTreeMap::new();
    labels.insert(inst, "AAA".to_string());
    let report = report_factor_availability(&[profile], &labels);
    assert_eq!(report.n_profiles, 1);
    assert_eq!(report.by_concept["Trend"].available, 1);
    assert_eq!(report.by_concept["Momentum"].unavailable, 1);
    assert_eq!(report.by_concept["Volatility"].unavailable, 1);
    assert_eq!(report.by_instrument["AAA"]["Trend"].available, 1);
    assert_eq!(report.by_year[&2021]["Trend"].available, 1);
}

#[test]
fn observations_after_t_do_not_enter_the_decision() {
    use chronosentiment_adapter::decision_support::replay::{
        observations_at_or_before, ReplayObservation,
    };
    let future = ReplayObservation {
        id: Uuid::from_u128(99),
        effective_from: t() + chrono::Duration::days(1),
    };
    let past = ReplayObservation {
        id: Uuid::from_u128(1),
        effective_from: t(),
    };
    let obs = [past.clone(), future];
    let kept = observations_at_or_before(&obs, t());
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].id, Uuid::from_u128(1));
    assert!(kept.iter().all(|o| o.effective_from <= t()));
}

#[test]
fn enrichment_profiles_are_structurally_admissible() {
    use chronosentiment_adapter::decision_support::factor_availability::certify_enrichment_profiles;
    let profile = AssessmentEngine.assess_at(&full_metrics(), &ENRICHMENT_CONCEPTS, t(), None);
    let failures = certify_enrichment_profiles(&[profile]);
    assert!(failures.is_empty(), "{failures:?}");
}

#[test]
fn bars_after_t_do_not_change_factor_status() {
    use chronosentiment_adapter::decision_support::enrichment_certify::assess_from_bars_at_t;
    use chronosentiment_adapter::ingestion::yahoo::YahooHistoricalBar;
    let mut bars = Vec::new();
    let start = Utc
        .with_ymd_and_hms(2021, 8, 1, 0, 0, 0)
        .unwrap()
        .timestamp();
    for i in 0..80 {
        let close = 100.0 + i as f64;
        bars.push(YahooHistoricalBar {
            timestamp: start + i * 86400,
            open: close,
            high: close + 1.0,
            low: close - 1.0,
            close,
            adj_close: close,
            volume: 1_000.0,
        });
    }
    let future = YahooHistoricalBar {
        timestamp: t().timestamp() + 86400,
        open: 9_999.0,
        high: 9_999.0,
        low: 9_999.0,
        close: 9_999.0,
        adj_close: 9_999.0,
        volume: 1.0,
    };
    let mut with_future = bars.clone();
    with_future.push(future);
    let inst = Uuid::from_u128(7);
    let (a, _, max_a) = assess_from_bars_at_t(&bars, t(), inst);
    let (b, _, max_b) = assess_from_bars_at_t(&with_future, t(), inst);
    assert_eq!(a.factor_status, b.factor_status);
    assert!(max_a.unwrap() <= t());
    assert!(max_b.unwrap() <= t());
}
