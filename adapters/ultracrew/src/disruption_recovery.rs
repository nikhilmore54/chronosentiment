// Disruption recovery workflow for UltraCrew.
//
// Provides a structured end-to-end workflow:
//   DisruptionEvent recorded → affected shifts identified →
//   recovery options generated → options ranked → recovery accepted →
//   DisruptionRecord stored in the Scheduling Workspace.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::models::{Shift, Worker};
use crate::schedule_solution::ScheduleSolution;

// ── Disruption Event ──────────────────────────────────────────────────────────

/// The kind of operational disruption affecting a workforce schedule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisruptionKind {
    /// A worker is unavailable (illness, emergency, no-show).
    WorkerUnavailable { worker_id: u64 },
    /// A shift has been cancelled (operational change, demand drop).
    ShiftCancelled { shift_id: u64 },
    /// A shift has been added urgently (demand spike, emergency coverage).
    ShiftAdded { shift_id: u64 },
    /// A worker's assignment must be swapped (qualification change, preference).
    AssignmentSwap { shift_id: u64, from_worker_id: u64 },
}

/// A recorded disruption event — immutable once created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisruptionEvent {
    pub event_id: String,
    pub kind: DisruptionKind,
    pub description: String,
    pub timestamp: u64,
    pub severity: DisruptionSeverity,
}

/// Severity of a disruption — used to prioritise recovery options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DisruptionSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl DisruptionEvent {
    pub fn new(
        event_id: impl Into<String>,
        kind: DisruptionKind,
        description: impl Into<String>,
        timestamp: u64,
        severity: DisruptionSeverity,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            kind,
            description: description.into(),
            timestamp,
            severity,
        }
    }
}

// ── Recovery Option ───────────────────────────────────────────────────────────

/// A single recovery option — a proposed change to the current schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryOption {
    pub option_id: String,
    pub description: String,
    pub action: RecoveryAction,
    /// Impact score — lower is better (0.0 = no impact, 1.0 = maximum impact).
    pub impact_score: f64,
    /// Whether this option satisfies all hard constraints.
    pub is_feasible: bool,
}

/// The concrete action taken by a recovery option.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryAction {
    /// Reassign a shift from one worker to another.
    Reassign { shift_id: u64, to_worker_id: u64 },
    /// Leave a shift uncovered (only valid if shift is non-critical).
    LeaveUncovered { shift_id: u64 },
    /// Cancel a shift entirely.
    CancelShift { shift_id: u64 },
    /// Split a shift between two workers.
    SplitShift { shift_id: u64, worker_a: u64, worker_b: u64 },
}

// ── Recovery Result ───────────────────────────────────────────────────────────

/// The outcome of a recovery attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryResult {
    pub event_id: String,
    /// Ranked recovery options (best first).
    pub options: Vec<RecoveryOption>,
    /// The accepted option (if any).
    pub accepted_option: Option<RecoveryOption>,
    /// The repaired schedule solution (if recovery was accepted).
    pub repaired_solution: Option<ScheduleSolution>,
    /// Shifts that could not be recovered.
    pub unrecovered_shifts: Vec<u64>,
}

// ── Disruption Record ─────────────────────────────────────────────────────────

/// Immutable record of a disruption event and its resolution.
/// Stored in the Scheduling Workspace as an evidence item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisruptionRecord {
    pub record_id: String,
    pub event: DisruptionEvent,
    pub result: RecoveryResult,
    pub resolved: bool,
    pub resolution_notes: Option<String>,
}

// ── Disruption Recovery Engine ────────────────────────────────────────────────

/// End-to-end disruption recovery workflow for UltraCrew.
///
/// Workflow:
///   1. `record_event` — record the disruption event (immutable)
///   2. `identify_affected_shifts` — find all shifts affected by the disruption
///   3. `generate_options` — generate ranked recovery options
///   4. `accept_option` — accept a recovery option and apply it to the schedule
///   5. `record_resolution` — store the complete disruption record
pub struct DisruptionRecoveryEngine {
    pub records: Vec<DisruptionRecord>,
}

impl DisruptionRecoveryEngine {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    /// Step 1 — Record a disruption event. Returns the event_id.
    pub fn record_event(&self, event: &DisruptionEvent) -> String {
        // Events are immutable once recorded; this method validates and returns the ID.
        event.event_id.clone()
    }

