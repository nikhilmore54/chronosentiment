//! Disruption recovery for crew scheduling.
//!
//! A [`Disruption`] represents an operational event that invalidates part of a
//! roster — a delayed flight, a sick crew member, or a cancelled pairing.
//! [`DisruptionRecovery`] produces a repaired roster by re-assigning affected
//! pairings using a greedy strategy with the Layer 2 legality checker as the
//! feasibility oracle.

use crate::domain::pairing::Pairing;
use crate::domain::roster::Roster;
use crate::domain::rotation::Rotation;
use crate::legality::LegalityChecker;
use chrono::Duration;

// ── Disruption types ──────────────────────────────────────────────────────────

/// The kind of operational disruption.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisruptionKind {
    /// A pairing has been cancelled and must be removed from its rotation.
    PairingCancelled {
        rotation_index: usize,
        pairing_index: usize,
    },
    /// A crew member is unavailable; all pairings in their rotation are orphaned.
    CrewUnavailable { rotation_index: usize },
    /// A flight is delayed, pushing back its scheduled times.
    FlightDelayed { leg_id: String, delay_mins: i64 },
    /// A flight is cancelled and must be removed from its duty.
    FlightCancelled { leg_id: String },
}

/// A single disruption event.
#[derive(Debug, Clone)]
pub struct Disruption {
    pub kind: DisruptionKind,
    pub description: String,
}

impl Disruption {
    pub fn new(kind: DisruptionKind, description: impl Into<String>) -> Self {
        Self {
            kind,
            description: description.into(),
        }
    }
}

// ── Recovery result ───────────────────────────────────────────────────────────

/// The outcome of a recovery attempt.
#[derive(Debug)]
pub struct RecoveryResult {
    /// The repaired roster.
    pub roster: Roster,
    /// Pairings that could not be re-assigned.
    pub unrecovered: Vec<Pairing>,
    /// Number of pairings successfully re-assigned.
    pub recovered_count: usize,
}

// ── Recovery engine ───────────────────────────────────────────────────────────

/// Greedy disruption recovery engine.
pub struct DisruptionRecovery<'a> {
    checker: &'a LegalityChecker,
}

impl<'a> DisruptionRecovery<'a> {
    pub fn new(checker: &'a LegalityChecker) -> Self {
        Self { checker }
    }

    /// Apply `disruptions` to `roster` and attempt to recover.
    pub fn recover(&self, roster: &Roster, disruptions: &[Disruption]) -> RecoveryResult {
        let mut rotations: Vec<Rotation> = roster.rotations().cloned().collect();
        let mut orphaned: Vec<Pairing> = Vec::new();
        let mut unrecovered: Vec<Pairing> = Vec::new();

        let mut crew_unavailable_indices: Vec<usize> = Vec::new();

        for disruption in disruptions {
            match &disruption.kind {
                DisruptionKind::PairingCancelled {
                    rotation_index,
                    pairing_index,
                } => {
                    if let Some(rot) = rotations.get(*rotation_index) {
                        if *pairing_index < rot.pairings().len() {
                            let pairing = rot.pairings()[*pairing_index].clone();
                            orphaned.push(pairing);
                            let new_pairings: Vec<_> = rot
                                .pairings()
                                .iter()
                                .enumerate()
                                .filter(|(i, _)| *i != *pairing_index)
                                .map(|(_, p)| p.clone())
                                .collect();
                            if let Ok(new_rot) =
                                Rotation::new(rot.id.clone(), rot.crew_id.clone(), new_pairings)
                            {
                                rotations[*rotation_index] = new_rot;
                            }
                        }
                    }
                }
                DisruptionKind::CrewUnavailable { rotation_index } => {
                    if let Some(rot) = rotations.get(*rotation_index) {
                        orphaned.extend(rot.pairings().iter().cloned());
                    }
                    crew_unavailable_indices.push(*rotation_index);
                }
                DisruptionKind::FlightDelayed { leg_id, delay_mins } => {
                    self.apply_flight_mod(
                        roster.period.clone(),
                        leg_id,
                        &mut rotations,
                        &mut orphaned,
                        &mut unrecovered,
                        |leg| {
                            let mut new_leg = leg.clone();
                            new_leg.scheduled_departure += Duration::minutes(*delay_mins);
                            new_leg.scheduled_arrival += Duration::minutes(*delay_mins);
                            Some(new_leg)
                        },
                    );
                }
                DisruptionKind::FlightCancelled { leg_id } => {
                    self.apply_flight_mod(
                        roster.period.clone(),
                        leg_id,
                        &mut rotations,
                        &mut orphaned,
                        &mut unrecovered,
                        |_| None,
                    );
                }
            }
        }

        crew_unavailable_indices.sort_unstable();
        crew_unavailable_indices.dedup();
        for idx in crew_unavailable_indices.iter().rev() {
            if *idx < rotations.len() {
                rotations.remove(*idx);
            }
        }

        let mut recovered_count = 0;

        'outer: for pairing in orphaned {
            for rot_idx in 0..rotations.len() {
                let rot = &rotations[rot_idx];
                let mut new_pairings = rot.pairings().to_vec();
                new_pairings.push(pairing.clone());
                new_pairings.sort_by_key(|p| p.duties()[0].start());

                if let Ok(new_rot) =
                    Rotation::new(rot.id.clone(), rot.crew_id.clone(), new_pairings)
                {
                    let temp_roster = Roster::new(
                        roster.id.clone(),
                        roster.period.clone(),
                        vec![],
                        vec![new_rot.clone()],
                    );
                    let is_legal = match temp_roster {
                        Ok(ref r) => !self.checker.check(r).iter().any(|v| v.is_error()),
                        Err(_) => false,
                    };
                    if is_legal {
                        rotations[rot_idx] = new_rot;
                        recovered_count += 1;
                        continue 'outer;
                    }
                }
            }
            unrecovered.push(pairing);
        }

