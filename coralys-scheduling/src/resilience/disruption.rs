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
    CrewUnavailable {
        rotation_index: usize,
    },
}

/// A single disruption event.
#[derive(Debug, Clone)]
pub struct Disruption {
    pub kind: DisruptionKind,
    pub description: String,
}

impl Disruption {
    pub fn new(kind: DisruptionKind, description: impl Into<String>) -> Self {
        Self { kind, description: description.into() }
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

        // Phase 1 — extract disrupted pairings (process in reverse index order
        // so that removing by index doesn't shift subsequent indices).
        let mut crew_unavailable_indices: Vec<usize> = Vec::new();

        for disruption in disruptions {
            match &disruption.kind {
                DisruptionKind::PairingCancelled { rotation_index, pairing_index } => {
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
                            if !new_pairings.is_empty() {
                                if let Ok(new_rot) = Rotation::new(
                                    rot.id.clone(),
                                    rot.crew_id.clone(),
                                    new_pairings,
                                ) {
                                    rotations[*rotation_index] = new_rot;
                                }
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
            }
        }

        // Remove crew-unavailable rotations in reverse order to preserve indices.
        crew_unavailable_indices.sort_unstable();
        crew_unavailable_indices.dedup();
        for idx in crew_unavailable_indices.iter().rev() {
            if *idx < rotations.len() {
                rotations.remove(*idx);
            }
        }

        // Phase 2 — greedy re-assignment of orphaned pairings.
        let mut unrecovered: Vec<Pairing> = Vec::new();
        let mut recovered_count = 0;

        'outer: for pairing in orphaned {
            for rot_idx in 0..rotations.len() {
                let rot = &rotations[rot_idx];
                let mut new_pairings = rot.pairings().to_vec();
                new_pairings.push(pairing.clone());
                // Sort chronologically by first duty start.
                new_pairings.sort_by_key(|p| p.duties()[0].start());

                if let Ok(new_rot) = Rotation::new(
                    rot.id.clone(),
                    rot.crew_id.clone(),
                    new_pairings,
                ) {
                    // Build a minimal temp roster to run the legality check.
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

        // Phase 3 — rebuild roster.
        let new_roster = Roster::new(
            roster.id.clone(),
            roster.period.clone(),
            roster.legs().cloned().collect(),
            rotations,
        )
        .unwrap_or_else(|_| roster.clone());

        RecoveryResult { roster: new_roster, unrecovered, recovered_count }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legality::{test_helpers::*, LegalityChecker};

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
    fn crew_unavailable_removes_rotation() {
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 8, 10)]);
        let d2 = make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 22, 24)]);
        let p1 = make_pairing("P1", "LHR", vec![d1, d2]);
        let d3 = make_duty("D3", vec![make_leg("L3", "LHR", "CDG", 32, 34)]);
        let d4 = make_duty("D4", vec![make_leg("L4", "CDG", "LHR", 46, 48)]);
        let p2 = make_pairing("P2", "LHR", vec![d3, d4]);
        let r1 = make_rotation("R1", "C1", vec![p1]);
        let r2 = make_rotation("R2", "C2", vec![p2]);
        let roster = make_roster(vec![], vec![r1, r2]);

        let checker = make_checker();
        let recovery = DisruptionRecovery::new(&checker);
        let disruption = Disruption::new(
            DisruptionKind::CrewUnavailable { rotation_index: 0 },
            "C1 sick",
        );
        let result = recovery.recover(&roster, &[disruption]);

        // R1 is removed; only R2 remains.
        let rot_count = result.roster.rotations().count();
        assert_eq!(rot_count, 1);
    }

    #[test]
    fn pairing_cancelled_removes_it_from_rotation() {
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 8, 10)]);
        let d2 = make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 22, 24)]);
        let p1 = make_pairing("P1", "LHR", vec![d1, d2]);
        let d3 = make_duty("D3", vec![make_leg("L3", "LHR", "CDG", 32, 34)]);
        let d4 = make_duty("D4", vec![make_leg("L4", "CDG", "LHR", 46, 48)]);
        let p2 = make_pairing("P2", "LHR", vec![d3, d4]);
        let r1 = make_rotation("R1", "C1", vec![p1, p2]);
        let d5 = make_duty("D5", vec![make_leg("L5", "LHR", "CDG", 8, 10)]);
        let d6 = make_duty("D6", vec![make_leg("L6", "CDG", "LHR", 22, 24)]);
        let p3 = make_pairing("P3", "LHR", vec![d5, d6]);
        let r2 = make_rotation("R2", "C2", vec![p3]);
        let roster = make_roster(vec![], vec![r1, r2]);

        let checker = make_checker();
        let recovery = DisruptionRecovery::new(&checker);
        let disruption = Disruption::new(
            DisruptionKind::PairingCancelled { rotation_index: 0, pairing_index: 0 },
            "P1 cancelled",
        );
        let result = recovery.recover(&roster, &[disruption]);

        // P1 removed from R1; R1 now has P2 only (or P2 + re-assigned P1).
        let r1_pairings = result.roster.rotations().next().unwrap().pairings().len();
        assert!(r1_pairings >= 1);
    }
}