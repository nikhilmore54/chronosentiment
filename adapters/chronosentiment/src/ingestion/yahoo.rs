use std::error::Error;
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Duration, Utc, TimeZone};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use async_trait::async_trait;
use crate::instrument::Instrument;
use crate::ingestion::provider::{MarketDataProvider, ValidatedObservationTranslator, TimeRange};
use crate::observation::RawObservation;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

        // Load any bars already on disk.
        let existing = read_yahoo_cache(ticker)?.unwrap_or_default();

        // Determine the last stored timestamp so we only fetch newer bars.
        let last_stored_ts: Option<i64> = existing.iter().map(|b| b.timestamp).max();

        // Build the fetch URL. When we have existing data we request only the
        // period after the last stored bar; otherwise fall back to the full
        // 5-year range.
        //
        // Yahoo daily bars are timestamped at market-open (UTC). We add 1 second
        // to period1 so we don't re-fetch the last stored bar, and we add 1 day
        // to period2 to ensure today's bar is included when the market has closed.
        let url = match last_stored_ts {
            Some(last_ts) => {
                let period1 = last_ts + 1;
                let period2 = (Utc::now() + Duration::days(1)).timestamp();
                format!(
                    "https://query1.finance.yahoo.com/v8/finance/chart/{}?period1={}&period2={}&interval=1d",
                    ticker, period1, period2
                )
            }
            None => format!(
                "https://query1.finance.yahoo.com/v8/finance/chart/{}?range=5y&interval=1d",
                ticker
            ),
        };

        let response = self.client.get(&url).send().await?.json::<Value>().await?;
        let result = &response["chart"]["result"][0];

        // Yahoo returns a null result when there are no new bars (e.g. the
        // symbol is fully up-to-date). In that case return what we already have.
        if result.is_null() {
            if !existing.is_empty() {
                return Ok(existing);
            }
            return Err("No data returned from Yahoo Finance".into());
        }

        let timestamps = result["timestamp"].as_array().ok_or("Missing timestamp array")?;
        let quote = &result["indicators"]["quote"][0];

        let opens     = quote["open"].as_array().ok_or("Missing opens")?;
        let highs     = quote["high"].as_array().ok_or("Missing highs")?;
        let lows      = quote["low"].as_array().ok_or("Missing lows")?;
        let closes    = quote["close"].as_array().ok_or("Missing closes")?;
        let volumes   = quote["volume"].as_array().ok_or("Missing volumes")?;
        let adj_closes = result["indicators"]["adjclose"][0]["adjclose"]
            .as_array()
            .ok_or("Missing adjclose")?;

        let mut new_bars: Vec<YahooHistoricalBar> = Vec::new();

        for i in 0..timestamps.len() {
            let ts = timestamps[i].as_i64().unwrap_or(0);

            // Skip any bar whose timestamp is not strictly after the last
            // stored one (guards against off-by-one or Yahoo overlap).
            if let Some(last_ts) = last_stored_ts {
                if ts <= last_ts {
                    continue;
                }
            }

            new_bars.push(YahooHistoricalBar {
                timestamp: ts,
                open:      opens[i].as_f64().unwrap_or(0.0),
                high:      highs[i].as_f64().unwrap_or(0.0),
                low:       lows[i].as_f64().unwrap_or(0.0),
                close:     closes[i].as_f64().unwrap_or(0.0),
                adj_close: adj_closes[i].as_f64().unwrap_or(0.0),
                volume:    volumes[i].as_f64().unwrap_or(0.0),
            });
        }

        // Merge: existing bars first, then newly fetched bars (already filtered
        // to be strictly newer), sorted by timestamp, deduped.
        let mut merged = existing;
        merged.extend(new_bars);
        merged.sort_by_key(|b| b.timestamp);
        merged.dedup_by_key(|b| b.timestamp);

        write_yahoo_cache(ticker, &merged)?;
        Ok(merged)
    }
}

fn yahoo_cache_path(ticker: &str) -> Option<PathBuf> {
    let dir = std::env::var("CHRONO_YAHOO_CACHE_DIR").ok()?;
    if dir.trim().is_empty() {
        return None;
    }
    Some(PathBuf::from(dir).join(format!("{ticker}.json")))
}

fn read_yahoo_cache(ticker: &str) -> Result<Option<Vec<YahooHistoricalBar>>, Box<dyn Error>> {
    let Some(path) = yahoo_cache_path(ticker) else {
        return Ok(None);
    };
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path)?;
    let bars: Vec<YahooHistoricalBar> = serde_json::from_slice(&bytes)?;
    Ok(Some(bars))
}

fn write_yahoo_cache(ticker: &str, bars: &[YahooHistoricalBar]) -> Result<(), Box<dyn Error>> {
    let Some(path) = yahoo_cache_path(ticker) else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(bars)?)?;
    Ok(())
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
