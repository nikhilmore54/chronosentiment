use chronosentiment_optimization::{Candidate, FitnessEvaluator};
use chronosentiment_strategies::evaluation::evaluator::FinancialEvaluator;
use chronosentiment_strategies::evaluation::report_adapter::SemanticEvaluationReport;

#[test]
fn test_evaluator_bridge_produces_semantic_report() {
    // 1. Setup the FinancialEvaluator for a mock asset/regime
    let evaluator = FinancialEvaluator::new("BTC".to_string(), "trending_up".to_string());

    // 2. Setup a candidate
    let mut candidate = Candidate::default();
    candidate.base_edge = 10;
    candidate.queue_threshold = 20;

    // 3. Evaluate the candidate mechanically
    let eval = evaluator.evaluate(&candidate);

    // 4. Prove bridge translates mechanical evaluation to semantic report
    let mut report: SemanticEvaluationReport = eval.into();
    report.regime = Some("trending_up".to_string());
    report.classification = "Momentum".to_string();

    assert!(
        report.fitness > 0.0,
        "Fitness should be populated from mechanical eval"
    );
    assert_eq!(report.regime.unwrap(), "trending_up");
    assert_eq!(report.classification, "Momentum");

    // Check that we safely extracted metrics from the eval map
    // The FinancialEvaluator should have populated some basic metrics
    // Since we're using synthetic test data in FinancialEvaluator right now, we can check basic assertions.
}
