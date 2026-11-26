//! Coralys Decision Engine Integration for Airline Scheduling.
//!
//! Implements the platform [`coralys_core::DecisionPlugin`] contract for the airline domain,
//! guaranteeing that all operational decisions flow through the domain adapter,
//! are simulated and legality-certified under statutory rules, and maintain an immutable lineage.

use crate::domain::crew::CrewId;
use crate::domain::flight::FlightLeg;
use crate::domain::pairing::{Pairing, PairingId};
use crate::domain::roster::Roster;
use crate::domain::rotation::Rotation;
use crate::legality::{LegalityChecker, ViolationSeverity};

use chrono::Utc;
use coralys_core::models::decision_lineage::LineageNode;
use coralys_core::{
    DecisionLineage, DecisionPlugin, DecisionProposal, EvaluationResult, ReasoningEngine,
    SimulationResult, StateReference, Violation,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::optimization::cost::CostEvaluator;
use crate::optimization::metrics::OptimizationMetrics;
use crate::optimization::search::evolutionary::{EvolutionaryConfig, EvolutionaryScheduler};

/// Operational domain state managed by the airline adapter.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AirlineState {
    pub roster: Roster,
    pub unassigned_pairings: Vec<Pairing>,
    pub unassigned_legs: Vec<FlightLeg>,
    pub reference: StateReference,
}

impl AirlineState {
    pub fn new(
        roster: Roster,
        unassigned_pairings: Vec<Pairing>,
        unassigned_legs: Vec<FlightLeg>,
    ) -> Self {
        let reference = StateReference {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            plugin: "airline".to_string(),
            metadata: HashMap::new(),
        };
        Self {
            roster,
            unassigned_pairings,
            unassigned_legs,
            reference,
        }
    }
}

/// Request parameters for Day-of-Operations IROPS Recovery (OCC).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IropsRepairRequest {
    /// Timestamp up to which the schedule is frozen (already operated/in-progress).
    pub frozen_until: chrono::DateTime<chrono::Utc>,
    /// List of crew members calling in sick / unavailable after frozen_until.
    pub sick_crew: Vec<CrewId>,
    /// Specific cancelled flight leg IDs.
    pub cancelled_legs: Vec<String>,
    /// Delayed flight leg IDs with delay minutes.
    pub delayed_legs: Vec<(String, i64)>,
}

/// Operational actions carried in `DecisionProposal.payload`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AirlineAction {
    /// Apply a complete newly generated or optimized roster.
    ApplyRoster {
        roster: Roster,
        unassigned_pairings: Vec<Pairing>,
        unassigned_legs: Vec<FlightLeg>,
    },
    /// Assign a specific pairing to a crew rotation.
    AssignPairing {
        crew_id: CrewId,
        pairing: Pairing,
    },
    /// Rebundle unassigned monolithic pairings at base touches into out-and-back turns.
    RebundleAndAssign {
        rescued_pairings: Vec<Pairing>,
        assignments: Vec<(CrewId, PairingId)>,
    },
    /// Allocate reserve standby crew to uncovered pairings.
    AllocateReserve {
        assignments: Vec<(CrewId, PairingId)>,
    },
    /// Reserve crew sizing recommendation (capacity augmentation).
    RecommendReserveCapacity {
        station_deficits: HashMap<String, usize>,
        total_crew_needed: usize,
    },
    /// Deploy an outstation tour where an away crew operates multiple station pairings.
    ApplyOutstationTour {
        tour: crate::domain::pairing::OutstationTour,
        crew_id: CrewId,
    },
    /// Day-of-Operations IROPS recovery action preserving the frozen historical prefix.
    ApplyIropsRepair {
        repaired_roster: Roster,
        uncovered_pairings: Vec<Pairing>,
        disrupted_count: usize,
        repaired_count: usize,
    },
    /// Standard airline practice: month-end rollover keeping crew intact across the boundary.
    /// Only pre-midnight block time counts against January's 85h cap, and February rollover legs
    /// are locked as pre-assigned Carry-In for February bidding.
    ApplyMonthEndRollover {
        crew_id: CrewId,
        pairing: Pairing,
        carry_in_lock: crate::domain::pairing::CarryInLock,
    },
    /// Exception fallback: Hub split when the assigned pilot has zero January headroom on Jan 31.
    /// Pilot A operates January legs into the hub; a fresh February reserve pilot picks up the aircraft on Feb 1.
    ApplyHubSplitFallback {
        hub: crate::domain::flight::AirportCode,
        pilot_a_crew_id: CrewId,
        pilot_a_pairing: Pairing,
        february_reserve_crew_id: Option<CrewId>,
        february_pairing: Pairing,
    },
}