    /// Step 2 — Identify all shift IDs affected by a disruption event.
    pub fn identify_affected_shifts(
        &self,
        event: &DisruptionEvent,
        current_solution: &ScheduleSolution,
    ) -> Vec<u64> {
        match &event.kind {
            DisruptionKind::WorkerUnavailable { worker_id } => {
                // All shifts currently assigned to this worker are affected.
                current_solution
                    .assignments
                    .iter()
                    .filter(|(_, &wid)| wid == *worker_id)
                    .map(|(&sid, _)| sid)
                    .collect()
            }
            DisruptionKind::ShiftCancelled { shift_id } => vec![*shift_id],
            DisruptionKind::ShiftAdded { shift_id } => vec![*shift_id],
            DisruptionKind::AssignmentSwap { shift_id, .. } => vec![*shift_id],
        }
    }

    /// Step 3 — Generate ranked recovery options for the affected shifts.
    ///
    /// Options are generated by trying each available worker against each
    /// affected shift. Options are ranked by impact score (lower = better).
    pub fn generate_options(
        &self,
        event: &DisruptionEvent,
        affected_shifts: &[u64],
        current_solution: &ScheduleSolution,
        available_workers: &[Worker],
        all_shifts: &[Shift],
    ) -> Vec<RecoveryOption> {
        let mut options: Vec<RecoveryOption> = Vec::new();
        let mut option_counter = 0u32;

        let shift_map: HashMap<u64, &Shift> =
            all_shifts.iter().map(|s| (s.id, s)).collect();

        for &shift_id in affected_shifts {
            let Some(shift) = shift_map.get(&shift_id) else { continue };

            match &event.kind {
                DisruptionKind::WorkerUnavailable { worker_id } |
                DisruptionKind::AssignmentSwap { shift_id: _, from_worker_id: worker_id } => {
                    // Try reassigning to each available worker.
                    for worker in available_workers {
                        if worker.id == *worker_id {
                            continue; // skip the unavailable worker
                        }
                        // Check skill match.
                        let skill_match = worker.skills.iter().any(|s| s == &shift.required_skill);
                        // Check for existing assignment conflict (worker already assigned to overlapping shift).
                        let has_conflict = current_solution.assignments.iter().any(|(other_sid, &wid)| {
                            wid == worker.id && *other_sid != shift_id && {
                                if let Some(other_shift) = shift_map.get(other_sid) {
                                    shift.overlaps_with(other_shift)
                                } else {
                                    false
                                }
                            }
                        });

                        let is_feasible = skill_match && !has_conflict;
                        // Impact: prefer workers with fewer current assignments.
                        let current_load = current_solution
                            .assignments
                            .values()
                            .filter(|&&wid| wid == worker.id)
                            .count() as f64;
                        let impact_score = if is_feasible {
                            current_load / (available_workers.len() as f64).max(1.0)
                        } else {
                            1.0
                        };

                        option_counter += 1;
                        options.push(RecoveryOption {
                            option_id: format!("opt-{}", option_counter),
                            description: format!(
                                "Reassign shift {} to worker {} (skill_match={}, conflict={})",
                                shift_id, worker.id, skill_match, has_conflict
                            ),
                            action: RecoveryAction::Reassign {
                                shift_id,
                                to_worker_id: worker.id,
                            },
                            impact_score,
                            is_feasible,
                        });
                    }

                    // Also offer "leave uncovered" as a last resort.
                    option_counter += 1;
                    options.push(RecoveryOption {
                        option_id: format!("opt-{}", option_counter),
                        description: format!("Leave shift {} uncovered (last resort)", shift_id),
                        action: RecoveryAction::LeaveUncovered { shift_id },
                        impact_score: 0.9,
                        is_feasible: true, // always feasible, but high impact
                    });
                }

                DisruptionKind::ShiftCancelled { .. } => {
                    option_counter += 1;
                    options.push(RecoveryOption {
                        option_id: format!("opt-{}", option_counter),
                        description: format!("Cancel shift {} (operational change)", shift_id),
                        action: RecoveryAction::CancelShift { shift_id },
                        impact_score: 0.1,
                        is_feasible: true,
                    });
                }

                DisruptionKind::ShiftAdded { .. } => {
                    // Try assigning the new shift to each available worker.
                    for worker in available_workers {
                        let skill_match = worker.skills.iter().any(|s| s == &shift.required_skill);
                        let has_conflict = current_solution.assignments.iter().any(|(other_sid, &wid)| {
                            wid == worker.id && {
                                if let Some(other_shift) = shift_map.get(other_sid) {
                                    shift.overlaps_with(other_shift)
                                } else {
                                    false
                                }
                            }
                        });
                        let is_feasible = skill_match && !has_conflict;
                        let current_load = current_solution
                            .assignments
                            .values()
                            .filter(|&&wid| wid == worker.id)
                            .count() as f64;
                        let impact_score = if is_feasible {
                            current_load / (available_workers.len() as f64).max(1.0)
                        } else {
                            1.0
                        };

                        option_counter += 1;
                        options.push(RecoveryOption {
                            option_id: format!("opt-{}", option_counter),
                            description: format!(
                                "Assign new shift {} to worker {}",
                                shift_id, worker.id
                            ),
                            action: RecoveryAction::Reassign {
                                shift_id,
                                to_worker_id: worker.id,
                            },
                            impact_score,
                            is_feasible,
                        });
                    }
                }
            }
        }

        // Rank: feasible options first, then by impact score ascending.
        options.sort_by(|a, b| {
            b.is_feasible
                .cmp(&a.is_feasible)
                .then(a.impact_score.partial_cmp(&b.impact_score).unwrap_or(std::cmp::Ordering::Equal))
        });

        options
    }

