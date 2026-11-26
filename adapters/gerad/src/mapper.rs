//! GERAD → Coralys airline domain model mapper.
//!
//! [`GeradMapper`] translates a [`RawGeradDataset`] into the Coralys airline
//! domain model ([`Roster`]).  After this step the rest of the system has no
//! knowledge that the data originated from the GERAD benchmark.
//!
//! # Mapping rules
//!
//! | GERAD concept        | Coralys domain type          |
//! |----------------------|------------------------------|
//! | `RawFlightLeg`       | [`FlightLeg`]                |
//! | `RawCrewMember`      | [`CrewMember`]               |
//! | `RawDutyLeg` rows    | [`Duty`] (grouped by duty_id)|
//! | `RawPairingDuty` rows| [`Pairing`] (grouped)        |
//! | `RawAssignment` rows | [`Rotation`] (grouped)       |
//! | All of the above     | [`Roster`]                   |
//!
//! # Timestamp handling
//!
//! GERAD timestamps are `YYYY-MM-DDTHH:MM` strings.  The mapper parses them
//! as UTC (the benchmark dataset uses a single timezone throughout).  If a
//! future dataset requires timezone offsets, add a station-offset table and
//! pass it to [`GeradMapper::new`].

use std::collections::HashMap;

use chrono::{NaiveDateTime, TimeZone, Utc};

use coralys_airline::domain::{
    AircraftType, AirportCode, CrewId, CrewMember, CrewRole, Duty, DutyId, FlightLeg, FlightLegId,
    FlightNumber, Pairing, PairingId, PlanningPeriod, Qualification, Roster, RosterId, Rotation,
    RotationId,
};

use crate::error::GeradError;
use crate::models::RawGeradDataset;

/// Translates a [`RawGeradDataset`] into a Coralys [`Roster`].
pub struct GeradMapper;

impl GeradMapper {
    /// Create a new mapper.
    pub fn new() -> Self {
        Self
    }

    /// Perform the full translation.
    ///
    /// # Errors
    /// Returns [`GeradError`] if any referential integrity check fails or if
    /// the domain model rejects a constructed entity.
    pub fn map(&self, raw: &RawGeradDataset) -> Result<Roster, GeradError> {
        // Step 1 — map flight legs (indexed by leg_id for later lookup).
        let legs = self.map_legs(raw)?;

        // Step 2 — map crew members (indexed by crew_id).
        let crew_members = self.map_crew(raw)?;

        // Step 3 — reconstruct duties from duty-leg membership rows.
        let duties = self.map_duties(raw, &legs)?;

        // Step 4 — reconstruct pairings from pairing-duty membership rows.
        let pairings = self.map_pairings(raw, &duties)?;

        // Step 5 — reconstruct rotations from assignment rows.
        let rotations = self.map_rotations(raw, &crew_members, &pairings)?;

        // Step 6 — derive the planning period from the leg schedule.
        let period = self.derive_planning_period(&legs)?;

        // Step 7 — assemble the roster.
        let roster = Roster::with_crew(
            RosterId::new("GERAD-G-2014-22"),
            period,
            legs.into_values().collect(),
            rotations,
            crew_members.into_values().collect(),
        )
        .map_err(|e| GeradError::RosterConstruction {
            detail: e.to_string(),
        })?;

        Ok(roster)
    }

    // ── Step 1: flight legs ───────────────────────────────────────────────────

    fn map_legs(
        &self,
        raw: &RawGeradDataset,
    ) -> Result<HashMap<FlightLegId, FlightLeg>, GeradError> {
        let mut map = HashMap::with_capacity(raw.legs.len());

        for r in &raw.legs {
            let id = FlightLegId::new(&r.leg_id);
            let dep = parse_timestamp(
                "flight_leg",
                &r.leg_id,
                "scheduled_departure",
                &r.scheduled_departure,
            )?;
            let arr = parse_timestamp(
                "flight_leg",
                &r.leg_id,
                "scheduled_arrival",
                &r.scheduled_arrival,
            )?;

            let leg = FlightLeg::new(
                id.clone(),
                FlightNumber::new(&r.flight_number),
                AirportCode::new(&r.origin),
                AirportCode::new(&r.destination),
                dep,
                arr,
                AircraftType::new(&r.aircraft_type),
            );

            map.insert(id, leg);
        }

        Ok(map)
    }

