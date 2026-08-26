//! Swap move: exchange a pairing between two rotations.
//!
//! [`swap_pairings`] takes a roster and two (rotation_index, pairing_index)
//! pairs and returns a new roster with those pairings exchanged.  The move
//! is pure — it does not modify the input roster.
//!
//! Returns `None` if any index is out of bounds or if both indices refer to
//! the same rotation.

use crate::domain::roster::Roster;

/// Swap pairing `pairing_a` in rotation `rotation_a` with pairing `pairing_b`
/// in rotation `rotation_b`.
///
/// Returns `None` if:
/// - `rotation_a == rotation_b` (same rotation — no-op)
/// - any index is out of bounds
pub fn swap_pairings(
    roster: &Roster,
    rotation_a: usize,
    pairing_a: usize,
    rotation_b: usize,
    pairing_b: usize,
) -> Option<Roster> {
    if rotation_a == rotation_b {
        return None;
    }

    let rotations: Vec<_> = roster.rotations().collect();
    if rotation_a >= rotations.len() || rotation_b >= rotations.len() {
        return None;
    }

    let rot_a = rotations[rotation_a];
    let rot_b = rotations[rotation_b];

    // pairings() returns &[Pairing] — use directly as a slice
    let pairings_a = rot_a.pairings();
    let pairings_b = rot_b.pairings();

    if pairing_a >= pairings_a.len() || pairing_b >= pairings_b.len() {
        return None;
    }

    // Build new pairing lists with the swap applied.
    // pairings_a/b are &[Pairing]; .iter() yields &Pairing.
    let new_pairings_a: Vec<_> = pairings_a
        .iter()
        .enumerate()
        .map(|(i, p)| {
            if i == pairing_a {
                pairings_b[pairing_b].clone()
            } else {
                p.clone()
            }
        })
        .collect();

    let new_pairings_b: Vec<_> = pairings_b
        .iter()
        .enumerate()
        .map(|(i, p)| {
            if i == pairing_b {
                pairings_a[pairing_a].clone()
            } else {
                p.clone()
            }
        })
        .collect();

    // Rebuild rotations.
    use crate::domain::rotation::Rotation;
    let new_rot_a = Rotation::new(rot_a.id.clone(), rot_a.crew_id.clone(), new_pairings_a).ok()?;
    let new_rot_b = Rotation::new(rot_b.id.clone(), rot_b.crew_id.clone(), new_pairings_b).ok()?;

    // Rebuild the full rotation list.
    // rotations is Vec<&Rotation>; .iter() yields &&Rotation.
    let new_rotations: Vec<_> = rotations
        .iter()
        .enumerate()
        .map(|(i, r)| {
            if i == rotation_a {
                new_rot_a.clone()
            } else if i == rotation_b {
                new_rot_b.clone()
            } else {
                (*r).clone()
            }
        })
        .collect();

    Roster::new(
        roster.id.clone(),
        roster.period.clone(),
        roster.legs().cloned().collect(),
        new_rotations,
    )
    .ok()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legality::test_helpers::*;

    fn make_two_rotation_roster() -> Roster {
        let d1a = make_duty("D1a", vec![make_leg("L1a", "LHR", "CDG", 8, 10)]);
        let d1b = make_duty("D1b", vec![make_leg("L1b", "CDG", "LHR", 22, 24)]);
        let d2a = make_duty("D2a", vec![make_leg("L2a", "LHR", "CDG", 8, 10)]);
        let d2b = make_duty("D2b", vec![make_leg("L2b", "CDG", "LHR", 22, 24)]);
        let p1 = make_pairing("P1", "LHR", vec![d1a, d1b]);
        let p2 = make_pairing("P2", "LHR", vec![d2a, d2b]);
        let r1 = make_rotation("R1", "C1", vec![p1]);
        let r2 = make_rotation("R2", "C2", vec![p2]);
        make_roster(vec![], vec![r1, r2])
    }

    #[test]
    fn swap_exchanges_pairings() {
        use crate::domain::crew::CrewId;

        let roster = make_two_rotation_roster();
        let result = swap_pairings(&roster, 0, 0, 1, 0);
        assert!(result.is_some());
        let new_roster = result.unwrap();

        // rotations() iterates a HashMap — order is non-deterministic.
        // Look up by crew ID instead of by index.
        let rot_c1 = new_roster
            .rotation_for(&CrewId::new("C1"))
            .expect("C1 rotation missing");
        let rot_c2 = new_roster
            .rotation_for(&CrewId::new("C2"))
            .expect("C2 rotation missing");

        // C1 originally had P1; after swap it should have P2.
        assert_eq!(rot_c1.pairings()[0].id.as_str(), "P2");
        // C2 originally had P2; after swap it should have P1.
        assert_eq!(rot_c2.pairings()[0].id.as_str(), "P1");
    }

    #[test]
    fn swap_same_rotation_returns_none() {
        let roster = make_two_rotation_roster();
        assert!(swap_pairings(&roster, 0, 0, 0, 0).is_none());
    }

    #[test]
    fn swap_out_of_bounds_rotation_returns_none() {
        let roster = make_two_rotation_roster();
        assert!(swap_pairings(&roster, 0, 0, 99, 0).is_none());
    }

    #[test]
    fn swap_out_of_bounds_pairing_returns_none() {
        let roster = make_two_rotation_roster();
        assert!(swap_pairings(&roster, 0, 99, 1, 0).is_none());
    }
}
