use chronosentiment_adapter::metrics::concepts::Concept;
use chronosentiment_adapter::reasoning::assessment::AssessmentEngine;
use chronosentiment_adapter::reasoning::evidence::EvidenceEngine;
// use chronosentiment_adapter::reasoning::similarity::HistoricalSimilarityEngine;
use coralys_moga::runtime::optimization::metric::{MetricReport, MetricValue};

fn main() {
    println!("=== ChronoSentiment Phase 2: Week 2 Demo ===");
    println!("Testing the Assessment, Evidence, and Similarity Engines pipeline.\n");

    // 1. Simulate a MetricReport from the MetricEngine
    let mut metrics = MetricReport::default();
    metrics
        .metrics
        .insert("ma_20".to_string(), MetricValue::Float(142.0));
    metrics
        .metrics
        .insert("ma_50".to_string(), MetricValue::Float(137.0));
    metrics
        .metrics
        .insert("roc_20".to_string(), MetricValue::Float(8.3));
    metrics
        .metrics
        .insert("volume_20d".to_string(), MetricValue::Float(1.7));
    metrics
        .metrics
        .insert("atr_14".to_string(), MetricValue::Float(2.1));

    println!("1. Metric Report Generated:");
    for (key, val) in &metrics.metrics {
        println!("   - {}: {}", key, val);
    }
    println!();

    // 2. Assessment Engine
    let active_concepts = vec![
        Concept::Trend,
        Concept::Momentum,
        Concept::Liquidity,
        Concept::Volatility,
    ];
    let assess_engine = AssessmentEngine;
    let assessments = assess_engine.assess(&metrics, &active_concepts);

    println!("2. Domain Assessments (Analyst Synthesis):");
    for assessment in &assessments.assessments {
        println!(
            "   - {:?} is {:?} (Confidence: {:.0}%)",
            assessment.concept,
            assessment.direction,
            assessment.confidence * 100.0
        );
    }
    println!();

    // 3. Evidence Engine
    let evidence_engine = EvidenceEngine;
    let mut evidence = evidence_engine.evaluate(&assessments);

    println!("3. Evidence Statements:");
    for e in &evidence.evidence {
        println!(
            "   - [{:?}] {} (Supports Continuation: {})",
            e.concept, e.description, e.supports_continuation
        );
    }
    println!();

    // 4. Historical Similarity Engine
    // (Disabled due to phase 5 architecture refactoring)
    // let similarity_engine = HistoricalSimilarityEngine;
    // let context = similarity_engine.find_similar(&evidence);

    // let similarity_evidence = similarity_engine.context_to_evidence(&context);
    // evidence.evidence.push(similarity_evidence.clone());

    // println!("4. Historical Similarity Engine:");
    // println!("   - Found {} similar historical decisions.", context.similar_cases_count);
    // println!("   - Added as New Evidence: [{:?}] {} (Supports Continuation: {})",
    //          similarity_evidence.concept, similarity_evidence.description, similarity_evidence.supports_continuation);

    println!("\n=== Pipeline Execution Complete ===");
}