    /// Step 4 — Accept a recovery option and apply it to the current schedule.
    ///
    /// Returns the repaired `ScheduleSolution` with the accepted option applied.
    pub fn accept_option(
        &self,
        option: &RecoveryOption,
        current_solution: &ScheduleSolution,
    ) -> ScheduleSolution {
        let mut repaired = current_solution.clone();

        match &option.action {
            RecoveryAction::Reassign { shift_id, to_worker_id } => {
                repaired.assignments.insert(*shift_id, *to_worker_id);
            }
            RecoveryAction::LeaveUncovered { shift_id } => {
                repaired.assignments.remove(shift_id);
                repaired.hard_violations += 1; // uncovered shift is a hard violation
            }
            RecoveryAction::CancelShift { shift_id } => {
                repaired.assignments.remove(shift_id);
            }
            RecoveryAction::SplitShift { shift_id, worker_a, .. } => {
                // Simplified: assign to worker_a (split scheduling requires richer model).
                repaired.assignments.insert(*shift_id, *worker_a);
            }
        }

        repaired
    }

    /// Step 5 — Record the complete disruption resolution.
    ///
    /// The record is immutable once stored.
    pub fn record_resolution(&mut self, record: DisruptionRecord) {
        self.records.push(record);
    }

    /// Full workflow: record event → identify affected shifts → generate options →
    /// auto-accept the best feasible option → record resolution.
    ///
    /// Returns the `DisruptionRecord` for the resolved event.
    pub fn recover(
        &mut self,
        event: DisruptionEvent,
        current_solution: &ScheduleSolution,
        available_workers: &[Worker],
        all_shifts: &[Shift],
    ) -> DisruptionRecord {
        let affected_shifts =
            self.identify_affected_shifts(&event, current_solution);

        let options = self.generate_options(
            &event,
            &affected_shifts,
            current_solution,
            available_workers,
            all_shifts,
        );

        // Auto-accept the best feasible option.
        let best = options.iter().find(|o| o.is_feasible).cloned();

        let (accepted_option, repaired_solution, unrecovered_shifts) = match &best {
            Some(opt) => {
                let repaired = self.accept_option(opt, current_solution);
                let unrecovered = match &opt.action {
                    RecoveryAction::LeaveUncovered { shift_id } => vec![*shift_id],
                    _ => vec![],
                };
                (Some(opt.clone()), Some(repaired), unrecovered)
            }
            None => (None, None, affected_shifts.clone()),
        };

        let result = RecoveryResult {
            event_id: event.event_id.clone(),
            options,
            accepted_option,
            repaired_solution,
            unrecovered_shifts,
        };

        let resolved = result.accepted_option.is_some()
            && result.unrecovered_shifts.is_empty();

        let record = DisruptionRecord {
            record_id: format!("rec-{}", event.event_id),
            event,
            result,
            resolved,
            resolution_notes: None,
        };

        self.records.push(record.clone());
        record
    }

    /// Return all disruption records (immutable evidence items).
    pub fn records(&self) -> &[DisruptionRecord] {
        &self.records
    }