/// Decision plugin for the airline scheduling domain, implementing [`coralys_core::DecisionPlugin`].
pub struct AirlineDecisionPlugin {
    pub checker: LegalityChecker,
    pub current_state: Mutex<AirlineState>,
}

impl AirlineDecisionPlugin {
    pub fn new(initial_state: AirlineState, checker: LegalityChecker) -> Self {
        Self {
            checker,
            current_state: Mutex::new(initial_state),
        }
    }
}

impl DecisionPlugin for AirlineDecisionPlugin {
    type State = AirlineState;
    type Evaluation = EvaluationResult;

    fn current_state(&self) -> Self::State {
        self.current_state.lock().unwrap().clone()
    }

    fn evaluate(&self, state: &Self::State) -> Self::Evaluation {
        // Run legality checker against the state's roster
        let violations = self.checker.check(&state.roster);

        let mut hard_violations = Vec::new();
        let mut soft_violations = Vec::new();

        for v in violations {
            let core_v = Violation {
                constraint_id: v.rule_id.clone(),
                severity: format!("{:?}", v.severity),
                value: Some(v.observed),
                expected: format!("{}", v.threshold),
                actual: format!("{}", v.observed),
                description: format!("{}: {}", v.entity, v.message),
                penalty: match v.severity {
                    ViolationSeverity::Error => 100_000,
                    ViolationSeverity::Warning => 1_000,
                    ViolationSeverity::Advisory => 10,
                },
            };

            if v.is_error() {
                hard_violations.push(core_v);
            } else {
                soft_violations.push(core_v);
            }
        }

        let assigned_pairings: usize = state.roster.rotations().map(|r| r.pairings().len()).sum();
        let total_pairings = assigned_pairings + state.unassigned_pairings.len();
        let covered_legs = state.roster.legs().count();
        let total_legs = covered_legs + state.unassigned_legs.len();

        let pairing_cov_pct = if total_pairings > 0 {
            (assigned_pairings as f64 / total_pairings as f64) * 100.0
        } else {
            100.0
        };

        let leg_cov_pct = if total_legs > 0 {
            (covered_legs as f64 / total_legs as f64) * 100.0
        } else {
            100.0
        };

        let mut metrics = HashMap::new();
        metrics.insert("assigned_pairings".to_string(), assigned_pairings as f64);
        metrics.insert(
            "unassigned_pairings".to_string(),
            state.unassigned_pairings.len() as f64,
        );
        metrics.insert(
            "unassigned_legs".to_string(),
            state.unassigned_legs.len() as f64,
        );
        metrics.insert("pairing_coverage_pct".to_string(), pairing_cov_pct);
        metrics.insert("flight_coverage_pct".to_string(), leg_cov_pct);
        metrics.insert(
            "crew_count".to_string(),
            state.roster.rotations().count() as f64,
        );
        metrics.insert("hard_violations".to_string(), hard_violations.len() as f64);
        metrics.insert("soft_violations".to_string(), soft_violations.len() as f64);

        // Objectives: higher is better in Coralys.
        // Objective 0: flight coverage percentage (maximized, priority 1)
        // Objective 1: -hard_violations * 10,000.0 (strictly 0)
        // Objective 2: pairing coverage percentage
        let objectives = vec![
            leg_cov_pct,
            -(hard_violations.len() as f64) * 10_000.0,
            pairing_cov_pct,
        ];

        EvaluationResult {
            objectives,
            hard_constraint_violations: hard_violations,
            soft_constraint_violations: soft_violations,
            metrics,
        }
    }

