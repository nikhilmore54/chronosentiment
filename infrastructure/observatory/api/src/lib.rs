pub mod certify;
pub mod dto;
pub mod errors;
pub mod events;
pub mod inspect_projection;
pub mod inspector;
pub mod market_adapter;
pub mod market_data_simulate;
pub mod replay;
pub mod scenario;
pub mod signatures;
pub mod simulate;
pub mod timeline;

pub use errors::ApiError;

pub use certify::*;
pub use events::*;
pub use inspector::{
    build_trade_inspector, handle_inspect, to_minimal_event, MinimalEvent, TradeInspectorResponse,
};
pub use market_adapter::*;
pub use market_data_simulate::*;
pub use replay::reduce_replay_state;
pub use simulate::*;
pub use timeline::*;

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
