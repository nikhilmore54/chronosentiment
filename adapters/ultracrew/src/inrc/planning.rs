//! `PlanningScenario` implementation for the INRC2 nurse scheduling domain.
//!
//! This module implements the `coralys-planning` execution contract for
//! `InrcScenario` without semantic distortion (Principle 10).
//!
//! # Domain mapping
//!
//! | `coralys-planning` trait | INRC2 type                                      |
//! |--------------------------|-------------------------------------------------|
//! | `Worker`                 | `InrcNurse`                                     |
//! | `PlanningUnit`           | `InrcShiftAssignment` (nurse × shift × day)     |
//! | `CoverageDemand`         | `InrcDemandSlot` (shift × skill × day → min/opt)|
//! | `PlanningSolution`       | `InrcSchedule`                                  |
//! | `PlanningScenario`       | `InrcScenario`                                  |
//!
//! The INRC2 Atomic Planning Unit is a shift assignment
//! `(nurse_id, shift_type_id, day_index)`.  It is atomic and indivisible —
//! a nurse is either assigned to a shift on a day or not.

use coralys_planning::{CoverageDemand, PlanningScenario, PlanningSolution, PlanningUnit, Worker};

use super::models::{InrcNurse, InrcRequirement, InrcScenario};

// ── Worker ────────────────────────────────────────────────────────────────────

impl Worker for InrcNurse {
    fn id(&self) -> &str {
        &self.id
    }
}

// ── PlanningUnit ──────────────────────────────────────────────────────────────

/// The atomic planning unit in the INRC2 domain: a single shift assignment.
///
/// Represents the decision "nurse `nurse_id` works shift `shift_type_id` on
/// day `day_index`".  This is the smallest indivisible unit of work in the
/// INRC2 problem.
#[derive(Clone, Debug)]
pub struct InrcShiftAssignment {
    /// Composite identifier: `"{nurse_id}:{shift_type_id}:{day_index}"`.
    pub id: String,
    pub nurse_id: String,
    pub shift_type_id: String,
    pub day_index: usize,
}

impl InrcShiftAssignment {
    pub fn new(nurse_id: &str, shift_type_id: &str, day_index: usize) -> Self {
        Self {
            id: format!("{}:{}:{}", nurse_id, shift_type_id, day_index),
            nurse_id: nurse_id.to_string(),
            shift_type_id: shift_type_id.to_string(),
            day_index,
        }
    }
}

impl PlanningUnit for InrcShiftAssignment {
    fn id(&self) -> &str {
        &self.id
    }
}

// ── CoverageDemand ────────────────────────────────────────────────────────────

/// A single (shift_type × skill × day) coverage slot from `InrcRequirement`.
///
/// `InrcRequirement` covers all seven days; this struct represents one day's
/// slice so that each `CoverageDemand` has a single minimum and optimal value.
#[derive(Clone, Debug)]
pub struct InrcDemandSlot {
    pub id: String,
    pub shift_type: String,
    pub skill: String,
    pub day_index: usize,
    pub minimum: usize,
    pub optimal: usize,
}

impl CoverageDemand for InrcDemandSlot {
    fn minimum(&self) -> usize {
        self.minimum
    }

    fn optimal(&self) -> usize {
        self.optimal
    }
}

/// Expand an `InrcRequirement` (which covers all 7 days) into 7 `InrcDemandSlot`s.
pub fn expand_requirement(req: &InrcRequirement) -> Vec<InrcDemandSlot> {
    let days = [
        (0usize, req.monday.minimum, req.monday.optimal),
        (1, req.tuesday.minimum, req.tuesday.optimal),
        (2, req.wednesday.minimum, req.wednesday.optimal),
        (3, req.thursday.minimum, req.thursday.optimal),
        (4, req.friday.minimum, req.friday.optimal),
        (5, req.saturday.minimum, req.saturday.optimal),
        (6, req.sunday.minimum, req.sunday.optimal),
    ];
    days.iter()
        .map(|&(day_index, minimum, optimal)| InrcDemandSlot {
            id: format!("{}:{}:{}", req.shift_type, req.skill, day_index),
            shift_type: req.shift_type.clone(),
            skill: req.skill.clone(),
            day_index,
            minimum,
            optimal,
        })
        .collect()
}

// ── PlanningSolution ──────────────────────────────────────────────────────────

/// A complete INRC2 schedule: a set of `(InrcNurse, InrcShiftAssignment)` pairs.
#[derive(Clone, Debug)]
pub struct InrcSchedule {
    pub assignments: Vec<(InrcNurse, InrcShiftAssignment)>,
    pub feasible: bool,
}

impl PlanningSolution for InrcSchedule {
    type W = InrcNurse;
    type U = InrcShiftAssignment;

    fn assignments(&self) -> impl Iterator<Item = (&InrcNurse, &InrcShiftAssignment)> {
        self.assignments.iter().map(|(n, a)| (n, a))
    }

    fn is_feasible(&self) -> bool {
        self.feasible
    }
}

// ── PlanningScenario ──────────────────────────────────────────────────────────

/// Adapter that presents `InrcScenario` as a `PlanningScenario`.
///
/// This struct is constructed once from an `InrcScenario` and pre-computes
/// the planning units and coverage demands so that the trait methods are O(1).
pub struct InrcPlanningScenario {
    scenario: InrcScenario,
    planning_units: Vec<InrcShiftAssignment>,
    coverage_demands: Vec<InrcDemandSlot>,
}

impl InrcPlanningScenario {
    /// Build an `InrcPlanningScenario` from an `InrcScenario`.
    ///
    /// Planning units are generated for every (nurse × shift_type × day)
    /// combination across the full planning horizon.  Coverage demands are
    /// expanded from the week data requirements.
    ///
    /// `week_requirements` should come from `InrcWeekData::requirements`.
    pub fn new(scenario: InrcScenario, week_requirements: &[InrcRequirement]) -> Self {
        let days_per_week = 7usize;
        let total_days = scenario.number_of_weeks * days_per_week;

        // Generate all possible shift assignments (nurse × shift × day).
        let mut planning_units = Vec::new();
        for nurse in &scenario.nurses {
            for shift in &scenario.shift_types {
                for day in 0..total_days {
                    planning_units.push(InrcShiftAssignment::new(&nurse.id, &shift.id, day));
                }
            }
        }

        // Expand each InrcRequirement into per-day demand slots.
        let coverage_demands: Vec<InrcDemandSlot> = week_requirements
            .iter()
            .flat_map(expand_requirement)
            .collect();

        Self {
            scenario,
            planning_units,
            coverage_demands,
        }
    }
}

impl PlanningScenario for InrcPlanningScenario {
    type W = InrcNurse;
    type U = InrcShiftAssignment;
    type D = InrcDemandSlot;
    type S = InrcSchedule;

    fn id(&self) -> &str {
        &self.scenario.id
    }

    fn workers(&self) -> &[InrcNurse] {
        &self.scenario.nurses
    }

    fn planning_units(&self) -> &[InrcShiftAssignment] {
        &self.planning_units
    }

    fn coverage_demands(&self) -> &[InrcDemandSlot] {
        &self.coverage_demands
    }
}