    fn simulate(
        &self,
        state: &Self::State,
        proposal: &DecisionProposal,
    ) -> SimulationResult<Self::State> {
        let action: AirlineAction = serde_json::from_value(proposal.payload.clone())
            .map_err(|e| format!("Failed to deserialize AirlineAction: {}", e))?;

        let mut next_state = state.clone();

        match action {
            AirlineAction::ApplyRoster {
                roster,
                unassigned_pairings,
                unassigned_legs,
            } => {
                next_state.roster = roster;
                next_state.unassigned_pairings = unassigned_pairings;
                next_state.unassigned_legs = unassigned_legs;
            }
            AirlineAction::AssignPairing { crew_id, pairing } => {
                let pid = pairing.id.clone();
                let mut rotations: Vec<Rotation> = next_state.roster.rotations().cloned().collect();
                if let Some(rot) = rotations.iter_mut().find(|r| r.crew_id == crew_id) {
                    let mut pairings = rot.pairings().to_vec();
                    pairings.push(pairing);
                    pairings.sort_by_key(|p| p.start());
                    let new_rot = Rotation::new(rot.id.clone(), crew_id.clone(), pairings)
                        .map_err(|e| format!("Rotation error: {:?}", e))?;
                    *rot = new_rot;
                } else {
                    return Err(format!("Crew {} not found in roster", crew_id));
                }
                next_state.roster = Roster::with_crew(
                    next_state.roster.id.clone(),
                    next_state.roster.period.clone(),
                    next_state.roster.legs().cloned().collect(),
                    rotations,
                    next_state.roster.crew_members().cloned().collect(),
                )
                .map_err(|e| format!("Roster error: {:?}", e))?;

                next_state.unassigned_pairings.retain(|p| p.id != pid);
            }
            AirlineAction::RebundleAndAssign {
                rescued_pairings,
                assignments,
            } => {
                let mut pairing_lookup: HashMap<PairingId, Pairing> = HashMap::new();
                for p in rescued_pairings {
                    pairing_lookup.insert(p.id.clone(), p);
                }

                let mut rotations: Vec<Rotation> = next_state.roster.rotations().cloned().collect();
                for (crew_id, pid) in assignments {
                    if let Some(p) = pairing_lookup.get(&pid) {
                        if let Some(rot) = rotations.iter_mut().find(|r| r.crew_id == crew_id) {
                            let mut pairings = rot.pairings().to_vec();
                            pairings.push(p.clone());
                            pairings.sort_by_key(|p| p.start());
                            let new_rot = Rotation::new(rot.id.clone(), crew_id.clone(), pairings)
                                .map_err(|e| format!("Rotation error: {:?}", e))?;
                            *rot = new_rot;
                        }
                    }
                }
                next_state.roster = Roster::with_crew(
                    next_state.roster.id.clone(),
                    next_state.roster.period.clone(),
                    next_state.roster.legs().cloned().collect(),
                    rotations,
                    next_state.roster.crew_members().cloned().collect(),
                )
                .map_err(|e| format!("Roster error: {:?}", e))?;
            }
            AirlineAction::AllocateReserve { assignments } => {
                let mut rotations: Vec<Rotation> = next_state.roster.rotations().cloned().collect();
                for (crew_id, pid) in assignments {
                    if let Some(idx) = next_state
                        .unassigned_pairings
                        .iter()
                        .position(|p| p.id == pid)
                    {
                        let pairing = next_state.unassigned_pairings.remove(idx);
                        if let Some(rot) = rotations.iter_mut().find(|r| r.crew_id == crew_id) {
                            let mut pairings = rot.pairings().to_vec();
                            pairings.push(pairing);
                            pairings.sort_by_key(|p| p.start());
                            let new_rot = Rotation::new(rot.id.clone(), crew_id.clone(), pairings)
                                .map_err(|e| format!("Rotation error: {:?}", e))?;
                            *rot = new_rot;
                        }
                    }
                }
                next_state.roster = Roster::with_crew(
                    next_state.roster.id.clone(),
                    next_state.roster.period.clone(),
                    next_state.roster.legs().cloned().collect(),
                    rotations,
                    next_state.roster.crew_members().cloned().collect(),
                )
                .map_err(|e| format!("Roster error: {:?}", e))?;
            }
            AirlineAction::RecommendReserveCapacity { .. } => {
                // Advisory proposal; does not modify roster directly
            }
            AirlineAction::ApplyOutstationTour { tour, crew_id } => {
                let comp_id = PairingId::new(format!(
                    "TOUR_{}_{}",
                    tour.outstation_base.as_str(),
                    tour.home_base.as_str()
                ));
                let composite = tour
                    .to_composite_pairing(comp_id)
                    .map_err(|e| format!("OutstationTour composition failed: {:?}", e))?;

                let mut rotations: Vec<Rotation> =
                    next_state.roster.rotations().cloned().collect();
                if let Some(rot) = rotations.iter_mut().find(|r| r.crew_id == crew_id) {
                    let mut pairings = rot.pairings().to_vec();
                    pairings.push(composite);
                    pairings.sort_by_key(|p| p.start());
                    let new_rot = Rotation::new(rot.id.clone(), crew_id.clone(), pairings)
                        .map_err(|e| format!("Rotation error: {:?}", e))?;
                    *rot = new_rot;
                } else {
                    return Err(format!("Crew {} not found in roster", crew_id));
                }

                let covered_ids: std::collections::HashSet<PairingId> =
                    tour.station_pairings.iter().map(|p| p.id.clone()).collect();
                next_state
                    .unassigned_pairings
                    .retain(|p| !covered_ids.contains(&p.id));

                next_state.roster = Roster::with_crew(
                    next_state.roster.id.clone(),
                    next_state.roster.period.clone(),
                    next_state.roster.legs().cloned().collect(),
                    rotations,
                    next_state.roster.crew_members().cloned().collect(),
                )
                .map_err(|e| format!("Roster error: {:?}", e))?;
            }
            AirlineAction::ApplyIropsRepair {
                repaired_roster,
                uncovered_pairings,
                ..
            } => {
                next_state.roster = repaired_roster;
                next_state.unassigned_pairings = uncovered_pairings;
            }
            AirlineAction::ApplyMonthEndRollover {
                crew_id,
                pairing,
                carry_in_lock: _,
            } => {
                let pid = pairing.id.clone();
                let mut rotations: Vec<Rotation> = next_state.roster.rotations().cloned().collect();
                if let Some(rot) = rotations.iter_mut().find(|r| r.crew_id == crew_id) {
                    let mut pairings = rot.pairings().to_vec();
                    pairings.push(pairing.clone());
                    pairings.sort_by_key(|p| p.start());
                    let new_rot = Rotation::new(rot.id.clone(), crew_id.clone(), pairings)
                        .map_err(|e| format!("Rotation error: {:?}", e))?;
                    *rot = new_rot;
                } else {
                    return Err(format!("Crew {} not found in roster", crew_id));
                }

                // Check monthly cap using prorated January hours (excluding Feb rollover legs)
                let rot = rotations.iter().find(|r| r.crew_id == crew_id).unwrap();
                let jan_prorated_minutes: i64 = rot
                    .pairings()
                    .iter()
                    .map(|p| p.prorated_block_time(&next_state.roster.period).num_minutes())
                    .sum();
                if jan_prorated_minutes > 85 * 60 {
                    return Err(format!(
                        "Crew {} exceeds 85.0h January statutory cap (prorated: {:.2}h)",
                        crew_id,
                        jan_prorated_minutes as f64 / 60.0
                    ));
                }

                next_state.unassigned_pairings.retain(|p| p.id != pid);
                next_state.roster = Roster::with_crew(
                    next_state.roster.id.clone(),
                    next_state.roster.period.clone(),
                    next_state.roster.legs().cloned().collect(),
                    rotations,
                    next_state.roster.crew_members().cloned().collect(),
                )
                .map_err(|e| format!("Roster error: {:?}", e))?;
            }
            AirlineAction::ApplyHubSplitFallback {
                hub: _,
                pilot_a_crew_id,
                pilot_a_pairing,
                february_reserve_crew_id,
                february_pairing,
            } => {
                let mut rotations: Vec<Rotation> = next_state.roster.rotations().cloned().collect();

                // 1. Assign Pilot A pairing
                if let Some(rot) = rotations.iter_mut().find(|r| r.crew_id == pilot_a_crew_id) {
                    let mut pairings = rot.pairings().to_vec();
                    pairings.push(pilot_a_pairing.clone());
                    pairings.sort_by_key(|p| p.start());
                    let new_rot = Rotation::new(rot.id.clone(), pilot_a_crew_id.clone(), pairings)
                        .map_err(|e| format!("Rotation error for Pilot A: {:?}", e))?;
                    *rot = new_rot;
                } else {
                    return Err(format!("Pilot A {} not found in roster", pilot_a_crew_id));
                }

                // 2. Assign February pairing to fresh Reserve Pilot on Feb 1 if designated
                if let Some(res_crew) = february_reserve_crew_id {
                    if let Some(rot) = rotations.iter_mut().find(|r| r.crew_id == res_crew) {
                        let mut pairings = rot.pairings().to_vec();
                        pairings.push(february_pairing.clone());
                        pairings.sort_by_key(|p| p.start());
                        let new_rot = Rotation::new(rot.id.clone(), res_crew.clone(), pairings)
                            .map_err(|e| format!("Rotation error for Reserve Pilot: {:?}", e))?;
                        *rot = new_rot;
                    }
                } else {
                    // Retain for February planning
                    next_state.unassigned_pairings.push(february_pairing);
                }

                next_state.roster = Roster::with_crew(
                    next_state.roster.id.clone(),
                    next_state.roster.period.clone(),
                    next_state.roster.legs().cloned().collect(),
                    rotations,
                    next_state.roster.crew_members().cloned().collect(),
                )
                .map_err(|e| format!("Roster error: {:?}", e))?;
            }
        }

        // Statutory Legality Gate: Verify no hard constraint errors introduced!
        let errors = self.checker.errors(&next_state.roster);
        if !errors.is_empty() {
            let msgs: Vec<String> = errors
                .iter()
                .map(|e| format!("{}: {}", e.rule_id, e.message))
                .collect();
            return Err(format!(
                "Decision proposal rejected by statutory legality gate: {:?}",
                msgs
            ));
        }

        // Update state reference metadata
        next_state.reference = StateReference {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            plugin: "airline".to_string(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("parent_id".to_string(), state.reference.id.to_string());
                m.insert("confidence".to_string(), format!("{:.4}", proposal.confidence));
                m
            },
        };

        Ok(next_state)
    }

