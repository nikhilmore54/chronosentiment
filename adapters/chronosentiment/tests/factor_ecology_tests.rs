use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::factor_ecology::{
    analyze, row_from_profile, state_key,
};
use chronosentiment_adapter::reasoning::assessment::{AssessmentEngine, ENRICHMENT_CONCEPTS};
use coralys_moga::runtime::optimization::metric::{MetricReport, MetricValue};

fn t() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2021, 10, 31, 15, 30, 0).unwrap()
}

#[test]
fn outcomes_do_not_enter_the_state_key() {
    let mut metrics = MetricReport::default();
    metrics
        .metrics
        .insert("ma_20".to_string(), MetricValue::Float(2100.0));
    metrics
        .metrics
        .insert("ma_50".to_string(), MetricValue::Float(2050.0));
    metrics
        .metrics
        .insert("roc_20".to_string(), MetricValue::Float(8.3));
    metrics
        .metrics
        .insert("atr_14".to_string(), MetricValue::Float(12.0));
    let profile = AssessmentEngine.assess_at(&metrics, &ENRICHMENT_CONCEPTS, t(), None);
    let mut a = row_from_profile(&profile, "AAA".into(), Some(8.3), Some(12.0));
    let mut b = a.clone();
    a.outcome_60d = Some(0.50);
    b.outcome_60d = Some(-0.50);
    assert_eq!(state_key(&a), state_key(&b));
    let report = analyze(&[a, b]);
    assert_eq!(report.trend_x_momentum_x_vol.len(), 1);
    assert!(!report.design_constraints.is_empty());
    assert!(report
        .design_constraints
        .iter()
        .any(|c| c.contains("measurement")));
}
