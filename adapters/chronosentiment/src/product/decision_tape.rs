//! DecisionTape Event Sequence Ledger (RP-600 / RP-601 Substrate)
//!
//! Represents a reproducible, immutable sequence of DecisionCard View Models.

use serde::{Deserialize, Serialize};
use super::decision_card::DecisionCardViewModel;

/// Immutable DecisionTape Ledger Struct
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecisionTape {
    pub tape_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub universe: Option<String>,
    pub information_cutoff_policy: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_status: Option<String>,
    pub engine_version: String,
    pub interpreter_config: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated_utc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replay_start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replay_end: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cards: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_events: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cards: Option<Vec<DecisionCardViewModel>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<DecisionCardViewModel>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tapes: Option<Vec<DecisionTape>>,
}

impl DecisionTape {
    /// Creates a new DecisionTape ledger instance for live container cards
    pub fn new(
        tape_id: String,
        symbol: Option<String>,
        universe: Option<String>,
        policy: String,
        domain_status: Option<String>,
        engine_version: String,
        interpreter_config: String,
        cards: Vec<DecisionCardViewModel>,
    ) -> Self {
        let now_str = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let total_cards = cards.len();
        Self {
            tape_id,
            symbol,
            universe,
            information_cutoff_policy: policy,
            domain_status,
            engine_version,
            interpreter_config,
            last_updated_utc: Some(now_str),
            replay_start: None,
            replay_end: None,
            total_cards: Some(total_cards),
            total_events: None,
            cards: Some(cards),
            events: None,
            tapes: None,
        }
    }

    /// Creates a replay event tape for a specific symbol
    pub fn create_replay_tape(
        symbol: String,
        policy: String,
        engine_version: String,
        interpreter_config: String,
        events: Vec<DecisionCardViewModel>,
    ) -> Self {
        let now_str = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        let replay_start = events.first().map(|e| e.datetime.clone());
        let replay_end = events.last().map(|e| e.datetime.clone());
        let total_events = events.len();
        Self {
            tape_id: format!("tape_crypto_{}_{}", symbol, now_str),
            symbol: Some(symbol),
            universe: None,
            information_cutoff_policy: policy,
            domain_status: Some("LIVE — EXPERIMENTAL / UNVALIDATED FOR CRYPTO".into()),
            engine_version,
            interpreter_config,
            last_updated_utc: None,
            replay_start,
            replay_end,
            total_cards: None,
            total_events: Some(total_events),
            cards: None,
            events: Some(events),
            tapes: None,
        }
    }
}
