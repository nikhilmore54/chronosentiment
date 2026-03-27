pub mod simulate;
pub mod ga;
pub mod inspector;
pub mod timeline;
pub mod replay;
pub mod events;
pub mod certify;
pub mod market_adapter;
pub mod market_data_simulate;

pub use simulate::*;
pub use ga::*;
pub use inspector::{TradeInspectorResponse, MinimalEvent, build_trade_inspector, handle_inspect, to_minimal_event};
pub use timeline::*;
pub use replay::*;
pub use events::*;
pub use certify::*;
pub use market_adapter::*;
pub use market_data_simulate::*;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SimulateOutputDto {
    pub pnl: i64,
    pub trade_count: u64,
    pub events: Vec<MinimalEvent>,
    pub state_hash: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EventsResponseDto {
    pub events: Vec<MinimalEvent>,
}

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
