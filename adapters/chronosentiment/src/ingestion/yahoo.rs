//! Yahoo daily ingest (AUD-025).
//!
//! Null / non-positive OHLC+volume are **not** zero-filled into the usable
//! metric sequence. They are retained as excluded provenance (`INVALID` vs
//! `MISSING`). ATR is unchanged and never filters.

use crate::ingestion::provider::{MarketDataProvider, TimeRange, ValidatedObservationTranslator};
use crate::instrument::Instrument;
use crate::observation::RawObservation;
use async_trait::async_trait;
use chrono::{Duration, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

const DAILY_CACHE_SCHEMA: &str = "yahoo_daily_v2";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YahooHistoricalBar {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub adj_close: f64,
    pub volume: f64,
}

/// Received Yahoo daily row that is not a usable trading-session observation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DailyBarClassification {
    Invalid,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExcludedDailyBar {
    pub timestamp: Option<i64>,
    pub classification: DailyBarClassification,
    pub reason: String,
    pub open: Option<f64>,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub close: Option<f64>,
    pub adj_close: Option<f64>,
    pub volume: Option<f64>,
}

/// Partitioned daily ingest: usable metric bars plus excluded provenance.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct YahooDailyIngest {
    pub usable: Vec<YahooHistoricalBar>,
    pub excluded: Vec<ExcludedDailyBar>,
}

impl YahooDailyIngest {
    pub fn last_stored_ts(&self) -> Option<i64> {
        let usable_max = self.usable.iter().map(|b| b.timestamp).max();
        let excluded_max = self.excluded.iter().filter_map(|b| b.timestamp).max();
        match (usable_max, excluded_max) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.usable.extend(other.usable);
        self.excluded.extend(other.excluded);
        self.normalize();
        self
    }

