use super::decision_proposal::DecisionProposal;
use super::evaluation_result::EvaluationResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageNode {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub proposal: Option<DecisionProposal>,
    pub evaluation: EvaluationResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionLineage {
    pub nodes: HashMap<Uuid, LineageNode>,
    pub root_id: Uuid,
    pub current_id: Uuid,
}