    fn execute(&mut self, proposal: &DecisionProposal) {
        if let Ok(new_state) = self.simulate(&self.current_state(), proposal) {
            *self.current_state.lock().unwrap() = new_state;
        }
    }
}

/// The authoritative domain adapter orchestrating Coralys decisions, legality certification, and lineage tracking.
pub struct AirlineDecisionAdapter {
    pub plugin: AirlineDecisionPlugin,
    pub lineage: DecisionLineage,
}

impl AirlineDecisionAdapter {
    /// Create a new adapter wrapping an initial state and legality checker.
    pub fn new(initial_state: AirlineState, checker: LegalityChecker) -> Self {
        let initial_eval = {
            let plugin = AirlineDecisionPlugin::new(initial_state.clone(), LegalityChecker::new());
            plugin.evaluate(&initial_state)
        };

        let root_node = LineageNode {
            id: initial_state.reference.id,
            parent_id: None,
            proposal: None,
            evaluation: initial_eval,
        };

        let mut nodes = HashMap::new();
        nodes.insert(root_node.id, root_node);

        let lineage = DecisionLineage {
            nodes,
            root_id: initial_state.reference.id,
            current_id: initial_state.reference.id,
        };

        let plugin = AirlineDecisionPlugin::new(initial_state, checker);

        Self { plugin, lineage }
    }

