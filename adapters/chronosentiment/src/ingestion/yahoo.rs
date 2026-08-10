use std::error::Error;
use chrono::{DateTime, Utc, TimeZone};
use serde_json::Value;

pub struct YahooFinanceWorker {
    client: reqwest::Client,
}

impl YahooFinanceWorker {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Fetches historical OHLCV data for a ticker and returns a list of Observations.
    pub async fn fetch_historical_prices(&self, ticker: &str) -> Result<Vec<Value>, Box<dyn Error>> {
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?range=1mo&interval=1d",
            ticker
        );

        let response = self.client.get(&url).send().await?.json::<Value>().await?;

        let result = &response["chart"]["result"][0];
        if result.is_null() {
            return Err("No data returned from Yahoo Finance".into());
        }

        let timestamps = result["timestamp"].as_array().ok_or("Missing timestamp array")?;
        let quote = &result["indicators"]["quote"][0];

        let opens = quote["open"].as_array().ok_or("Missing opens")?;
        let highs = quote["high"].as_array().ok_or("Missing highs")?;
        let lows = quote["low"].as_array().ok_or("Missing lows")?;
        let closes = quote["close"].as_array().ok_or("Missing closes")?;
        let volumes = quote["volume"].as_array().ok_or("Missing volumes")?;

        let mut observations = Vec::new();

        for i in 0..timestamps.len() {
            let ts = timestamps[i].as_i64().unwrap_or(0);
            let observed_at = Utc.timestamp_opt(ts, 0).unwrap();

            let open = opens[i].as_f64().unwrap_or(0.0);
            let high = highs[i].as_f64().unwrap_or(0.0);
            let low = lows[i].as_f64().unwrap_or(0.0);
            let close = closes[i].as_f64().unwrap_or(0.0);
            let volume = volumes[i].as_f64().unwrap_or(0.0);

            let payload = serde_json::json!({
                "open": open,
                "high": high,
                "low": low,
                "close": close,
                "volume": volume
            });

            let observation = serde_json::json!({
                "observation_type": "PriceAction",
                "observed_at": observed_at.to_rfc3339(),
                "symbol": ticker,
                "numerical_value": close,
                "payload": payload,
                "source_name": "YahooFinance",
                "coverage": "Complete"
            });

            observations.push(observation);
        }

        Ok(observations)
    }
}
