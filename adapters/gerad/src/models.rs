//! GERAD G-2014-22 native schema types.
//!
//! These structs mirror the raw CSV columns published in the GERAD G-2014-22
//! technical report ("Airline crew scheduling: Models, algorithms, and data
//! sets", Desaulniers et al., 2014).  They are **schema-faithful** — field
//! names and types match the dataset exactly so that the parser can deserialise
//! directly into them.
//!
//! Nothing outside this module and [`crate::parser`] should ever reference
//! these types.  The mapper translates them into the Coralys airline domain
//! model and the rest of the system is unaware of the GERAD origin.
//!
//! # File layout expected on disk
//!
//! ```text
//! <dataset_dir>/
//!   flights.csv      — one row per flight leg
//!   crew.csv         — one row per crew member
//!   duties.csv       — one row per (duty_id, leg_id) membership record
//!   pairings.csv     — one row per (pairing_id, duty_id) membership record
//!   assignments.csv  — one row per (crew_id, pairing_id) roster assignment
//! ```

use serde::Deserialize;

// ── Flight legs ───────────────────────────────────────────────────────────────

/// A single flight leg as it appears in `flights.csv`.
///
/// Timestamps use the format `YYYY-MM-DDTHH:MM` (local station time in the
/// original dataset; the mapper converts to UTC using the station offset table
/// or treats them as UTC when no offset is available).
#[derive(Debug, Clone, Deserialize)]
pub struct RawFlightLeg {
    /// Unique leg identifier within the dataset, e.g. `"FL0001"`.
    pub leg_id: String,
    /// Marketed flight number, e.g. `"AA1234"`.
    pub flight_number: String,
    /// IATA origin airport code, e.g. `"ORD"`.
    pub origin: String,
    /// IATA destination airport code, e.g. `"LAX"`.
    pub destination: String,
    /// Scheduled departure timestamp (`YYYY-MM-DDTHH:MM`).
    pub scheduled_departure: String,
    /// Scheduled arrival timestamp (`YYYY-MM-DDTHH:MM`).
    pub scheduled_arrival: String,
    /// ICAO aircraft type designator, e.g. `"B738"`.
    pub aircraft_type: String,
}

// ── Crew members ──────────────────────────────────────────────────────────────

/// A crew member record as it appears in `crew.csv`.
#[derive(Debug, Clone, Deserialize)]
pub struct RawCrewMember {
    /// Unique crew identifier, e.g. `"C0042"`.
    pub crew_id: String,
    /// Full display name.
    pub name: String,
    /// Role string as used in the dataset: `"captain"`, `"first_officer"`,
    /// `"cabin_crew_senior"`, `"cabin_crew"`, or `"relief_pilot"`.
    pub role: String,
    /// Comma-separated list of ICAO aircraft type designators for which this
    /// crew member holds a type rating, e.g. `"B738,A320"`.
    pub qualifications: String,
    /// Home base IATA airport code.
    pub base: String,
}

// ── Duty membership ───────────────────────────────────────────────────────────

/// One row of `duties.csv` — a (duty, leg) membership record.
///
/// A duty is reconstructed by collecting all rows with the same `duty_id` and
/// ordering them by `sequence_number`.
#[derive(Debug, Clone, Deserialize)]
pub struct RawDutyLeg {
    /// Duty identifier, e.g. `"D0001"`.
    pub duty_id: String,
    /// Flight leg identifier (foreign key into `flights.csv`).
    pub leg_id: String,
    /// 1-based position of this leg within the duty.
    pub sequence_number: u32,
}

// ── Pairing membership ────────────────────────────────────────────────────────

/// One row of `pairings.csv` — a (pairing, duty) membership record.
///
/// A pairing is reconstructed by collecting all rows with the same `pairing_id`
/// and ordering them by `sequence_number`.
#[derive(Debug, Clone, Deserialize)]
pub struct RawPairingDuty {
    /// Pairing identifier, e.g. `"P0001"`.
    pub pairing_id: String,
    /// Duty identifier (foreign key into `duties.csv`).
    pub duty_id: String,
    /// 1-based position of this duty within the pairing.
    pub sequence_number: u32,
    /// IATA base airport where this pairing starts and ends.
    pub base: String,
}

// ── Roster assignments ────────────────────────────────────────────────────────

/// One row of `assignments.csv` — a (crew member, pairing) assignment.
///
/// The full roster is reconstructed by grouping all assignments by `crew_id`.
#[derive(Debug, Clone, Deserialize)]
pub struct RawAssignment {
    /// Crew member identifier (foreign key into `crew.csv`).
    pub crew_id: String,
    /// Pairing identifier (foreign key into `pairings.csv`).
    pub pairing_id: String,
}

// ── Collected raw dataset ─────────────────────────────────────────────────────

/// The complete raw GERAD dataset as parsed from disk.
///
/// This is the output of [`crate::parser::GeradParser`] and the input to
/// [`crate::mapper::GeradMapper`].  It is an internal type — callers use
/// [`crate::importer::GeradImporter`] which returns the mapped domain model.
#[derive(Debug, Default)]
pub struct RawGeradDataset {
    /// All flight legs from `flights.csv`.
    pub legs: Vec<RawFlightLeg>,
    /// All crew member records from `crew.csv`.
    pub crew: Vec<RawCrewMember>,
    /// All duty-leg membership rows from `duties.csv`.
    pub duty_legs: Vec<RawDutyLeg>,
    /// All pairing-duty membership rows from `pairings.csv`.
    pub pairing_duties: Vec<RawPairingDuty>,
    /// All roster assignment rows from `assignments.csv`.
    pub assignments: Vec<RawAssignment>,
}
