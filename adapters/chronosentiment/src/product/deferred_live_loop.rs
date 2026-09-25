//! Live `MarketObservation` loop for deferred-live paper trading.
//!
//! Does not fetch Yahoo. Does not rewrite [`DeferredLiveDriver`] or Paper Trader v0.2.
//! The loop is the clock: each observation is an event.
//!
//! ```text
//! MarketObservation
//!       ↓
//! DeferredLiveRuntime.ingest
//!       ├── arm DecisionBrief → open_from_brief (live fill)
//!       └── DeferredLiveDriver.on_observation (shared v0.2 lifecycle)
//!       ↓
//! Deferred Live Ledger
//! ```
//!
//! Cached 1m / Yahoo 1m bars may be converted into observations by a
//! controlled clock (`CACHED_1M` / `YAHOO_1M`). That is not
//! `paper_trader_v2_*.csv` replay and not a broker feed.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::deferred_live::{
    overlay_live_quote, DeferredLiveConfig, DeferredLiveDriver, LivePaperEvent, LivePaperLedger,
    MarketObservation, OpenError,
};
use super::intraday_decision::{DecisionBrief, ExecutionFacts};
use super::live_observation::{
    ControlledObservationTape, ObservationFeedSnapshot, ObservationProducer, ObservationSourceKind,
    SourcedObservation,
};

/// One OHLC bar as stored in `intraday_capture/yahoo_cache_1m/*.json`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CachedOhlcBar {
    pub timestamp: i64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    #[serde(default)]
    pub volume: Option<f64>,
}

