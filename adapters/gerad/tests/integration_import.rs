//! Integration test: load the fixture dataset through [`GeradImporter`] and
//! verify the resulting [`Roster`] has the expected shape.
//!
//! The fixture files in `tests/fixtures/` represent a small but realistic
//! GERAD G-2014-22 dataset:
//!   - 10 flight legs across 5 days
//!   - 8 crew members (captains, first officers, cabin crew)
//!   - 5 duties (each covering an outbound + return leg)
//!   - 5 pairings (one duty each, all based at ORD)
//!   - 15 assignments (3 crew per pairing)

use std::path::PathBuf;

use coralys_gerad::GeradImporter;

fn fixtures_dir() -> PathBuf {
    // `CARGO_MANIFEST_DIR` is set by Cargo to the crate root at test time.
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set — run via `cargo test`");
    PathBuf::from(manifest).join("tests").join("fixtures")
}

#[test]
fn fixture_dataset_loads_without_error() {
    let roster = GeradImporter::new(fixtures_dir())
        .load()
        .expect("fixture dataset should import cleanly");

    // ── Flight legs ───────────────────────────────────────────────────────────
    assert_eq!(roster.leg_count(), 10, "expected 10 flight legs");

    // ── Crew members ──────────────────────────────────────────────────────────
    assert_eq!(
        roster.crew_member_count(),
        8,
        "expected 8 crew member records"
    );

    // ── Rotations (one per crew member that has at least one assignment) ──────
    // 8 crew members, all assigned to at least one pairing.
    assert_eq!(roster.crew_count(), 8, "expected 8 rotations");
}

#[test]
fn fixture_planning_period_spans_all_legs() {
    use chrono::{TimeZone, Utc};

    let roster = GeradImporter::new(fixtures_dir())
        .load()
        .expect("fixture dataset should import cleanly");

    // Earliest departure: FL0001 2014-01-06T08:00 UTC
    // Latest arrival:     FL0010 2014-01-10T13:30 UTC
    // Period is extended by ±1 day by the mapper.
    let expected_start = Utc.with_ymd_and_hms(2014, 1, 5, 8, 0, 0).unwrap();
    let expected_end = Utc.with_ymd_and_hms(2014, 1, 11, 13, 30, 0).unwrap();

    assert!(
        roster.period.start <= expected_start,
        "period start {:?} should be on or before {:?}",
        roster.period.start,
        expected_start
    );
    assert!(
        roster.period.end >= expected_end,
        "period end {:?} should be on or after {:?}",
        roster.period.end,
        expected_end
    );
}

#[test]
fn fixture_all_legs_are_accessible_by_id() {
    use coralys_airline::domain::FlightLegId;

    let roster = GeradImporter::new(fixtures_dir())
        .load()
        .expect("fixture dataset should import cleanly");

    for n in 1..=10u32 {
        let id = FlightLegId::new(format!("FL{n:04}"));
        assert!(
            roster.leg(&id).is_some(),
            "leg {id} should be present in the roster"
        );
    }
}

#[test]
fn fixture_crew_qualifications_are_mapped() {
    use coralys_airline::domain::{AircraftType, CrewId};

    let roster = GeradImporter::new(fixtures_dir())
        .load()
        .expect("fixture dataset should import cleanly");

    // C0008 holds both B738 and A320 qualifications.
    let c8 = roster
        .crew_member(&CrewId::new("C0008"))
        .expect("C0008 should be present");

    assert!(
        c8.is_qualified_for(&AircraftType::new("B738")),
        "C0008 should be qualified for B738"
    );
    assert!(
        c8.is_qualified_for(&AircraftType::new("A320")),
        "C0008 should be qualified for A320"
    );
    assert!(
        !c8.is_qualified_for(&AircraftType::new("B744")),
        "C0008 should NOT be qualified for B744"
    );
}
