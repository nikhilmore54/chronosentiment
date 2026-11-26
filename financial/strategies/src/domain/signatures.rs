use super::scoring::ConvictionOutcome;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlphaPorosity {
    Dense,
    Porous,
    Sparse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalType {
    EntryLong,
    EntryShort,
    ExitLong,
    ExitShort,
    Neutral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalSource {
    Organic,
    Synthetic,
    Ensemble,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalSignature {
    pub archetype: u8,
    pub regime: u8,
    pub momentum: i8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlphaConsensus {
    pub average_confidence: f64,
    pub dominant_direction: i8,
    pub signal_count: usize,
    pub porosity: AlphaPorosity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalAlpha {
    pub ts: usize,
    pub price: f64,
    pub archetype: u8,
    pub direction: i8,
    pub strength: f64,
    pub source: SignalSource,
    pub conviction: ConvictionOutcome,
    pub is_probe: bool,
}
