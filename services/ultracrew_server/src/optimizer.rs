// ScheduleGenome, UltraCrewEvaluator, UltraCrewMutator, and AssignmentSlot have been
// moved to the UltraCrew Solution Adapter (adapters/ultracrew/src/inrc/schedule_optimizer.rs).
// Re-exported here for backward compatibility with research binaries and existing code.
pub use ultracrew::inrc::schedule_optimizer::{
    AssignmentSlot,
    ScheduleGenome,
    UltraCrewEvaluator,
    UltraCrewMutator,
};
