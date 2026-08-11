use chronosentiment_adapter::metrics::concepts::Concept;
use chronosentiment_adapter::reasoning::assessment::AssessmentEngine;
use chronosentiment_adapter::reasoning::evidence::EvidenceEngine;
use chronosentiment_adapter::reasoning::historical_reasoning::HistoricalReasoningEngine;
use chronosentiment_adapter::reasoning::hypothesis::HypothesisEngine;
use chronosentiment_adapter::reasoning::decision::{Decision, Opportunity, ConfidenceDecomposition, ExpectedHorizon as DecExpectedHorizon};
use chronosentiment_adapter::reasoning::strategy::{OpportunityStrategy, PriceRange, Horizon};
use chronosentiment_adapter::validation::replay_decision::{TemporalFirewall, DecisionReplay};
use chronosentiment_adapter::validation::outcome::{OutcomeEngine, Horizon as OutcomeHorizon};
use chronosentiment_adapter::validation::calibration::CalibrationEngine;
use coralys_moga::runtime::optimization::metric::{MetricReport, MetricValue};
use chrono::{Utc, TimeZone};
use uuid::Uuid;

fn main() {
    println!("=== ChronoSentiment Phase 4: Financial Time Machine Demo ===\n");

    // 1. Set the temporal firewall
    // Let's travel back to exactly March 15, 2023, 10:30 AM
    let evaluation_time = Utc.with_ymd_and_hms(2023, 3, 15, 10, 30, 0).unwrap();
    let firewall = TemporalFirewall::new(evaluation_time);
    
    println!("1. TEMPORAL FIREWALL ENGAGED");
    println!("   Evaluation Timestamp: {}", firewall.evaluation_timestamp);
    println!("   Status: Future Knowledge STRICTLY LOCKED.\n");

    // Test the firewall
    // firewall.assert_historical(Utc.with_ymd_and_hms(2023, 3, 16, 0, 0, 0).unwrap(), "Test Case"); // Would panic

    // 2. Decision Replay
    let mut metrics = MetricReport::default();
    metrics.metrics.insert("ma_20".to_string(), MetricValue::Float(2100.0));
    metrics.metrics.insert("ma_50".to_string(), MetricValue::Float(2050.0));
    
    let profile = AssessmentEngine.assess(&metrics, &[Concept::Trend]);
    let evidence = EvidenceEngine.evaluate(&profile);
    let reasoning = HistoricalReasoningEngine.evaluate(&profile);
    let hypotheses = HypothesisEngine::new().evaluate(&evidence);
    
    let decision = Decision {
        decision_id: Uuid::new_v4(),
        evaluation_timestamp: evaluation_time,
        instrument_id: Uuid::new_v4(),
        universe: "NSE500".to_string(),
        market_context_id: None,
        evidence_ids: vec![],
        hypothesis_ids: vec![],
        scenario_ids: vec![],
        opportunity: Opportunity::Positive,
        confidence: ConfidenceDecomposition {
            evidence_quality: 0.8,
            evidence_agreement: 0.9,
            historical_reliability: 0.75,
            data_completeness: 1.0,
            model_stability: 0.9,
        },
        opportunity_score: 85.0,
        quality_score: 82.0,
        expected_horizon: DecExpectedHorizon::Medium,
        replay_context_hash: "ctx_15mar23".to_string(),
        knowledge_lake_version: "v1".to_string(),
        evaluation_profile_version: "v1".to_string(),
        concept_model_version: "v1".to_string(),
        metric_model_version: "v1".to_string(),
        evidence_rule_version: "v1".to_string(),
        assessment_engine_version: "v1".to_string(),
        hypothesis_engine_version: "v1".to_string(),
        validation_engine_version: "v1".to_string(),
        decision_engine_version: "v1".to_string(),
        scenario_projection_version: "v1".to_string(),
    };
    
    let strategy = OpportunityStrategy {
        decision_id: decision.decision_id,
        expected_horizon: Horizon::Swing,
        expected_holding_period_days: (8, 15),
        entry_zone: PriceRange { min: 2180.0, max: 2220.0 },
        target_zone: PriceRange { min: 2390.0, max: 2450.0 },
        stop_loss_zone: PriceRange { min: 2100.0, max: 2130.0 },
        expected_return: 0.1,
        expected_drawdown: 0.04,
        expected_volatility: 0.15,
        risk_reward_ratio: 2.5,
        confidence: 0.8,
    };

    let replay = DecisionReplay {
        decision_id: decision.decision_id,
        evaluation_timestamp: evaluation_time,
        replay_context_hash: "ctx_15mar23".to_string(),
        assessment_profile: profile,
        evidence_set: evidence,
        historical_reasoning_report: reasoning,
        hypotheses,
        decision,
        strategy,
    };

    println!("2. DECISION REPLAY");
    println!("   Reconstructed Decision ID: {}", replay.decision_id);
    println!("   Hypothesis: {}", replay.hypotheses.hypotheses.first().unwrap().name);
    println!("   Historical Reasoning: Found {} cases prior to {}", replay.historical_reasoning_report.cases.len(), evaluation_time);
    println!("   Strategy Generated:");
    println!("     - Entry: {}-{}", replay.strategy.entry_zone.min, replay.strategy.entry_zone.max);
    println!("     - Target: {}-{}", replay.strategy.target_zone.min, replay.strategy.target_zone.max);
    println!("     - Stop: {}-{}\n", replay.strategy.stop_loss_zone.min, replay.strategy.stop_loss_zone.max);

    // 3. Outcome Engine
    println!("3. LOOKING FORWARD (OUTCOME ENGINE)");
    let outcome = OutcomeEngine.measure_outcome(&replay.strategy, &[], evaluation_time);
    println!("   Observation End: {}", outcome.observation_end_timestamp);
    println!("   Outcome Return: {:.1}%", outcome.outcome_return * 100.0);
    println!("   Maximum Favourable Excursion (MFE): {:.1}%", outcome.mfe * 100.0);
    println!("   Maximum Adverse Excursion (MAE): {:.1}%", outcome.mae * 100.0);
    println!("   Exit Reason: {}\n", outcome.exit_reason);

    // 4. Calibration
    println!("4. RELIABILITY CALIBRATION");
    let calibration = CalibrationEngine.calibrate("prof_hash_abc", OutcomeHorizon::Swing);
    println!("   Aggregated across {} historical instances of this Assessment Profile:", calibration.sample_count);
    println!("   Win Rate: {:.1}%", calibration.win_rate * 100.0);
    println!("   Median Return: {:.1}%", calibration.median_return * 100.0);
    println!("   Median MFE: {:.1}%", calibration.median_mfe * 100.0);
    println!("   Median MAE: {:.1}%", calibration.median_mae * 100.0);
    println!("   Target Hit Rate: {:.1}%", calibration.target_hit_rate * 100.0);
    println!("   Stop Hit Rate: {:.1}%", calibration.stop_hit_rate * 100.0);
    
    println!("\n=== Phase 4 Execution Complete ===");
}
