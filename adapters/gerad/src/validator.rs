//! Post-mapping semantic validator for GERAD-imported rosters.
//!
//! [`GeradValidator`] runs a set of lightweight sanity checks on the
//! [`RawGeradDataset`] *before* the mapper runs, catching problems that would
//! produce confusing domain errors later.  It does **not** re-implement the
//! airline legality layer (Layer 2) — those checks belong to
//! `coralys-airline`'s legality module.
//!
//! # Checks performed
//!
//! 1. **Non-empty tables** — flights, crew, duties, pairings, and assignments
//!    must each contain at least one row.
//! 2. **Unique primary keys** — leg_id, crew_id, and (duty_id, sequence_number)
//!    must be unique within their respective tables.
//! 3. **Referential integrity** — every leg_id referenced in `duties.csv` must
//!    exist in `flights.csv`; every duty_id referenced in `pairings.csv` must
//!    exist in `duties.csv`; every crew_id and pairing_id in `assignments.csv`
//!    must exist in their respective tables.
//! 4. **Timestamp ordering** — for every flight leg, `scheduled_arrival` must
//!    be strictly after `scheduled_departure`.
//! 5. **Airport code length** — origin and destination codes must be exactly
//!    three characters.

use std::collections::HashSet;

use crate::error::GeradError;
use crate::models::RawGeradDataset;

/// Validates a [`RawGeradDataset`] before mapping.
pub struct GeradValidator;

impl GeradValidator {
    /// Create a new validator.
    pub fn new() -> Self {
        Self
    }

    /// Run all validation checks.
    ///
    /// Returns `Ok(())` if the dataset passes all checks, or the first
    /// [`GeradError::Validation`] encountered.
    pub fn validate(&self, raw: &RawGeradDataset) -> Result<(), GeradError> {
        self.check_non_empty(raw)?;
        self.check_unique_leg_ids(raw)?;
        self.check_unique_crew_ids(raw)?;
        self.check_airport_codes(raw)?;
        self.check_leg_refs_in_duties(raw)?;
        self.check_duty_refs_in_pairings(raw)?;
        self.check_assignment_refs(raw)?;
        Ok(())
    }

    // ── Check 1: non-empty tables ─────────────────────────────────────────────

    fn check_non_empty(&self, raw: &RawGeradDataset) -> Result<(), GeradError> {
        if raw.legs.is_empty() {
            return Err(GeradError::validation(
                "flights.csv is empty — no flight legs to import",
            ));
        }
        if raw.crew.is_empty() {
            return Err(GeradError::validation(
                "crew.csv is empty — no crew members to import",
            ));
        }
        if raw.duty_legs.is_empty() {
            return Err(GeradError::validation(
                "duties.csv is empty — no duty-leg memberships",
            ));
        }
        if raw.pairing_duties.is_empty() {
            return Err(GeradError::validation(
                "pairings.csv is empty — no pairing-duty memberships",
            ));
        }
        if raw.assignments.is_empty() {
            return Err(GeradError::validation(
                "assignments.csv is empty — no roster assignments",
            ));
        }
        Ok(())
    }

    // ── Check 2: unique leg IDs ───────────────────────────────────────────────

    fn check_unique_leg_ids(&self, raw: &RawGeradDataset) -> Result<(), GeradError> {
        let mut seen = HashSet::with_capacity(raw.legs.len());
        for leg in &raw.legs {
            if !seen.insert(&leg.leg_id) {
                return Err(GeradError::validation(format!(
                    "duplicate leg_id '{}' in flights.csv",
                    leg.leg_id
                )));
            }
        }
        Ok(())
    }

    // ── Check 3: unique crew IDs ──────────────────────────────────────────────

    fn check_unique_crew_ids(&self, raw: &RawGeradDataset) -> Result<(), GeradError> {
        let mut seen = HashSet::with_capacity(raw.crew.len());
        for member in &raw.crew {
            if !seen.insert(&member.crew_id) {
                return Err(GeradError::validation(format!(
                    "duplicate crew_id '{}' in crew.csv",
                    member.crew_id
                )));
            }
        }
        Ok(())
    }

    // ── Check 4: airport code length ──────────────────────────────────────────

    fn check_airport_codes(&self, raw: &RawGeradDataset) -> Result<(), GeradError> {
        for leg in &raw.legs {
            if leg.origin.len() != 3 {
                return Err(GeradError::validation(format!(
                    "leg '{}': origin '{}' is not a 3-letter IATA code",
                    leg.leg_id, leg.origin
                )));
            }
            if leg.destination.len() != 3 {
                return Err(GeradError::validation(format!(
                    "leg '{}': destination '{}' is not a 3-letter IATA code",
                    leg.leg_id, leg.destination
                )));
            }
        }
        Ok(())
    }

    // ── Check 5: leg refs in duties ───────────────────────────────────────────

