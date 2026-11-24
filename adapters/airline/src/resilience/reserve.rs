//! Reserve crew allocation.
//!
//! A [`ReservePool`] holds crew members on standby.  [`ReserveAllocator`]
//! matches reserve crew to disrupted rotations using a greedy first-fit
//! strategy, delegating feasibility checks to the Layer 2 legality engine.

use crate::domain::crew::{CrewId, CrewMember};
use crate::domain::pairing::Pairing;
use crate::domain::roster::Roster;
use crate::domain::rotation::Rotation;
use crate::legality::LegalityChecker;

// ── Reserve pool ──────────────────────────────────────────────────────────────

/// A pool of crew members available for reserve duty.
#[derive(Debug, Clone, Default)]
pub struct ReservePool {
    members: Vec<CrewMember>,
}

impl ReservePool {
    pub fn new() -> Self {
        Self {
            members: Vec::new(),
        }
    }

    pub fn from_members(members: Vec<CrewMember>) -> Self {
        Self { members }
    }

    pub fn add(&mut self, member: CrewMember) {
        self.members.push(member);
    }

    pub fn members(&self) -> &[CrewMember] {
        &self.members
    }

    pub fn get_by_base(&self, base: &crate::domain::flight::AirportCode) -> Vec<&CrewMember> {
        self.members.iter().filter(|m| &m.base == base).collect()
    }

    pub fn len(&self) -> usize {
        self.members.len()
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}

// ── Allocation result ─────────────────────────────────────────────────────────

/// A single reserve assignment.
#[derive(Debug, Clone)]
pub struct ReserveAssignment {
    pub crew_id: CrewId,
    pub rotation_index: usize,
}

/// The outcome of a reserve allocation attempt.
#[derive(Debug)]
pub struct AllocationResult {
    pub roster: Roster,
    pub assignments: Vec<ReserveAssignment>,
    pub uncovered_rotations: Vec<usize>,
}

// ── Allocator ─────────────────────────────────────────────────────────────────

/// Greedy reserve crew allocator.
pub struct ReserveAllocator<'a> {
    checker: &'a LegalityChecker,
}

impl<'a> ReserveAllocator<'a> {
    pub fn new(checker: &'a LegalityChecker) -> Self {
        Self { checker }
    }

    /// Attempt to cover rotations whose crew IDs are in `absent_crew` using
    /// members from `pool`.
    pub fn allocate(
        &self,
        roster: &Roster,
        absent_crew: &[CrewId],
        pool: &ReservePool,
    ) -> AllocationResult {
        let mut rotations: Vec<Rotation> = roster.rotations().cloned().collect();
        let mut assignments: Vec<ReserveAssignment> = Vec::new();
        let mut uncovered: Vec<usize> = Vec::new();
        let mut used_reserve: Vec<CrewId> = Vec::new();

        for (rot_idx, rot) in rotations.iter_mut().enumerate() {
            if !absent_crew.contains(&rot.crew_id) {
                continue;
            }

            let mut assigned = false;
            for reserve in pool.members() {
                if used_reserve.contains(&reserve.id) {
                    continue;
                }

                if let Ok(new_rot) =
                    Rotation::new(rot.id.clone(), reserve.id.clone(), rot.pairings().to_vec())
                {
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
                        *rot = new_rot;
                        assignments.push(ReserveAssignment {
                            crew_id: reserve.id.clone(),
                            rotation_index: rot_idx,
                        });
                        used_reserve.push(reserve.id.clone());
                        assigned = true;
                        break;
                    }
                }
            }

            if !assigned {
                uncovered.push(rot_idx);
            }
        }

        let new_roster = Roster::new(
            roster.id.clone(),
            roster.period.clone(),
            roster.legs().cloned().collect(),
            rotations,
        )
        .unwrap_or_else(|_| roster.clone());

