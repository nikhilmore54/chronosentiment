use chrono::{TimeZone, Utc};
use chronosentiment_adapter::metrics::concepts::Concept;
use chronosentiment_adapter::reasoning::assessment::AssessmentEngine;
use coralys_moga::runtime::optimization::metric::{MetricReport, MetricValue};

#[test]
fn assess_at_stamps_replay_dt_not_wall_clock() {
    let mut metrics = MetricReport::default();
    metrics
        .metrics
        .insert("ma_20".to_string(), MetricValue::Float(2100.0));
    metrics
        .metrics
        .insert("ma_50".to_string(), MetricValue::Float(2050.0));

    let dt = Utc.with_ymd_and_hms(2021, 10, 31, 15, 30, 0).unwrap();
    let profile = AssessmentEngine.assess_at(&metrics, &[Concept::Trend], dt, None);

    assert_eq!(profile.metadata.evaluation_timestamp, dt);
    assert!(
        profile.metadata.evaluation_timestamp < Utc::now() - chrono::Duration::days(365),
        "replay dt must not collapse to persist wall-clock"
    );
}

#[test]
fn assess_mock_path_is_not_the_population_timestamp() {
    let metrics = MetricReport::default();
    let profile = AssessmentEngine.assess(&metrics, &[Concept::Trend]);
    let now = Utc::now();
    let delta = (profile.metadata.evaluation_timestamp - now)
        .num_seconds()
        .abs();
    assert!(
        delta < 5,
        "assess() remains the mock/wall-clock helper and must not be used to persist Knowledge Lake rows"
    );
}
