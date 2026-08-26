use crate::models::Shift;
use crate::optimization::{ScheduleContext, ScheduleGenome};
use coralys_core::operators::RepairOperator;
use coralys_core::operators::{ConstraintModel, OperatorBudget};
use rand::seq::SliceRandom;
use std::sync::Arc;
// use rand::thread_rng; // removed unused import

#[derive(Debug, Clone)]
pub enum UltraCrewViolation {
    RestViolation {
        worker_id: u64,
        shift_1_id: u64,
        shift_2_id: u64,
    },
    OverlapViolation {
        worker_id: u64,
        shift_1_id: u64,
        shift_2_id: u64,
    },
    SkillViolation {
        worker_id: u64,
        shift_id: u64,
    },
}

pub struct RestConstraint {
    pub context: Arc<ScheduleContext>,
}

impl ConstraintModel<ScheduleGenome> for RestConstraint {
    type Violation = UltraCrewViolation;

    fn evaluate_violations(&self, genome: &ScheduleGenome) -> Vec<Self::Violation> {
        let mut violations = Vec::new();
        let mut worker_shifts: std::collections::HashMap<u64, Vec<&Shift>> =
            std::collections::HashMap::new();

        for shift in self.context.shifts.iter() {
            if let Some(worker_id) = genome.assignments.get(&shift.id) {
                worker_shifts.entry(*worker_id).or_default().push(shift);
            }
        }

        let min_rest = self
            .context
            .scenario
            .as_ref()
            .and_then(|s| s.minimum_rest_hours)
            .unwrap_or(10);

        for worker in self.context.workers.iter() {
            if let Some(shifts) = worker_shifts.get(&worker.id) {
                let mut sorted_shifts = shifts.clone();
                sorted_shifts.sort_by_key(|s| s.start_hour);

                // Overlaps
                for i in 0..sorted_shifts.len() {
                    for j in (i + 1)..sorted_shifts.len() {
                        let s_i = sorted_shifts[i];
                        let s_j = sorted_shifts[j];
                        if s_i.overlaps_with(s_j) {
                            violations.push(UltraCrewViolation::OverlapViolation {
                                worker_id: worker.id,
                                shift_1_id: s_i.id,
                                shift_2_id: s_j.id,
                            });
                        }
                    }
                }

                // Rest gaps
                for i in 0..sorted_shifts.len().saturating_sub(1) {
                    let s_i = sorted_shifts[i];
                    let s_next = sorted_shifts[i + 1];
                    let gap = if s_next.start_hour >= s_i.end_hour() {
                        s_next.start_hour - s_i.end_hour()
                    } else {
                        0
                    };
                    if gap < min_rest && !s_i.overlaps_with(s_next) {
                        violations.push(UltraCrewViolation::RestViolation {
                            worker_id: worker.id,
                            shift_1_id: s_i.id,
                            shift_2_id: s_next.id,
                        });
                    }
                }
            }
        }

        violations
    }
}

pub struct SkillConstraint {
    pub context: Arc<ScheduleContext>,
}

impl ConstraintModel<ScheduleGenome> for SkillConstraint {
    type Violation = UltraCrewViolation;

    fn evaluate_violations(&self, genome: &ScheduleGenome) -> Vec<Self::Violation> {
        let mut violations = Vec::new();
        for shift in self.context.shifts.iter() {
            if let Some(worker_id) = genome.assignments.get(&shift.id) {
                if let Some(worker) = self.context.workers.iter().find(|w| w.id == *worker_id) {
                    if !worker.skills.contains(&shift.required_skill) {
                        violations.push(UltraCrewViolation::SkillViolation {
                            worker_id: *worker_id,
                            shift_id: shift.id,
                        });
                    }
                }
            }
        }
        violations
    }
}

pub struct ReassignRepairOperator {
    pub context: Arc<ScheduleContext>,
    pub rng: std::sync::Mutex<rand::rngs::StdRng>,
}

// unified constraint model for ultra crew
pub struct InrcConstraintModel {
    pub rest: RestConstraint,
    pub skill: SkillConstraint,
}

impl ConstraintModel<ScheduleGenome> for InrcConstraintModel {
    type Violation = UltraCrewViolation;

    fn evaluate_violations(&self, candidate: &ScheduleGenome) -> Vec<Self::Violation> {
        let mut v = Vec::new();
        v.extend(self.rest.evaluate_violations(candidate));
        v.extend(self.skill.evaluate_violations(candidate));
        v
    }
}

impl RepairOperator<ScheduleGenome, InrcConstraintModel> for ReassignRepairOperator {
    type Error = crate::errors::UltraCrewError;

