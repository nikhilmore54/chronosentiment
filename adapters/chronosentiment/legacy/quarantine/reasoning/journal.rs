use coralys_moga::runtime::optimization::metric::MetricReport;
use crate::validation::context::MarketEvaluationContext as EvaluationContext;
use crate::evidence::EvidenceItem;
use crate::hypothesis::InvestmentThesis;
use crate::reasoning::scenario::Scenario;
use crate::reasoning::decision::Decision;

pub struct DecisionJournal;

impl DecisionJournal {
    /// A passive, append-only logger that records the exact state of the reasoning pipeline
    /// at the moment a decision is made. It generates nothing.
    pub fn record(
        &self,
        context: &EvaluationContext,
        _metrics: &MetricReport,
        evidence: &[EvidenceItem],
        hypotheses: &[InvestmentThesis],
        scenarios: &[Scenario],
        decision: &Decision,
    ) {
        println!("\n==================================================");
        println!("               DECISION JOURNAL LOG               ");
        println!("==================================================");
        println!("Timestamp:   {}", context.evaluation_timestamp);
        println!("Session ID:  {}", context.research_session_id);
        
        println!("\n[1. Facts & Evidence]");
        for e in evidence {
            println!("  - [{:?}] {}: {}", e.source_type, e.title, e.content);
        }

        println!("\n[2. Hypotheses]");
        for h in hypotheses {
            println!("  - {}", h.summary());
        }

        println!("\n[3. Scenarios Evaluated]");
        for s in scenarios {
            let marker = if s.scenario_id == decision.selected_scenario_id {
                "[*SELECTED*]"
            } else if decision.rejected_scenario_ids.contains(&s.scenario_id) {
                "[REJECTED]"
            } else {
                "[UNKNOWN]"
            };
            println!("  {} {} (ER: {:.1}%, Risk: {:.1}%)", marker, s.description, s.expected_return * 100.0, s.expected_risk * 100.0);
        }

        println!("\n[4. Final Decision]");
        println!("  Action:     {:?}", decision.action);
        println!("  Confidence: {:.1}%", decision.confidence * 100.0);
        println!("==================================================\n");
    }
}
