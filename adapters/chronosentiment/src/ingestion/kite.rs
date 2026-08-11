use std::env;
use reqwest::Client;
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::error::Error;
use uuid::Uuid;

use crate::observation::ValidatedObservation;
use crate::validation::ValidationEngine;

pub struct KiteGateway {
    client: Client,
    api_key: String,
    access_token: String,
}

impl KiteGateway {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let api_key = env::var("KITE_API_KEY")
            .map_err(|_| "KITE_API_KEY environment variable is not set")?;
            
        // For Phase 1 we read the generated access token from the python flow
        let token_path = "archive/transient_texts/kite_access_token.txt";
        let access_token = std::fs::read_to_string(token_path)
            .map_err(|_| "Failed to read Kite access token. Run kite_test_auth.py first.")?
            .trim()
            .to_string();

        Ok(Self {
            client: Client::new(),
            api_key,
            access_token,
        })
    }

    /// Fetches historical data for an instrument token over a specific date range,
    /// normalises it, and passes it through the Validation Engine to return a Canonical ValidatedObservation.
    pub async fn fetch_historical_candles(
        &self,
        instrument_master_id: Uuid,
        kite_instrument_token: &str,
        from_date: DateTime<Utc>,
        to_date: DateTime<Utc>,
        interval: &str, // e.g. "minute", "day"
    ) -> Result<ValidatedObservation, Box<dyn Error>> {
        
        let url = format!(
            "https://api.kite.trade/instruments/historical/{}/{}",
            kite_instrument_token, interval
        );

        let from_str = from_date.format("%Y-%m-%d %H:%M:%S").to_string();
        let to_str = to_date.format("%Y-%m-%d %H:%M:%S").to_string();

        let response = self.client.get(&url)
            .header("X-Kite-Version", "3")
            .header("Authorization", format!("token {}:{}", self.api_key, self.access_token))
            .query(&[("from", &from_str), ("to", &to_str)])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Kite API error: {}", response.status()).into());
        }

        let raw_payload: Value = response.json().await?;
        
        // Ensure Kite responded with "success"
        if raw_payload["status"] != "success" {
            return Err("Kite API returned non-success payload".into());
        }

        // --- Normalization Phase ---
        // Here we map Kite's specific layout (candles array [timestamp, open, high, low, close, volume])
        // into a standardized JSON structure.
        let candles = raw_payload["data"]["candles"].as_array().unwrap_or(&vec![]).clone();
        
        // This acts as the generalized ValidatedObservation structure.
        let normalized_payload = serde_json::json!({
            "interval": interval,
            "candle_count": candles.len(),
            "time_series": candles
        });

        // --- Enrichment and Validation Phase ---
        let canonical_observation = ValidationEngine::enrich_observation(
            instrument_master_id,
            "MarketPrice",
            "Kite",
            raw_payload,
            normalized_payload,
            0.95, // High confidence since it's a direct API from primary broker
            "Complete"
        );

        Ok(canonical_observation)
    }
}
