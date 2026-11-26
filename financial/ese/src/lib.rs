//! Deterministic ingest lineage — frozen substrate, chronology, dedupe, archive.
//! Causal persistence and live-path verification belong here (not in Python).

pub mod archive;
pub mod dedupe;
pub mod fixtures;
pub mod frozen_loader;
pub mod governor;
pub mod gzip_jsonl;
pub mod manifest;
pub mod observatory;
pub mod paths;
pub mod pca;
pub mod persist;
pub mod repair;
pub mod replay;
pub mod telemetry;
pub mod timeline;
pub mod verify;

pub use frozen_loader::{load_frozen_cohort, FrozenBar, FrozenManifest};
pub use repair::{RepairConfig, RepairStatus};
pub use replay::{run_replay_step, ReplayStepConfig, ReplayStepResult};
pub use timeline::{align_timeline, TimelineAlignment};
