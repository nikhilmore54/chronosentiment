use chronosentiment_adapter::metrics::concepts::Concept;
use chronosentiment_adapter::reasoning::assessment::AssessmentEngine;
use chronosentiment_adapter::reasoning::evidence::EvidenceEngine;
use chronosentiment_adapter::reasoning::historical_reasoning::HistoricalReasoningEngine;
use chronosentiment_adapter::reasoning::hypothesis::HypothesisEngine;
use coralys_moga::runtime::optimization::metric::{MetricReport, MetricValue};

fn main() {
    println!("=== ChronoSentiment Phase 3: Reasoning Intelligence Demo ===\n");

    // 1. Simulate a MetricReport
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

    let active_concepts = vec![Concept::Trend, Concept::Momentum];

    // 2. Assessment Profile (Rich Synthesis)
    let assess_engine = AssessmentEngine;
    let profile = assess_engine.assess(&metrics, &active_concepts);

    println!("1. Assessment Profile (Immutable State):");
    for assessment in &profile.assessments {
        println!(
            "   - [{:?}] Direction: {:?}, Strength: {:?}, Maturity: {:?}",
            assessment.concept, assessment.direction, assessment.strength, assessment.maturity
        );
        println!(
            "     Confidence: {:.0}%, Uncertainty: {:.0}% ({})",
            assessment.confidence * 100.0,
            assessment.uncertainty * 100.0,
            assessment.uncertainty_reason.clone().unwrap_or_default()
        );
    }
    println!();

    // 3. Evidence Engine (Provenance)
    let evidence_engine = EvidenceEngine;
    let evidence_set = evidence_engine.evaluate(&profile);

    println!("2. Evidence Set with Provenance:");
    for e in &evidence_set.evidence {
        println!("   - ID: {} | Type: {:?}", e.evidence_id, e.evidence_type);
        println!("     [{:?}] {}", e.concept, e.description);
    }
    println!();

    // 4. Historical Reasoning Engine (Analogy-Based Reasoning)
    let reasoning_engine = HistoricalReasoningEngine;
    let report = reasoning_engine.evaluate(&profile);

    println!("3. Historical Reasoning Report:");
    println!("   - Similarity Score: {:.2}", report.similarity_score);
    println!("   - Found {} historical cases.", report.cases.len());
    for case in &report.cases {
        println!(
            "     Case ID: {} | Outcome Return: {:.1}% | Hash: {}",
            case.case_id,
            case.outcome_return * 100.0,
            case.assessment_profile_hash
        );
    }
    println!(
        "   - Win Rate: {:.0}% | Median Return: {:.1}%",
        report.win_rate * 100.0,
        report.median_return * 100.0
    );
    for note in &report.notes {
        println!("   - Note: {}", note);
    }
    println!();

    // 5. Pluggable Competing Hypotheses
    let hypothesis_engine = HypothesisEngine::new();
    let hypotheses = hypothesis_engine.evaluate(&evidence_set);

    println!("4. Competing Hypotheses:");
    for h in &hypotheses.hypotheses {
        println!(
            "   - Hypothesis: {} (Confidence: {:.0}%)",
            h.name,
            h.confidence * 100.0
        );
        println!(
            "     Supports: {}, Contradicts: {}",
            h.supporting_evidence_count, h.contradicting_evidence_count
        );
    }

    println!("\n=== Phase 3 Execution Complete ===");
}
