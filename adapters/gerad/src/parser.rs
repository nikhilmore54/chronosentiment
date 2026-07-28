//! GERAD G-2014-22 CSV parser.
//!
//! [`GeradParser`] reads the five CSV files that make up a GERAD dataset
//! directory and returns a [`RawGeradDataset`].  It performs **no semantic
//! validation** — that is the responsibility of [`crate::validator`].  It only
//! ensures that every row can be deserialised into the expected raw type.
//!
//! # Expected directory layout
//!
//! ```text
//! <dataset_dir>/
//!   flights.csv
//!   crew.csv
//!   duties.csv
//!   pairings.csv
//!   assignments.csv
//! ```

use std::path::PathBuf;

use crate::error::GeradError;
use crate::models::{
    RawAssignment, RawCrewMember, RawDutyLeg, RawFlightLeg, RawGeradDataset, RawPairingDuty,
};

/// Reads a GERAD dataset directory and returns the raw parsed records.
pub struct GeradParser {
    dataset_dir: PathBuf,
}

impl GeradParser {
    /// Create a new parser rooted at `dataset_dir`.
    pub fn new(dataset_dir: impl Into<PathBuf>) -> Self {
        Self { dataset_dir: dataset_dir.into() }
    }

    /// Parse all five CSV files and return the collected raw dataset.
    ///
    /// # Errors
    /// Returns [`GeradError`] if any file cannot be opened, or if any row
    /// cannot be deserialised into the expected type.
    pub fn parse(&self) -> Result<RawGeradDataset, GeradError> {
        Ok(RawGeradDataset {
            legs: self.parse_csv::<RawFlightLeg>("flights.csv")?,
            crew: self.parse_csv::<RawCrewMember>("crew.csv")?,
            duty_legs: self.parse_csv::<RawDutyLeg>("duties.csv")?,
            pairing_duties: self.parse_csv::<RawPairingDuty>("pairings.csv")?,
            assignments: self.parse_csv::<RawAssignment>("assignments.csv")?,
        })
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    /// Parse a single CSV file into a `Vec<T>`.
    ///
    /// The file must have a header row whose column names match the field names
    /// of `T` (serde `rename` attributes on `T` are respected).
    fn parse_csv<T>(&self, filename: &str) -> Result<Vec<T>, GeradError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let path = self.dataset_dir.join(filename);
        let path_str = path.display().to_string();

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .trim(csv::Trim::All)
            .from_path(&path)
            .map_err(|e| GeradError::io(path_str.clone(), io_from_csv(e)))?;

        let mut records: Vec<T> = Vec::new();
        for (idx, result) in reader.deserialize().enumerate() {
            let record: T = result.map_err(|e| GeradError::CsvParse {
                path: path_str.clone(),
                record: idx + 1,
                source: e,
            })?;
            records.push(record);
        }

        Ok(records)
    }
}

/// Convert a [`csv::Error`] that wraps an I/O error into a plain
/// [`std::io::Error`] so we can store it in [`GeradError::Io`].
///
/// If the CSV error is not an I/O error we synthesise one with the display
/// string so the caller always gets a useful message.
fn io_from_csv(e: csv::Error) -> std::io::Error {
    match e.into_kind() {
        csv::ErrorKind::Io(io) => io,
        other => std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("csv open error: {other:?}"),
        ),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::Path;
    use tempfile::TempDir;

    /// Write a minimal valid dataset to a temp directory and parse it.
    fn write_minimal_dataset(dir: &Path) {
        write_file(
            dir,
            "flights.csv",
            "leg_id,flight_number,origin,destination,scheduled_departure,scheduled_arrival,aircraft_type\n\
             FL0001,AA1234,ORD,LAX,2014-01-06T08:00,2014-01-06T11:30,B738\n",
        );
        write_file(
            dir,
            "crew.csv",
            "crew_id,name,role,qualifications,base\n\
             C0001,Alice Smith,captain,B738,ORD\n",
        );
        write_file(
            dir,
            "duties.csv",
            "duty_id,leg_id,sequence_number\n\
             D0001,FL0001,1\n",
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
             C0001,P0001\n",
        );
    }

    fn write_file(dir: &Path, name: &str, content: &str) {
        let mut f = std::fs::File::create(dir.join(name)).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn parses_minimal_dataset() {
        let tmp = TempDir::new().unwrap();
        write_minimal_dataset(tmp.path());

        let parser = GeradParser::new(tmp.path());
        let dataset = parser.parse().unwrap();

        assert_eq!(dataset.legs.len(), 1);
        assert_eq!(dataset.crew.len(), 1);
        assert_eq!(dataset.duty_legs.len(), 1);
        assert_eq!(dataset.pairing_duties.len(), 1);
        assert_eq!(dataset.assignments.len(), 1);

        let leg = &dataset.legs[0];
        assert_eq!(leg.leg_id, "FL0001");
        assert_eq!(leg.origin, "ORD");
        assert_eq!(leg.destination, "LAX");
        assert_eq!(leg.aircraft_type, "B738");
    }

    #[test]
    fn error_on_missing_file() {
        let tmp = TempDir::new().unwrap();
        // Only write flights.csv — crew.csv is missing.
        write_file(
            tmp.path(),
            "flights.csv",
            "leg_id,flight_number,origin,destination,scheduled_departure,scheduled_arrival,aircraft_type\n",
        );

        let parser = GeradParser::new(tmp.path());
        let err = parser.parse().unwrap_err();
        assert!(matches!(err, GeradError::Io { .. }), "expected Io error, got {err:?}");
    }
}