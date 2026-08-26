use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// An Instrument represents a tradable asset, economic indicator, or entity.
/// It is the core anchor for observations, abstracting away provider-specific symbols.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instrument {
    /// Globally unique identifier for this instrument
    pub id: Uuid,

    /// The exchange this instrument trades on (e.g., 'NSE', 'BSE', 'NYSE', or 'FRED' for macro)
    pub exchange: String,

    /// A human-readable display symbol (e.g., 'RELIANCE', 'AAPL', 'FEDFUNDS')
    pub display_symbol: String,

    /// Provider-specific identifiers (e.g., {"kite_token": "738561", "isin": "INE002A01018"})
    pub provider_ids: HashMap<String, String>,

    /// When this instrument was first recorded
    pub created_at: DateTime<Utc>,
}

impl Instrument {
    pub fn new(exchange: String, display_symbol: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            exchange,
            display_symbol,
            provider_ids: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    /// Adds a provider-specific identifier to this instrument
    pub fn add_provider_id(&mut self, provider_key: &str, provider_value: &str) {
        self.provider_ids
            .insert(provider_key.to_string(), provider_value.to_string());
    }
}
