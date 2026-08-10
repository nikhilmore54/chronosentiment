use crate::runtime::model::network::OperationalModel;
use crate::runtime::optimization::constraint::{
    ConstraintModel, ConstraintEvaluation, ConstraintViolation, ConstraintTier,
    ConstraintSatisfactionConfig, ConstraintSatisfactionPolicy, ConstraintReport,
    ConstraintSatisfactionResult, ConstraintSatisfactionEngine, RepairOperator,
    RepairActionEvaluator, DefaultRepairEvaluator
};
use std::sync::Arc;
use crate::observatory::{PipelineObserver, RepairEvent, FeasibilityReport};

use crate::traits::Genome;

pub struct DefaultRepairEngine<G, V>
where
    G: OperationalModel + Genome,
    V: ConstraintViolation,
{
    pub models: Vec<Box<dyn ConstraintModel<G, V>>>,
    pub operators: Vec<Box<dyn RepairOperator<G, V>>>,
    pub config: ConstraintSatisfactionConfig,
    pub evaluator: Box<dyn RepairActionEvaluator<G>>,
    pub metric_engine: Option<Arc<dyn crate::runtime::optimization::metric::MetricEngine<G>>>,
    pub observer: Option<Arc<dyn PipelineObserver<G>>>,
}

impl<G, V> DefaultRepairEngine<G, V>
where
    G: OperationalModel + Genome,
    V: ConstraintViolation,
{
    pub fn new(config: ConstraintSatisfactionConfig) -> Self {
        Self {
            models: Vec::new(),
            operators: Vec::new(),
            config,
            evaluator: Box::new(DefaultRepairEvaluator),
            metric_engine: None,
            observer: None,
        }
    }
    
    pub fn with_observer(mut self, observer: Arc<dyn PipelineObserver<G>>) -> Self {
        self.observer = Some(observer);
        self
    }
    
    pub fn with_metric_engine(mut self, metric_engine: Arc<dyn crate::runtime::optimization::metric::MetricEngine<G>>) -> Self {
        self.metric_engine = Some(metric_engine);
        self
    }
    
    pub fn with_evaluator(mut self, evaluator: Box<dyn RepairActionEvaluator<G>>) -> Self {
        self.evaluator = evaluator;
        self
    }

    pub fn add_model(&mut self, model: Box<dyn ConstraintModel<G, V>>) {
        self.models.push(model);
    }

    pub fn add_operator(&mut self, operator: Box<dyn RepairOperator<G, V>>) {
        self.operators.push(operator);
    }

    pub fn evaluate(&self, model: &G, metrics: &crate::runtime::optimization::metric::MetricReport) -> ConstraintReport<V> {
        let mut assessments = Vec::new();
        let mut legal = true;
        
        for m in &self.models {
            let assessment = m.evaluate(model, metrics);
            if assessment.tier <= ConstraintTier::Legality && assessment.status == crate::runtime::optimization::constraint::AssessmentStatus::Failed {
                legal = false;
            }
            assessments.push(assessment);
        }
        
        ConstraintReport { assessments, legal }
    }
}