    fn normalize(&mut self) {
        self.usable.sort_by_key(|b| b.timestamp);
        self.usable.dedup_by_key(|b| b.timestamp);
        let usable_ts: HashSet<i64> = self.usable.iter().map(|b| b.timestamp).collect();
        self.excluded
            .retain(|e| e.timestamp.map(|t| !usable_ts.contains(&t)).unwrap_or(true));
        self.excluded.sort_by_key(|e| (e.timestamp.unwrap_or(0), e.reason.clone()));
        let mut seen = HashSet::new();
        self.excluded.retain(|e| {
            let key = (
                e.timestamp.unwrap_or(0),
                e.classification.clone(),
                e.reason.clone(),
            );
            seen.insert(key)
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct YahooDailyCacheFile {
    schema: String,
    usable: Vec<YahooHistoricalBar>,
    excluded: Vec<ExcludedDailyBar>,
}

#[derive(Debug, Clone, Copy)]
enum JsonNum {
    Absent,
    Null,
    Number(f64),
}

impl JsonNum {
    fn from_value(v: Option<&Value>) -> Self {
        match v {
            None => Self::Absent,
            Some(x) if x.is_null() => Self::Null,
            Some(x) => x
                .as_f64()
                .filter(|n| n.is_finite())
                .map(Self::Number)
                .unwrap_or(Self::Null),
        }
    }

    fn opt(self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(n),
            Self::Absent | Self::Null => None,
        }
    }

    fn is_nullish(self) -> bool {
        matches!(self, Self::Absent | Self::Null)
    }
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

        Self { client }
    }

    /// Daily fetch with AUD-025 provenance. `fetch_historical` returns `.usable` only.
    pub async fn fetch_daily_ingest(
        &self,
        instrument: &Instrument,
    ) -> Result<YahooDailyIngest, Box<dyn Error>> {
        let ticker = instrument
            .provider_ids
            .get("yahoo")
            .ok_or("Instrument missing 'yahoo' identity")?;

        let existing = read_yahoo_cache(ticker)?.unwrap_or_default();
        let last_stored_ts = existing.last_stored_ts();

        let url = match last_stored_ts {
            Some(last_ts) => {
                let period1 = last_ts + 1;
                let period2 = (Utc::now() + Duration::days(1)).timestamp();
                format!(
                    "https://query1.finance.yahoo.com/v8/finance/chart/{ticker}?period1={period1}&period2={period2}&interval=1d"
                )
            }
            None => format!(
                "https://query1.finance.yahoo.com/v8/finance/chart/{ticker}?range=5y&interval=1d"
            ),
        };

        let response = self.client.get(&url).send().await?.json::<Value>().await?;
        let result = &response["chart"]["result"][0];

        if result.is_null() {
            if !existing.usable.is_empty() || !existing.excluded.is_empty() {
                return Ok(existing);
            }
            return Err("No data returned from Yahoo Finance".into());
        }

        let fetched = ingest_yahoo_daily_chart(&response)?;
        let merged = existing.merge(fetched);
        write_yahoo_cache(ticker, &merged)?;
        Ok(merged)
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
        Ok(self.fetch_daily_ingest(instrument).await?.usable)
    }
}

/// Parse a Yahoo v8 daily chart. Never zero-fills null OHLC into usable bars.
pub fn ingest_yahoo_daily_chart(response: &Value) -> Result<YahooDailyIngest, String> {
    let result = &response["chart"]["result"][0];
    if result.is_null() {
        return Err("Yahoo returned no daily chart result".into());
    }
    let timestamps = result["timestamp"]
        .as_array()
        .ok_or("Yahoo chart missing timestamp")?;
    let quote = &result["indicators"]["quote"][0];
    let opens = quote["open"].as_array();
    let highs = quote["high"].as_array();
    let lows = quote["low"].as_array();
    let closes = quote["close"].as_array();
    let volumes = quote["volume"].as_array();
    let adj_closes = result["indicators"]["adjclose"][0]["adjclose"].as_array();

    let mut ingest = YahooDailyIngest::default();
    for i in 0..timestamps.len() {
        match classify_yahoo_daily_row(
            timestamps.get(i),
            opens.and_then(|a| a.get(i)),
            highs.and_then(|a| a.get(i)),
            lows.and_then(|a| a.get(i)),
            closes.and_then(|a| a.get(i)),
            adj_closes.and_then(|a| a.get(i)),
            volumes.and_then(|a| a.get(i)),
        ) {
            Ok(bar) => ingest.usable.push(bar),
            Err(excluded) => ingest.excluded.push(excluded),
        }
    }
    ingest.normalize();
    Ok(ingest)
}

/// Split a legacy (possibly zero-filled) daily vector into usable + excluded.
pub fn partition_yahoo_daily_bars(bars: Vec<YahooHistoricalBar>) -> YahooDailyIngest {
    let mut ingest = YahooDailyIngest::default();
    for bar in bars {
        match classify_stored_daily_bar(&bar) {
            Ok(usable) => ingest.usable.push(usable),
            Err(excluded) => ingest.excluded.push(excluded),
        }
    }
    ingest.normalize();
    ingest
}

pub fn parse_yahoo_daily_cache_bytes(bytes: &[u8]) -> Result<YahooDailyIngest, String> {
    let value: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if value.is_array() {
        let bars: Vec<YahooHistoricalBar> =
            serde_json::from_value(value).map_err(|e| e.to_string())?;
        return Ok(partition_yahoo_daily_bars(bars));
    }
    let file: YahooDailyCacheFile = serde_json::from_value(value).map_err(|e| e.to_string())?;
    let mut ingest = YahooDailyIngest {
        usable: file.usable,
        excluded: file.excluded,
    };
    // Legacy usable slots may still contain zero-filled bars.
    let partitioned = partition_yahoo_daily_bars(std::mem::take(&mut ingest.usable));
    ingest.excluded.extend(partitioned.excluded);
    ingest.usable = partitioned.usable;
    ingest.normalize();
    Ok(ingest)
}

fn classify_yahoo_daily_row(
    ts_val: Option<&Value>,
    open: Option<&Value>,
    high: Option<&Value>,
    low: Option<&Value>,
    close: Option<&Value>,
    adj_close: Option<&Value>,
    volume: Option<&Value>,
) -> Result<YahooHistoricalBar, ExcludedDailyBar> {
    let timestamp = match ts_val {
        None | Some(Value::Null) => None,
        Some(v) => v.as_i64().filter(|t| *t > 0),
    };
    let open = JsonNum::from_value(open);
    let high = JsonNum::from_value(high);
    let low = JsonNum::from_value(low);
    let close = JsonNum::from_value(close);
    let adj_close = JsonNum::from_value(adj_close);
    let volume = JsonNum::from_value(volume);

    if timestamp.is_none() {
        return Err(ExcludedDailyBar {
            timestamp,
            classification: DailyBarClassification::Missing,
            reason: "absent_timestamp".into(),
            open: open.opt(),
            high: high.opt(),
            low: low.opt(),
            close: close.opt(),
            adj_close: adj_close.opt(),
            volume: volume.opt(),
        });
    }

    let nullish = open.is_nullish()
        || high.is_nullish()
        || low.is_nullish()
        || close.is_nullish()
        || adj_close.is_nullish()
        || volume.is_nullish();
    if nullish {
        return Err(ExcludedDailyBar {
            timestamp,
            classification: DailyBarClassification::Invalid,
            reason: "null_ohlc".into(),
            open: open.opt(),
            high: high.opt(),
            low: low.opt(),
            close: close.opt(),
            adj_close: adj_close.opt(),
            volume: volume.opt(),
        });
    }

    let bar = YahooHistoricalBar {
        timestamp: timestamp.unwrap(),
        open: open.opt().unwrap(),
        high: high.opt().unwrap(),
        low: low.opt().unwrap(),
        close: close.opt().unwrap(),
        adj_close: adj_close.opt().unwrap(),
        volume: volume.opt().unwrap(),
    };
    classify_stored_daily_bar(&bar)
}

fn classify_stored_daily_bar(
    bar: &YahooHistoricalBar,
) -> Result<YahooHistoricalBar, ExcludedDailyBar> {
    if bar.timestamp <= 0 {
        return Err(excluded_from_stored(
            bar,
            DailyBarClassification::Missing,
            "absent_timestamp",
        ));
    }
    if !usable_ohlcv(bar) {
        return Err(excluded_from_stored(
            bar,
            DailyBarClassification::Invalid,
            "non_positive_ohlc",
        ));
    }
    Ok(bar.clone())
}

fn usable_ohlcv(bar: &YahooHistoricalBar) -> bool {
    bar.open > 0.0
        && bar.high > 0.0
        && bar.low > 0.0
        && bar.close > 0.0
        && bar.adj_close > 0.0
        && bar.volume > 0.0
        && bar.high >= bar.low
        && bar.open.is_finite()
        && bar.high.is_finite()
        && bar.low.is_finite()
        && bar.close.is_finite()
        && bar.adj_close.is_finite()
        && bar.volume.is_finite()
}

fn excluded_from_stored(
    bar: &YahooHistoricalBar,
    classification: DailyBarClassification,
    reason: &str,
) -> ExcludedDailyBar {
    ExcludedDailyBar {
        timestamp: (bar.timestamp > 0).then_some(bar.timestamp),
        classification,
        reason: reason.into(),
        open: Some(bar.open),
        high: Some(bar.high),
        low: Some(bar.low),
        close: Some(bar.close),
        adj_close: Some(bar.adj_close),
        volume: Some(bar.volume),
    }
}

fn yahoo_cache_path(ticker: &str) -> Option<PathBuf> {
    let dir = std::env::var("CHRONO_YAHOO_CACHE_DIR").ok()?;
    if dir.trim().is_empty() {
        return None;
    }
    Some(PathBuf::from(dir).join(format!("{ticker}.json")))
}

fn read_yahoo_cache(ticker: &str) -> Result<Option<YahooDailyIngest>, Box<dyn Error>> {
    let Some(path) = yahoo_cache_path(ticker) else {
        return Ok(None);
    };
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path)?;
    Ok(Some(parse_yahoo_daily_cache_bytes(&bytes)?))
}

fn write_yahoo_cache(ticker: &str, ingest: &YahooDailyIngest) -> Result<(), Box<dyn Error>> {
    let Some(path) = yahoo_cache_path(ticker) else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = YahooDailyCacheFile {
        schema: DAILY_CACHE_SCHEMA.into(),
        usable: ingest.usable.clone(),
        excluded: ingest.excluded.clone(),
    };
    fs::write(path, serde_json::to_vec_pretty(&file)?)?;
    Ok(())
}

pub struct YahooTranslator;

impl ValidatedObservationTranslator<YahooHistoricalBar> for YahooTranslator {
    fn translate(&self, raw: YahooHistoricalBar, _instrument: &Instrument) -> RawObservation {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn chart(ts: Vec<Value>, ohlcv: Vec<[Value; 6]>) -> Value {
        let mut open = Vec::new();
        let mut high = Vec::new();
        let mut low = Vec::new();
        let mut close = Vec::new();
        let mut adj = Vec::new();
        let mut vol = Vec::new();
        for row in ohlcv {
            open.push(row[0].clone());
            high.push(row[1].clone());
            low.push(row[2].clone());
            close.push(row[3].clone());
            adj.push(row[4].clone());
            vol.push(row[5].clone());
        }
        serde_json::json!({
            "chart": { "result": [{
                "timestamp": ts,
                "indicators": {
                    "quote": [{
                        "open": open,
                        "high": high,
                        "low": low,
                        "close": close,
                        "volume": vol
                    }],
                    "adjclose": [{ "adjclose": adj }]
                }
            }]}
        })
    }

    fn n(v: f64) -> Value {
        serde_json::json!(v)
    }

    #[test]
    fn null_ohlc_is_invalid_not_zero_filled() {
        let body = chart(
            vec![serde_json::json!(100), serde_json::json!(200)],
            vec![
                [n(10.0), n(11.0), n(9.0), n(10.5), n(10.5), n(1000.0)],
                [
                    Value::Null,
                    Value::Null,
                    Value::Null,
                    Value::Null,
                    Value::Null,
                    Value::Null,
                ],
            ],
        );
        let ingest = ingest_yahoo_daily_chart(&body).unwrap();
        assert_eq!(ingest.usable.len(), 1);
        assert_eq!(ingest.usable[0].timestamp, 100);
        assert_eq!(ingest.usable[0].close, 10.5);
        assert_eq!(ingest.excluded.len(), 1);
        assert_eq!(
            ingest.excluded[0].classification,
            DailyBarClassification::Invalid
        );
        assert_eq!(ingest.excluded[0].reason, "null_ohlc");
        assert_eq!(ingest.excluded[0].timestamp, Some(200));
        assert!(ingest.excluded[0].close.is_none());
        assert!(ingest.usable.iter().all(|b| b.close > 0.0));
    }

    #[test]
    fn non_positive_ohlc_is_invalid_and_retained() {
        let body = chart(
            vec![serde_json::json!(100), serde_json::json!(200)],
            vec![
                [n(10.0), n(11.0), n(9.0), n(10.5), n(10.5), n(1000.0)],
                [n(0.0), n(0.0), n(0.0), n(0.0), n(0.0), n(0.0)],
            ],
        );
        let ingest = ingest_yahoo_daily_chart(&body).unwrap();
        assert_eq!(ingest.usable.len(), 1);
        assert_eq!(ingest.excluded.len(), 1);
        assert_eq!(
            ingest.excluded[0].classification,
            DailyBarClassification::Invalid
        );
        assert_eq!(ingest.excluded[0].reason, "non_positive_ohlc");
        assert_eq!(ingest.excluded[0].close, Some(0.0));
    }

    #[test]
    fn absent_timestamp_is_missing_not_invalid() {
        let body = chart(
            vec![Value::Null],
            vec![[n(10.0), n(11.0), n(9.0), n(10.5), n(10.5), n(1000.0)]],
        );
        let ingest = ingest_yahoo_daily_chart(&body).unwrap();
        assert!(ingest.usable.is_empty());
        assert_eq!(ingest.excluded.len(), 1);
        assert_eq!(
            ingest.excluded[0].classification,
            DailyBarClassification::Missing
        );
        assert_eq!(ingest.excluded[0].reason, "absent_timestamp");
        assert!(ingest.excluded[0].timestamp.is_none());
    }

    #[test]
    fn legacy_cache_zero_bars_are_partitioned_not_usable() {
        let ingest = partition_yahoo_daily_bars(vec![
            YahooHistoricalBar {
                timestamp: 100,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.5,
                adj_close: 10.5,
                volume: 1000.0,
            },
            YahooHistoricalBar {
                timestamp: 200,
                open: 0.0,
                high: 0.0,
                low: 0.0,
                close: 0.0,
                adj_close: 0.0,
                volume: 0.0,
            },
        ]);
        assert_eq!(ingest.usable.len(), 1);
        assert_eq!(ingest.usable[0].timestamp, 100);
        assert_eq!(ingest.excluded.len(), 1);
        assert_eq!(ingest.excluded[0].timestamp, Some(200));
        assert_eq!(
            ingest.excluded[0].classification,
            DailyBarClassification::Invalid
        );
    }

    #[test]
    fn v2_cache_roundtrip_keeps_excluded_provenance() {
        let raw = serde_json::to_vec(&YahooDailyCacheFile {
            schema: DAILY_CACHE_SCHEMA.into(),
            usable: vec![YahooHistoricalBar {
                timestamp: 100,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.5,
                adj_close: 10.5,
                volume: 1000.0,
            }],
            excluded: vec![ExcludedDailyBar {
                timestamp: Some(200),
                classification: DailyBarClassification::Invalid,
                reason: "null_ohlc".into(),
                open: None,
                high: None,
                low: None,
                close: None,
                adj_close: None,
                volume: None,
            }],
        })
        .unwrap();
        let parsed = parse_yahoo_daily_cache_bytes(&raw).unwrap();
        assert_eq!(parsed.usable.len(), 1);
        assert_eq!(parsed.excluded.len(), 1);
        assert_eq!(parsed.excluded[0].reason, "null_ohlc");
    }

    fn bar(ts: i64, close: f64) -> YahooHistoricalBar {
        YahooHistoricalBar {
            timestamp: ts,
            open: close,
            high: close + 1.0,
            low: close - 1.0,
            close,
            adj_close: close,
            volume: 1_000.0,
        }
    }

    fn zero_bar(ts: i64) -> YahooHistoricalBar {
        YahooHistoricalBar {
            timestamp: ts,
            open: 0.0,
            high: 0.0,
            low: 0.0,
            close: 0.0,
            adj_close: 0.0,
            volume: 0.0,
        }
    }

    fn valid_series(n: usize) -> Vec<YahooHistoricalBar> {
        (0..n)
            .map(|i| bar(1_000 + (i as i64) * 86_400, 100.0 + i as f64))
            .collect()
    }

    #[test]
    fn invalid_daily_bar_does_not_poison_atr_after_ingest_partition() {
        use crate::decision_support::enrichment_certify::metrics_from_bars_at_t;
        use crate::decision_support::forward_tick::instrument_id_for;

        let mut mixed = valid_series(20);
        let last_ts = mixed.last().unwrap().timestamp;
        mixed.push(zero_bar(last_ts + 86_400));
        let t = Utc.timestamp_opt(last_ts + 86_400, 0).single().unwrap();
        let id = instrument_id_for("AUD025.NS");

        let poisoned = metrics_from_bars_at_t(&mixed, t, id).get_float("atr_14");
        assert!(poisoned.is_none());
        assert_ne!(poisoned, Some(0.0));

        let ingest = partition_yahoo_daily_bars(mixed);
        assert_eq!(ingest.usable.len(), 20);
        assert_eq!(ingest.excluded.len(), 1);

        let t_usable = Utc.timestamp_opt(last_ts, 0).single().unwrap();
        let atr = metrics_from_bars_at_t(&ingest.usable, t_usable, id).get_float("atr_14");
        let atr = atr.expect("20 valid bars must form ATR-14");
        assert!(atr.is_finite() && atr > 0.0);
    }

    #[test]
    fn fewer_than_14_valid_bars_yields_atr_none_not_zero() {
        use crate::decision_support::enrichment_certify::metrics_from_bars_at_t;
        use crate::decision_support::forward_tick::instrument_id_for;

        let bars = valid_series(13);
        let t = Utc
            .timestamp_opt(bars.last().unwrap().timestamp, 0)
            .single()
            .unwrap();
        let atr = metrics_from_bars_at_t(&bars, t, instrument_id_for("AUD025.NS")).get_float("atr_14");
        assert!(atr.is_none(), "LIVE-001 completeness requires atr_14.is_some()");
        assert_ne!(atr, Some(0.0));
    }
}
