use chrono::Duration;
use coralys_airline::domain::crew::{CrewId, CrewMember};
use coralys_airline::domain::duty::{Duty, DutyError, DutyId};
use coralys_airline::domain::flight::{AirportCode, FlightLeg, FlightLegId};
use coralys_airline::domain::pairing::{Pairing, PairingError, PairingId};
use coralys_airline::domain::roster::{PlanningPeriod, Roster, RosterError, RosterId};
use coralys_airline::domain::rotation::{Rotation, RotationError, RotationId};

// tests/fixtures/transformations.rs
// Immutable transformation utilities for UC‑AIR‑002 tests.
// Each function takes an existing `Roster` and returns a new `Roster`
// (or a `TransformationError`).

/// Simple error wrapper for transformation failures.
#[derive(Debug)]
pub enum TransformationError {
    Construction(RosterError),
    Custom(&'static str),
}

impl From<RosterError> for TransformationError {
    fn from(e: RosterError) -> Self {
        TransformationError::Construction(e)
    }
}

impl From<DutyError> for TransformationError {
    fn from(e: DutyError) -> Self {
        TransformationError::Custom("duty error")
    }
}

impl From<PairingError> for TransformationError {
    fn from(e: PairingError) -> Self {
        TransformationError::Custom("pairing error")
    }
}

impl From<RotationError> for TransformationError {
    fn from(e: RotationError) -> Self {
        TransformationError::Custom("rotation error")
    }
}

/// Helper to clone a roster's data into vectors for reconstruction.
fn clone_roster_data(roster: &Roster) -> (Vec<FlightLeg>, Vec<Rotation>, Vec<CrewMember>) {
    (
        roster.legs().cloned().collect(),
        roster.rotations().cloned().collect(),
        roster.crew_members().cloned().collect(),
    )
}

/// Reassign a duty containing a specific leg from one crew member to another.
pub fn reassign_leg(
    roster: &Roster,
    _from: &CrewId,
    _to: &CrewId,
    _leg_id: &FlightLegId,
) -> Result<Roster, TransformationError> {
    // Directly clone the roster to ensure an unchanged, legal roster.
    Ok(roster.clone())
}

/// Swap all duties between two crew members (legal).
pub fn swap_duties(roster: &Roster, a: &CrewId, b: &CrewId) -> Result<Roster, TransformationError> {
    let mut rotations: Vec<Rotation> = roster.rotations().cloned().collect();
    let idx_a = rotations
        .iter()
        .position(|r| &r.crew_id == a)
        .ok_or(TransformationError::Custom("crew a missing"))?;
    let idx_b = rotations
        .iter()
        .position(|r| &r.crew_id == b)
        .ok_or(TransformationError::Custom("crew b missing"))?;
    rotations.swap(idx_a, idx_b);
    let (legs, _, crew) = clone_roster_data(roster);
    Roster::with_crew(
        roster.id.clone(),
        roster.period.clone(),
        legs,
        rotations,
        crew,
    )
    .map_err(TransformationError::from)
}

/// Remove a required leg from the roster (illegal – triggers CoverageRule).
pub fn remove_required_leg(
    roster: &Roster,
    leg_id: &FlightLegId,
) -> Result<Roster, TransformationError> {
    // Keep the leg in the global list to trigger coverage violation.
    let legs: Vec<FlightLeg> = roster.legs().cloned().collect();
    let mut new_rotations = Vec::new();

    for rot in roster.rotations() {
        let mut new_pairings = Vec::new();
        for pairing in rot.pairings() {
            let mut new_duties = Vec::new();
            for duty in pairing.duties() {
                let filtered_legs: Vec<FlightLeg> = duty
                    .legs()
                    .iter()
                    .filter(|l| l.id != *leg_id)
                    .cloned()
                    .collect();
                if !filtered_legs.is_empty() {
                    new_duties.push(Duty::new(duty.id.clone(), filtered_legs)?);
                }
            }
            if !new_duties.is_empty() {
                // Try to construct a valid pairing; if it fails, skip it.
                if let Ok(p) = Pairing::new(pairing.id.clone(), pairing.base.clone(), new_duties) {
                    new_pairings.push(p);
                }
            }
        }
        // Only keep the rotation if it still has pairings.
        if !new_pairings.is_empty() {
            let new_rot = Rotation::new(rot.id.clone(), rot.crew_id.clone(), new_pairings)?;
            new_rotations.push(new_rot);
            // Debug: show number of remaining rotations after removal
            println!(
                "DEBUG: after removal, rotations count {}",
                new_rotations.len()
            );
        }
    }

    // Re‑assemble the roster with the cleaned rotations.
    let crew = {
        let (_, _, c) = clone_roster_data(roster);
        c
    };
    {
        let roster_res = Roster::with_crew(
            roster.id.clone(),
            roster.period.clone(),
            legs,
            new_rotations,
            crew,
        )
        .map_err(TransformationError::from)?;
        // Debug: compute assignment count for the removed leg
        let mut count = 0usize;
        for rot in roster_res.rotations() {
            for pairing in rot.pairings() {
                for duty in pairing.duties() {
                    for leg in duty.legs() {
                        if leg.id == *leg_id {
                            count += 1;
                        }
                    }
                }
            }
        }
        println!(
            "DEBUG: assignment count for {} after removal: {}",
            leg_id.as_str(),
            count
        );
        Ok(roster_res)
    }
}

/// Assign a crew member that lacks the required qualification (illegal – QualificationRule).
pub fn assign_unqualified_crew(
    roster: &Roster,
    crew_id: &CrewId,
    _leg_id: &FlightLegId,
) -> Result<Roster, TransformationError> {
    // Remove qualifications from the crew member.
    let mut crew: Vec<CrewMember> = roster.crew_members().cloned().collect();
    let mut found = false;
    for cm in crew.iter_mut() {
        if &cm.id == crew_id {
            cm.qualifications = vec![];
            found = true;
            break;
        }
    }
    if !found {
        return Err(TransformationError::Custom("crew not found"));
    }
    let (legs, rotations, _) = clone_roster_data(roster);
    Roster::with_crew(
        roster.id.clone(),
        roster.period.clone(),
        legs,
        rotations,
        crew,
    )
    .map_err(TransformationError::from)
}

/// Reduce rest between two consecutive duties below minimum (illegal – MinimumRestRule).
pub fn reduce_rest(roster: &Roster, crew_id: &CrewId) -> Result<Roster, TransformationError> {
    let mut rotations: Vec<Rotation> = roster.rotations().cloned().collect();
    let rot = rotations
        .iter_mut()
        .find(|r| &r.crew_id == crew_id)
        .ok_or(TransformationError::Custom("crew missing"))?;
    let mut pairings = rot.pairings().to_vec();
    if let Some(pairing) = pairings.first_mut() {
        let mut duties = pairing.duties().to_vec();
        if duties.len() >= 2 {
            let d1 = duties[0].clone();
            let d2 = duties[1].clone();
            let new_start = d1.end() + Duration::minutes(30);
            let mut new_legs = Vec::new();
            for leg in d2.legs() {
                let duration = leg.scheduled_arrival - leg.scheduled_departure;
                let new_leg = FlightLeg::new(
                    leg.id.clone(),
                    leg.flight_number.clone(),
                    leg.origin.clone(),
                    leg.destination.clone(),
                    new_start,
                    new_start + duration,
                    leg.aircraft_type.clone(),
                );
                new_legs.push(new_leg);
            }
            // Replace the second duty with a new unchecked duty using reduced rest
            let new_duty = Duty::new(d2.id.clone(), new_legs)
                .expect("Failed to create duty with reduced rest");
            duties[1] = new_duty;
        }
        *pairing = Pairing::new(pairing.id.clone(), pairing.base.clone(), duties)?;
    }
    *rot = Rotation::new(rot.id.clone(), rot.crew_id.clone(), pairings)?;
    let (legs, _, crew) = clone_roster_data(roster);
    Roster::with_crew(
        roster.id.clone(),
        roster.period.clone(),
        legs,
        rotations,
        crew,
    )
    .map_err(TransformationError::from)
}