    /// Retrieve the current evaluation of the operational state.
    pub fn current_evaluation(&self) -> EvaluationResult {
        let state = self.plugin.current_state();
        self.plugin.evaluate(&state)
    }

    /// Commit a decision proposal generated by Coralys.
    ///
    /// The proposal passes strictly through the adapter's simulation and statutory legality gate.
    /// If valid, the decision is executed, the operational state transitions, and a new lineage node is sealed.
    pub fn commit_decision(
        &mut self,
        proposal: DecisionProposal,
    ) -> Result<EvaluationResult, String> {
        let current_state = self.plugin.current_state();
        let new_state = self.plugin.simulate(&current_state, &proposal)?;
        let evaluation = self.plugin.evaluate(&new_state);

        let node_id = new_state.reference.id;
        let parent_id = self.lineage.current_id;

        // Commit execution
        self.plugin.execute(&proposal);

        // Record immutable lineage node
        let node = LineageNode {
            id: node_id,
            parent_id: Some(parent_id),
            proposal: Some(proposal),
            evaluation: evaluation.clone(),
        };

        self.lineage.nodes.insert(node_id, node);
        self.lineage.current_id = node_id;

        Ok(evaluation)
    }

    /// Generate decision proposals using the Coralys reasoning engine.
    pub fn generate_proposals(
        &mut self,
        engine: &CoralysAirlineEngine,
        config: &EvolutionaryConfig,
    ) -> Result<Vec<DecisionProposal>, String> {
        engine.solve(&mut self.plugin, config)
    }

    /// Reference to the complete decision lineage tree.
    pub fn lineage(&self) -> &DecisionLineage {
        &self.lineage
    }

