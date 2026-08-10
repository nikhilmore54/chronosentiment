use crate::optimization::{ScheduleGenome, ScheduleContext};
use coralys_moga::runtime::optimization::constraint::{ConstraintModel, RepairOperator, ConstraintViolation, ConstraintTier};
use std::sync::Arc;
use crate::models::{Worker, Shift};
use rand::seq::SliceRandom;
use rand::thread_rng;

pub enum UltraCrewViolation {
    RestViolation { worker_id: u64, shift_1_id: u64, shift_2_id: u64 },
    OverlapViolation { worker_id: u64, shift_1_id: u64, shift_2_id: u64 },
    SkillViolation { worker_id: u64, shift_id: u64 },
}

impl ConstraintViolation for UltraCrewViolation {
    fn description(&self) -> String {
        match self {
            Self::RestViolation { worker_id, shift_1_id, shift_2_id } => {
                format!("Rest violation for worker {} between shifts {} and {}", worker_id, shift_1_id, shift_2_id)
            },
            Self::OverlapViolation { worker_id, shift_1_id, shift_2_id } => {
                format!("Overlap violation for worker {} between shifts {} and {}", worker_id, shift_1_id, shift_2_id)
            },
            Self::SkillViolation { worker_id, shift_id } => {
                format!("Skill mismatch for worker {} on shift {}", worker_id, shift_id)
            },
        }
    }
}

pub struct RestConstraint {
    pub context: Arc<ScheduleContext>,
}

impl ConstraintModel<ScheduleGenome, UltraCrewViolation> for RestConstraint {
    fn tier(&self) -> ConstraintTier {
        ConstraintTier::Safety
    }

    fn name(&self) -> String {
        "Rest Gap & Overlap".to_string()
    }

    fn evaluate(&self, genome: &ScheduleGenome, metrics: &coralys_moga::runtime::optimization::metric::MetricReport) -> coralys_moga::runtime::optimization::constraint::ConstraintAssessment<UltraCrewViolation> {
        let mut violations = Vec::new();
        let mut worker_shifts: std::collections::HashMap<u64, Vec<&Shift>> = std::collections::HashMap::new();

        for shift in self.context.shifts.iter() {
            if let Some(worker_id) = genome.assignments.get(&shift.id) {
                worker_shifts.entry(*worker_id).or_default().push(shift);
            }
        }

        let min_rest = self.context.scenario
            .as_ref()
            .and_then(|s| s.minimum_rest_hours)
            .unwrap_or(10); 

        for (worker_id, shifts) in worker_shifts {
            let mut sorted_shifts = shifts.clone();
            sorted_shifts.sort_by_key(|s| s.start_hour);

            // Overlaps
            for i in 0..sorted_shifts.len() {
                for j in (i + 1)..sorted_shifts.len() {
                    let s_i = sorted_shifts[i];
                    let s_j = sorted_shifts[j];
                    if s_i.overlaps_with(s_j) {
                        violations.push(UltraCrewViolation::OverlapViolation {
                            worker_id,
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
                } else { 0 };
                if gap < min_rest && !s_i.overlaps_with(s_next) {
                    violations.push(UltraCrewViolation::RestViolation {
                        worker_id,
                        shift_1_id: s_i.id,
                        shift_2_id: s_next.id,
                    });
                }
            }
        }

        let status = if violations.is_empty() {
            coralys_moga::runtime::optimization::constraint::AssessmentStatus::Pass
        } else {
            coralys_moga::runtime::optimization::constraint::AssessmentStatus::Failed
        };

        coralys_moga::runtime::optimization::constraint::ConstraintAssessment {
            constraint_id: self.name(),
            tier: self.tier(),
            status,
            violations,
            metrics: std::collections::HashMap::new(),
            margins: std::collections::HashMap::new(),
            repairability: true,
            diagnostics: Vec::new(),
        }
    }
}

pub struct SkillConstraint {
    pub context: Arc<ScheduleContext>,
}

impl ConstraintModel<ScheduleGenome, UltraCrewViolation> for SkillConstraint {
    fn tier(&self) -> ConstraintTier {
        ConstraintTier::Safety
    }

    fn name(&self) -> String {
        "Required Skills".to_string()
    }

    fn evaluate(&self, genome: &ScheduleGenome, metrics: &coralys_moga::runtime::optimization::metric::MetricReport) -> coralys_moga::runtime::optimization::constraint::ConstraintAssessment<UltraCrewViolation> {
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
        let status = if violations.is_empty() {
            coralys_moga::runtime::optimization::constraint::AssessmentStatus::Pass
        } else {
            coralys_moga::runtime::optimization::constraint::AssessmentStatus::Failed
        };

        coralys_moga::runtime::optimization::constraint::ConstraintAssessment {
            constraint_id: self.name(),
            tier: self.tier(),
            status,
            violations,
            metrics: std::collections::HashMap::new(),
            margins: std::collections::HashMap::new(),
            repairability: true,
            diagnostics: Vec::new(),
        }
    }
}

pub struct ReassignRepairOperator {
    pub context: Arc<ScheduleContext>,
}

use coralys_moga::runtime::optimization::constraint::RepairAction;

pub struct UltraCrewRepairAction {
    pub shift_id: u64,
    pub new_worker_id: u64,
    pub priority: f64,
}

impl RepairAction<ScheduleGenome> for UltraCrewRepairAction {
    fn priority(&self) -> f64 {
        self.priority
    }

    fn description(&self) -> String {
        format!("Reassign shift {} to worker {}", self.shift_id, self.new_worker_id)
    }

    fn payload(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "action": "reassign_shift",
            "shift_id": self.shift_id,
            "new_worker_id": self.new_worker_id
        }))
    }

    fn apply(&self, model: &mut ScheduleGenome) -> Result<(), String> {
        model.assignments.insert(self.shift_id, self.new_worker_id);
        Ok(())
    }
}

impl RepairOperator<ScheduleGenome, UltraCrewViolation> for ReassignRepairOperator {
    fn repair(&self, _genome: &ScheduleGenome, violation: &UltraCrewViolation) -> Vec<Box<dyn RepairAction<ScheduleGenome>>> {
        let mut actions: Vec<Box<dyn RepairAction<ScheduleGenome>>> = Vec::new();
        match violation {
            UltraCrewViolation::RestViolation { shift_2_id, .. } | 
            UltraCrewViolation::OverlapViolation { shift_2_id, .. } |
            UltraCrewViolation::SkillViolation { shift_id: shift_2_id, .. } => {
                let target_shift = self.context.shifts.iter().find(|s| s.id == *shift_2_id);
                if let Some(shift) = target_shift {
                    let mut eligible_workers: Vec<u64> = self.context.workers.iter()
                        .filter(|w| w.skills.contains(&shift.required_skill))
                        .map(|w| w.id)
                        .collect();
                    
                    let mut rng = thread_rng();
                    eligible_workers.shuffle(&mut rng);

                    // Generate a proposal for each eligible worker (limit to top 3 to avoid explosion)
                    for worker_id in eligible_workers.into_iter().take(3) {
                        actions.push(Box::new(UltraCrewRepairAction {
                            shift_id: *shift_2_id,
                            new_worker_id: worker_id,
                            priority: 1.0,
                        }));
                    }
                }
            }
        }
        actions
    }
}
