pub mod architecture;
pub mod champion;
pub mod guidance;
pub mod memory;
pub mod metrics;
pub mod observer;
pub mod policy;
pub mod signal;

pub use architecture::*;
pub use champion::{ChampionLifecycle, ChampionStatus, ChampionTracker, ExitReason};
pub use guidance::EcologyGuidanceTarget;
pub use memory::{EcologyMemory, MeasureValue, ResourceId};
pub use metrics::{
    compute_pearson, compute_spearman, distribution_gini, distribution_variance, rank_array,
};
pub use observer::{EcologyObserver, ExternalObserver};
pub use policy::EcologyPolicy;
pub use signal::EcologySignal;