    /// Day-of-Operations IROPS Repair API:
    /// Locks historical assignments prior to `request.frozen_until`, identifies disrupted pairings,
    /// re-assigns them using available reserve crew or active crew with legal slack,
    /// and produces a certified DecisionProposal with minimal churn.
    pub fn repair_irops(
        &mut self,
        request: &IropsRepairRequest,
        reserve_pool: Option<&crate::resilience::reserve::ReservePool>,
    ) -> Result<DecisionProposal, String> {
        let current_state = self.plugin.current_state();
        let mut rotations: Vec<Rotation> = current_state.roster.rotations().cloned().collect();
        let mut disrupted_pairings = Vec::new();
        let mut disrupted_count = 0;

        // 1. Process sick crew: preserve pairings < frozen_until, orphan pairings >= frozen_until
        let sick_set: std::collections::HashSet<&CrewId> = request.sick_crew.iter().collect();
        for rot in rotations.iter_mut() {
            if sick_set.contains(&rot.crew_id) {
                let (frozen, active): (Vec<Pairing>, Vec<Pairing>) = rot
                    .pairings()
                    .iter()
                    .cloned()
                    .partition(|p| p.start() < request.frozen_until);

                disrupted_count += active.len();
                disrupted_pairings.extend(active);

                if !frozen.is_empty() {
                    let new_rot = Rotation::new(rot.id.clone(), rot.crew_id.clone(), frozen)
                        .map_err(|e| format!("Rotation error: {:?}", e))?;
                    *rot = new_rot;
                }
            }
        }

        // 2. Filter cancelled legs out of disrupted pairings
        let cancelled_set: std::collections::HashSet<&str> =
            request.cancelled_legs.iter().map(|s| s.as_str()).collect();
        let mut valid_to_reassign = Vec::new();
        for p in disrupted_pairings {
            let has_cancelled = p
                .duties()
                .iter()
                .flat_map(|d| d.legs())
                .any(|l| cancelled_set.contains(l.id.as_str()));
            if !has_cancelled {
                valid_to_reassign.push(p);
            }
        }

        // 3. Attempt to re-assign via Reserve Pool first
        let mut repaired_count = 0;
        let open_pairings = valid_to_reassign;

        let working_roster = Roster::with_crew(
            current_state.roster.id.clone(),
            current_state.roster.period.clone(),
            current_state.roster.legs().cloned().collect(),
            rotations.clone(),
            current_state.roster.crew_members().cloned().collect(),
        )
        .map_err(|e| format!("Roster error: {:?}", e))?;

        let (after_reserve_roster, _reserve_assignments, still_open) =
            if let Some(pool) = reserve_pool {
                let allocator =
                    crate::resilience::reserve::ReserveAllocator::new(&self.plugin.checker);
                let (r, a, o) =
                    allocator.allocate_open_pairings(&working_roster, &open_pairings, pool);
                repaired_count += a.len();
                (r, a, o)
            } else {
                (working_roster, vec![], open_pairings)
            };

        // 4. For remaining open pairings, attempt greedy assignment into healthy active lineholders
        let mut final_rotations: Vec<Rotation> =
            after_reserve_roster.rotations().cloned().collect();
        let mut unrecovered = Vec::new();

        for pairing in still_open {
            let mut covered = false;
            for rot in final_rotations.iter_mut() {
                if sick_set.contains(&rot.crew_id) {
                    continue;
                }
                // Check temporal conflict
                let p_start = pairing.start();
                let p_end = pairing.end();
                let overlaps = rot.pairings().iter().any(|p| {
                    !(p_end <= p.start() - chrono::Duration::minutes(720)
                        || p_start >= p.end() + chrono::Duration::minutes(720))
                });

                if !overlaps {
                    let mut new_pairings = rot.pairings().to_vec();
                    new_pairings.push(pairing.clone());
                    new_pairings.sort_by_key(|p| p.start());

                    let total_minutes: i64 = new_pairings
                        .iter()
                        .map(|p| p.total_block_time().num_minutes())
                        .sum();
                    if total_minutes <= 85 * 60 {
                        if let Ok(new_rot) =
                            Rotation::new(rot.id.clone(), rot.crew_id.clone(), new_pairings)
                        {
                            *rot = new_rot;
                            repaired_count += 1;
                            covered = true;
                            break;
                        }
                    }
                }
            }
            if !covered {
                unrecovered.push(pairing);
            }
        }

        let repaired_roster = Roster::with_crew(
            current_state.roster.id.clone(),
            current_state.roster.period.clone(),
            current_state.roster.legs().cloned().collect(),
            final_rotations,
            current_state.roster.crew_members().cloned().collect(),
        )
        .map_err(|e| format!("Roster error: {:?}", e))?;

        // 5. Build DecisionProposal
        let action = AirlineAction::ApplyIropsRepair {
            repaired_roster,
            uncovered_pairings: unrecovered,
            disrupted_count,
            repaired_count,
        };

        let payload = serde_json::to_value(action)
            .map_err(|e| format!("Failed to serialize IROPS proposal: {}", e))?;

        Ok(DecisionProposal {
            priority: 1.0,
            estimated_gain: repaired_count as f64,
            affected_resources: request.sick_crew.iter().map(|c| c.to_string()).collect(),
            violations_resolved: vec!["irops_disruption_recovered".to_string()],
            confidence: 0.99,
            payload,
        })
    }

