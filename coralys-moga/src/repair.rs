use crate::traits::{Genome, ImprovementOperator};
use rand::SeedableRng;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub trait ConstraintChecker<G: Genome>: Send + Sync {
    type Violation: std::fmt::Debug;
    
    /// Friendly name of this constraint checker for telemetry.
    fn name(&self) -> &'static str {
        "UnnamedChecker"
    }

    /// Identifies all violations of this constraint on the given candidate.
    fn check_violations(&self, candidate: &G) -> Vec<Self::Violation>;
}

pub trait RepairHeuristic<G: Genome, V>: Send + Sync {
    /// Friendly name of this heuristic for telemetry.
    fn name(&self) -> &'static str {
        "UnnamedHeuristic"
    }

    /// Attempts to repair the candidate based on the specific violation.
    /// Returns true if the repair made progress/modified the genome, false otherwise.
    fn repair_violation(&self, candidate: &mut G, violation: &V, rng: &mut rand::rngs::StdRng) -> bool;
}

#[derive(Debug, Clone, Default)]
pub struct RepairStats {
    pub total_invocations: usize,
    pub successful_repairs: usize,
    pub failed_repairs: usize,
    pub total_iterations: usize,
    pub violations_encountered: HashMap<String, usize>,
    pub heuristic_successes: HashMap<String, usize>,
    pub heuristic_attempts: HashMap<String, usize>,
}

pub struct FeasibilityRepairFramework<G: Genome, V> {
    pub checkers: Vec<Box<dyn ConstraintChecker<G, Violation = V>>>,
    pub heuristics: Vec<Box<dyn RepairHeuristic<G, V>>>,
    pub max_iterations: usize,
    pub stats: Arc<Mutex<RepairStats>>,
}

impl<G: Genome, V: std::fmt::Debug> FeasibilityRepairFramework<G, V> {
    pub fn new(max_iterations: usize) -> Self {
        Self {
            checkers: Vec::new(),
            heuristics: Vec::new(),
            max_iterations,
            stats: Arc::new(Mutex::new(RepairStats::default())),
        }
    }

    pub fn add_checker(&mut self, checker: Box<dyn ConstraintChecker<G, Violation = V>>) {
        self.checkers.push(checker);
    }

    pub fn add_heuristic(&mut self, heuristic: Box<dyn RepairHeuristic<G, V>>) {
        self.heuristics.push(heuristic);
    }

    /// Returns a snapshot of the current repair statistics.
    pub fn stats_snapshot(&self) -> RepairStats {
        self.stats.lock().unwrap().clone()
    }
}

impl<G: Genome, V: std::fmt::Debug> ImprovementOperator<G> for FeasibilityRepairFramework<G, V> {
    fn improve(&self, candidate: &mut G) {
        let mut rng = rand::rngs::StdRng::from_entropy();
        let mut iterations_run = 0;
        let mut violations_logged = Vec::new();
        let mut heuristic_attempts_logged = Vec::new();
        let mut heuristic_successes_logged = Vec::new();
        
        let mut final_feasible = false;

        for _iter in 0..self.max_iterations {
            iterations_run += 1;
            // Collect all current violations
            let mut all_violations = Vec::new();
            for checker in &self.checkers {
                let v = checker.check_violations(candidate);
                for violation in &v {
                    violations_logged.push(format!("{:?}", violation));
                }
                all_violations.extend(v);
            }

            if all_violations.is_empty() {
                final_feasible = true;
                break;
            }

            // Attempt to repair each violation
            let mut any_repaired = false;
            for violation in &all_violations {
                for heuristic in &self.heuristics {
                    let h_name = heuristic.name();
                    heuristic_attempts_logged.push(h_name.to_string());
                    if heuristic.repair_violation(candidate, violation, &mut rng) {
                        heuristic_successes_logged.push(h_name.to_string());
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

        // Post-verification for final status log
        if !final_feasible {
            let mut final_violations = Vec::new();
            for checker in &self.checkers {
                final_violations.extend(checker.check_violations(candidate));
            }
            if final_violations.is_empty() {
                final_feasible = true;
            }
        }

        // Update statistics
        let mut stats = self.stats.lock().unwrap();
        stats.total_invocations += 1;
        stats.total_iterations += iterations_run;
        if final_feasible {
            stats.successful_repairs += 1;
        } else {
            stats.failed_repairs += 1;
        }
        for v in violations_logged {
            *stats.violations_encountered.entry(v).or_insert(0) += 1;
        }
        for h in heuristic_attempts_logged {
            *stats.heuristic_attempts.entry(h).or_insert(0) += 1;
        }
        for h in heuristic_successes_logged {
            *stats.heuristic_successes.entry(h).or_insert(0) += 1;
        }
    }
}
