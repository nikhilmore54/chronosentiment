//! Controlled Deferred Live observation producer.
//!
//! Does not rewrite [`super::deferred_live::DeferredLiveDriver`].
//! Does not replay `paper_trader_v2_*.csv`. Does not flip
//! `LIVE_YAHOO_FETCH_AUTHORIZED`.
//!
//! ```text
//! CACHED_1M | YAHOO_1M bars
//!         ↓  ControlledObservationTape (speed)
//! MarketObservation
//!         ↓
//! DeferredLiveRuntime.ingest
//! ```
//!
//! `mode` stays `DEFERRED_LIVE`. The source label is `CACHED_1M` or
//! `YAHOO_1M` — a controlled clock, not a broker feed.
//!
//! `DEFERRED_LIVE_SOURCE=yahoo` polls Yahoo 1m bars into the same ingest
//! path, labeled `YAHOO_1M`. That is **not** Observatory live fetch
//! (`LIVE_YAHOO_FETCH_AUTHORIZED` stays false).

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use chrono::{FixedOffset, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::deferred_live::MarketObservation;
use crate::decision_support::observatory_live_execution::LIVE_YAHOO_FETCH_AUTHORIZED;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ObservationSourceKind {
    None,
    ExternalLive,
    #[serde(rename = "CACHED_1M")]
    Cached1m,
    #[serde(rename = "YAHOO_1M")]
    Yahoo1m,
    YahooUnauthorized,
}

impl ObservationSourceKind {
    pub fn freshness(self) -> &'static str {
        match self {
            Self::ExternalLive => "LIVE",
            Self::Cached1m => "CACHED_1M",
            Self::Yahoo1m => "YAHOO_1M",
            Self::None | Self::YahooUnauthorized => "STALE",
        }
    }

    pub fn is_controlled_clock(self) -> bool {
        matches!(self, Self::Cached1m | Self::Yahoo1m)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourcedObservation {
    pub observation: MarketObservation,
    pub source: ObservationSourceKind,
    #[serde(default)]
    pub volume: Option<f64>,
}

impl SourcedObservation {
    pub fn external_live(observation: MarketObservation) -> Self {
        Self {
            observation,
            source: ObservationSourceKind::ExternalLive,
            volume: None,
        }
    }

    pub fn cached_1m(observation: MarketObservation) -> Self {
        Self {
            observation,
            source: ObservationSourceKind::Cached1m,
            volume: None,
        }
    }

    pub fn yahoo_1m(observation: MarketObservation) -> Self {
        Self {
            observation,
            source: ObservationSourceKind::Yahoo1m,
            volume: None,
        }
    }

    pub fn with_volume(mut self, volume: Option<f64>) -> Self {
        self.volume = volume;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ObservationFeedStatus {
    WaitingForFeed,
    Polling,
    WaitingForInbox,
    Playing,
    TapeExhausted,
    YahooRefused,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservationFeedSnapshot {
    pub background: ObservationSourceKind,
    /// Explicit producer label: `CACHED_1M` / `YAHOO_1M` / `NONE` / …
    pub observation_source: ObservationSourceKind,
    pub status: ObservationFeedStatus,
    pub yahoo_authorized: bool,
    pub clock_simulation: bool,
    pub speed: Option<f64>,
    pub inbox_path: Option<String>,
    pub last_ingest_unix: Option<i64>,
    pub last_source_kind: Option<ObservationSourceKind>,
    pub note: String,
}

impl ObservationFeedSnapshot {
    pub fn none() -> Self {
        Self {
            background: ObservationSourceKind::None,
            observation_source: ObservationSourceKind::None,
            status: ObservationFeedStatus::WaitingForFeed,
            yahoo_authorized: LIVE_YAHOO_FETCH_AUTHORIZED,
            clock_simulation: false,
            speed: None,
            inbox_path: None,
            last_ingest_unix: None,
            last_source_kind: None,
            note: none_note(),
        }
    }

    pub fn record_ingest(&mut self, sourced: &SourcedObservation) {
        self.last_ingest_unix = Some(sourced.observation.unix);
        self.last_source_kind = Some(sourced.source);
        self.observation_source = sourced.source;
    }
}

fn none_note() -> String {
    "No observation producer attached. LIVE_YAHOO_FETCH_AUTHORIZED=false. \
     Deferred Live can use a controlled CACHED_1M / YAHOO_1M clock \
     (`deferred_live --source cached`) — not historical paper_trader_v2 replay, \
     not a broker feed."
        .into()
}

pub fn yahoo_fetch_authorized() -> bool {
    LIVE_YAHOO_FETCH_AUTHORIZED
}

pub fn refuse_yahoo_fetch() -> Result<(), String> {
    if LIVE_YAHOO_FETCH_AUTHORIZED {
        return Ok(());
    }
    Err(
        "Observatory Yahoo fetch remains unauthorized (LIVE_YAHOO_FETCH_AUTHORIZED=false). \
         Deferred Live Yahoo 1m is a separate producer labeled YAHOO_1M, not that flag."
            .into(),
    )
}

pub fn parse_speed(raw: &str) -> Result<f64, String> {
    let t = raw.trim().trim_end_matches(['x', 'X']);
    if t.eq_ignore_ascii_case("instant") {
        return Ok(0.0);
    }
    let v: f64 = t
        .parse()
        .map_err(|_| format!("invalid --speed {raw}"))?;
    if !v.is_finite() || v < 0.0 {
        return Err(format!("invalid --speed {raw}"));
    }
    Ok(v)
}

/// Wall-clock wait between two market timestamps at `speed`.
/// `60x` → 60s of market time elapses in 1s. `speed == 0` is instant (tests).
pub fn inter_bar_wait_millis(prev_unix: i64, next_unix: i64, speed: f64) -> u64 {
    if !speed.is_finite() || speed <= 0.0 {
        return 0;
    }
    let dt = (next_unix - prev_unix).max(0) as f64;
    (dt * 1000.0 / speed).round().max(0.0) as u64
}

fn ist() -> FixedOffset {
    FixedOffset::east_opt(5 * 3600 + 30 * 60).expect("IST offset")
}

pub fn unix_ist_date(unix: i64) -> String {
    ist()
        .timestamp_opt(unix, 0)
        .single()
        .map(|t| t.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

pub fn filter_ist_date(obs: Vec<MarketObservation>, date: &str) -> Vec<MarketObservation> {
    obs.into_iter()
        .filter(|o| unix_ist_date(o.unix) == date)
        .collect()
}

/// Sorted bar tape. Emits one `MarketObservation` at a time. Does not trade.
pub struct ControlledObservationTape {
    source: ObservationSourceKind,
    observations: Vec<SourcedObservation>,
    cursor: usize,
    speed: f64,
    last_unix: Option<i64>,
}

impl ControlledObservationTape {
    pub fn new(
        source: ObservationSourceKind,
        mut observations: Vec<MarketObservation>,
        speed: f64,
    ) -> Self {
        observations.sort_by_key(|o| o.unix);
        let sourced = observations
            .into_iter()
            .map(|o| SourcedObservation {
                observation: o,
                source,
                volume: None,
            })
            .collect();
        Self {
            source,
            observations: sourced,
            cursor: 0,
            speed,
            last_unix: None,
        }
    }

    pub fn new_sourced(
        source: ObservationSourceKind,
        mut observations: Vec<SourcedObservation>,
        speed: f64,
    ) -> Self {
        observations.sort_by_key(|s| s.observation.unix);
        Self {
            source,
            observations,
            cursor: 0,
            speed,
            last_unix: None,
        }
    }

    pub fn source(&self) -> ObservationSourceKind {
        self.source
    }

    pub fn speed(&self) -> f64 {
        self.speed
    }

    pub fn len(&self) -> usize {
        self.observations.len()
    }

    pub fn remaining(&self) -> usize {
        self.observations.len().saturating_sub(self.cursor)
    }

    pub fn wait_millis_for_next(&self) -> u64 {
        let Some(next) = self.observations.get(self.cursor) else {
            return 1_000;
        };
        match self.last_unix {
            None => 0,
            Some(prev) => inter_bar_wait_millis(prev, next.observation.unix, self.speed),
        }
    }

    pub fn next(&mut self) -> Option<SourcedObservation> {
        let sourced = self.observations.get(self.cursor)?.clone();
        self.cursor += 1;
        self.last_unix = Some(sourced.observation.unix);
        Some(sourced)
    }

    pub fn snapshot(&self) -> ObservationFeedSnapshot {
        let exhausted = self.cursor >= self.observations.len();
        ObservationFeedSnapshot {
            background: self.source,
            observation_source: self.source,
            status: if exhausted {
                ObservationFeedStatus::TapeExhausted
            } else {
                ObservationFeedStatus::Playing
            },
            yahoo_authorized: LIVE_YAHOO_FETCH_AUTHORIZED,
            clock_simulation: true,
            speed: Some(self.speed),
            inbox_path: None,
            last_ingest_unix: self.last_unix,
            last_source_kind: if self.cursor == 0 {
                None
            } else {
                Some(self.source)
            },
            note: format!(
                "Deferred Live controlled clock · observation_source={} · speed={}x · {}/{} bars. \
                 Not a broker feed. Not paper_trader_v2 replay.",
                self.source.freshness(),
                self.speed,
                self.cursor,
                self.observations.len()
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackgroundSourceKind {
    None,
    Jsonl,
    Cached,
    Yahoo,
    YahooUnauthorized,
}

#[derive(Debug, Clone)]
pub struct FeedConfig {
    pub kind: BackgroundSourceKind,
    pub inbox_path: Option<PathBuf>,
    pub ticker: Option<String>,
    pub date: Option<String>,
    pub speed: f64,
    pub cache_dir: Option<PathBuf>,
}

impl FeedConfig {
    pub fn from_env() -> Self {
        Self::from_env_vars(|k| std::env::var(k).ok())
    }

    pub fn from_env_vars<F: Fn(&str) -> Option<String>>(get: F) -> Self {
        let source = get("DEFERRED_LIVE_SOURCE")
            .unwrap_or_default()
            .to_ascii_lowercase();
        let jsonl = get("DEFERRED_LIVE_OBS_JSONL").filter(|s| !s.is_empty());
        let ticker = get("DEFERRED_LIVE_TICKER").filter(|s| !s.is_empty());
        let date = get("DEFERRED_LIVE_DATE").filter(|s| !s.is_empty());
        let speed = get("DEFERRED_LIVE_SPEED")
            .and_then(|s| parse_speed(&s).ok())
            .unwrap_or(60.0);
        let cache_dir = get("DEFERRED_LIVE_CACHE_DIR")
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);
        match source.as_str() {
            "yahoo" => Self {
                kind: BackgroundSourceKind::Yahoo,
                inbox_path: None,
                ticker,
                date,
                speed,
                cache_dir,
            },
            "cached" | "cached_1m" | "clock_sim" | "clock-simulation" => Self {
                kind: BackgroundSourceKind::Cached,
                inbox_path: None,
                ticker,
                date,
                speed,
                cache_dir,
            },
            "jsonl" => Self {
                kind: BackgroundSourceKind::Jsonl,
                inbox_path: jsonl.map(PathBuf::from),
                ticker: None,
                date: None,
                speed,
                cache_dir: None,
            },
            _ if jsonl.is_some() => Self {
                kind: BackgroundSourceKind::Jsonl,
                inbox_path: jsonl.map(PathBuf::from),
                ticker: None,
                date: None,
                speed,
                cache_dir: None,
            },
            _ => Self {
                kind: BackgroundSourceKind::None,
                inbox_path: None,
                ticker: None,
                date: None,
                speed: 60.0,
                cache_dir: None,
            },
        }
    }
}

pub enum ObservationProducer {
    None,
    Jsonl(JsonlInboxSource),
    Tape(ControlledObservationTape),
    YahooLive(Yahoo1mLiveSource),
    YahooUnauthorized { reason: String },
}

impl ObservationProducer {
    pub fn none() -> Self {
        Self::None
    }

    pub fn tape(tape: ControlledObservationTape) -> Self {
        Self::Tape(tape)
    }

    pub fn from_config(cfg: FeedConfig) -> Self {
        match cfg.kind {
            BackgroundSourceKind::None => Self::None,
            BackgroundSourceKind::YahooUnauthorized => Self::YahooUnauthorized {
                reason: refuse_yahoo_fetch()
                    .err()
                    .unwrap_or_else(|| "Yahoo live fetch refused.".into()),
            },
            BackgroundSourceKind::Jsonl => match cfg.inbox_path {
                Some(path) => Self::Jsonl(JsonlInboxSource::new(path)),
                None => Self::None,
            },
            BackgroundSourceKind::Cached => Self::None,
            BackgroundSourceKind::Yahoo => {
                let date = cfg.date.unwrap_or_else(today_ist_date);
                Self::YahooLive(Yahoo1mLiveSource::new(date))
            }
        }
    }

    pub fn from_env() -> Self {
        Self::from_config(FeedConfig::from_env())
    }

    pub fn snapshot(&self) -> ObservationFeedSnapshot {
        match self {
            Self::None => ObservationFeedSnapshot::none(),
            Self::Tape(tape) => tape.snapshot(),
            Self::Jsonl(src) => {
                let exists = src.path.exists();
                ObservationFeedSnapshot {
                    background: ObservationSourceKind::ExternalLive,
                    observation_source: ObservationSourceKind::ExternalLive,
                    status: if exists {
                        ObservationFeedStatus::Polling
                    } else {
                        ObservationFeedStatus::WaitingForInbox
                    },
                    yahoo_authorized: LIVE_YAHOO_FETCH_AUTHORIZED,
                    clock_simulation: false,
                    speed: None,
                    inbox_path: Some(src.path.display().to_string()),
                    last_ingest_unix: None,
                    last_source_kind: None,
                    note: "Tailing operator-supplied JSONL. Not paper_trader_v2 replay."
                        .into(),
                }
            }
            Self::YahooLive(src) => src.snapshot(),
            Self::YahooUnauthorized { reason } => ObservationFeedSnapshot {
                background: ObservationSourceKind::YahooUnauthorized,
                observation_source: ObservationSourceKind::YahooUnauthorized,
                status: ObservationFeedStatus::YahooRefused,
                yahoo_authorized: LIVE_YAHOO_FETCH_AUTHORIZED,
                clock_simulation: false,
                speed: None,
                inbox_path: None,
                last_ingest_unix: None,
                last_source_kind: None,
                note: reason.clone(),
            },
        }
    }

    pub fn wait_millis_for_next(&self) -> u64 {
        match self {
            Self::Tape(t) => t.wait_millis_for_next(),
            Self::YahooLive(src) => src.wait_millis_for_next(),
            _ => 1_000,
        }
    }

    pub fn poll(&mut self, tickers: &[String]) -> Result<Vec<SourcedObservation>, String> {
        let _ = tickers;
        match self {
            Self::None | Self::YahooUnauthorized { .. } => Ok(vec![]),
            Self::YahooLive(_) => Ok(vec![]),
            Self::Jsonl(src) => src.poll_new_lines(),
            Self::Tape(tape) => Ok(tape.next().into_iter().collect()),
        }
    }

    /// Yahoo 1m uses async HTTP. JSONL / tape stay on [`Self::poll`].
    pub async fn poll_async(&mut self, tickers: &[String]) -> Result<Vec<SourcedObservation>, String> {
        match self {
            Self::YahooLive(src) => src.poll_tickers(tickers).await,
            _ => self.poll(tickers),
        }
    }
}

pub fn today_ist_date() -> String {
    unix_ist_date(Utc::now().timestamp())
}

fn yahoo_symbol_from_brief_ticker(ticker: &str) -> String {
    if ticker.ends_with("_NS") {
        format!("{}.NS", ticker.trim_end_matches("_NS"))
    } else if ticker.ends_with(".NS") {
        ticker.to_string()
    } else {
        format!("{ticker}.NS")
    }
}

fn brief_ticker_from_yahoo_symbol(symbol: &str) -> String {
    symbol.replace(".NS", "_NS")
}

fn take_new_session_bars(
    date: &str,
    after_unix: i64,
    obs: impl IntoIterator<Item = MarketObservation>,
) -> (Vec<MarketObservation>, i64) {
    let mut newest = after_unix;
    let mut out = Vec::new();
    for o in obs {
        if unix_ist_date(o.unix) != date {
            continue;
        }
        if o.unix <= after_unix {
            continue;
        }
        newest = newest.max(o.unix);
        out.push(o);
    }
    (out, newest)
}

/// Parse a Yahoo v8 chart JSON into observations. Skips null/non-positive closes.
pub fn observations_from_yahoo_chart(
    ticker: &str,
    response: &Value,
) -> Result<Vec<MarketObservation>, String> {
    let result = &response["chart"]["result"][0];
    if result.is_null() {
        return Err("Yahoo returned no 1m chart result".into());
    }
    let timestamps = result["timestamp"]
        .as_array()
        .ok_or("Yahoo chart missing timestamp")?;
    let quote = &result["indicators"]["quote"][0];
    let highs = quote["high"].as_array().ok_or("Yahoo chart missing high")?;
    let lows = quote["low"].as_array().ok_or("Yahoo chart missing low")?;
    let closes = quote["close"].as_array().ok_or("Yahoo chart missing close")?;
    let ticker = brief_ticker_from_yahoo_symbol(ticker);
    let mut out = Vec::new();
    for i in 0..timestamps.len() {
        let unix = timestamps[i].as_i64().unwrap_or(0);
        let price = closes.get(i).and_then(|v| v.as_f64()).unwrap_or(0.0);
        if unix <= 0 || !price.is_finite() || price <= 0.0 {
            continue;
        }
        let high = highs.get(i).and_then(|v| v.as_f64()).filter(|v| v.is_finite());
        let low = lows.get(i).and_then(|v| v.as_f64()).filter(|v| v.is_finite());
        out.push(MarketObservation {
            ticker: ticker.clone(),
            unix,
            price,
            high,
            low,
        });
    }
    out.sort_by(|a, b| a.unix.cmp(&b.unix).then(a.ticker.cmp(&b.ticker)));
    Ok(out)
}

pub async fn fetch_yahoo_1m_bars(symbol: &str) -> Result<Vec<MarketObservation>, String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .build()
        .map_err(|e| e.to_string())?;
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{symbol}?interval=1m&range=1d"
    );
    let response: Value = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("yahoo fetch {symbol}: {e}"))?
        .json()
        .await
        .map_err(|e| format!("yahoo json {symbol}: {e}"))?;
    observations_from_yahoo_chart(symbol, &response)
}

/// Incremental Yahoo 1m producer. Observatory `LIVE_YAHOO_FETCH_AUTHORIZED` stays false.
pub struct Yahoo1mLiveSource {
    date: String,
    last_unix: HashMap<String, i64>,
    rounds: usize,
}

impl Yahoo1mLiveSource {
    pub fn new(date: impl Into<String>) -> Self {
        Self {
            date: date.into(),
            last_unix: HashMap::new(),
            rounds: 0,
        }
    }

    pub fn wait_millis_for_next(&self) -> u64 {
        if self.rounds == 0 {
            0
        } else {
            30_000
        }
    }

    pub fn snapshot(&self) -> ObservationFeedSnapshot {
        ObservationFeedSnapshot {
            background: ObservationSourceKind::Yahoo1m,
            observation_source: ObservationSourceKind::Yahoo1m,
            status: ObservationFeedStatus::Polling,
            yahoo_authorized: LIVE_YAHOO_FETCH_AUTHORIZED,
            clock_simulation: false,
            speed: None,
            inbox_path: None,
            last_ingest_unix: None,
            last_source_kind: None,
            note: format!(
                "Deferred Live Yahoo 1m poller · session {} · not Observatory live fetch · LIVE_YAHOO_FETCH_AUTHORIZED=false.",
                self.date
            ),
        }
    }

    pub async fn poll_tickers(
        &mut self,
        tickers: &[String],
    ) -> Result<Vec<SourcedObservation>, String> {
        let mut batch = Vec::new();
        for (i, ticker) in tickers.iter().enumerate() {
            if i > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            }
            let symbol = yahoo_symbol_from_brief_ticker(ticker);
            match fetch_yahoo_1m_bars(&symbol).await {
                Ok(obs) => {
                    let after = self.last_unix.get(ticker).copied().unwrap_or(0);
                    let (fresh, newest) = take_new_session_bars(&self.date, after, obs);
                    if newest > after {
                        self.last_unix.insert(ticker.clone(), newest);
                    }
                    for o in fresh {
                        batch.push(SourcedObservation::yahoo_1m(o));
                    }
                }
                Err(e) => eprintln!("[deferred-live] yahoo 1m {symbol}: {e}"),
            }
        }
        batch.sort_by(|a, b| {
            a.observation
                .unix
                .cmp(&b.observation.unix)
                .then(a.observation.ticker.cmp(&b.observation.ticker))
        });
        self.rounds += 1;
        Ok(batch)
    }
}

pub struct JsonlInboxSource {
    path: PathBuf,
    offset: u64,
}

impl JsonlInboxSource {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            offset: 0,
        }
    }

    pub fn poll_new_lines(&mut self) -> Result<Vec<SourcedObservation>, String> {
        if !self.path.exists() {
            return Ok(vec![]);
        }
        let mut file = File::open(&self.path)
            .map_err(|e| format!("open {}: {e}", self.path.display()))?;
        let meta = file
            .metadata()
            .map_err(|e| format!("stat {}: {e}", self.path.display()))?;
        if meta.len() < self.offset {
            self.offset = 0;
        }
        file.seek(SeekFrom::Start(self.offset))
            .map_err(|e| format!("seek {}: {e}", self.path.display()))?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)
            .map_err(|e| format!("read {}: {e}", self.path.display()))?;
        self.offset = meta.len();
        let mut out = Vec::new();
        for line in buf.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            match parse_observation_line(line) {
                Ok(obs) => out.push(SourcedObservation::external_live(obs)),
                Err(_) => continue,
            }
        }
        Ok(out)
    }
}

#[derive(Debug, Deserialize)]
struct ObservationLine {
    ticker: String,
    unix: i64,
    price: f64,
    high: Option<f64>,
    low: Option<f64>,
}

pub fn parse_observation_line(line: &str) -> Result<MarketObservation, String> {
    let row: ObservationLine =
        serde_json::from_str(line).map_err(|e| format!("observation JSON: {e}"))?;
    if row.ticker.is_empty() || !row.price.is_finite() || row.price <= 0.0 {
        return Err("invalid ticker or price".into());
    }
    Ok(MarketObservation {
        ticker: row.ticker,
        unix: row.unix,
        price: row.price,
        high: row.high,
        low: row.low,
    })
}

pub fn append_observation_line(path: &Path, obs: &MarketObservation) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    let line = serde_json::json!({
        "ticker": obs.ticker,
        "unix": obs.unix,
        "price": obs.price,
        "high": obs.high,
        "low": obs.low,
    });
    use std::io::Write;
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("append {}: {e}", path.display()))?;
    writeln!(f, "{line}").map_err(|e| format!("write {}: {e}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yahoo_observatory_flag_remains_false() {
        assert!(!yahoo_fetch_authorized());
        assert!(refuse_yahoo_fetch().is_err());
    }

    #[test]
    fn labels_are_cached_or_yahoo_not_broker() {
        assert_eq!(ObservationSourceKind::Cached1m.freshness(), "CACHED_1M");
        assert_eq!(ObservationSourceKind::Yahoo1m.freshness(), "YAHOO_1M");
        assert_ne!(ObservationSourceKind::Cached1m.freshness(), "LIVE");
        assert!(ObservationSourceKind::Cached1m.is_controlled_clock());
    }

    #[test]
    fn speed_60x_maps_one_minute_to_one_second() {
        assert_eq!(inter_bar_wait_millis(1_000, 1_060, 60.0), 1_000);
        assert_eq!(inter_bar_wait_millis(1_000, 1_060, 300.0), 200);
        assert_eq!(inter_bar_wait_millis(1_000, 1_060, 1.0), 60_000);
        assert_eq!(inter_bar_wait_millis(1_000, 1_060, 0.0), 0);
        assert_eq!(parse_speed("60x").unwrap(), 60.0);
        assert_eq!(parse_speed("instant").unwrap(), 0.0);
    }

    #[test]
    fn tape_emits_in_timestamp_order_and_does_not_invent_prices() {
        let mut tape = ControlledObservationTape::new(
            ObservationSourceKind::Cached1m,
            vec![
                MarketObservation::last("JUBLFOOD_NS", 20, 2.0),
                MarketObservation::last("JUBLFOOD_NS", 10, 1.0),
            ],
            0.0,
        );
        let a = tape.next().unwrap();
        let b = tape.next().unwrap();
        assert_eq!(a.observation.unix, 10);
        assert_eq!(a.observation.price, 1.0);
        assert_eq!(b.observation.unix, 20);
        assert_eq!(a.source, ObservationSourceKind::Cached1m);
        assert!(tape.next().is_none());
        assert_eq!(tape.snapshot().observation_source, ObservationSourceKind::Cached1m);
        assert!(tape.snapshot().clock_simulation);
        assert_eq!(tape.snapshot().status, ObservationFeedStatus::TapeExhausted);
    }

    #[test]
    fn ist_date_filter_keeps_only_that_session() {
        // 2026-09-11 09:15 IST = 2026-09-11 03:45 UTC
        let open = 1_788_911_100;
        let obs = vec![
            MarketObservation::last("T", open - 86_400, 1.0),
            MarketObservation::last("T", open, 2.0),
            MarketObservation::last("T", open + 60, 3.0),
        ];
        let day = unix_ist_date(open);
        let kept = filter_ist_date(obs, &day);
        assert_eq!(kept.len(), 2);
        assert_eq!(kept[0].price, 2.0);
    }

    #[test]
    fn default_env_is_idle() {
        let cfg = FeedConfig::from_env_vars(|_| None);
        assert_eq!(cfg.kind, BackgroundSourceKind::None);
        let snap = ObservationProducer::from_config(cfg).snapshot();
        assert_eq!(snap.observation_source, ObservationSourceKind::None);
        assert_eq!(snap.status, ObservationFeedStatus::WaitingForFeed);
        assert!(!snap.yahoo_authorized);
    }

    #[test]
    fn cached_env_selects_cached_kind() {
        let cfg = FeedConfig::from_env_vars(|k| match k {
            "DEFERRED_LIVE_SOURCE" => Some("cached".into()),
            "DEFERRED_LIVE_TICKER" => Some("JUBLFOOD_NS".into()),
            "DEFERRED_LIVE_DATE" => Some("2026-09-11".into()),
            "DEFERRED_LIVE_SPEED" => Some("60x".into()),
            _ => None,
        });
        assert_eq!(cfg.kind, BackgroundSourceKind::Cached);
        assert_eq!(cfg.ticker.as_deref(), Some("JUBLFOOD_NS"));
        assert_eq!(cfg.speed, 60.0);
    }

    #[test]
    fn yahoo_env_builds_yahoo_1m_poller_without_observatory_flag() {
        let cfg = FeedConfig::from_env_vars(|k| match k {
            "DEFERRED_LIVE_SOURCE" => Some("yahoo".into()),
            "DEFERRED_LIVE_DATE" => Some("2026-09-15".into()),
            _ => None,
        });
        assert_eq!(cfg.kind, BackgroundSourceKind::Yahoo);
        let producer = ObservationProducer::from_config(cfg);
        let snap = producer.snapshot();
        assert_eq!(snap.observation_source, ObservationSourceKind::Yahoo1m);
        assert_eq!(snap.status, ObservationFeedStatus::Polling);
        assert!(!snap.yahoo_authorized);
        assert!(!snap.clock_simulation);
        assert!(snap.note.contains("LIVE_YAHOO_FETCH_AUTHORIZED=false"));
    }

    #[test]
    fn yahoo_chart_skips_null_closes_and_maps_ns_ticker() {
        let body = serde_json::json!({
            "chart": { "result": [{
                "timestamp": [100, 160, 220],
                "indicators": { "quote": [{
                    "high": [11.0, 12.0, 13.0],
                    "low": [9.0, 10.0, 11.0],
                    "close": [10.0, null, 12.5]
                }]}
            }]}
        });
        let obs = observations_from_yahoo_chart("JUBLFOOD.NS", &body).unwrap();
        assert_eq!(obs.len(), 2);
        assert_eq!(obs[0].ticker, "JUBLFOOD_NS");
        assert_eq!(obs[0].unix, 100);
        assert_eq!(obs[0].price, 10.0);
        assert_eq!(obs[1].unix, 220);
        assert_eq!(obs[1].price, 12.5);
    }

    #[test]
    fn yahoo_poller_emits_only_new_same_session_bars() {
        let open = 1_788_752_700 + 8 * 86_400;
        assert_eq!(unix_ist_date(open), "2026-09-15");
        let obs = vec![
            MarketObservation::last("T_NS", open - 86_400, 1.0),
            MarketObservation::last("T_NS", open, 2.0),
            MarketObservation::last("T_NS", open + 60, 3.0),
            MarketObservation::last("T_NS", open + 120, 4.0),
        ];
        let date = unix_ist_date(open);
        let (first, last) = take_new_session_bars(&date, 0, obs.clone());
        assert_eq!(first.len(), 3);
        assert_eq!(last, open + 120);
        let (second, last2) = take_new_session_bars(&date, last, obs);
        assert!(second.is_empty());
        assert_eq!(last2, last);
    }
}