        let new_roster = Roster::new(
            roster.id.clone(),
            roster.period.clone(),
            roster.legs().cloned().collect(),
            rotations,
        )
        .unwrap_or_else(|_| roster.clone());

        RecoveryResult {
            roster: new_roster,
            unrecovered,
            recovered_count,
        }
    }

    /// Helper to find a flight leg and modify or remove it.
    fn apply_flight_mod<F>(
        &self,
        period: crate::domain::roster::PlanningPeriod,
        leg_id: &str,
        rotations: &mut Vec<Rotation>,
        orphaned: &mut Vec<Pairing>,
        unrecovered: &mut Vec<Pairing>,
        modifier: F,
    ) where
        F: Fn(&crate::domain::flight::FlightLeg) -> Option<crate::domain::flight::FlightLeg>,
    {
        for (rot_idx, rot) in rotations.clone().iter().enumerate() {
            for (p_idx, pairing) in rot.pairings().iter().enumerate() {
                if pairing
                    .duties()
                    .iter()
                    .any(|d| d.legs().iter().any(|l| l.id.as_str() == leg_id))
                {
                    // We found the pairing containing the leg. Rebuild it.
                    let mut rebuild_failed = false;
                    let mut new_duties = Vec::new();

                    for duty in pairing.duties() {
                        let mut new_legs = Vec::new();
                        for leg in duty.legs() {
                            if leg.id.as_str() == leg_id {
                                if let Some(modified) = modifier(leg) {
                                    new_legs.push(modified);
                                }
                            } else {
                                new_legs.push(leg.clone());
                            }
                        }

                        if new_legs.is_empty() {
                            continue; // Duty is empty now, skip it.
                        }

                        if let Ok(new_duty) =
                            crate::domain::duty::Duty::new(duty.id.clone(), new_legs)
                        {
                            new_duties.push(new_duty);
                        } else {
                            rebuild_failed = true;
                            break;
                        }
                    }

                    if new_duties.is_empty() {
                        rebuild_failed = true;
                    }

                    let mut new_pairing_opt = None;
                    if !rebuild_failed {
                        if let Ok(new_pairing) =
                            Pairing::new(pairing.id.clone(), pairing.base.clone(), new_duties)
                        {
                            new_pairing_opt = Some(new_pairing);
                        } else {
                            rebuild_failed = true;
                        }
                    }

                    // Remove the old pairing from the rotation
                    let mut new_pairings = rot.pairings().to_vec();
                    new_pairings.remove(p_idx);

                    if !rebuild_failed {
                        if let Some(new_pairing) = new_pairing_opt {
                            let mut temp_pairings = new_pairings.clone();
                            temp_pairings.push(new_pairing.clone());
                            temp_pairings.sort_by_key(|p| p.duties()[0].start());

                            let mut is_legal_for_crew = false;
                            if let Ok(test_rot) =
                                Rotation::new(rot.id.clone(), rot.crew_id.clone(), temp_pairings)
                            {
                                if let Ok(test_roster) = Roster::new(
                                    crate::domain::roster::RosterId::new("temp"),
                                    period.clone(),
                                    vec![],
                                    vec![test_rot.clone()],
                                ) {
                                    if !self
                                        .checker
                                        .check(&test_roster)
                                        .iter()
                                        .any(|v| v.is_error())
                                    {
                                        is_legal_for_crew = true;
                                        rotations[rot_idx] = test_rot;
                                    }
                                }
                            }

                            if !is_legal_for_crew {
                                orphaned.push(new_pairing);
                                if new_pairings.is_empty() {
                                    rotations.remove(rot_idx);
                                } else if let Ok(new_rot) =
                                    Rotation::new(rot.id.clone(), rot.crew_id.clone(), new_pairings)
                                {
                                    rotations[rot_idx] = new_rot;
                                }
                            }
                        }
                    } else {
                        unrecovered.push(pairing.clone());
                        if new_pairings.is_empty() {
                            rotations.remove(rot_idx);
                        } else if let Ok(new_rot) =
                            Rotation::new(rot.id.clone(), rot.crew_id.clone(), new_pairings)
                        {
                            rotations[rot_idx] = new_rot;
                        }
                    }

                    return; // Done
                }
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legality::{LegalityChecker, test_helpers::*};

    fn make_checker() -> LegalityChecker {
        LegalityChecker::new()
    }

    #[test]
    fn no_disruptions_returns_unchanged_roster() {
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 8, 10)]);
        let d2 = make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 22, 24)]);
        let p1 = make_pairing("P1", "LHR", vec![d1, d2]);
        let r1 = make_rotation("R1", "C1", vec![p1]);
        let roster = make_roster(vec![], vec![r1]);

        let checker = make_checker();
        let recovery = DisruptionRecovery::new(&checker);
        let result = recovery.recover(&roster, &[]);

        let total: usize = result.roster.rotations().map(|r| r.pairings().len()).sum();
        assert_eq!(total, 1);
        assert_eq!(result.unrecovered.len(), 0);
        assert_eq!(result.recovered_count, 0);
    }

    #[test]
    fn flight_delayed_legal_stays_assigned() {
        let d1 = make_duty(
            "D1",
            vec![
                make_leg("L1", "LHR", "CDG", 8, 10),
                make_leg("L2", "CDG", "LHR", 12, 14),
            ],
        );
        let p1 = make_pairing("P1", "LHR", vec![d1]);
        let r1 = make_rotation("R1", "C1", vec![p1]);
        let roster = make_roster(vec![], vec![r1]);

        let checker = make_checker();
        let recovery = DisruptionRecovery::new(&checker);
        let disruption = Disruption::new(
            DisruptionKind::FlightDelayed {
                leg_id: "L1".to_string(),
                delay_mins: 60,
            },
            "L1 delayed 1h",
        );
        let result = recovery.recover(&roster, &[disruption]);

        // Still legal (connection is 1h instead of 2h).
        let total: usize = result.roster.rotations().map(|r| r.pairings().len()).sum();
        assert_eq!(total, 1);
        assert_eq!(result.unrecovered.len(), 0);
        assert_eq!(result.recovered_count, 0);
    }

    #[test]
    fn flight_cancelled_breaks_pairing() {
        let d1 = make_duty(
            "D1",
            vec![
                make_leg("L1", "LHR", "CDG", 8, 10),
                make_leg("L2", "CDG", "FRA", 12, 14),
                make_leg("L3", "FRA", "LHR", 16, 18),
            ],
        );
        let p1 = make_pairing("P1", "LHR", vec![d1]);
        let r1 = make_rotation("R1", "C1", vec![p1]);
        let roster = make_roster(vec![], vec![r1]);

        let checker = make_checker();
        let recovery = DisruptionRecovery::new(&checker);
        let disruption = Disruption::new(
            DisruptionKind::FlightCancelled {
                leg_id: "L2".to_string(),
            },
            "L2 cancelled",
        );
        let result = recovery.recover(&roster, &[disruption]);

        // Cancelling L2 means L1 (CDG) doesn't connect to L3 (FRA).
        // Duty rebuild fails. Pairing dropped to unrecovered.
        let total: usize = result.roster.rotations().map(|r| r.pairings().len()).sum();
        assert_eq!(total, 0);
        assert_eq!(result.unrecovered.len(), 1);
        assert_eq!(result.recovered_count, 0);
    }
}
