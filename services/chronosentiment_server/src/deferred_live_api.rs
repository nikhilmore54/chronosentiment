//! Deferred-live HTTP projection — separate from frozen `/paper-trades`.
//!
//! ```text
//! LIVE MARKET OBSERVATION (JSONL inbox / HTTP POST)
//!       ↓
//! DecisionLoopRuntime / AsOfSessionDriver
//!       ↓
//! DeferredLiveRuntime.ingest
//!       ↓
//! AsOfIcAssembler
//!       ↓
//! DecisionSurface
//!       ↓
//! GET /api/v1/intraday/deferred-live → Cockpit
//! ```
//!
//! Same ingest path as the cached session harness. Does not load
//! `paper_trader_v2_*.csv`. Does not invent prices. Does not flip
//! Observatory `LIVE_YAHOO_FETCH_AUTHORIZED`. `DEFERRED_LIVE_SOURCE=yahoo`
//! polls Yahoo 1m into the same `ingest_sourced` path labeled `YAHOO_1M`.
//! Does not modify `deferred_live.rs`.

use axum::{
    Json,
    extract::State,
    http::StatusCode,
};
use chronosentiment_adapter::product::{
    cached_session_tape, observation_producer_from_env, session_tape_tickers, AsOfSessionDriver,
    AsOfSessionEvent, DecisionSurface, DeferredLiveConfig, LivePaperLedger, MarketObservation,
    ObservationFeedSnapshot, ObservationProducer, SessionState, DeferredLivePerformance,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};

use super::intraday_api::IntradayStore;

#[derive(Clone)]
pub struct DeferredLiveState {
    pub briefs: IntradayStore,
    pub driver: Arc<RwLock<AsOfSessionDriver>>,
    producer: Arc<Mutex<ObservationProducer>>,
    session_date: Option<String>,
}

impl DeferredLiveState {
    pub fn new(briefs: IntradayStore) -> Self {
        let src = std::env::var("DEFERRED_LIVE_SOURCE").unwrap_or_default();
        let ticker = std::env::var("DEFERRED_LIVE_TICKER").ok().filter(|s| !s.is_empty());
        let mut date = std::env::var("DEFERRED_LIVE_DATE").ok().filter(|s| !s.is_empty());
        let cached = src.eq_ignore_ascii_case("cached") || src.eq_ignore_ascii_case("cached_1m");
        let yahoo = src.eq_ignore_ascii_case("yahoo") || src.eq_ignore_ascii_case("yahoo_1m");
        if yahoo && date.is_none() {
            date = Some(chronosentiment_adapter::product::today_ist_date());
        }
        let session_mode = cached && date.is_some() && ticker.is_none();
        let attach_session = date.is_some() && !(cached && ticker.is_some());

        let mut config = DeferredLiveConfig::default();
        if std::env::var("STRICT_T0_ADMISSION").map(|v| v == "1").unwrap_or(false) {
            config.strict_t0_admission = true;
        }
        let mut driver = AsOfSessionDriver::new(config);
        let (producer, session_date) = if session_mode {
            let date = date.clone().expect("session_mode requires date");
            let cache_dir = std::env::var("DEFERRED_LIVE_CACHE_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| std::path::PathBuf::from("intraday_capture/yahoo_cache_1m"));
            let speed = std::env::var("DEFERRED_LIVE_SPEED")
                .ok()
                .and_then(|s| chronosentiment_adapter::product::parse_speed(&s).ok())
                .unwrap_or(60.0);
            let armed_ids = driver.install_session(briefs.as_slice(), &date);
            let tickers = session_tape_tickers(briefs.as_slice(), &date);
            eprintln!(
                "[deferred-live] session {date} auto-arm {} ACT tickers (no POST /arm); as-of watches on same ingest path",
                armed_ids.len()
            );
            let producer = match cached_session_tape(&cache_dir, &tickers, &date, speed) {
                Ok(tape) => {
                    driver.set_feed_snapshot(tape.snapshot());
                    ObservationProducer::tape(tape)
                }
                Err(e) => {
                    eprintln!("[deferred-live] session tape: {e}");
                    ObservationProducer::none()
                }
            };
            (producer, Some(date))
        } else {
            let producer = observation_producer_from_env();
            driver.set_feed_snapshot(producer.snapshot());
            if attach_session {
                let date = date.clone().expect("attach_session requires date");
                let armed_ids = driver.install_session(briefs.as_slice(), &date);
                eprintln!(
                    "[deferred-live] live observation session {date} auto-arm {} ACT; JSONL/HTTP/YAHOO_1M uses DecisionLoopRuntime.ingest",
                    armed_ids.len()
                );
                if yahoo && armed_ids.is_empty() {
                    if let Some(latest) = briefs.iter().map(|b| b.date.as_str()).max() {
                        eprintln!(
                            "[deferred-live] no DecisionBriefs for {date}; Yahoo 1m will fetch names from {latest} without auto-arming that book"
                        );
                    }
                }
                (producer, Some(date))
            } else {
                if cached {
                    if let Some(ticker) = ticker {
                        let mut hits: Vec<_> = briefs.iter().filter(|d| d.ticker == ticker).cloned().collect();
                        if let Some(date) = date {
                            let dated: Vec<_> = hits.iter().filter(|d| d.date == date).cloned().collect();
                            if !dated.is_empty() {
                                hits = dated;
                            }
                        }
                        hits.sort_by(|a, b| b.date.cmp(&a.date).then(b.oqs.cmp(&a.oqs)));
                        if let Some(brief) = hits.into_iter().next() {
                            eprintln!("[deferred-live] armed {} ({})", brief.ticker, brief.id);
                            driver.arm(brief);
                        }
                    }
                }
                (producer, None)
            }
        };

        if let Some(feed) = driver.session() {
            eprintln!("[deferred-live] {}", feed.note);
        } else {
            eprintln!(
                "[deferred-live] observation producer: {}",
                driver.feed().note
            );
        }
        Self {
            briefs,
            driver: Arc::new(RwLock::new(driver)),
            producer: Arc::new(Mutex::new(producer)),
            session_date,
        }
    }
}

fn env_truthy(key: &str) -> bool {
    std::env::var(key)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
        .unwrap_or(false)
}

/// Background loop: JSONL tail or CACHED_1M / YAHOO_1M controlled clock.
/// Same ingest as the cached session harness. Never loads paper_trader_v2 CSVs.
/// Does not enable observatory Yahoo fetch.
pub async fn run_observation_loop(state: DeferredLiveState) {
    loop {
        let (batch, snap, wait) = {
            let tickers = {
                let mut driver = state.driver.write().await;
                if let Some(date) = &state.session_date {
                    driver.begin_session(state.briefs.as_slice(), date);
                }
                let mut tickers = driver.watched_tickers();
                if tickers.is_empty() {
                    if let Some(latest) = state.briefs.iter().map(|b| b.date.as_str()).max() {
                        tickers = session_tape_tickers(state.briefs.as_slice(), latest);
                    }
                }
                tickers
            };
            let mut producer = state.producer.lock().await;
            let wait = producer.wait_millis_for_next();
            let batch = match producer.poll_async(&tickers).await {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("[deferred-live] observation poll: {e}");
                    vec![]
                }
            };
            let snap = producer.snapshot();
            (batch, snap, wait)
        };
        {
            let mut driver = state.driver.write().await;
            driver.set_feed_snapshot(snap);
            for sourced in batch {
                driver.ingest_sourced(sourced);
            }
        }
        tokio::time::sleep(Duration::from_millis(wait.max(10))).await;
    }
}