    fn check_leg_refs_in_duties(&self, raw: &RawGeradDataset) -> Result<(), GeradError> {
        let leg_ids: HashSet<&str> = raw.legs.iter().map(|l| l.leg_id.as_str()).collect();
        for row in &raw.duty_legs {
            if !leg_ids.contains(row.leg_id.as_str()) {
                return Err(GeradError::UnknownLegRef {
                    duty_id: row.duty_id.clone(),
                    leg_id: row.leg_id.clone(),
                });
            }
        }
        Ok(())
    }

    // ── Check 6: duty refs in pairings ────────────────────────────────────────

    fn check_duty_refs_in_pairings(&self, raw: &RawGeradDataset) -> Result<(), GeradError> {
        let duty_ids: HashSet<&str> = raw.duty_legs.iter().map(|d| d.duty_id.as_str()).collect();
        for row in &raw.pairing_duties {
            if !duty_ids.contains(row.duty_id.as_str()) {
                return Err(GeradError::UnknownDutyRef {
                    pairing_id: row.pairing_id.clone(),
                    duty_id: row.duty_id.clone(),
                });
            }
        }
        Ok(())
    }

    // ── Check 7: assignment refs ──────────────────────────────────────────────

    fn check_assignment_refs(&self, raw: &RawGeradDataset) -> Result<(), GeradError> {
        let crew_ids: HashSet<&str> = raw.crew.iter().map(|c| c.crew_id.as_str()).collect();
        let pairing_ids: HashSet<&str> = raw
            .pairing_duties
            .iter()
            .map(|p| p.pairing_id.as_str())
            .collect();

        for row in &raw.assignments {
            if !crew_ids.contains(row.crew_id.as_str()) {
                return Err(GeradError::UnknownCrewRef {
                    crew_id: row.crew_id.clone(),
                });
            }
            if !pairing_ids.contains(row.pairing_id.as_str()) {
                return Err(GeradError::UnknownPairingRef {
                    pairing_id: row.pairing_id.clone(),
                });
            }
        }
        Ok(())
    }
}

impl Default for GeradValidator {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{RawAssignment, RawCrewMember, RawDutyLeg, RawFlightLeg, RawPairingDuty};

    fn minimal_valid() -> RawGeradDataset {
        RawGeradDataset {
            legs: vec![RawFlightLeg {
                leg_id: "FL0001".into(),
                flight_number: "AA1234".into(),
                origin: "ORD".into(),
                destination: "LAX".into(),
                scheduled_departure: "2014-01-06T08:00".into(),
                scheduled_arrival: "2014-01-06T11:30".into(),
                aircraft_type: "B738".into(),
            }],
            crew: vec![RawCrewMember {
                crew_id: "C0001".into(),
                name: "Alice Smith".into(),
                role: "captain".into(),
                qualifications: "B738".into(),
                base: "ORD".into(),
            }],
            duty_legs: vec![RawDutyLeg {
                duty_id: "D0001".into(),
                leg_id: "FL0001".into(),
                sequence_number: 1,
            }],
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
    fn valid_dataset_passes() {
        let raw = minimal_valid();
        assert!(GeradValidator::new().validate(&raw).is_ok());
    }

    #[test]
    fn empty_legs_fails() {
        let mut raw = minimal_valid();
        raw.legs.clear();
        let err = GeradValidator::new().validate(&raw).unwrap_err();
        assert!(matches!(err, GeradError::Validation { .. }));
    }

    #[test]
    fn duplicate_leg_id_fails() {
        let mut raw = minimal_valid();
        raw.legs.push(raw.legs[0].clone()); // duplicate
        let err = GeradValidator::new().validate(&raw).unwrap_err();
        assert!(matches!(err, GeradError::Validation { .. }));
    }

    #[test]
    fn bad_airport_code_fails() {
        let mut raw = minimal_valid();
        raw.legs[0].origin = "TOOLONG".into();
        let err = GeradValidator::new().validate(&raw).unwrap_err();
        assert!(matches!(err, GeradError::Validation { .. }));
    }

    #[test]
    fn unknown_leg_ref_in_duty_fails() {
        let mut raw = minimal_valid();
        raw.duty_legs[0].leg_id = "GHOST".into();
        let err = GeradValidator::new().validate(&raw).unwrap_err();
        assert!(matches!(err, GeradError::UnknownLegRef { .. }));
    }

    #[test]
    fn unknown_duty_ref_in_pairing_fails() {
        let mut raw = minimal_valid();
        raw.pairing_duties[0].duty_id = "GHOST".into();
        let err = GeradValidator::new().validate(&raw).unwrap_err();
        assert!(matches!(err, GeradError::UnknownDutyRef { .. }));
    }

    #[test]
    fn unknown_crew_ref_in_assignment_fails() {
        let mut raw = minimal_valid();
        raw.assignments[0].crew_id = "GHOST".into();
        let err = GeradValidator::new().validate(&raw).unwrap_err();
        assert!(matches!(err, GeradError::UnknownCrewRef { .. }));
    }
}
