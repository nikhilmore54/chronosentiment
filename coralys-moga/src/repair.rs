use crate::traits::{Genome, ImprovementOperator};
use rand::SeedableRng;

pub trait ConstraintChecker<G: Genome>: Send + Sync {
    type Violation: std::fmt::Debug;
    /// Identifies all violations of this constraint on the given candidate.
    fn check_violations(&self, candidate: &G) -> Vec<Self::Violation>;
}

pub trait RepairHeuristic<G: Genome, V>: Send + Sync {
    /// Attempts to repair the candidate based on the specific violation.
    /// Returns true if the repair made progress/modified the genome, false otherwise.
    fn repair_violation(&self, candidate: &mut G, violation: &V, rng: &mut rand::rngs::StdRng) -> bool;
}

pub struct FeasibilityRepairFramework<G: Genome, V> {
    pub checkers: Vec<Box<dyn ConstraintChecker<G, Violation = V>>>,
    pub heuristics: Vec<Box<dyn RepairHeuristic<G, V>>>,
    pub max_iterations: usize,
}

impl<G: Genome, V: std::fmt::Debug> FeasibilityRepairFramework<G, V> {
    pub fn new(max_iterations: usize) -> Self {
        Self {
            checkers: Vec::new(),
            heuristics: Vec::new(),
            max_iterations,
        }
    }

    pub fn add_checker(&mut self, checker: Box<dyn ConstraintChecker<G, Violation = V>>) {
        self.checkers.push(checker);
    }

    pub fn add_heuristic(&mut self, heuristic: Box<dyn RepairHeuristic<G, V>>) {
        self.heuristics.push(heuristic);
    }
}

impl<G: Genome, V: std::fmt::Debug> ImprovementOperator<G> for FeasibilityRepairFramework<G, V> {
    fn improve(&self, candidate: &mut G) {
        let mut rng = rand::rngs::StdRng::from_entropy();
        
        for _iter in 0..self.max_iterations {
            // Collect all current violations
            let mut all_violations = Vec::new();
            for checker in &self.checkers {
                all_violations.extend(checker.check_violations(candidate));
            }

            if all_violations.is_empty() {
                // Feasible!
                return;
            }

            // Attempt to repair each violation
            let mut any_repaired = false;
            for violation in &all_violations {
                for heuristic in &self.heuristics {
                    if heuristic.repair_violation(candidate, violation, &mut rng) {
                        any_repaired = true;
                        break; // Move to the next violation
                    }
                }
            }

            // If no heuristic could make any repair progress, we abort to prevent infinite loop
            if !any_repaired {
                break;
            }
        }
    }
}