    // ── Step 2: crew members ──────────────────────────────────────────────────

    fn map_crew(&self, raw: &RawGeradDataset) -> Result<HashMap<CrewId, CrewMember>, GeradError> {
        let mut map = HashMap::with_capacity(raw.crew.len());

        for r in &raw.crew {
            let id = CrewId::new(&r.crew_id);
            let role = parse_role(&r.crew_id, &r.role)?;
            let qualifications = parse_qualifications(&r.qualifications);
            let base = AirportCode::new(&r.base);

            let member = CrewMember::new(id.clone(), &r.name, role, qualifications, base);
            map.insert(id, member);
        }

        Ok(map)
    }

    // ── Step 3: duties ────────────────────────────────────────────────────────

    fn map_duties(
        &self,
        raw: &RawGeradDataset,
        legs: &HashMap<FlightLegId, FlightLeg>,
    ) -> Result<HashMap<DutyId, Duty>, GeradError> {
        // Group duty-leg rows by duty_id, preserving sequence order.
        let mut groups: HashMap<String, Vec<(u32, FlightLeg)>> = HashMap::new();
        for r in &raw.duty_legs {
            let leg_id = FlightLegId::new(&r.leg_id);
            let leg = legs.get(&leg_id).ok_or_else(|| GeradError::UnknownLegRef {
                duty_id: r.duty_id.clone(),
                leg_id: r.leg_id.clone(),
            })?;
            groups
                .entry(r.duty_id.clone())
                .or_default()
                .push((r.sequence_number, leg.clone()));
        }

        let mut map = HashMap::with_capacity(groups.len());
        for (duty_id_str, mut seq_legs) in groups {
            // Sort by sequence number to guarantee correct order.
            seq_legs.sort_by_key(|(seq, _)| *seq);
            let ordered_legs: Vec<FlightLeg> = seq_legs.into_iter().map(|(_, l)| l).collect();

            let duty_id = DutyId::new(&duty_id_str);
            let duty = Duty::new(duty_id.clone(), ordered_legs).map_err(|e| {
                GeradError::DutyConstruction {
                    duty_id: duty_id_str.clone(),
                    detail: e.to_string(),
                }
            })?;

            map.insert(duty_id, duty);
        }

        Ok(map)
    }

    // ── Step 4: pairings ──────────────────────────────────────────────────────

    fn map_pairings(
        &self,
        raw: &RawGeradDataset,
        duties: &HashMap<DutyId, Duty>,
    ) -> Result<HashMap<PairingId, Pairing>, GeradError> {
        // Group pairing-duty rows by pairing_id.
        // We also need the base airport — it is the same for all rows of a
        // pairing, so we take it from the first row encountered.
        let mut groups: HashMap<String, (String, Vec<(u32, Duty)>)> = HashMap::new();

        for r in &raw.pairing_duties {
            let duty_id = DutyId::new(&r.duty_id);
            let duty = duties
                .get(&duty_id)
                .ok_or_else(|| GeradError::UnknownDutyRef {
                    pairing_id: r.pairing_id.clone(),
                    duty_id: r.duty_id.clone(),
                })?;

            let entry = groups
                .entry(r.pairing_id.clone())
                .or_insert_with(|| (r.base.clone(), Vec::new()));
            entry.1.push((r.sequence_number, duty.clone()));
        }

        let mut map = HashMap::with_capacity(groups.len());
        for (pairing_id_str, (base_str, mut seq_duties)) in groups {
            seq_duties.sort_by_key(|(seq, _)| *seq);
            let ordered_duties: Vec<Duty> = seq_duties.into_iter().map(|(_, d)| d).collect();

            let pairing_id = PairingId::new(&pairing_id_str);
            let base = AirportCode::new(&base_str);

            let pairing = Pairing::new(pairing_id.clone(), base, ordered_duties).map_err(|e| {
                GeradError::PairingConstruction {
                    pairing_id: pairing_id_str.clone(),
                    detail: e.to_string(),
                }
            })?;

            map.insert(pairing_id, pairing);
        }

        Ok(map)
    }

