//! # coralys-gerad
//!
//! GERAD G-2014-22 dataset adapter for the Coralys airline crew scheduling
//! platform.
//!
//! This crate translates the GERAD G-2014-22 benchmark dataset ("Airline crew
//! scheduling: Models, algorithms, and data sets", Desaulniers et al., 2014)
//! into the Coralys airline domain model.  After import, the rest of the
//! system has no knowledge that the data originated from the GERAD benchmark.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │  GeradImporter  (public API — adapters/gerad/src/importer.rs)   │
//! │                                                                 │
//! │   1. GeradParser    — CSV files → RawGeradDataset               │
//! │   2. GeradValidator — referential integrity + semantic checks   │
//! │   3. GeradMapper    — RawGeradDataset → Roster (domain model)   │
//! └─────────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//!              coralys-airline domain model
//!         (FlightLeg, Duty, Pairing, CrewMember, Roster)
//! ```
//!
//! ## Usage
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
//!
//! ## Dataset directory layout
//!
//! ```text
//! <dataset_dir>/
//!   flights.csv      — one row per flight leg
//!   crew.csv         — one row per crew member
//!   duties.csv       — one row per (duty_id, leg_id) membership record
//!   pairings.csv     — one row per (pairing_id, duty_id) membership record
//!   assignments.csv  — one row per (crew_id, pairing_id) roster assignment
//! ```

// ── Internal modules (not part of the public API) ─────────────────────────────

pub(crate) mod models;
pub(crate) mod parser;
pub(crate) mod mapper;
pub(crate) mod validator;

// ── Public API ────────────────────────────────────────────────────────────────

pub mod error;
pub mod importer;

// ── Top-level re-exports ──────────────────────────────────────────────────────

pub use error::GeradError;
pub use importer::GeradImporter;