    /// Return unresolved disruption records.
    pub fn unresolved(&self) -> Vec<&DisruptionRecord> {
        self.records.iter().filter(|r| !r.resolved).collect()
    }
}

impl Default for DisruptionRecoveryEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Shift, Skill, Worker};

    fn make_solution(assignments: Vec<(u64, u64)>) -> ScheduleSolution {
        ScheduleSolution {
            assignments: assignments.into_iter().collect(),
            fitness: 100.0,
            hard_violations: 0,
            fairness_penalty: 0.0,
            fatigue_penalty: 0.0,
            rest_violations: 0,
            recommendations: None,
            telemetry: None,
        }
    }

    fn make_worker(id: u64, skill: &str) -> Worker {
        Worker { id, skills: vec![Skill::new(skill)] }
    }

    fn make_shift(id: u64, start: u64, skill: &str) -> Shift {
        Shift { id, start_hour: start, duration_hours: 8, required_skill: Skill::new(skill) }
    }

    #[test]
    fn worker_unavailable_identifies_affected_shifts() {
        let solution = make_solution(vec![(1, 10), (2, 10), (3, 20)]);
        let event = DisruptionEvent::new(
            "evt-001",
            DisruptionKind::WorkerUnavailable { worker_id: 10 },
            "Worker 10 sick",
            1000,
            DisruptionSeverity::High,
        );
        let engine = DisruptionRecoveryEngine::new();
        let affected = engine.identify_affected_shifts(&event, &solution);
        assert!(affected.contains(&1));
        assert!(affected.contains(&2));
        assert!(!affected.contains(&3));
    }

    #[test]
    fn recovery_reassigns_to_best_available_worker() {
        // Shift 1 assigned to worker 10 (now unavailable).
        let solution = make_solution(vec![(1, 10)]);
        let workers = vec![
            make_worker(10, "Forklift"),
            make_worker(20, "Forklift"), // available replacement
            make_worker(30, "GeneralLabor"), // wrong skill
        ];
        let shifts = vec![make_shift(1, 8, "Forklift")];

        let event = DisruptionEvent::new(
            "evt-002",
            DisruptionKind::WorkerUnavailable { worker_id: 10 },
            "Worker 10 no-show",
            2000,
            DisruptionSeverity::High,
        );

        let mut engine = DisruptionRecoveryEngine::new();
        let record = engine.recover(event, &solution, &workers, &shifts);

        assert!(record.resolved);
        assert!(record.result.accepted_option.is_some());
        // The accepted option should reassign shift 1 to worker 20.
        if let Some(opt) = &record.result.accepted_option {
            assert!(opt.is_feasible);
            if let RecoveryAction::Reassign { shift_id, to_worker_id } = opt.action {
                assert_eq!(shift_id, 1);
                assert_eq!(to_worker_id, 20);
            }
        }
    }

    #[test]
    fn shift_cancelled_generates_cancel_option() {
        let solution = make_solution(vec![(5, 10)]);
        let workers = vec![make_worker(10, "Forklift")];
        let shifts = vec![make_shift(5, 8, "Forklift")];

        let event = DisruptionEvent::new(
            "evt-003",
            DisruptionKind::ShiftCancelled { shift_id: 5 },
            "Shift 5 cancelled — demand drop",
            3000,
            DisruptionSeverity::Low,
        );

        let mut engine = DisruptionRecoveryEngine::new();
        let record = engine.recover(event, &solution, &workers, &shifts);

        assert!(record.resolved);
        if let Some(opt) = &record.result.accepted_option {
            assert!(matches!(opt.action, RecoveryAction::CancelShift { shift_id: 5 }));
        }
    }

    #[test]
    fn records_are_stored_after_recovery() {
        let solution = make_solution(vec![(1, 10)]);
        let workers = vec![make_worker(10, "Forklift"), make_worker(20, "Forklift")];
        let shifts = vec![make_shift(1, 8, "Forklift")];

        let event = DisruptionEvent::new(
            "evt-004",
            DisruptionKind::WorkerUnavailable { worker_id: 10 },
            "Worker 10 sick",
            4000,
            DisruptionSeverity::Medium,
        );

        let mut engine = DisruptionRecoveryEngine::new();
        engine.recover(event, &solution, &workers, &shifts);

        assert_eq!(engine.records().len(), 1);
        assert_eq!(engine.unresolved().len(), 0);
    }
}