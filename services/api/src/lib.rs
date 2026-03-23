pub mod simulate;
pub mod ga;
pub mod inspector;
pub mod timeline;
pub mod replay;
pub mod events;
pub mod certify;

pub use simulate::*;
pub use ga::*;
pub use inspector::*;
pub use timeline::*;
pub use replay::*;
pub use events::*;
pub use certify::*;
pub use replay::*;

#[derive(Debug, Clone, serde::Serialize)]
pub struct EventStreamResponse {
    pub events: Vec<chronosentiment_core::SimEvent>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CertificationResponse {
    pub status: String,
    pub hash_1: String,
    pub hash_2: String,
    pub divergence_point: Option<u64>,
    pub fingerprint: Option<DeterminismFingerprint>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DeterminismFingerprint {
    pub engine_version: String,
    pub event_count: usize,
    pub final_hash: String,
    pub config_hash: String,
}

#[derive(Debug, Clone)]
pub enum ApiError {
    InvalidInput(String),
    InternalError(String),
}
