use chronosentiment_adapter::metrics::concepts::Concept;
use chronosentiment_adapter::reasoning::assessment::{AssessmentEngine, AssessmentValue};
use chronosentiment_adapter::reasoning::evidence::{EvidenceEngine};
use coralys_moga::runtime::optimization::metric::{MetricReport, MetricValue};
use std::collections::HashMap;

#[test]
fn test_assessment_engine_contract() {
    let mut metrics = MetricReport::default();
    metrics.metrics.insert("ma_20".to_string(), MetricValue::Float(142.0));
    metrics.metrics.insert("ma_50".to_string(), MetricValue::Float(137.0));
    metrics.metrics.insert("roc_20".to_string(), MetricValue::Float(8.3));
    metrics.metrics.insert("volume_20d".to_string(), MetricValue::Float(1.7));
    metrics.metrics.insert("atr_14".to_string(), MetricValue::Float(2.1));

    let active_concepts = vec![Concept::Trend, Concept::Momentum, Concept::Liquidity, Concept::Volatility];

    let engine = AssessmentEngine;
    let assessments = engine.assess(&metrics, &active_concepts);

    assert_eq!(assessments.len(), 4);
    
    // Trend should be Bullish
    let trend = assessments.iter().find(|a| a.concept == Concept::Trend).unwrap();
    assert_eq!(trend.value, AssessmentValue::Bullish);

    // Momentum should be Positive
    let momentum = assessments.iter().find(|a| a.concept == Concept::Momentum).unwrap();
    assert_eq!(momentum.value, AssessmentValue::Positive);
}

#[test]
fn test_evidence_engine_contract() {
    let mut metrics = MetricReport::default();
    metrics.metrics.insert("ma_20".to_string(), MetricValue::Float(142.0));
    metrics.metrics.insert("ma_50".to_string(), MetricValue::Float(137.0));
    
    let active_concepts = vec![Concept::Trend];
    
    let assess_engine = AssessmentEngine;
    let assessments = assess_engine.assess(&metrics, &active_concepts);
    
    let evidence_engine = EvidenceEngine;
    let evidence = evidence_engine.evaluate(&assessments);
    
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0].concept, Concept::Trend);
    assert!(evidence[0].supports_continuation);
    assert!(evidence[0].description.contains("Bullish trend supports continuation"));
}

use chronosentiment_adapter::reasoning::similarity::HistoricalSimilarityEngine;

#[test]
fn test_similarity_engine_contract() {
    let mut metrics = MetricReport::default();
    metrics.metrics.insert("ma_20".to_string(), MetricValue::Float(142.0));
    metrics.metrics.insert("ma_50".to_string(), MetricValue::Float(137.0));
    
    let active_concepts = vec![Concept::Trend];
    let assess_engine = AssessmentEngine;
    let assessments = assess_engine.assess(&metrics, &active_concepts);
    
    let evidence_engine = EvidenceEngine;
    let evidence = evidence_engine.evaluate(&assessments);
    
    let similarity_engine = HistoricalSimilarityEngine;
    let context = similarity_engine.find_similar(&evidence);
    
    let similarity_evidence = similarity_engine.context_to_evidence(&context);
    
    assert_eq!(context.similar_cases_count, 187);
    assert!(similarity_evidence.supports_continuation);
    assert!(similarity_evidence.description.contains("81%"));
}