    /// Export all February Carry-In Locks across the current roster for the next month's planning cycle.
    ///
    /// Implements standard airline rollover practice: flight legs that extend past midnight on Jan 31
    /// (e.g. LEG_31_72, LEG_31_65, LEG_31_202, LEG_31_145) are exported as pre-assigned Carry-In Locks
    /// for February bidding, keeping the crew intact without artificial schedule splitting.
    pub fn export_february_carry_in_locks(
        &self,
        period: &crate::domain::roster::PlanningPeriod,
    ) -> Vec<crate::domain::pairing::CarryInLock> {
        let state = self.plugin.current_state.lock().unwrap();
        let mut locks = Vec::new();

        for rotation in state.roster.rotations() {
            for pairing in rotation.pairings() {
                let carry_legs = pairing.carry_over_legs(period);
                if !carry_legs.is_empty() {
                    let carry_block: chrono::Duration = carry_legs.iter().map(|l| l.block_time()).sum();
                    locks.push(crate::domain::pairing::CarryInLock {
                        crew_id: rotation.crew_id.clone(),
                        parent_pairing_id: pairing.id.clone(),
                        start_time: carry_legs.first().unwrap().scheduled_departure,
                        end_time: carry_legs.last().unwrap().scheduled_arrival,
                        carry_in_block_time: carry_block,
                        carry_in_legs: carry_legs,
                    });
                }
            }
        }
        locks
    }

    /// Evaluates all unassigned month-end rollover pairings under the airline's hierarchy of rules:
    /// 1. Default: Month-End Rollover keeping crew intact and locking February carry-in.
    /// 2. Fallback: Hub split only when candidate pilots have zero January headroom.
    pub fn evaluate_month_end_pairings(
        &self,
        period: &crate::domain::roster::PlanningPeriod,
        hub_candidates: &[crate::domain::flight::AirportCode],
    ) -> Vec<crate::domain::pairing::RolloverDecision> {
        let state = self.plugin.current_state.lock().unwrap();
        let mut decisions = Vec::new();

        // Calculate current January block hours per pilot
        let mut crew_hours: HashMap<CrewId, chrono::Duration> = HashMap::new();
        for rotation in state.roster.rotations() {
            let total_jan_block: chrono::Duration = rotation
                .pairings()
                .iter()
                .map(|p| p.prorated_block_time(period))
                .sum();
            crew_hours.insert(rotation.crew_id.clone(), total_jan_block);
        }

        for pairing in &state.unassigned_pairings {
            let carry_legs = pairing.carry_over_legs(period);
            if carry_legs.is_empty() {
                continue;
            }

            // Find best candidate pilot at the same home base
            let cand_crew = state
                .roster
                .rotations()
                .find(|r| {
                    if let Some(c) = state.roster.crew_members().find(|cm| cm.id == r.crew_id) {
                        c.base == pairing.base
                    } else {
                        false
                    }
                })
                .map(|r| r.crew_id.clone());

            if let Some(cid) = cand_crew {
                let cur_hours = crew_hours.get(&cid).cloned().unwrap_or(chrono::Duration::zero());
                if let Some(decision) = pairing.evaluate_month_end_transition(&cid, cur_hours, period, hub_candidates) {
                    decisions.push(decision);
                }
            }
        }

        decisions
    }
}

/// The Coralys Decision Engine generating operational scheduling proposals.
pub struct CoralysAirlineEngine {
    pub evaluator: CostEvaluator,
    pub weights: Vec<f64>,
}

