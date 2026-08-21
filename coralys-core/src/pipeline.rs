use std::error::Error;
use crate::operators::{ConstraintModel, ImprovementOperator, OperatorBudget, RepairOperator};

pub struct EvolutionaryPipeline<G, M, E>
where
    M: ConstraintModel<G>,
    E: Error + Send + Sync + 'static,
{
    pub constraint_model: M,
    pub repair_operators: Vec<Box<dyn RepairOperator<G, M, Error = E>>>,
    pub improvement_operators: Vec<Box<dyn ImprovementOperator<G, M, Error = E>>>,
    pub repair_budget: OperatorBudget,
    pub improve_budget: OperatorBudget,
}

impl<G, M, E> EvolutionaryPipeline<G, M, E>
where
    G: Clone,
    M: ConstraintModel<G>,
    E: Error + Send + Sync + 'static,
{
    /// Executes the full Repair -> Improve lifecycle on a newly varied genome.
    pub fn process_offspring(&self, candidate: &mut G) -> Result<bool, E> {
        // Step 1: Repair Gate (if infeasible)
        if !self.constraint_model.is_feasible(candidate) {
            let mut repaired = false;
            for op in &self.repair_operators {
                if op.repair(candidate, &self.constraint_model, &self.repair_budget)? {
                    repaired = true;
                    break;
                }
            }
            if !repaired || !self.constraint_model.is_feasible(candidate) {
                return Ok(false); // Candidate remains infeasible; drop from Pareto front
            }
        }

        // Step 2: Improvement Gate (candidate is guaranteed feasible here)
        for op in &self.improvement_operators {
            op.improve(candidate, &self.constraint_model, &self.improve_budget)?;
            debug_assert!(
                self.constraint_model.is_feasible(candidate),
                "Feasibility Preservation Invariant violated by ImprovementOperator"
            );
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operators::NoOpConstraintModel;
    use std::fmt;

    #[derive(Debug, Clone, PartialEq)]
    struct DummyGenome {
        val: i32,
    }

    #[derive(Debug, Clone)]
    struct DummyViolation;

    struct DummyModel {
        // if true, genome is feasible only if val > 0
        require_positive: bool,
    }
    impl ConstraintModel<DummyGenome> for DummyModel {
        type Violation = DummyViolation;
        fn evaluate_violations(&self, candidate: &DummyGenome) -> Vec<Self::Violation> {
            if self.require_positive && candidate.val <= 0 {
                vec![DummyViolation]
            } else {
                vec![]
            }
        }
    }

    #[derive(Debug)]
    struct DummyError;
    impl fmt::Display for DummyError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "DummyError")
        }
    }
    impl Error for DummyError {}

    struct SuccessfulRepair;
    impl RepairOperator<DummyGenome, DummyModel> for SuccessfulRepair {
        type Error = DummyError;
        fn repair(&self, candidate: &mut DummyGenome, _model: &DummyModel, _budget: &OperatorBudget) -> Result<bool, Self::Error> {
            candidate.val = 1;
            Ok(true)
        }
    }

    struct FailedRepair;
    impl RepairOperator<DummyGenome, DummyModel> for FailedRepair {
        type Error = DummyError;
        fn repair(&self, _candidate: &mut DummyGenome, _model: &DummyModel, _budget: &OperatorBudget) -> Result<bool, Self::Error> {
            // Fails to repair
            Ok(false)
        }
    }

    struct SafeImprovement;
    impl ImprovementOperator<DummyGenome, DummyModel> for SafeImprovement {
        type Error = DummyError;
        fn improve(&self, candidate: &mut DummyGenome, _model: &DummyModel, _budget: &OperatorBudget) -> Result<bool, Self::Error> {
            candidate.val += 1;
            Ok(true)
        }
    }

    #[test]
    fn test_pipeline_successful_repair() {
        let model = DummyModel { require_positive: true };
        let pipeline = EvolutionaryPipeline {
            constraint_model: model,
            repair_operators: vec![Box::new(SuccessfulRepair)],
            improvement_operators: vec![Box::new(SafeImprovement)],
            repair_budget: OperatorBudget { max_iterations: 10, max_time_ms: 100 },
            improve_budget: OperatorBudget { max_iterations: 10, max_time_ms: 100 },
        };

        let mut candidate = DummyGenome { val: -5 };
        let result = pipeline.process_offspring(&mut candidate).unwrap();
        
        assert!(result); // Reached evaluation
        assert!(pipeline.constraint_model.is_feasible(&candidate));
        assert_eq!(candidate.val, 2); // 1 from repair, +1 from improve
    }

    #[test]
    fn test_pipeline_failed_repair() {
        let model = DummyModel { require_positive: true };
        let pipeline = EvolutionaryPipeline {
            constraint_model: model,
            repair_operators: vec![Box::new(FailedRepair)],
            improvement_operators: vec![Box::new(SafeImprovement)],
            repair_budget: OperatorBudget { max_iterations: 10, max_time_ms: 100 },
            improve_budget: OperatorBudget { max_iterations: 10, max_time_ms: 100 },
        };

        let mut candidate = DummyGenome { val: -5 };
        let result = pipeline.process_offspring(&mut candidate).unwrap();
        
        assert!(!result); // Did not reach evaluation
        assert!(!pipeline.constraint_model.is_feasible(&candidate));
        assert_eq!(candidate.val, -5); // Improve was never called
    }
}
