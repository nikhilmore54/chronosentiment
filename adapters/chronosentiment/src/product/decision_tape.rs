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
    pub total_cards: usize,
    pub cards: Vec<DecisionCardViewModel>,
}

impl DecisionTape {
    /// Creates a new DecisionTape ledger instance
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
            total_cards,
            cards,
        }
    }
}