impl CoralysAirlineEngine {
    pub fn new(evaluator: CostEvaluator, weights: Vec<f64>) -> Self {
        Self { evaluator, weights }
    }
}

impl ReasoningEngine for CoralysAirlineEngine {
    type Plugin = AirlineDecisionPlugin;
    type Config = EvolutionaryConfig;

    fn solve(
        &self,
        plugin: &mut Self::Plugin,
        config: &Self::Config,
    ) -> Result<Vec<DecisionProposal>, String> {
        let state = plugin.current_state();
        let mut proposals = Vec::new();

        let mut pairings = state.unassigned_pairings.clone();
        for rot in state.roster.rotations() {
            pairings.extend(rot.pairings().iter().cloned());
        }
        pairings.sort_by_key(|p| p.start());
        pairings.dedup_by_key(|p| p.id.clone());

        if pairings.is_empty() {
            return Ok(proposals);
        }

        // 1. Primary Roster Generation via Evolutionary Search with Duty Rescue
        let scheduler = EvolutionaryScheduler::with_config(
            &self.evaluator,
            self.weights.clone(),
            config.clone(),
        );
        let mut metrics = OptimizationMetrics::new();
        let (rescued_roster, _still_unassigned) =
            scheduler.assign_with_duty_rescue(&state.roster, pairings.clone(), &mut metrics);

        let assigned_ids: std::collections::HashSet<PairingId> = rescued_roster
            .rotations()
            .flat_map(|r| r.pairings())
            .map(|p| p.id.clone())
            .collect();

        let unassigned_pairings: Vec<Pairing> = pairings
            .into_iter()
            .filter(|p| !assigned_ids.contains(&p.id))
            .collect();

        let assigned_leg_ids: std::collections::HashSet<crate::domain::flight::FlightLegId> =
            rescued_roster
                .rotations()
                .flat_map(|r| r.pairings())
                .flat_map(|p| p.duties())
                .flat_map(|d| d.legs())
                .map(|l| l.id.clone())
                .collect();

        let unassigned_legs: Vec<FlightLeg> = state
            .roster
            .legs()
            .filter(|l| !assigned_leg_ids.contains(&l.id))
            .cloned()
            .collect();

        let action = AirlineAction::ApplyRoster {
            roster: rescued_roster.clone(),
            unassigned_pairings: unassigned_pairings.clone(),
            unassigned_legs: unassigned_legs.clone(),
        };

        let payload = serde_json::to_value(action)
            .map_err(|e| format!("Failed to serialize proposal action: {}", e))?;

        let total_legs = state.roster.legs().count();
        let covered_legs = total_legs.saturating_sub(unassigned_legs.len());
        let leg_cov = if total_legs > 0 {
            covered_legs as f64 / total_legs as f64 * 100.0
        } else {
            100.0
        };

        proposals.push(DecisionProposal {
            priority: 1.0,
            estimated_gain: leg_cov,
            affected_resources: rescued_roster
                .rotations()
                .map(|r| r.crew_id.to_string())
                .collect(),
            violations_resolved: vec!["zero_uncrewed_flights".to_string()],
            confidence: 0.98,
            payload,
        });

        // 2. Reserve Capacity Sizing Proposal (if any pairings remain uncovered)
        if !unassigned_pairings.is_empty() {
            let mut station_deficits: HashMap<String, usize> = HashMap::new();
            for p in &unassigned_pairings {
                *station_deficits.entry(p.base.as_str().to_string()).or_insert(0) += 1;
            }

            let mut pilot_deficits: HashMap<String, usize> = HashMap::new();
            let mut total_crew_needed = 0;
            for (base, open_pairings) in &station_deficits {
                let needed = (open_pairings + 4) / 5;
                pilot_deficits.insert(base.clone(), needed);
                total_crew_needed += needed;
            }

            let reserve_action = AirlineAction::RecommendReserveCapacity {
                station_deficits: pilot_deficits,
                total_crew_needed,
            };
            let reserve_payload = serde_json::to_value(reserve_action)
                .map_err(|e| format!("Failed to serialize reserve action: {}", e))?;

            proposals.push(DecisionProposal {
                priority: 2.0,
                estimated_gain: total_crew_needed as f64,
                affected_resources: station_deficits.keys().cloned().collect(),
                violations_resolved: vec!["capacity_shortfall".to_string()],
                confidence: 1.0,
                payload: reserve_payload,
            });
        }

        Ok(proposals)
    }
}
