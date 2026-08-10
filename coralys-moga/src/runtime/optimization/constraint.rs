use crate::runtime::model::network::OperationalModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConstraintTier {
    Safety = 1,
    Regulatory = 2,
    Legality = 3,
    Operational = 4,
    Business = 5,
}

pub trait ConstraintViolation {
    fn description(&self) -> String;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssessmentStatus {
    Pass,
    Warning,
    Failed,
}

#[derive(Debug, Clone)]
pub struct ConstraintAssessment<V: ConstraintViolation> {
    pub constraint_id: String,
    pub tier: ConstraintTier,
    pub status: AssessmentStatus,
    pub violations: Vec<V>,
    pub metrics: std::collections::HashMap<String, crate::runtime::optimization::metric::MetricValue>,
    pub margins: std::collections::HashMap<String, f64>,
    pub repairability: bool,
    pub diagnostics: Vec<String>,
}

pub trait OperationalMutation<M: OperationalModel> {
    fn apply(&self, model: &mut M) -> Result<(), String>;
}

pub trait RepairAction<M: OperationalModel> {
    fn priority(&self) -> f64;
    fn description(&self) -> String;
    fn payload(&self) -> Option<serde_json::Value>;
    fn apply(&self, model: &mut M) -> Result<(), String>;
}

pub trait RepairOperator<M: OperationalModel, V: ConstraintViolation> {
    fn repair(&self, model: &M, violation: &V) -> Vec<Box<dyn RepairAction<M>>>;
}

pub trait ConstraintModel<M: OperationalModel, V: ConstraintViolation> {
    fn tier(&self) -> ConstraintTier;
    fn name(&self) -> String;
    fn evaluate(&self, model: &M, metrics: &crate::runtime::optimization::metric::MetricReport) -> ConstraintAssessment<V>;
}

pub struct ConstraintEvaluation {
    pub mandatory: bool,
    pub advisory_score: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintSatisfactionPolicy {
    Disabled,
    OnViolation,
    Always,
}

#[derive(Debug, Clone, Copy)]
pub struct ConstraintSatisfactionConfig {
    pub policy: ConstraintSatisfactionPolicy,
    pub repair_until: ConstraintTier,
    pub max_iterations: usize,
    pub stop_after_first_success: bool,
}

impl Default for ConstraintSatisfactionConfig {
    fn default() -> Self {
        Self {
            policy: ConstraintSatisfactionPolicy::OnViolation,
            repair_until: ConstraintTier::Legality,
            max_iterations: 10,
            stop_after_first_success: false,
        }
    }
}

pub struct ConstraintReport<V: ConstraintViolation> {
    pub assessments: Vec<ConstraintAssessment<V>>,
    pub legal: bool,
}

pub struct ConstraintSatisfactionResult {
    pub legal: bool,
    pub repaired: bool,
    pub iterations: usize,
    pub final_metrics: crate::runtime::optimization::metric::MetricReport,
}

pub trait ConstraintSatisfactionEngine<M> {
    fn satisfy(&self, model: &mut M) -> ConstraintSatisfactionResult;
}

pub trait RepairActionEvaluator<M: OperationalModel> {
    fn evaluate(&self, model: &M, violation: &dyn ConstraintViolation, actions: Vec<Box<dyn RepairAction<M>>>) -> Option<Box<dyn RepairAction<M>>>;
}

pub struct DefaultRepairEvaluator;
impl<M: OperationalModel> RepairActionEvaluator<M> for DefaultRepairEvaluator {
    fn evaluate(&self, _model: &M, _violation: &dyn ConstraintViolation, actions: Vec<Box<dyn RepairAction<M>>>) -> Option<Box<dyn RepairAction<M>>> {
        actions.into_iter()
            .max_by(|a, b| a.priority().partial_cmp(&b.priority()).unwrap_or(std::cmp::Ordering::Equal))
    }
}