    // ── Step 5: rotations ─────────────────────────────────────────────────────

    fn map_rotations(
        &self,
        raw: &RawGeradDataset,
        crew_members: &HashMap<CrewId, CrewMember>,
        pairings: &HashMap<PairingId, Pairing>,
    ) -> Result<Vec<Rotation>, GeradError> {
        // Group assignments by crew_id.
        let mut groups: HashMap<String, Vec<Pairing>> = HashMap::new();

        for r in &raw.assignments {
            let crew_id = CrewId::new(&r.crew_id);
            if !crew_members.contains_key(&crew_id) {
                return Err(GeradError::UnknownCrewRef {
                    crew_id: r.crew_id.clone(),
                });
            }

            let pairing_id = PairingId::new(&r.pairing_id);
            let pairing =
                pairings
                    .get(&pairing_id)
                    .ok_or_else(|| GeradError::UnknownPairingRef {
                        pairing_id: r.pairing_id.clone(),
                    })?;

            groups
                .entry(r.crew_id.clone())
                .or_default()
                .push(pairing.clone());
        }

        let mut rotations = Vec::with_capacity(groups.len());
        for (crew_id_str, mut crew_pairings) in groups {
            // Sort pairings chronologically so the rotation is well-ordered.
            crew_pairings.sort_by_key(|p| p.start());

            let crew_id = CrewId::new(&crew_id_str);
            let rotation_id = RotationId::new(format!("ROT-{crew_id_str}"));

            // Rotation::new validates that pairings are non-overlapping.
            let rotation =
                coralys_airline::domain::Rotation::new(rotation_id, crew_id, crew_pairings)
                    .map_err(|e| GeradError::RosterConstruction {
                        detail: e.to_string(),
                    })?;

            rotations.push(rotation);
        }

        Ok(rotations)
    }

    // ── Step 6: planning period ───────────────────────────────────────────────

    fn derive_planning_period(
        &self,
        legs: &HashMap<FlightLegId, FlightLeg>,
    ) -> Result<PlanningPeriod, GeradError> {
        if legs.is_empty() {
            return Err(GeradError::validation(
                "dataset contains no flight legs; cannot derive planning period",
            ));
        }

        let earliest = legs.values().map(|l| l.scheduled_departure).min().unwrap();
        let latest = legs.values().map(|l| l.scheduled_arrival).max().unwrap();

        // Extend the period by one day on each side so that all legs and
        // rotations fall comfortably within the declared window.
        let start = earliest - chrono::Duration::days(1);
        let end = latest + chrono::Duration::days(1);

        Ok(PlanningPeriod::new(start, end))
    }
}

impl Default for GeradMapper {
    fn default() -> Self {
        Self::new()
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Parse a `YYYY-MM-DDTHH:MM` string into a UTC [`chrono::DateTime`].
fn parse_timestamp(
    entity: &'static str,
    record_id: &str,
    _field: &'static str,
    value: &str,
) -> Result<chrono::DateTime<Utc>, GeradError> {
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M")
        .map(|ndt| Utc.from_utc_datetime(&ndt))
        .map_err(|_| GeradError::InvalidTimestamp {
            entity,
            record_id: record_id.to_owned(),
            value: value.to_owned(),
        })
}

/// Parse a role string into a [`CrewRole`].
fn parse_role(crew_id: &str, role: &str) -> Result<CrewRole, GeradError> {
    match role.trim().to_lowercase().as_str() {
        "captain" => Ok(CrewRole::Captain),
        "first_officer" | "firstofficer" | "fo" => Ok(CrewRole::FirstOfficer),
        "cabin_crew_senior" | "cabincrewsenior" | "purser" => Ok(CrewRole::CabinCrewSenior),
        "cabin_crew" | "cabincrew" | "cc" => Ok(CrewRole::CabinCrew),
        "relief_pilot" | "reliefpilot" | "rp" => Ok(CrewRole::ReliefPilot),
        other => Err(GeradError::InvalidField {
            entity: "crew_member",
            record_id: crew_id.to_owned(),
            field: "role",
            detail: format!("unknown role '{other}'"),
        }),
    }
}

/// Parse a comma-separated qualifications string into a [`Vec<Qualification>`].
///
/// Empty strings and whitespace-only tokens are silently skipped.
fn parse_qualifications(raw: &str) -> Vec<Qualification> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| Qualification::new(AircraftType::new(s)))
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{RawAssignment, RawCrewMember, RawDutyLeg, RawFlightLeg, RawPairingDuty};

