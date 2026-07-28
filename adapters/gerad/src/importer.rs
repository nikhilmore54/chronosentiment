//! Public import API for the GERAD G-2014-22 adapter.
//!
//! [`GeradImporter`] is the single entry point for loading a GERAD dataset.
//! It orchestrates the three internal stages:
//!
//! 1. **Parse** — [`GeradParser`] reads the five CSV files into raw structs.
//! 2. **Validate** — [`GeradValidator`] checks referential integrity and
//!    semantic constraints before any domain objects are constructed.
//! 3. **Map** — [`GeradMapper`] translates the raw structs into the Coralys
//!    airline domain model.
//!
//! After [`GeradImporter::load`] returns, the caller holds a [`Roster`] that
//! is indistinguishable from one built directly against the domain model.  No
//! GERAD-specific types leak past this boundary.
//!
//! # Example
//!
//! ```no_run
//! use coralys_gerad::GeradImporter;
//!
//! let roster = GeradImporter::new("path/to/gerad/dataset")
//!     .load()
//!     .expect("failed to import GERAD dataset");
//!
//! println!("Imported {} flight legs", roster.leg_count());
//! println!("Imported {} crew members", roster.crew_member_count());
//! ```

use std::path::PathBuf;

use coralys_airline::domain::Roster;

use crate::error::GeradError;
use crate::mapper::GeradMapper;
use crate::parser::GeradParser;
use crate::validator::GeradValidator;

/// Loads a GERAD G-2014-22 dataset directory and returns a Coralys [`Roster`].
pub struct GeradImporter {
    dataset_dir: PathBuf,
}

impl GeradImporter {
    /// Create a new importer rooted at `dataset_dir`.
    ///
    /// The directory must contain the five CSV files described in
    /// [`crate::models`].
    pub fn new(dataset_dir: impl Into<PathBuf>) -> Self {
        Self { dataset_dir: dataset_dir.into() }
    }

    /// Parse, validate, and map the dataset.
    ///
    /// # Errors
    /// Returns [`GeradError`] if any stage fails.  The error carries enough
    /// context (file name, record index, field name) to pinpoint the problem.
    pub fn load(&self) -> Result<Roster, GeradError> {
        // Stage 1 — parse CSV files into raw structs.
        let raw = GeradParser::new(&self.dataset_dir).parse()?;

        // Stage 2 — validate referential integrity and semantic constraints.
        GeradValidator::new().validate(&raw)?;

        // Stage 3 — translate raw structs into the domain model.
        GeradMapper::new().map(&raw)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::Path;
    use tempfile::TempDir;

    fn write_file(dir: &Path, name: &str, content: &str) {
        let mut f = std::fs::File::create(dir.join(name)).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    fn write_full_dataset(dir: &Path) {
        write_file(
            dir,
            "flights.csv",
            "leg_id,flight_number,origin,destination,scheduled_departure,scheduled_arrival,aircraft_type\n\
             FL0001,AA1234,ORD,LAX,2014-01-06T08:00,2014-01-06T11:30,B738\n\
             FL0002,AA1235,LAX,ORD,2014-01-06T14:00,2014-01-06T19:30,B738\n",
        );
        write_file(
            dir,
            "crew.csv",
            "crew_id,name,role,qualifications,base\n\
             C0001,Alice Smith,captain,B738,ORD\n\
             C0002,Bob Jones,first_officer,B738,ORD\n",
        );
        write_file(
            dir,
            "duties.csv",
            "duty_id,leg_id,sequence_number\n\
             D0001,FL0001,1\n\
             D0001,FL0002,2\n",
        );
        write_file(
            dir,
            "pairings.csv",
            "pairing_id,duty_id,sequence_number,base\n\
             P0001,D0001,1,ORD\n",
        );
        write_file(
            dir,
            "assignments.csv",
            "crew_id,pairing_id\n\
             C0001,P0001\n\
             C0002,P0001\n",
        );
    }

    #[test]
    fn end_to_end_import() {
        let tmp = TempDir::new().unwrap();
        write_full_dataset(tmp.path());

        let roster = GeradImporter::new(tmp.path()).load().unwrap();

        assert_eq!(roster.leg_count(), 2, "expected 2 flight legs");
        assert_eq!(roster.crew_member_count(), 2, "expected 2 crew members");
        assert_eq!(roster.crew_count(), 2, "expected 2 rotations");
    }

    #[test]
    fn missing_file_returns_io_error() {
        let tmp = TempDir::new().unwrap();
        // Write only flights.csv — the rest are missing.
        write_file(
            tmp.path(),
            "flights.csv",
            "leg_id,flight_number,origin,destination,scheduled_departure,scheduled_arrival,aircraft_type\n",
        );

        let err = GeradImporter::new(tmp.path()).load().unwrap_err();
        assert!(matches!(err, GeradError::Io { .. }), "expected Io error, got {err:?}");
    }
}