    fn repair(
        &self,
        genome: &mut ScheduleGenome,
        model: &InrcConstraintModel,
        _budget: &OperatorBudget,
    ) -> Result<bool, Self::Error> {
        let mut repaired_any = false;
        let mut rng = self.rng.lock().unwrap();

        let violations = model.evaluate_violations(genome);
        if violations.is_empty() {
            return Ok(true);
        }

        for violation in violations {
            match violation {
                UltraCrewViolation::RestViolation { shift_2_id, .. }
                | UltraCrewViolation::OverlapViolation { shift_2_id, .. }
                | UltraCrewViolation::SkillViolation {
                    shift_id: shift_2_id,
                    ..
                } => {
                    let target_shift = self.context.shifts.iter().find(|s| s.id == shift_2_id);
                    if let Some(shift) = target_shift {
                        let mut eligible_workers: Vec<u64> = self
                            .context
                            .workers
                            .iter()
                            .filter(|w| w.skills.contains(&shift.required_skill))
                            .map(|w| w.id)
                            .collect();

                        if !eligible_workers.is_empty() {
                            eligible_workers.shuffle(&mut *rng);
                            let new_worker_id = eligible_workers[0];
                            genome.assignments.insert(shift_2_id, new_worker_id);
                            repaired_any = true;
                        }
                    }
                }
            }
        }

        Ok(repaired_any)
    }
}

#[cfg(test)]
mod contract_tests {
    use super::*;
    use crate::models::{Shift, Worker};
    use crate::optimization::ScheduleGenome;
    use coralys_core::operators::{ConstraintModel, OperatorBudget};

    fn dummy_context() -> Arc<ScheduleContext> {
        let mut workers = Vec::new();
        let mut shifts = Vec::new();
        // Create 2 workers
        workers.push(Worker {
            id: 1,
            skills: vec![crate::models::Skill::new("RN")],
        });
        workers.push(Worker {
            id: 2,
            skills: vec![crate::models::Skill::new("RN")],
        });

        // Create shifts that violate rest if assigned to same worker
        shifts.push(Shift {
            id: 101,
            start_hour: 8,
            duration_hours: 8,
            required_skill: crate::models::Skill::new("RN"),
        });
        shifts.push(Shift {
            id: 102,
            start_hour: 16,
            duration_hours: 8,
            required_skill: crate::models::Skill::new("RN"),
        }); // 0 gap

        Arc::new(ScheduleContext {
            workers: Arc::new(workers),
            shifts: Arc::new(shifts),
            ecology: crate::ecology::WorkforceEcology::new(),
            rng_seed: 42,
            observatory: Arc::new(std::sync::Mutex::new(
                crate::optimization::Observatory::new(),
            )),
            locked_assignments: None,
            scenario: Some(crate::public_contracts::InrcScenario {
                minimum_rest_hours: Some(10),
                max_hours_per_worker: Some(40.0),
                planning_horizon_hours: None,
                leave_requests: None,
            }),
        })
    }

    #[test]
    fn test_repair_contract_moves_to_feasible() {
        let ctx = dummy_context();
        let model = InrcConstraintModel {
            rest: RestConstraint {
                context: ctx.clone(),
            },
            skill: SkillConstraint {
                context: ctx.clone(),
            },
        };
        let repair_op = ReassignRepairOperator {
            context: ctx.clone(),
            rng: std::sync::Mutex::new(rand::SeedableRng::seed_from_u64(42)),
        };

        let mut genome = ScheduleGenome {
            assignments: std::collections::HashMap::new(),
        };
        // Assign both to Alice, causing a RestViolation (0 gap < 10)
        genome.assignments.insert(101, 1);
        genome.assignments.insert(102, 1);

        assert!(!model.is_feasible(&genome));

        let budget = OperatorBudget {
            max_iterations: 10,
            max_time_ms: 1000,
        };
        let repaired = repair_op
            .repair(&mut genome, &model, &budget)
            .expect("Repair failed");

        // It should have reassigned one shift to Bob (2)
        assert!(repaired);
        assert!(model.is_feasible(&genome));
    }

    #[test]
    fn test_feasible_genome_remains_feasible() {
        let ctx = dummy_context();
        let model = InrcConstraintModel {
            rest: RestConstraint {
                context: ctx.clone(),
            },
            skill: SkillConstraint {
                context: ctx.clone(),
            },
        };
        let repair_op = ReassignRepairOperator {
            context: ctx.clone(),
            rng: std::sync::Mutex::new(rand::SeedableRng::seed_from_u64(42)),
        };

        let mut genome = ScheduleGenome {
            assignments: std::collections::HashMap::new(),
        };
        // Feasible assignment
        genome.assignments.insert(101, 1);
        genome.assignments.insert(102, 2);

        assert!(model.is_feasible(&genome));

        let budget = OperatorBudget {
            max_iterations: 10,
            max_time_ms: 1000,
        };
        let repaired = repair_op
            .repair(&mut genome, &model, &budget)
            .expect("Repair failed");

        // No repair needed
        assert!(repaired); // or false if your op returns false when no repair
        assert!(model.is_feasible(&genome)); // Remains feasible
    }
}