#[derive(Debug, Serialize)]
pub struct DeferredLiveResponse {
    pub mode: String,
    pub observation_feed: ObservationFeedSnapshot,
    pub session: Option<SessionState>,
    pub performance: DeferredLivePerformance,
    pub armed: Vec<ArmedDto>,
    pub ledger: LivePaperLedger,
    pub asof_events: Vec<AsOfSessionEvent>,
    pub last_surface: Option<DecisionSurface>,
}

#[derive(Debug, Serialize)]
pub struct ArmedDto {
    pub decision_id: String,
    pub ticker: String,
}

#[derive(Debug, Deserialize)]
pub struct ArmRequest {
    pub decision_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ObservationRequest {
    pub ticker: String,
    pub unix: i64,
    pub price: f64,
    pub high: Option<f64>,
    pub low: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct IngestResponse {
    pub opened: bool,
    pub open_error: Option<String>,
    pub events: usize,
    pub source: String,
}

/// GET /api/v1/intraday/deferred-live
pub async fn get_deferred_live(State(state): State<DeferredLiveState>) -> Json<DeferredLiveResponse> {
    let driver = state.driver.read().await;
    Json(DeferredLiveResponse {
        mode: driver.ledger().mode.clone(),
        observation_feed: driver.feed().clone(),
        session: driver.session().cloned(),
        performance: driver.performance(state.briefs.as_slice()),
        armed: driver
            .armed()
            .into_iter()
            .map(|a| ArmedDto {
                decision_id: a.decision_id,
                ticker: a.ticker,
            })
            .collect(),
        ledger: driver.ledger().clone(),
        asof_events: driver.asof_events().to_vec(),
        last_surface: driver.last_surface().cloned(),
    })
}

/// POST /api/v1/intraday/deferred-live/arm
pub async fn post_arm(
    State(state): State<DeferredLiveState>,
    Json(body): Json<ArmRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let brief = state
        .briefs
        .iter()
        .find(|d| d.id == body.decision_id)
        .cloned()
        .ok_or((StatusCode::NOT_FOUND, format!("unknown decision_id {}", body.decision_id)))?;
    let mut driver = state.driver.write().await;
    driver.arm(brief);
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/intraday/deferred-live/observations
/// Operator-supplied live tick. Same ingest as JSONL / cached tape. Not Yahoo.
pub async fn post_observation(
    State(state): State<DeferredLiveState>,
    Json(body): Json<ObservationRequest>,
) -> Json<IngestResponse> {
    let obs = MarketObservation {
        ticker: body.ticker,
        unix: body.unix,
        price: body.price,
        high: body.high,
        low: body.low,
    };
    let mut driver = state.driver.write().await;
    let surface = driver.ingest(obs);
    Json(IngestResponse {
        opened: surface.opened,
        open_error: surface.open_error,
        events: surface.events.len(),
        source: "EXTERNAL_LIVE".into(),
    })
}
