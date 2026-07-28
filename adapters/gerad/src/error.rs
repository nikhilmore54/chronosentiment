//! Unified error type for the GERAD G-2014-22 adapter.
//!
//! All errors produced by the parser, mapper, validator, and importer are
//! collected here so callers only need to handle a single error type.

use thiserror::Error;

/// Every error that can occur during a GERAD import.
#[derive(Debug, Error)]
pub enum GeradError {
    // ── I/O ──────────────────────────────────────────────────────────────────
    /// A required dataset file could not be opened or read.
    #[error("I/O error reading '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    // ── Parse ─────────────────────────────────────────────────────────────────
    /// The CSV reader encountered a malformed record.
    #[error("CSV parse error in '{path}' at record {record}: {source}")]
    CsvParse {
        path: String,
        record: usize,
        #[source]
        source: csv::Error,
    },

    /// A required field was missing or empty in a raw record.
    #[error("missing field '{field}' in {entity} record {record_id}")]
    MissingField {
        entity: &'static str,
        record_id: String,
        field: &'static str,
    },

    /// A field value could not be parsed into the expected type.
    #[error("invalid value for field '{field}' in {entity} record {record_id}: {detail}")]
    InvalidField {
        entity: &'static str,
        record_id: String,
        field: &'static str,
        detail: String,
    },

    /// A timestamp string did not match the expected GERAD format.
    #[error(
        "invalid timestamp '{value}' in {entity} record {record_id}: expected YYYY-MM-DDTHH:MM"
    )]
    InvalidTimestamp {
        entity: &'static str,
        record_id: String,
        value: String,
    },

    // ── Referential integrity ─────────────────────────────────────────────────
    /// A pairing references a duty ID that was not found in the duties table.
    #[error("pairing '{pairing_id}' references unknown duty '{duty_id}'")]
    UnknownDutyRef {
        pairing_id: String,
        duty_id: String,
    },

    /// A duty references a flight leg ID that was not found in the legs table.
    #[error("duty '{duty_id}' references unknown flight leg '{leg_id}'")]
    UnknownLegRef { duty_id: String, leg_id: String },

    /// A roster assignment references a crew member ID that was not found.
    #[error("roster assignment references unknown crew member '{crew_id}'")]
    UnknownCrewRef { crew_id: String },

    /// A roster assignment references a pairing ID that was not found.
    #[error("roster assignment references unknown pairing '{pairing_id}'")]
    UnknownPairingRef { pairing_id: String },

    // ── Domain construction ───────────────────────────────────────────────────
    /// The airline domain model rejected a constructed Duty.
    #[error("domain error constructing duty '{duty_id}': {detail}")]
    DutyConstruction { duty_id: String, detail: String },

    /// The airline domain model rejected a constructed Pairing.
    #[error("domain error constructing pairing '{pairing_id}': {detail}")]
    PairingConstruction { pairing_id: String, detail: String },

    /// The airline domain model rejected the constructed Roster.
    #[error("domain error constructing roster: {detail}")]
    RosterConstruction { detail: String },

    // ── Validation ────────────────────────────────────────────────────────────
    /// The mapped dataset failed a semantic validation check.
    #[error("validation failed: {message}")]
    Validation { message: String },
}

impl GeradError {
    /// Convenience constructor for [`GeradError::Io`].
    pub fn io(path: impl Into<String>, source: std::io::Error) -> Self {
        Self::Io { path: path.into(), source }
    }

    /// Convenience constructor for [`GeradError::Validation`].
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation { message: message.into() }
    }
}
