use chronosentiment_adapter::metrics::{
    profile::{EvaluationProfile, LargeCapCoreProfile},
    concepts::Concept,
    instrument::{SimpleMovingAverageMetric, RateOfChangeMetric, InstrumentMetricEngine},
};
use coralys_moga::runtime::optimization::metric::MetricEngine;

#[test]
fn test_evaluation_profile_contract() {
    let profile = LargeCapCoreProfile;
    
    // 1. Verify intended concepts are activated
    let active_concepts = profile.active_concepts();
    assert!(active_concepts.contains(&Concept::Trend));
    assert!(active_concepts.contains(&Concept::Momentum));
    assert!(active_concepts.contains(&Concept::Volatility));
    assert!(active_concepts.contains(&Concept::Liquidity));
    
    // Ensure Macro and Sector are NOT activated for LargeCapCore directly in this way
    assert!(!active_concepts.contains(&Concept::Macro));
    
    // 2. Verify metric mapping contract
    let trend_metrics = profile.metrics_for_concept(&Concept::Trend);
    assert_eq!(trend_metrics, vec!["ma_20", "ma_50"]);
    
    let momentum_metrics = profile.metrics_for_concept(&Concept::Momentum);
    assert_eq!(momentum_metrics, vec!["roc_20"]);
}

#[test]
fn test_metric_models_ontology() {
    use chronosentiment_adapter::metrics::concepts::ConceptModel;
    
    let ma = SimpleMovingAverageMetric::new(20);
    assert_eq!(ma.concept(), Concept::Trend);
    assert_eq!(ma.name(), "ma_20");

    let roc = RateOfChangeMetric::new(20);
    assert_eq!(roc.concept(), Concept::Momentum);
    assert_eq!(roc.name(), "roc_20");
}
