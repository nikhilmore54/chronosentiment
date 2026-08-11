use std::error::Error;
use chrono::{DateTime, Utc, TimeZone};
use serde_json::Value;
use async_trait::async_trait;
use crate::instrument::Instrument;
use crate::ingestion::provider::{MarketDataProvider, ValidatedObservationTranslator, TimeRange};
use crate::observation::RawObservation;

pub struct YahooHistoricalBar {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub adj_close: f64,
    pub volume: f64,
}

pub struct YahooProvider {
    client: reqwest::Client,
}

impl YahooProvider {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36")
            .build()
            .unwrap();
            
        Self {
            client,
        }
    }
}

#[async_trait]
impl MarketDataProvider for YahooProvider {
    type RawRecord = YahooHistoricalBar;

    async fn fetch_historical(
        &self,
        instrument: &Instrument,
        _range: TimeRange,
    ) -> Result<Vec<Self::RawRecord>, Box<dyn Error>> {
        let ticker = instrument.provider_ids.get("yahoo")
            .ok_or("Instrument missing 'yahoo' identity")?;
            
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?range=5y&interval=1d",
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

        let adj_closes = result["indicators"]["adjclose"][0]["adjclose"].as_array().ok_or("Missing adjclose")?;

        let mut bars = Vec::new();

        for i in 0..timestamps.len() {
            let ts = timestamps[i].as_i64().unwrap_or(0);
            let open = opens[i].as_f64().unwrap_or(0.0);
            let high = highs[i].as_f64().unwrap_or(0.0);
            let low = lows[i].as_f64().unwrap_or(0.0);
            let close = closes[i].as_f64().unwrap_or(0.0);
            let adj_close = adj_closes[i].as_f64().unwrap_or(0.0);
            let volume = volumes[i].as_f64().unwrap_or(0.0);

            bars.push(YahooHistoricalBar {
                timestamp: ts,
                open,
                high,
                low,
                close,
                adj_close,
                volume,
            });
        }

        Ok(bars)
    }
}

pub struct YahooTranslator;

impl ValidatedObservationTranslator<YahooHistoricalBar> for YahooTranslator {
    fn translate(
        &self,
        raw: YahooHistoricalBar,
        _instrument: &Instrument,
    ) -> RawObservation {
        let observed_at = Utc.timestamp_opt(raw.timestamp, 0).unwrap();
        
        let payload = serde_json::json!({
            "open": raw.open,
            "high": raw.high,
            "low": raw.low,
            "unadjusted_close": raw.close,
            "close": raw.adj_close,
            "volume": raw.volume
        });

        RawObservation {
            observation_type: "MarketPrice".to_string(),
            source: "YahooFinance".to_string(),
            source_identifier: Some(raw.timestamp.to_string()),
            observed_at,
            raw_payload: payload.clone(),
            normalized_payload: payload,
        }
    }
}
