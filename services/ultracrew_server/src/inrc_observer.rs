// InrcScoreComponents, score_inrc_official, to_inrc_genome, and OBSERVER_ID have been
// moved to the UltraCrew Solution Adapter (adapters/ultracrew/src/inrc/observer.rs).
// Re-exported here for backward compatibility with research binaries and existing code.
pub use ultracrew::inrc::observer::{
    InrcScoreComponents, OBSERVER_ID, score_inrc_official, to_inrc_genome,
};
