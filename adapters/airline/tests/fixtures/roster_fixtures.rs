// tests/fixtures/roster_fixtures.rs
//! Test‑fixture utilities for constructing canonical rosters used in UC‑AIR‑002.
//! These are **only** for test purposes and do not expose any production API.

use chrono::TimeZone;
use chrono::Utc;
use coralys_airline::domain::crew::{CrewId, CrewMember, CrewRole, Qualification};
use coralys_airline::domain::duty::{Duty, DutyId};
use coralys_airline::domain::flight::{AircraftType, AirportCode, FlightLeg, FlightLegId, FlightNumber};
use coralys_airline::domain::pairing::{Pairing, PairingId};
use coralys_airline::domain::rotation::{Rotation, RotationId};
use coralys_airline::domain::roster::{Roster, RosterId, PlanningPeriod};

/// Helper to create a simple `FlightLeg`.
fn make_leg(id: &str, origin: &str, dest: &str, dep_h: i64, arr_h: i64) -> FlightLeg {
    let base = Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap();
    FlightLeg::new(
        FlightLegId::new(id),
        FlightNumber::new(format!("XX{id}")),
        AirportCode::new(origin),
        AirportCode::new(dest),
        base + chrono::Duration::hours(dep_h),
        base + chrono::Duration::hours(arr_h),
        AircraftType::new("B738"),
    )
}

/// Helper to create a simple `Rotation` with two duties.
fn make_rotation(crew_id: &str) -> Rotation {
    let leg1 = make_leg("L1", "LHR", "CDG", 8, 10);
    let leg2 = make_leg("L2", "CDG", "LHR", 14, 16);
    let d1 = Duty::new(DutyId::new("D1"), vec![leg1]).unwrap();
    let d2 = Duty::new(DutyId::new("D2"), vec![leg2]).unwrap();
    let pairing = Pairing::new(
        PairingId::new("P1"),
        AirportCode::new("LHR"),
        vec![d1, d2],
    )
    .unwrap();
    Rotation::new(
        RotationId::new(format!("R-{crew_id}")),
        CrewId::new(crew_id),
        vec![pairing],
    )
    .unwrap()
}

/// Helper to create a simple `CrewMember`.
fn make_crew_member(id: &str) -> CrewMember {
    CrewMember::new(
        CrewId::new(id),
        format!("Crew {id}"),
        CrewRole::Captain,
        vec![Qualification::new(AircraftType::new("B738"))],
        AirportCode::new("LHR"),
    )
}

/// Returns a canonical, valid roster with two crews, each having a round‑trip pairing split into two duties.
// Returns a canonical, valid roster with two crews, each having a round‑trip pairing split into two duties.
pub fn canonical_roster() -> Roster {
    // Legs for the schedule.
    let leg_out_c1 = make_leg("L1", "LHR", "CDG", 8, 10);
    let leg_return_c1 = make_leg("L1R", "CDG", "LHR", 20, 22);
    let leg_out_c2 = make_leg("L2", "LHR", "CDG", 8, 10);
    let leg_return_c2 = make_leg("L2R", "CDG", "LHR", 24, 26);

    // Crew C1 duties.
    let duty_out_c1 = Duty::new(DutyId::new("D1"), vec![leg_out_c1.clone()]).unwrap();
    let duty_return_c1 = Duty::new(DutyId::new("D1R"), vec![leg_return_c1.clone()]).unwrap();
    let pairing_c1 = Pairing::new(
        PairingId::new("P1"),
        AirportCode::new("LHR"),
        vec![duty_out_c1, duty_return_c1],
    )
    .unwrap();
    let rot_c1 = Rotation::new(
        RotationId::new("R-C1"),
        CrewId::new("C1"),
        vec![pairing_c1],
    )
    .unwrap();

    // Crew C2 duties.
    let duty_out_c2 = Duty::new(DutyId::new("D2"), vec![leg_out_c2.clone()]).unwrap();
    let duty_return_c2 = Duty::new(DutyId::new("D2R"), vec![leg_return_c2.clone()]).unwrap();
    let pairing_c2 = Pairing::new(
        PairingId::new("P2"),
        AirportCode::new("LHR"),
        vec![duty_out_c2, duty_return_c2],
    )
    .unwrap();
    let rot_c2 = Rotation::new(
        RotationId::new("R-C2"),
        CrewId::new("C2"),
        vec![pairing_c2],
    )
    .unwrap();

    // All legs for the roster (including return legs).
    let legs = vec![leg_out_c1, leg_return_c1, leg_out_c2, leg_return_c2];
    let rotations = vec![rot_c1, rot_c2];
    let crew = vec![make_crew_member("C1"), make_crew_member("C2")];
    Roster::with_crew(
        RosterId::new("R0"),
        PlanningPeriod::new(
            Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 7, 31, 23, 59, 59).unwrap(),
        ),
        legs,
        rotations,
        crew,
    )
    .expect("canonical roster construction should succeed")
}

/// Convenience to produce a roster with a single duty for a given crew.
pub fn roster_with_one_duty(crew_id: &str) -> Roster {
    let leg = make_leg("L1", "LHR", "CDG", 8, 10);
    let duty = Duty::new(DutyId::new("D1"), vec![leg]).unwrap();
    let pairing = Pairing::new(
        PairingId::new("P1"),
        AirportCode::new("LHR"),
        vec![duty],
    )
    .unwrap();
    let rotation = Rotation::new(
        RotationId::new(format!("R-{crew_id}")),
        CrewId::new(crew_id),
        vec![pairing],
    )
    .unwrap();
    let crew = vec![make_crew_member(crew_id)];
    Roster::with_crew(
        RosterId::new("R1"),
        PlanningPeriod::new(
            Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 7, 31, 23, 59, 59).unwrap(),
        ),
        vec![],
        vec![rotation],
        crew,
    )
    .expect("roster with one duty should succeed")
}
