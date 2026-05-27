//! Deterministic ingest lineage — frozen substrate, chronology, dedupe, archive.
//! Python orchestration may call this binary; causal persistence belongs here.

pub mod archive;
pub mod dedupe;
pub mod frozen_loader;
pub mod manifest;
pub mod observatory;
pub mod pca;
pub mod persist;
pub mod repair;
pub mod replay;
pub mod telemetry;
pub mod timeline;

pub use frozen_loader::{load_frozen_cohort, FrozenBar, FrozenManifest};
pub use repair::{RepairConfig, RepairStatus};
pub use replay::{run_replay_step, ReplayStepConfig, ReplayStepResult};
pub use timeline::{align_timeline, TimelineAlignment};
