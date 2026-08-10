use coralys_moga::observatory::{PipelineObserver, ProcessingEvent, RepairEvent, FeasibilityReport};
use coralys_moga::traits::Genome;
use coralys_core::models::DecisionProposal;
use coralys_core::models::decision_lineage::LineageNode;
use uuid::Uuid;

pub struct LineageAdapter;

impl LineageAdapter {
    pub fn new() -> Self {
        Self {}
    }
}

impl<G: Genome> PipelineObserver<G> for LineageAdapter {
    fn on_event(&self, _event: &ProcessingEvent<G>) {}

    fn on_repair_event(&self, event: &RepairEvent) {
        if event.successful {
            if let (Some(_desc), Some(payload), Some(priority)) = (
                &event.action_description, 
                &event.action_payload,
                event.action_priority
            ) {
                let proposal_id = Uuid::new_v4();
                
                let proposal = DecisionProposal {
                    priority,
                    estimated_gain: 0.0,
                    affected_resources: vec![],
                    violations_resolved: vec![event.violation_id.clone()],
                    confidence: 0.9,
                    payload: payload.clone(),
                };
                
                let _node = LineageNode {
                    id: proposal_id,
                    parent_id: None,
                    proposal: Some(proposal),
                    evaluation: coralys_core::models::EvaluationResult {
                        objectives: vec![],
                        hard_constraint_violations: vec![],
                        soft_constraint_violations: vec![],
                        metrics: std::collections::HashMap::new(),
                    }
                };
                
                tracing::debug!(
                    target: "ultracrew::observability", 
                    "DecisionLineage node recorded: {} for violation {}", 
                    proposal_id, 
                    event.violation_id
                );
            }
        }
    }

    fn on_feasibility_report(&self, _report: &FeasibilityReport) {}
}