/// Adapter-owned live runtime. Separate ledger from frozen CSV replay.
pub struct DeferredLiveRuntime {
    driver: DeferredLiveDriver,
    /// One armed brief per ticker. Next matching observation opens the paper fill.
    armed: HashMap<String, DecisionBrief>,
    feed: ObservationFeedSnapshot,
    session: Option<super::deferred_live_session::SessionState>,
    /// Live update capture sidecar.
    update_capture: super::live_update_capture::LiveUpdateCapture,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArmedBrief {
    pub decision_id: String,
    pub ticker: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IngestOutcome {
    pub opened: bool,
    pub open_error: Option<String>,
    pub events: Vec<LivePaperEvent>,
}

impl DeferredLiveRuntime {
    pub fn new(config: DeferredLiveConfig) -> Self {
        Self {
            driver: DeferredLiveDriver::new(config),
            armed: HashMap::new(),
            feed: ObservationFeedSnapshot::none(),
            session: None,
            update_capture: super::live_update_capture::LiveUpdateCapture::new(),
        }
    }

    pub fn arm(&mut self, brief: DecisionBrief) {
        self.armed.insert(brief.ticker.clone(), brief);
    }

    pub fn session(&self) -> Option<&super::deferred_live_session::SessionState> {
        self.session.as_ref()
    }

    pub fn performance(
        &self,
        briefs: &[DecisionBrief],
    ) -> super::deferred_live_performance::DeferredLivePerformance {
        super::deferred_live_performance::score_deferred_live(
            self.ledger(),
            briefs,
            self.session.as_ref().map(|s| s.date.as_str()),
        )
    }

    fn decision_taken(&self, decision_id: &str) -> bool {
        self.driver
            .ledger()
            .positions
            .iter()
            .any(|p| p.paper.decision_id.as_deref() == Some(decision_id))
    }

    /// Arm ACT session briefs that are not already on the ledger.
    /// Does not implement TARGET / STOP / HORIZON.
    pub fn auto_arm(&mut self, briefs: impl IntoIterator<Item = DecisionBrief>) -> Vec<String> {
        use super::deferred_live_session::eligible_for_auto_arm;
        let mut ids = Vec::new();
        for brief in briefs {
            if eligible_for_auto_arm(&brief).is_err() {
                continue;
            }
            if self.decision_taken(&brief.id) {
                continue;
            }
            if self.armed.contains_key(&brief.ticker) {
                continue;
            }
            ids.push(brief.id.clone());
            self.arm(brief);
        }
        ids
    }

    /// Select and arm ACT briefs for `date`. Safe to call again (idempotent).
    pub fn begin_session(&mut self, briefs: &[DecisionBrief], date: &str) -> Vec<String> {
        use super::deferred_live_session::{select_session_briefs, SessionState};
        let selected = select_session_briefs(briefs, date);
        let ids = self.auto_arm(selected);
        let n = self.armed.len() + self.ledger().positions.len();
        self.session = Some(SessionState::summarize(briefs, date, n));
        
        let ledger_dir = std::env::var("LIVE_LEDGER_DIR").unwrap_or_else(|_| "live_capture/ledger".into());
        self.update_capture.enable(date, std::path::PathBuf::from(ledger_dir));
        
        ids
    }

    pub fn disarm_ticker(&mut self, ticker: &str) {
        self.armed.remove(ticker);
    }

    pub fn armed(&self) -> Vec<ArmedBrief> {
        let mut v: Vec<ArmedBrief> = self
            .armed
            .values()
            .map(|b| ArmedBrief {
                decision_id: b.id.clone(),
                ticker: b.ticker.clone(),
            })
            .collect();
        v.sort_by(|a, b| a.ticker.cmp(&b.ticker).then(a.decision_id.cmp(&b.decision_id)));
        v
    }

    pub fn ledger(&self) -> &LivePaperLedger {
        self.driver.ledger()
    }

    pub fn feed(&self) -> &ObservationFeedSnapshot {
        &self.feed
    }

    pub fn set_feed_snapshot(&mut self, snap: ObservationFeedSnapshot) {
        let last_ingest_unix = self.feed.last_ingest_unix;
        let last_source_kind = self.feed.last_source_kind;
        self.feed = snap;
        self.feed.last_ingest_unix = last_ingest_unix;
        self.feed.last_source_kind = last_source_kind;
    }

    /// Tickers that still need observations: armed names plus open live paper.
    pub fn watched_tickers(&self) -> Vec<String> {
        let mut t: Vec<String> = self.armed.keys().cloned().collect();
        for p in &self.driver.ledger().positions {
            if p.walk.is_open() {
                let ticker = p.paper.ticker.clone();
                if !t.iter().any(|x| x == &ticker) {
                    t.push(ticker);
                }
            }
        }
        t
    }

    /// Event-driven ingest. Opens from the armed DecisionBrief on the first
    /// matching tick, using `obs.price` as `paper_entry_price`. Then runs the
    /// existing driver lifecycle on the same observation.
    /// HTTP POST / operator JSONL is treated as an external live producer.
    pub fn ingest(&mut self, obs: MarketObservation) -> IngestOutcome {
        self.ingest_sourced(SourcedObservation::external_live(obs))
    }

    pub fn ingest_sourced(&mut self, sourced: SourcedObservation) -> IngestOutcome {
        if sourced.source == ObservationSourceKind::YahooUnauthorized {
            return IngestOutcome {
                opened: false,
                open_error: Some("observatory Yahoo fetch is not authorized".into()),
                events: vec![],
            };
        }
        self.feed.record_ingest(&sourced);
        let obs = sourced.observation;
        let mut opened = false;
        let mut open_error = None;

        if let Some(brief) = self.armed.get(&obs.ticker).cloned() {
            let already_taken = self.driver.ledger().positions.iter().any(|p| {
                p.paper.decision_id.as_deref() == Some(brief.id.as_str())
            });
            if !already_taken {
                let mut live_brief = brief.clone();
                overlay_live_quote(&mut live_brief, obs.price, obs.unix);
                live_brief.execution.freshness = sourced.source.freshness().to_string();
                match self.driver.open_from_brief(&live_brief, &obs) {
                    Ok(_) => {
                        opened = true;
                        self.disarm_ticker(&obs.ticker);
                    }
                    Err(OpenError::AlreadyOpen) => {}
                    Err(e) => open_error = Some(e.to_string()),
                }
            }
        }

        let events = self.driver.on_observation(&obs);
        
        // Update capture sidecar
        self.update_capture.observe(self.driver.ledger(), &obs);
        
        IngestOutcome {
            opened,
            open_error,
            events,
        }
    }

    pub fn ingest_all<I: IntoIterator<Item = MarketObservation>>(&mut self, observations: I) -> Vec<IngestOutcome> {
        observations.into_iter().map(|o| self.ingest(o)).collect()
    }
}

/// Map Yahoo/cache `.NS` file names onto DecisionBrief `_NS` tickers.
pub fn brief_ticker_from_cache_symbol(symbol: &str) -> String {
    symbol.replace(".NS", "_NS")
}
#[derive(Debug, Clone, PartialEq)]
pub struct CachedMarketBar {
    pub observation: MarketObservation,
    pub volume: Option<f64>,
}

pub fn cached_market_bars(
    ticker: &str,
    bars: &[CachedOhlcBar],
) -> Vec<CachedMarketBar> {
    let ticker = brief_ticker_from_cache_symbol(ticker);

    let mut out: Vec<CachedMarketBar> = bars
        .iter()
        .filter(|b| b.close.is_finite() && b.close > 0.0)
        .map(|b| CachedMarketBar {
            observation: MarketObservation {
                ticker: ticker.clone(),
                unix: b.timestamp,
                price: b.close,
                high: Some(b.high),
                low: Some(b.low),
            },
            volume: b.volume.filter(|v| v.is_finite() && *v > 0.0),
        })
        .collect();

    out.sort_by_key(|b| b.observation.unix);
    out
}

/// Convert cached OHLC bars into observations. `price` is the bar close.
/// High/low are the bar extremes for that observation only (no look-ahead).
pub fn observations_from_cached_bars(ticker: &str, bars: &[CachedOhlcBar]) -> Vec<MarketObservation> {
    let ticker = brief_ticker_from_cache_symbol(ticker);
    let mut out: Vec<MarketObservation> = bars
        .iter()
        .filter(|b| b.close.is_finite() && b.close > 0.0)
        .map(|b| MarketObservation {
            ticker: ticker.clone(),
            unix: b.timestamp,
            price: b.close,
            high: Some(b.high),
            low: Some(b.low),
        })
        .collect();
    out.sort_by_key(|o| o.unix);
    out
}

pub fn load_cached_1m_market_bars(cache_dir: &Path, symbol: &str) -> Result<Vec<CachedMarketBar>, String> {
    let path = cache_dir.join(format!("{symbol}.json"));
    let bytes = fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let bars: Vec<CachedOhlcBar> =
        serde_json::from_slice(&bytes).map_err(|e| format!("parse {}: {e}", path.display()))?;
    Ok(cached_market_bars(symbol, &bars))
}

pub fn load_cached_1m_observations(cache_dir: &Path, symbol: &str) -> Result<Vec<MarketObservation>, String> {
    let path = cache_dir.join(format!("{symbol}.json"));
    let bytes = fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let bars: Vec<CachedOhlcBar> =
        serde_json::from_slice(&bytes).map_err(|e| format!("parse {}: {e}", path.display()))?;
    Ok(observations_from_cached_bars(symbol, &bars))
}

pub fn cache_symbol_from_brief_ticker(ticker: &str) -> String {
    if ticker.ends_with("_NS") {
        format!("{}.NS", ticker.trim_end_matches("_NS"))
    } else if ticker.ends_with(".NS") {
        ticker.to_string()
    } else {
        format!("{ticker}.NS")
    }
}

/// Cached 1m tape labeled `CACHED_1M`. Does not open positions; caller arms + ingest.
pub fn cached_1m_tape(
    cache_dir: &Path,
    ticker: &str,
    date: &str,
    speed: f64,
) -> Result<super::live_observation::ControlledObservationTape, String> {
    use super::live_observation::{filter_ist_date, ControlledObservationTape, ObservationSourceKind};
    let symbol = cache_symbol_from_brief_ticker(ticker);
    let all = load_cached_1m_observations(cache_dir, &symbol)?;
    let day = filter_ist_date(all, date);
    if day.is_empty() {
        return Err(format!(
            "no cached 1m bars for {symbol} on {date} in {}",
            cache_dir.display()
        ));
    }
    Ok(ControlledObservationTape::new(
        ObservationSourceKind::Cached1m,
        day,
        speed,
    ))
}

/// Continue a cached session until the first observation at or after `until_unix`.
/// Used to reach live H300 after the session tape would otherwise exhaust.
/// Does not invent prices. Does not change the driver.
pub fn cached_1m_through_unix(
    cache_dir: &Path,
    ticker: &str,
    from_date: &str,
    until_unix: i64,
    speed: f64,
) -> Result<ControlledObservationTape, String> {
    let symbol = cache_symbol_from_brief_ticker(ticker);
    let all = load_cached_1m_observations(cache_dir, &symbol)?;
    let mut tape = filter_ist_date_owned(&all, from_date);
    if tape.is_empty() {
        return Err(format!(
            "no cached 1m bars for {symbol} on {from_date} in {}",
            cache_dir.display()
        ));
    }
    let last_session = tape.last().expect("non-empty").unix;
    let mut rest: Vec<MarketObservation> = all
        .into_iter()
        .filter(|o| o.unix > last_session)
        .collect();
    rest.sort_by_key(|o| o.unix);
    for obs in rest {
        let reached = obs.unix >= until_unix;
        tape.push(obs);
        if reached {
            break;
        }
    }
    if tape.last().map(|o| o.unix < until_unix).unwrap_or(true) {
        return Err(format!(
            "cached tape for {symbol} from {from_date} never reaches unix {until_unix}"
        ));
    }
    Ok(ControlledObservationTape::new(
        ObservationSourceKind::Cached1m,
        tape,
        speed,
    ))
}

/// Merge cached 1m bars for many tickers on one IST date, ordered by (unix, ticker).
/// Missing cache files are skipped. Empty after skip is an error.
pub fn cached_session_tape(
    cache_dir: &Path,
    tickers: &[String],
    date: &str,
    speed: f64,
) -> Result<ControlledObservationTape, String> {
    use super::live_observation::{unix_ist_date, ObservationSourceKind, SourcedObservation};
    let mut merged = Vec::new();
    let mut loaded = 0usize;
    for ticker in tickers {
        let symbol = cache_symbol_from_brief_ticker(ticker);
        match load_cached_1m_market_bars(cache_dir, &symbol) {
            Ok(all) => {
                let day: Vec<CachedMarketBar> = all
                    .into_iter()
                    .filter(|b| unix_ist_date(b.observation.unix) == date)
                    .collect();
                if day.is_empty() {
                    eprintln!("[deferred-live] session: no {symbol} bars on {date}");
                    continue;
                }
                loaded += 1;
                for b in day {
                    merged.push(SourcedObservation {
                        observation: b.observation,
                        source: ObservationSourceKind::Cached1m,
                        volume: b.volume,
                    });
                }
            }
            Err(e) => eprintln!("[deferred-live] session skip {ticker}: {e}"),
        }
    }
    if merged.is_empty() {
        return Err(format!(
            "no cached 1m bars for session {date} ({loaded} tickers loaded)"
        ));
    }
    merged.sort_by(|a, b| {
        a.observation
            .unix
            .cmp(&b.observation.unix)
            .then(a.observation.ticker.cmp(&b.observation.ticker))
    });
    Ok(ControlledObservationTape::new_sourced(
        ObservationSourceKind::Cached1m,
        merged,
        speed,
    ))
}

fn filter_ist_date_owned(obs: &[MarketObservation], date: &str) -> Vec<MarketObservation> {
    super::live_observation::filter_ist_date(obs.to_vec(), date)
}

/// Controlled-clock tapes that exercise TARGET / STOP / HORIZON via ingest only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleScenario {
    Target,
    Stop,
    Horizon,
}

impl LifecycleScenario {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "target" => Ok(Self::Target),
            "stop" => Ok(Self::Stop),
            "horizon" => Ok(Self::Horizon),
            other => Err(format!(
                "unknown scenario {other} (expected target|stop|horizon)"
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Target => "TARGET",
            Self::Stop => "STOP",
            Self::Horizon => "HORIZON",
        }
    }
}

const SCENARIO_TICKER: &str = "JUBLFOOD_NS";
const SCENARIO_DECISION_ENTRY: f64 = 473.0;
const SCENARIO_REFERENCE: f64 = 475.4;
const SCENARIO_FILL: f64 = 480.5;
const SCENARIO_SNAP: i64 = 1_000_000;

/// Synthetic CACHED_1M tape + armed brief. The producer still only emits
/// `MarketObservation`; TARGET / STOP / HORIZON remain the driver's job.
/// Horizon scenario uses a 180s unix horizon so the tape stays short.
pub fn lifecycle_scenario_tape(
    kind: LifecycleScenario,
    speed: f64,
) -> (DecisionBrief, ControlledObservationTape, DeferredLiveConfig) {
    let (target, risk, horizon_secs, observations) = match kind {
        LifecycleScenario::Target => (
            475.0,
            490.0,
            300 * 60,
            vec![
                MarketObservation {
                    ticker: SCENARIO_TICKER.into(),
                    unix: SCENARIO_SNAP,
                    price: SCENARIO_FILL,
                    high: Some(481.0),
                    low: Some(480.5),
                },
                // Same bar would also pierce STOP; TARGET is first.
                MarketObservation {
                    ticker: SCENARIO_TICKER.into(),
                    unix: SCENARIO_SNAP + 60,
                    price: 474.8,
                    high: Some(491.0),
                    low: Some(474.5),
                },
            ],
        ),
        LifecycleScenario::Stop => (
            450.0,
            485.0,
            300 * 60,
            vec![
                MarketObservation {
                    ticker: SCENARIO_TICKER.into(),
                    unix: SCENARIO_SNAP,
                    price: SCENARIO_FILL,
                    high: Some(481.0),
                    low: Some(480.0),
                },
                MarketObservation {
                    ticker: SCENARIO_TICKER.into(),
                    unix: SCENARIO_SNAP + 60,
                    price: 485.2,
                    high: Some(486.0),
                    low: Some(479.0),
                },
            ],
        ),
        LifecycleScenario::Horizon => (
            450.0,
            490.0,
            180,
            vec![
                MarketObservation {
                    ticker: SCENARIO_TICKER.into(),
                    unix: SCENARIO_SNAP,
                    price: SCENARIO_FILL,
                    high: Some(481.0),
                    low: Some(480.0),
                },
                MarketObservation {
                    ticker: SCENARIO_TICKER.into(),
                    unix: SCENARIO_SNAP + 60,
                    price: 479.0,
                    high: Some(479.5),
                    low: Some(478.5),
                },
                MarketObservation {
                    ticker: SCENARIO_TICKER.into(),
                    unix: SCENARIO_SNAP + 180,
                    price: 475.4,
                    high: Some(476.0),
                    low: Some(475.0),
                },
            ],
        ),
    };
    let brief = scenario_brief(kind, target, risk);
    let tape = ControlledObservationTape::new(
        ObservationSourceKind::Cached1m,
        observations,
        speed,
    );
    (
        brief,
        tape,
        DeferredLiveConfig {
            horizon_secs,
            strict_t0_admission: false,
            ..Default::default()
        },
    )
}

fn scenario_brief(kind: LifecycleScenario, target: f64, risk: f64) -> DecisionBrief {
    brief(
        &format!("SCENARIO-{}-JUBLFOOD_NS", kind.as_str()),
        SCENARIO_TICKER,
        "SHORT",
        SCENARIO_DECISION_ENTRY,
        SCENARIO_REFERENCE,
        facts(target, risk, SCENARIO_SNAP),
    )
}

fn facts(target: f64, risk: f64, snap: i64) -> ExecutionFacts {
    ExecutionFacts {
        snap_unix: Some(snap),
        adaptive_target: Some(target),
        adaptive_risk: Some(risk),
        adaptive_horizon_sessions: Some(1.0),
        target_distance_abs: None,
        target_distance_pct: None,
        risk_distance_abs: None,
        risk_distance_pct: None,
        expected_move_pct: None,
        current_price: None,
        last_tick_unix: None,
        freshness: "STALE".into(),
        horizon_elapsed: Some(false),
    }
}

fn brief(
    id: &str,
    ticker: &str,
    dir: &str,
    entry: f64,
    reference: f64,
    exec: ExecutionFacts,
) -> DecisionBrief {
    DecisionBrief {
        id: id.into(),
        ticker: ticker.into(),
        date: "2026-09-07".into(),
        direction: dir.into(),
        oqs: 53,
        h60_class: "WAIT".into(),
        reference_price: Some(reference),
        entry_price: Some(entry),
        execution: exec,
        entry_state: "WAIT-HIGH".into(),
        entry_action: "ACT".into(),
        entry_confidence: "HIGH".into(),
        entry_horizon: "H300".into(),
        entry_why: "scenario".into(),
        entry_risk: "scenario".into(),
        h120_state: "WAIT-HIGH".into(),
        h120_action: "ACT".into(),
        h120_confidence: "HIGH".into(),
        h120_horizon: "H300".into(),
        h120_why: "scenario".into(),
        h120_risk: "scenario".into(),
        h15_ret: None,
        h30_ret: None,
        h60_ret: None,
        h120_ret: None,
        h180_ret: None,
        h300_ret: None,
        mfe_h60: None,
        mfe_h120: None,
        outcome: None,
        pnl: None,
        hist_win: None,
        hist_pf: None,
        hist_med: None,
    }
}

/// Play a controlled tape into the runtime. Sleep is the caller's job.
pub fn ingest_tape(runtime: &mut DeferredLiveRuntime, tape: ControlledObservationTape) {
    runtime.set_feed_snapshot(tape.snapshot());
    let mut producer = ObservationProducer::tape(tape);
    loop {
        let batch = producer.poll(&[]).unwrap_or_default();
        if batch.is_empty() {
            break;
        }
        for sourced in batch {
            runtime.ingest_sourced(sourced);
        }
    }
    runtime.set_feed_snapshot(producer.snapshot());
}

/// Same tape interface, labeled `YAHOO_1M` (already-fetched or freshly fetched bars).
pub fn yahoo_1m_tape(
    ticker: &str,
    bars: &[CachedOhlcBar],
    date: Option<&str>,
    speed: f64,
) -> Result<super::live_observation::ControlledObservationTape, String> {
    use super::live_observation::{filter_ist_date, ControlledObservationTape, ObservationSourceKind};
    let mut obs = observations_from_cached_bars(ticker, bars);
    if let Some(date) = date {
        obs = filter_ist_date(obs, date);
    }
    if obs.is_empty() {
        return Err("no Yahoo 1m bars to emit".into());
    }
    Ok(ControlledObservationTape::new(
        ObservationSourceKind::Yahoo1m,
        obs,
        speed,
    ))
}

/// Env: `DEFERRED_LIVE_SOURCE=cached` + ticker/date/speed. Default is idle.
pub fn observation_producer_from_env() -> super::live_observation::ObservationProducer {
    use super::live_observation::{BackgroundSourceKind, FeedConfig, ObservationProducer};
    let cfg = FeedConfig::from_env();
    match cfg.kind {
        BackgroundSourceKind::Cached => {
            let Some(ticker) = cfg.ticker.clone() else {
                eprintln!("[deferred-live] DEFERRED_LIVE_SOURCE=cached requires DEFERRED_LIVE_TICKER");
                return ObservationProducer::none();
            };
            let Some(date) = cfg.date.clone() else {
                eprintln!("[deferred-live] DEFERRED_LIVE_SOURCE=cached requires DEFERRED_LIVE_DATE");
                return ObservationProducer::none();
            };
            let dir = cfg
                .cache_dir
                .clone()
                .unwrap_or_else(|| PathBuf::from("intraday_capture/yahoo_cache_1m"));
            match cached_1m_tape(&dir, &ticker, &date, cfg.speed) {
                Ok(tape) => ObservationProducer::tape(tape),
                Err(e) => {
                    eprintln!("[deferred-live] cached tape: {e}");
                    ObservationProducer::none()
                }
            }
        }
        _ => ObservationProducer::from_config(cfg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::product::intraday_decision::ExecutionFacts;
    use crate::product::LivePaperStatus;

    fn brief(id: &str, ticker: &str, dir: &str, entry: f64, reference: f64, exec: ExecutionFacts) -> DecisionBrief {
        DecisionBrief {
            id: id.into(),
            ticker: ticker.into(),
            date: "2026-09-14".into(),
            direction: dir.into(),
            oqs: 88,
            h60_class: "WAIT".into(),
            reference_price: Some(reference),
            entry_price: Some(entry),
            execution: exec,
            entry_state: "WAIT-MID".into(),
            entry_action: "MONITOR".into(),
            entry_confidence: "MODERATE".into(),
            entry_horizon: "H300".into(),
            entry_why: "test".into(),
            entry_risk: "test".into(),
            h120_state: "WAIT-MID".into(),
            h120_action: "CONTINUE".into(),
            h120_confidence: "MODERATE".into(),
            h120_horizon: "H300".into(),
            h120_why: "test".into(),
            h120_risk: "test".into(),
            h15_ret: None,
            h30_ret: None,
            h60_ret: None,
            h120_ret: None,
            h180_ret: None,
            h300_ret: None,
            mfe_h60: None,
            mfe_h120: None,
            outcome: None,
            pnl: None,
            hist_win: None,
            hist_pf: None,
            hist_med: None,
        }
    }

    #[test]
    fn ingest_without_arm_does_not_open() {
        let mut rt = DeferredLiveRuntime::new(DeferredLiveConfig::default());
        let out = rt.ingest(MarketObservation::last("JUBLFOOD_NS", 1, 470.04));
        assert!(!out.opened);
        assert!(rt.ledger().positions.is_empty());
    }

    #[test]
    fn first_live_tick_opens_at_observation_price_not_decision_entry() {
        let mut rt = DeferredLiveRuntime::new(DeferredLiveConfig::default());
        let b = brief(
            "LIVE-005-20260914-1000-JUBLFOOD_NS",
            "JUBLFOOD_NS",
            "SHORT",
            469.00,
            470.00,
            facts(467.30, 472.10, 1_000),
        );
        rt.arm(b.clone());
        let out = rt.ingest(MarketObservation::last("JUBLFOOD_NS", 1_000, 470.04));
        assert!(out.opened);
        assert!(out.open_error.is_none());
        let pos = &rt.ledger().positions[0];
        assert_eq!(pos.paper_entry_price(), 470.04);
        assert_ne!(pos.paper_entry_price(), b.entry_price.unwrap());
        assert_eq!(pos.paper.decision_id.as_deref(), Some(b.id.as_str()));
    }

    #[test]
    fn later_ticks_use_driver_lifecycle() {
        let mut rt = DeferredLiveRuntime::new(DeferredLiveConfig { horizon_secs: 60, strict_t0_admission: false, ..Default::default() });
        rt.arm(brief(
            "d1",
            "JUBLFOOD_NS",
            "SHORT",
            469.0,
            470.0,
            facts(467.30, 472.10, 1_000),
        ));
        rt.ingest(MarketObservation::last("JUBLFOOD_NS", 1_000, 470.04));
        rt.ingest(MarketObservation::last("JUBLFOOD_NS", 1_030, 468.82));
        assert_eq!(rt.ledger().positions[0].status(), LivePaperStatus::Open);
        rt.ingest(MarketObservation::last("JUBLFOOD_NS", 1_060, 468.50));
        assert_eq!(rt.ledger().positions[0].status(), LivePaperStatus::Horizon);
        assert_eq!(rt.ledger().positions[0].paper_exit_price(), Some(468.50));
        rt.ingest(MarketObservation::last("JUBLFOOD_NS", 1_090, 468.10));
        assert_eq!(rt.ledger().positions.len(), 1);
    }

    #[test]
    fn cached_bars_map_ns_ticker_and_ohlc() {
        let bars = vec![CachedOhlcBar {
            timestamp: 10,
            high: 102.0,
            low: 99.0,
            close: 100.5,
            volume: None,
        }];
        let obs = observations_from_cached_bars("JUBLFOOD.NS", &bars);
        assert_eq!(obs[0].ticker, "JUBLFOOD_NS");
        assert_eq!(obs[0].price, 100.5);
        assert_eq!(obs[0].high, Some(102.0));
        assert_eq!(obs[0].low, Some(99.0));
    }

    #[test]
    fn ingest_all_is_deterministic() {
        let run = || {
            let mut rt = DeferredLiveRuntime::new(DeferredLiveConfig { horizon_secs: 1_000, strict_t0_admission: false, ..Default::default() });
            rt.arm(brief("d1", "AAA_NS", "LONG", 99.0, 100.0, facts(102.0, 98.0, 0)));
            rt.ingest_all([
                MarketObservation::last("AAA_NS", 0, 100.0),
                MarketObservation::last("AAA_NS", 1, 100.2),
                MarketObservation {
                    ticker: "AAA_NS".into(),
                    unix: 2,
                    price: 98.1,
                    high: Some(98.2),
                    low: Some(97.9),
                },
            ]);
            rt.ledger().positions[0].clone()
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn cached_1m_ingest_opens_at_observation_price() {
        let mut rt = DeferredLiveRuntime::new(DeferredLiveConfig::default());
        rt.arm(brief(
            "d1",
            "AAA_NS",
            "LONG",
            99.0,
            100.0,
            facts(102.0, 98.0, 0),
        ));
        let out = rt.ingest_sourced(SourcedObservation::cached_1m(MarketObservation::last(
            "AAA_NS", 0, 100.5,
        )));
        assert!(out.opened);
        assert_eq!(rt.ledger().positions[0].paper_entry_price(), 100.5);
        assert_eq!(
            rt.feed().last_source_kind,
            Some(ObservationSourceKind::Cached1m)
        );
        assert_eq!(rt.feed().observation_source, ObservationSourceKind::Cached1m);
        assert_ne!(rt.ledger().positions[0].paper_entry_price(), 99.0);
    }

    #[test]
    fn yahoo_1m_tape_uses_same_ingest_path() {
        let mut rt = DeferredLiveRuntime::new(DeferredLiveConfig::default());
        rt.arm(brief("d1", "AAA_NS", "SHORT", 101.0, 100.0, facts(98.0, 102.0, 0)));
        let out = rt.ingest_sourced(SourcedObservation::yahoo_1m(MarketObservation::last(
            "AAA_NS", 1, 100.2,
        )));
        assert!(out.opened);
        assert_eq!(rt.ledger().positions[0].paper_entry_price(), 100.2);
        assert_eq!(
            rt.feed().observation_source,
            ObservationSourceKind::Yahoo1m
        );
    }

    #[test]
    fn http_ingest_records_external_live_provenance() {
        let mut rt = DeferredLiveRuntime::new(DeferredLiveConfig::default());
        rt.ingest(MarketObservation::last("AAA_NS", 5, 1.0));
        assert_eq!(
            rt.feed().last_source_kind,
            Some(ObservationSourceKind::ExternalLive)
        );
        assert_eq!(rt.feed().last_ingest_unix, Some(5));
    }

    #[test]
    fn cached_1m_jublfood_20260911_tape_is_deterministic() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../intraday_capture/yahoo_cache_1m");
        if !dir.join("JUBLFOOD.NS.json").exists() {
            return;
        }
        let mut a = cached_1m_tape(&dir, "JUBLFOOD_NS", "2026-09-11", 0.0).unwrap();
        let mut b = cached_1m_tape(&dir, "JUBLFOOD_NS", "2026-09-11", 0.0).unwrap();
        assert_eq!(a.len(), b.len());
        assert!(a.len() > 50);
        let first = a.next().unwrap();
        assert_eq!(first, b.next().unwrap());
        assert_eq!(first.source, ObservationSourceKind::Cached1m);
        assert_eq!(first.observation.ticker, "JUBLFOOD_NS");
        assert!(first.observation.price.is_finite() && first.observation.price > 0.0);
    }

    #[test]
    fn yahoo_1m_tape_is_same_ingest_interface() {
        let bars = [CachedOhlcBar {
            timestamp: 10,
            high: 2.0,
            low: 1.0,
            close: 1.5,
            volume: None,
        }];
        let mut tape = yahoo_1m_tape("AAA_NS", &bars, None, 0.0).unwrap();
        let obs = tape.next().unwrap();
        assert_eq!(obs.source, ObservationSourceKind::Yahoo1m);
        assert_eq!(obs.observation.price, 1.5);
        assert_eq!(obs.observation.ticker, "AAA_NS");
    }

    fn play_scenario(kind: LifecycleScenario) -> (crate::product::LivePaperPosition, ObservationFeedSnapshot) {
        let (brief, tape, cfg) = lifecycle_scenario_tape(kind, 0.0);
        let mut rt = DeferredLiveRuntime::new(cfg);
        rt.arm(brief);
        ingest_tape(&mut rt, tape);
        (rt.ledger().positions[0].clone(), rt.feed().clone())
    }

    #[test]
    fn producer_target_tape_exits_target_not_stop() {
        let (pos, feed) = play_scenario(LifecycleScenario::Target);
        assert_eq!(pos.paper_entry_price(), 480.5);
        assert_ne!(pos.paper_entry_price(), 473.0);
        assert_eq!(pos.status(), LivePaperStatus::Exited);
        assert_eq!(pos.exit_reason(), Some("TARGET"));
        assert_eq!(pos.paper_exit_price(), Some(475.0));
        assert_eq!(pos.events.last().map(|e| e.kind.as_str()), Some("PAPER_EXIT"));
        assert_eq!(pos.events.last().and_then(|e| e.note.as_deref()), Some("TARGET"));
        assert_eq!(feed.observation_source, ObservationSourceKind::Cached1m);
        assert_eq!(feed.status, crate::product::live_observation::ObservationFeedStatus::TapeExhausted);
    }

    #[test]
    fn producer_stop_tape_exits_stop() {
        let (pos, _) = play_scenario(LifecycleScenario::Stop);
        assert_eq!(pos.paper_entry_price(), 480.5);
        assert_eq!(pos.status(), LivePaperStatus::Exited);
        assert_eq!(pos.exit_reason(), Some("STOP"));
        assert_eq!(pos.paper_exit_price(), Some(485.0));
        assert_eq!(pos.events.last().and_then(|e| e.note.as_deref()), Some("STOP"));
    }

    #[test]
    fn producer_horizon_tape_exits_horizon_at_last_not_barrier() {
        let (pos, _) = play_scenario(LifecycleScenario::Horizon);
        assert_eq!(pos.paper_entry_price(), 480.5);
        assert_eq!(pos.status(), LivePaperStatus::Horizon);
        assert_eq!(pos.exit_reason(), Some("HORIZON"));
        assert_eq!(pos.paper_exit_price(), Some(475.4));
        assert_eq!(pos.events.last().map(|e| e.kind.as_str()), Some("HORIZON"));
        assert_eq!(pos.current_price, 475.4);
    }

    #[test]
    fn jublfood_next_session_bar_reaches_horizon() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../intraday_capture/yahoo_cache_1m");
        let dataset = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../datasets/p4_opportunity_dataset.json");
        if !dir.join("JUBLFOOD.NS.json").exists() || !dataset.exists() {
            return;
        }
        let briefs = crate::product::load_intraday_briefs(dataset.to_str().unwrap()).unwrap();
        let brief = briefs
            .into_iter()
            .find(|b| b.id == "LIVE-005-20260907-1000-JUBLFOOD_NS")
            .expect("JUBLFOOD 2026-09-07 brief");
        let snap = brief.execution.snap_unix.expect("snap_unix");
        let until = snap + 300 * 60;
        let tape = cached_1m_through_unix(&dir, "JUBLFOOD_NS", "2026-09-07", until, 0.0).unwrap();
        assert!(tape.len() > 359, "session plus first post-horizon bar");
        let mut rt = DeferredLiveRuntime::new(DeferredLiveConfig::default());
        rt.arm(brief);
        ingest_tape(&mut rt, tape);
        let pos = &rt.ledger().positions[0];
        assert_eq!(pos.paper_entry_price(), 480.5);
        assert_eq!(pos.status(), LivePaperStatus::Horizon);
        assert_eq!(pos.exit_reason(), Some("HORIZON"));
        assert!((pos.paper_exit_price().unwrap() - 474.8999938964844).abs() < 1e-6);
        assert_eq!(rt.ledger().positions.len(), 1);
    }

    #[test]
    fn tcs_cached_1m_hits_stop() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../intraday_capture/yahoo_cache_1m");
        let dataset = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../datasets/p4_opportunity_dataset.json");
        if !dir.join("TCS.NS.json").exists() || !dataset.exists() {
            return;
        }
        let briefs = crate::product::load_intraday_briefs(dataset.to_str().unwrap()).unwrap();
        let brief = briefs
            .into_iter()
            .find(|b| b.id == "LIVE-005-20260904-1000-TCS_NS")
            .expect("TCS 2026-09-04 brief");
        let tape = cached_1m_tape(&dir, "TCS_NS", "2026-09-04", 0.0).unwrap();
        let mut rt = DeferredLiveRuntime::new(DeferredLiveConfig::default());
        rt.arm(brief);
        ingest_tape(&mut rt, tape);
        let pos = &rt.ledger().positions[0];
        assert_eq!(pos.paper_entry_price(), 2346.0);
        assert_eq!(pos.status(), LivePaperStatus::Exited);
        assert_eq!(pos.exit_reason(), Some("STOP"));
        assert!((pos.paper_exit_price().unwrap() - 2363.10).abs() < 0.01);
        assert_eq!(rt.ledger().positions.len(), 1);
    }

    fn act_brief(id: &str, ticker: &str, date: &str, target: f64, risk: f64, snap: i64) -> DecisionBrief {
        let mut b = brief(id, ticker, "SHORT", 473.0, 475.0, facts(target, risk, snap));
        b.entry_action = "ACT".into();
        b.date = date.into();
        b
    }

    #[test]
    fn session_auto_arm_skips_monitor_and_opens_act_on_first_observation() {
        let date = "2026-09-07";
        let act = act_brief("act-1", "AAA_NS", date, 450.0, 490.0, 1_000);
        let mut monitor = brief("mon-1", "BBB_NS", "SHORT", 99.0, 100.0, facts(95.0, 105.0, 1_000));
        monitor.date = date.into();
        monitor.entry_action = "MONITOR".into();
        let mut rt = DeferredLiveRuntime::new(DeferredLiveConfig::default());
        let armed = rt.begin_session(&[act, monitor], date);
        assert_eq!(armed, ["act-1"]);
        assert_eq!(rt.armed().len(), 1);
        rt.ingest(MarketObservation::last("AAA_NS", 1_000, 480.5));
        rt.ingest(MarketObservation::last("BBB_NS", 1_000, 100.0));
        assert_eq!(rt.ledger().positions.len(), 1);
        assert_eq!(rt.ledger().positions[0].paper_entry_price(), 480.5);
        assert_eq!(rt.ledger().positions[0].paper.ticker, "AAA_NS");
        rt.begin_session(
            &[act_brief("act-1", "AAA_NS", date, 450.0, 490.0, 1_000)],
            date,
        );
        assert!(rt.armed().is_empty());
        assert_eq!(rt.ledger().positions.len(), 1);
    }

    #[test]
    fn session_two_tickers_maintain_book_without_manual_arm() {
        let date = "2026-09-07";
        let a = act_brief("a", "AAA_NS", date, 450.0, 490.0, 10);
        let b = act_brief("b", "BBB_NS", date, 90.0, 110.0, 10);
        let mut rt = DeferredLiveRuntime::new(DeferredLiveConfig::default());
        rt.begin_session(&[a, b], date);
        rt.ingest(MarketObservation::last("BBB_NS", 10, 100.0));
        rt.ingest(MarketObservation::last("AAA_NS", 10, 480.5));
        assert_eq!(rt.ledger().positions.len(), 2);
        let mut tickers: Vec<_> = rt
            .ledger()
            .positions
            .iter()
            .map(|p| p.paper.ticker.as_str())
            .collect();
        tickers.sort();
        assert_eq!(tickers, ["AAA_NS", "BBB_NS"]);
        assert!(rt.armed().is_empty());
        assert_eq!(rt.session().map(|s| s.auto_arm), Some(true));
    }

    #[test]
    fn session_auto_arm_opens_on_observation_before_snap_unix() {
        // Implemented contract (CS-P-001-F): join is calendar date + first
        // matching tick. There is no snap_unix gate. Fill may precede snap.
        // Do not treat this test as a vote to add a gate.
        let date = "2026-09-07";
        let snap = 1_788_775_200; // 15:30 IST
        let fill_unix = 1_788_752_700; // 09:15 IST
        let act = act_brief("act-1", "AAA_NS", date, 450.0, 490.0, snap);
        let mut rt = DeferredLiveRuntime::new(DeferredLiveConfig::default());
        rt.begin_session(&[act], date);
        rt.ingest(MarketObservation::last("AAA_NS", fill_unix, 480.5));
        assert_eq!(rt.ledger().positions.len(), 1);
        assert_eq!(rt.ledger().positions[0].paper_entry_price(), 480.5);
        assert_eq!(rt.ledger().positions[0].opened_at, fill_unix);
        assert!(fill_unix < snap);
    }

    #[test]
    fn cached_session_tape_is_ordered_by_unix_then_ticker() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../intraday_capture/yahoo_cache_1m");
        if !dir.join("JUBLFOOD.NS.json").exists() || !dir.join("TCS.NS.json").exists() {
            return;
        }
        let tickers = ["JUBLFOOD_NS".into(), "TCS_NS".into()];
        let mut tape = cached_session_tape(&dir, &tickers, "2026-09-07", 0.0).unwrap();
        let mut prev = (i64::MIN, String::new());
        let mut n = 0;
        while let Some(s) = tape.next() {
            assert_eq!(s.source, ObservationSourceKind::Cached1m);
            let key = (s.observation.unix, s.observation.ticker.clone());
            assert!(key >= prev);
            prev = key;
            n += 1;
        }
        assert!(n > 400);
    }
}
