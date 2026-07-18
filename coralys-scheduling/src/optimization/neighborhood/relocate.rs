//! Relocate move: move a pairing from one rotation to another.
//!
//! [`relocate_pairing`] removes a pairing from a source rotation and appends
//! it to a destination rotation.  The move is pure — it does not modify the
//! input roster.
//!
//! Returns `None` if any index is out of bounds, if source and destination
//! are the same rotation, or if removing the pairing would leave the source
//! rotation empty (a rotation must have at least one pairing).

use crate::domain::roster::Roster;

/// Move pairing at `pairing_index` in `src_rotation` to `dst_rotation`.
///
/// Returns `None` if:
/// - `src_rotation == dst_rotation`
/// - any index is out of bounds
/// - the source rotation has only one pairing (removing it would leave it empty)
pub fn relocate_pairing(
    roster: &Roster,
    src_rotation: usize,
    pairing_index: usize,
    dst_rotation: usize,
) -> Option<Roster> {
    if src_rotation == dst_rotation {
        return None;
    }

    let rotations: Vec<_> = roster.rotations().collect();
    if src_rotation >= rotations.len() || dst_rotation >= rotations.len() {
        return None;
    }

    let src = rotations[src_rotation];
    let dst = rotations[dst_rotation];

    // pairings() returns &[Pairing] — use as slice directly
    let src_pairings = src.pairings();
    if pairing_index >= src_pairings.len() {
        return None;
    }

    // A rotation must retain at least one pairing.
    if src_pairings.len() == 1 {
        return None;
    }

    let pairing_to_move = src_pairings[pairing_index].clone();

    // Build new source pairing list (without the moved pairing).
    // src_pairings is &[Pairing]; .iter() yields &Pairing.
    let new_src_pairings: Vec<_> = src_pairings
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != pairing_index)
        .map(|(_, p)| p.clone())
        .collect();

    // Build new destination pairing list (with the moved pairing appended).
    // dst.pairings() returns &[Pairing]; convert to Vec via .to_vec().
    let mut new_dst_pairings: Vec<_> = dst.pairings().to_vec();
    new_dst_pairings.push(pairing_to_move);

    use crate::domain::rotation::Rotation;
    let new_src = Rotation::new(src.id.clone(), src.crew_id.clone(), new_src_pairings).ok()?;
    let new_dst = Rotation::new(dst.id.clone(), dst.crew_id.clone(), new_dst_pairings).ok()?;

    // rotations is Vec<&Rotation>; .iter() yields &&Rotation.
    let new_rotations: Vec<_> = rotations
        .iter()
        .enumerate()
        .map(|(i, r)| {
            if i == src_rotation {
                new_src.clone()
            } else if i == dst_rotation {
                new_dst.clone()
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
    ).ok()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legality::test_helpers::*;

    fn make_two_rotation_roster() -> Roster {
        // P1: day 1 (hours 8–24), P3: day 2 (hours 32–48) — must be chronologically ordered.
        let d1a = make_duty("D1a", vec![make_leg("L1a", "LHR", "CDG", 8, 10)]);
        let d1b = make_duty("D1b", vec![make_leg("L1b", "CDG", "LHR", 22, 24)]);
        let d2a = make_duty("D2a", vec![make_leg("L2a", "LHR", "CDG", 8, 10)]);
        let d2b = make_duty("D2b", vec![make_leg("L2b", "CDG", "LHR", 22, 24)]);
        // R1 has two pairings so it can donate one.
        let p1 = make_pairing("P1", "LHR", vec![d1a, d1b]);
        let p2 = make_pairing("P2", "LHR", vec![d2a, d2b]);
        // P3 is on day 2 (base+32h .. base+48h) so it sorts after P1 in R1.
        let d3a = make_duty("D3a", vec![make_leg("L3a", "LHR", "AMS", 32, 34)]);
        let d3b = make_duty("D3b", vec![make_leg("L3b", "AMS", "LHR", 46, 48)]);
        let p3 = make_pairing("P3", "LHR", vec![d3a, d3b]);
        let r1 = make_rotation("R1", "C1", vec![p1, p3]);
        let r2 = make_rotation("R2", "C2", vec![p2]);
        make_roster(vec![], vec![r1, r2])
    }

    #[test]
    fn relocate_moves_pairing() {
        let roster = make_two_rotation_roster();
        let result = relocate_pairing(&roster, 0, 1, 1); // move P3 from R1 to R2
        assert!(result.is_some());
        let new_roster = result.unwrap();
        let rots: Vec<_> = new_roster.rotations().collect();
        assert_eq!(rots[0].pairings().len(), 1); // R1 now has 1 pairing
        assert_eq!(rots[1].pairings().len(), 2); // R2 now has 2 pairings
    }

    #[test]
    fn relocate_same_rotation_returns_none() {
        let roster = make_two_rotation_roster();
        assert!(relocate_pairing(&roster, 0, 0, 0).is_none());
    }

    #[test]
    fn relocate_out_of_bounds_rotation_returns_none() {
        let roster = make_two_rotation_roster();
        assert!(relocate_pairing(&roster, 0, 0, 99).is_none());
    }

    #[test]
    fn relocate_last_pairing_returns_none() {
        let roster = make_two_rotation_roster();
        // R2 has only one pairing — cannot relocate it.
        assert!(relocate_pairing(&roster, 1, 0, 0).is_none());
    }
}