impl<G, V> ConstraintSatisfactionEngine<G> for DefaultRepairEngine<G, V>
where
    G: OperationalModel + Genome,
    V: ConstraintViolation,
{
    fn satisfy(&self, model: &mut G) -> ConstraintSatisfactionResult {
        let empty_metrics = crate::runtime::optimization::metric::MetricReport::default();
        let mut current_metrics = match &self.metric_engine {
            Some(engine) => engine.evaluate(model),
            None => empty_metrics,
        };

        let initial_report = self.evaluate(model, &current_metrics);
        
        if self.config.policy == ConstraintSatisfactionPolicy::Disabled {
            return ConstraintSatisfactionResult {
                legal: initial_report.legal,
                repaired: false,
                iterations: 0,
                final_metrics: current_metrics,
            };
        }
        
        let target_violations: Vec<_> = initial_report.assessments.iter()
            .filter(|a| a.tier <= self.config.repair_until)
            .flat_map(|a| a.violations.iter().map(move |v| (a.constraint_id.clone(), a.tier, v.clone())))
            .collect();
            
        if self.config.policy == ConstraintSatisfactionPolicy::OnViolation && target_violations.is_empty() {
            return ConstraintSatisfactionResult {
                legal: initial_report.legal,
                repaired: false,
                iterations: 0,
                final_metrics: current_metrics,
            };
        }
        
        let mut iterations = 0;
        let mut improved = true;
        let mut total_repair_attempts = 0;
        let mut any_repair_successful = false;

        let mut sorted_models: Vec<&dyn ConstraintModel<G, V>> = self.models.iter().map(|m| m.as_ref()).collect();
        sorted_models.sort_by(|a, b| a.tier().cmp(&b.tier()));

        while improved && iterations < self.config.max_iterations {
            improved = false;
            if any_repair_successful {
                if let Some(engine) = &self.metric_engine {
                    current_metrics = engine.evaluate(model);
                }
            }
            let current_report = self.evaluate(model, &current_metrics);
            let current_target_violations: Vec<_> = current_report.assessments.iter()
                .filter(|a| a.tier <= self.config.repair_until)
                .flat_map(|a| a.violations.iter().map(move |v| (a.constraint_id.clone(), a.tier, v.clone())))
                .collect();
                
            if current_target_violations.is_empty() {
                break;
            }
            
            for (model_name, tier, violation) in current_target_violations {
                let mut repaired = false;
                for operator in &self.operators {
                    let actions = operator.repair(model, violation);
                    if !actions.is_empty() {
                        total_repair_attempts += 1;
                        
                        if let Some(best_action) = self.evaluator.evaluate(model, violation, actions) {
                            let desc = best_action.description();
                            let payload = best_action.payload();
                            let priority = best_action.priority();
                                
                            if best_action.apply(model).is_ok() {
                                repaired = true;
                                any_repair_successful = true;
                                if let Some(obs) = &self.observer {
                                    obs.on_repair_event(&RepairEvent {
                                        generation: 0,
                                        violation_id: violation.description(),
                                        action_description: Some(desc),
                                        action_payload: payload,
                                        action_priority: Some(priority),
                                        attempts: 1,
                                        successful: true,
                                    });
                                }
                                break;
                            }
                        }
                    }
                }
                
                if !repaired {
                    total_repair_attempts += 1;
                    if let Some(obs) = &self.observer {
                        obs.on_repair_event(&RepairEvent {
                            generation: 0,
                            violation_id: violation.description(),
                            action_description: None,
                            action_payload: None,
                            action_priority: None,
                            attempts: 1,
                            successful: false,
                        });
                    }
                }
                
                if repaired {
                    improved = true;
                    break; // Restart evaluation
                }
            }
            
            iterations += 1;
            if any_repair_successful && self.config.stop_after_first_success {
                break;
            }
        }
        
        if any_repair_successful {
            if let Some(engine) = &self.metric_engine {
                current_metrics = engine.evaluate(model);
            }
        }
        let final_report = self.evaluate(model, &current_metrics);
        let hard_violations = final_report.assessments.iter().filter(|a| a.tier <= ConstraintTier::Legality).flat_map(|a| &a.violations).count();
        let soft_violations = final_report.assessments.iter().filter(|a| a.tier > ConstraintTier::Legality).flat_map(|a| &a.violations).count();
        
        if let Some(obs) = &self.observer {
            obs.on_feasibility_report(&FeasibilityReport {
                hard_violations_remaining: hard_violations,
                soft_violations_remaining: soft_violations,
                repair_attempts: total_repair_attempts,
                constraint_coverage: if self.models.is_empty() { 0.0 } else { 1.0 },
            });
        }
        
        ConstraintSatisfactionResult {
            legal: final_report.legal,
            repaired: any_repair_successful,
            iterations,
            final_metrics: current_metrics,
        }
    }
}