    fn minimal_raw() -> RawGeradDataset {
        RawGeradDataset {
            legs: vec![
                RawFlightLeg {
                    leg_id: "FL0001".into(),
                    flight_number: "AA1234".into(),
                    origin: "ORD".into(),
                    destination: "LAX".into(),
                    scheduled_departure: "2014-01-06T08:00".into(),
                    scheduled_arrival: "2014-01-06T11:30".into(),
                    aircraft_type: "B738".into(),
                },
                RawFlightLeg {
                    leg_id: "FL0002".into(),
                    flight_number: "AA1235".into(),
                    origin: "LAX".into(),
                    destination: "ORD".into(),
                    scheduled_departure: "2014-01-06T14:00".into(),
                    scheduled_arrival: "2014-01-06T19:30".into(),
                    aircraft_type: "B738".into(),
                },
            ],
            crew: vec![RawCrewMember {
                crew_id: "C0001".into(),
                name: "Alice Smith".into(),
                role: "captain".into(),
                qualifications: "B738".into(),
                base: "ORD".into(),
            }],
            duty_legs: vec![
                RawDutyLeg {
                    duty_id: "D0001".into(),
                    leg_id: "FL0001".into(),
                    sequence_number: 1,
                },
                RawDutyLeg {
                    duty_id: "D0001".into(),
                    leg_id: "FL0002".into(),
                    sequence_number: 2,
                },
            ],
            pairing_duties: vec![RawPairingDuty {
                pairing_id: "P0001".into(),
                duty_id: "D0001".into(),
                sequence_number: 1,
                base: "ORD".into(),
            }],
            assignments: vec![RawAssignment {
                crew_id: "C0001".into(),
                pairing_id: "P0001".into(),
            }],
        }
    }

    #[test]
    fn maps_minimal_dataset_to_roster() {
        let raw = minimal_raw();
        let mapper = GeradMapper::new();
        let roster = mapper.map(&raw).unwrap();

        assert_eq!(roster.leg_count(), 2);
        assert_eq!(roster.crew_count(), 1);
        assert_eq!(roster.crew_member_count(), 1);
    }

    #[test]
    fn parse_role_variants() {
        assert!(matches!(parse_role("C1", "captain"), Ok(CrewRole::Captain)));
        assert!(matches!(
            parse_role("C1", "first_officer"),
            Ok(CrewRole::FirstOfficer)
        ));
        assert!(matches!(parse_role("C1", "fo"), Ok(CrewRole::FirstOfficer)));
        assert!(matches!(
            parse_role("C1", "cabin_crew"),
            Ok(CrewRole::CabinCrew)
        ));
        assert!(matches!(
            parse_role("C1", "relief_pilot"),
            Ok(CrewRole::ReliefPilot)
        ));
        assert!(parse_role("C1", "unknown_role").is_err());
    }

    #[test]
    fn parse_qualifications_comma_separated() {
        let quals = parse_qualifications("B738,A320, B777 ");
        assert_eq!(quals.len(), 3);
    }

    #[test]
    fn parse_qualifications_empty_string() {
        let quals = parse_qualifications("");
        assert!(quals.is_empty());
    }

    #[test]
    fn unknown_leg_ref_returns_error() {
        let mut raw = minimal_raw();
        raw.duty_legs[0].leg_id = "NONEXISTENT".into();
        let err = GeradMapper::new().map(&raw).unwrap_err();
        assert!(matches!(err, GeradError::UnknownLegRef { .. }));
    }
}