        AllocationResult {
            roster: new_roster,
            assignments,
            uncovered_rotations: uncovered,
        }
    }

    /// Match standby reserve crew to open/uncovered pairings (e.g. during the end-of-month push).
    pub fn allocate_open_pairings(
        &self,
        roster: &Roster,
        open_pairings: &[Pairing],
        pool: &ReservePool,
    ) -> (Roster, Vec<ReserveAssignment>, Vec<Pairing>) {
        let mut rotations: Vec<Rotation> = roster.rotations().cloned().collect();
        let mut assignments: Vec<ReserveAssignment> = Vec::new();
        let mut still_uncovered: Vec<Pairing> = Vec::new();

        for pairing in open_pairings {
            let mut covered = false;

            // Find matching reserve crew members at the pairing's base
            for reserve in pool.members() {
                if reserve.base != pairing.base {
                    continue;
                }

                // Look for an existing rotation for this reserve crew or create one
                if let Some((rot_idx, rot)) = rotations
                    .iter_mut()
                    .enumerate()
                    .find(|(_, r)| r.crew_id == reserve.id)
                {
                    // Check chronological overlap and rest with existing pairings
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

                        // Verify monthly 85h flying cap
                        let total_minutes: i64 = new_pairings
                            .iter()
                            .map(|p| p.total_block_time().num_minutes())
                            .sum();

                        if total_minutes <= 85 * 60 {
                            if let Ok(new_rot) =
                                Rotation::new(rot.id.clone(), reserve.id.clone(), new_pairings)
                            {
                                // Legality check
                                let temp_roster = Roster::with_crew(
                                    roster.id.clone(),
                                    roster.period.clone(),
                                    roster.legs().cloned().collect(),
                                    vec![new_rot.clone()],
                                    vec![reserve.clone()],
                                );

                                if let Ok(ref tr) = temp_roster {
                                    if !self.checker.check(tr).iter().any(|v| v.is_error()) {
                                        *rot = new_rot;
                                        assignments.push(ReserveAssignment {
                                            crew_id: reserve.id.clone(),
                                            rotation_index: rot_idx,
                                        });
                                        covered = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Reserve crew has no rotation yet; create a fresh rotation for them
                    if let Ok(new_rot) = Rotation::new(
                        crate::domain::rotation::RotationId::new(format!("RES_{}", reserve.id)),
                        reserve.id.clone(),
                        vec![pairing.clone()],
                    ) {
                        let temp_roster = Roster::with_crew(
                            roster.id.clone(),
                            roster.period.clone(),
                            roster.legs().cloned().collect(),
                            vec![new_rot.clone()],
                            vec![reserve.clone()],
                        );

                        if let Ok(ref tr) = temp_roster {
                            if !self.checker.check(tr).iter().any(|v| v.is_error()) {
                                let new_rot_idx = rotations.len();
                                rotations.push(new_rot);
                                assignments.push(ReserveAssignment {
                                    crew_id: reserve.id.clone(),
                                    rotation_index: new_rot_idx,
                                });
                                covered = true;
                                break;
                            }
                        }
                    }
                }
            }

            if !covered {
                still_uncovered.push(pairing.clone());
            }
        }

        let new_roster = Roster::with_crew(
            roster.id.clone(),
            roster.period.clone(),
            roster.legs().cloned().collect(),
            rotations,
            roster.crew_members().cloned().collect(),
        )
        .unwrap_or_else(|_| roster.clone());

        (new_roster, assignments, still_uncovered)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::crew::{CrewRole, Qualification};
    use crate::domain::flight::{AircraftType, AirportCode};
    use crate::legality::{LegalityChecker, test_helpers::*};

    fn make_checker() -> LegalityChecker {
        LegalityChecker::new()
    }

    fn make_reserve(id: &str, base: &str) -> CrewMember {
        CrewMember::new(
            CrewId::new(id),
            format!("Reserve {id}"),
            CrewRole::FirstOfficer,
            vec![Qualification::new(AircraftType::new("B738"))],
            AirportCode::new(base),
        )
    }

    #[test]
    fn empty_absent_list_makes_no_assignments() {
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 8, 10)]);
        let d2 = make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 22, 24)]);
        let p1 = make_pairing("P1", "LHR", vec![d1, d2]);
        let r1 = make_rotation("R1", "C1", vec![p1]);
        let roster = make_roster(vec![], vec![r1]);

        let mut pool = ReservePool::new();
        pool.add(make_reserve("RES1", "LHR"));

        let checker = make_checker();
        let allocator = ReserveAllocator::new(&checker);
        let result = allocator.allocate(&roster, &[], &pool);

        assert_eq!(result.assignments.len(), 0);
        assert_eq!(result.uncovered_rotations.len(), 0);
    }

    #[test]
    fn absent_crew_with_empty_pool_is_uncovered() {
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 8, 10)]);
        let d2 = make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 22, 24)]);
        let p1 = make_pairing("P1", "LHR", vec![d1, d2]);
        let r1 = make_rotation("R1", "C1", vec![p1]);
        let roster = make_roster(vec![], vec![r1]);

        let pool = ReservePool::new();
        let checker = make_checker();
        let allocator = ReserveAllocator::new(&checker);
        let result = allocator.allocate(&roster, &[CrewId::new("C1")], &pool);

        assert_eq!(result.assignments.len(), 0);
        assert_eq!(result.uncovered_rotations, vec![0]);
    }

    #[test]
    fn reserve_pool_len_and_is_empty() {
        let mut pool = ReservePool::new();
        assert!(pool.is_empty());
        pool.add(make_reserve("RES1", "LHR"));
        assert_eq!(pool.len(), 1);
        assert!(!pool.is_empty());
    }
}
