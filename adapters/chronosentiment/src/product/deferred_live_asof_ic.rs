//! Increment 2 — As-of IC assembler sidecar.
//!
//! Assembles an IC-legal [`DecisionBrief`] once H60 evidence exists on a tape
//! prefix. Does not mutate [`super::deferred_live::DeferredLiveDriver`], fill
//! semantics, H, D/E/G, reassessment, or INVERT.
//!
//! Live JSONL / HTTP and cached session tapes share [`AsOfSessionDriver`]:
//!
//! ```text
//! LIVE MARKET OBSERVATION
//!         ↓
//! DecisionLoopRuntime
//!         ↓
//! DeferredLiveRuntime.ingest
//!         ↓
//! AsOfIcAssembler
//!         ↓
//! DecisionSurface
//!         ↓
//! Cockpit
//! ```
//!
//! Earliest same-session point matches P4 replay: 12 session bars (~10:15 IST),
//! not 09:32. Direction and T0 geometry are copied from a frozen Watch fixture.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::deferred_live::{DeferredLiveConfig, MarketObservation};
use super::deferred_live_decision_loop::{DecisionLoopRuntime, DecisionSurface};
use super::deferred_live_loop::cached_session_tape;
use super::deferred_live_session::{eligible_for_auto_arm, select_session_briefs};
use super::intraday_decision::{DecisionBrief, ExecutionFacts};
use super::live_observation::SourcedObservation;
use crate::reasoning::intraday_classification::{
    classify_at_entry, resolve_action, Checkpoint, Direction, EntryInput, H60Classification,
};

/// P4 replay H60 bar index (session bar 12). Not 60 one-minute bars.
pub const H60_BAR_COUNT: usize = 12;

/// Frozen Watch inputs. Prices are copied, never invented.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchFixture {
    pub ticker: String,
    pub date: String,
    pub direction: String,
    pub reference_price: f64,
    pub entry_price: f64,
    pub adaptive_target: f64,
    pub adaptive_risk: f64,
    pub t0_entry_action: String,
}

impl WatchFixture {
    /// Copy frozen Watch / LIVE-005 fields. None if geometry or prices are missing.
    pub fn try_from_brief(brief: &DecisionBrief) -> Option<Self> {
        Some(Self {
            ticker: brief.ticker.clone(),
            date: brief.date.clone(),
            direction: brief.direction.clone(),
            reference_price: brief.reference_price.filter(|v| v.is_finite() && *v > 0.0)?,
            entry_price: brief.entry_price.filter(|v| v.is_finite() && *v > 0.0)?,
            adaptive_target: brief.execution.adaptive_target.filter(|v| v.is_finite() && *v > 0.0)?,
            adaptive_risk: brief.execution.adaptive_risk.filter(|v| v.is_finite() && *v > 0.0)?,
            t0_entry_action: brief.entry_action.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AsOfPathFeatures {
    pub h15_ret: Option<f64>,
    pub h30_ret: Option<f64>,
    pub h60_ret: Option<f64>,
    pub mfe_h60: Option<f64>,
    pub mae_h60: Option<f64>,
    pub momentum_persistence: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AsOfOffer {
    Armed,
    NotReady,
    NotAct,
    WithheldOpen,
    WithheldArmed,
    WithheldForCapacity,
}

/// Sidecar assembler. Existing `classify_at_entry` remains authoritative.
pub struct AsOfIcAssembler;

impl AsOfIcAssembler {
    pub fn assemble(watch: &WatchFixture, prefix: &[MarketObservation]) -> Option<DecisionBrief> {
        let bars = ticker_prefix(watch.ticker.as_str(), prefix);
        if bars.len() < H60_BAR_COUNT {
            return None;
        }
        let as_of = bars[H60_BAR_COUNT - 1].unix;
        let features = path_features(&watch.direction, watch.entry_price, &bars[..H60_BAR_COUNT]);
        let h60_class = classify_h60(&watch.direction, features.h60_ret);
        let oqs = compute_time_safe_oqs(
            &watch.direction,
            features.h15_ret,
            features.h60_ret,
            features.mfe_h60,
            features.mae_h60,
            features.momentum_persistence,
        );
        let direction = parse_direction(&watch.direction);
        let h60_enum = parse_h60(&h60_class);
        let entry_state = classify_at_entry(&EntryInput {
            direction,
            opportunity_quality_score: oqs,
            h60_classification: h60_enum,
        });
        let entry_action = resolve_action(Checkpoint::Entry, entry_state, None);
        Some(DecisionBrief {
            id: format!("ASOF-{}-{}-{}", watch.date, as_of, watch.ticker),
            ticker: watch.ticker.clone(),
            date: watch.date.clone(),
            direction: watch.direction.clone(),
            oqs,
            h60_class,
            reference_price: Some(watch.reference_price),
            entry_price: Some(watch.entry_price),
            execution: ExecutionFacts {
                snap_unix: Some(as_of),
                adaptive_target: Some(watch.adaptive_target),
                adaptive_risk: Some(watch.adaptive_risk),
                adaptive_horizon_sessions: Some(1.0),
                target_distance_abs: None,
                target_distance_pct: None,
                risk_distance_abs: None,
                risk_distance_pct: None,
                expected_move_pct: None,
                current_price: None,
                last_tick_unix: Some(as_of),
                freshness: "CACHED_1M".into(),
                horizon_elapsed: Some(false),
            },
            entry_state: entry_state.label().to_string(),
            entry_action: entry_action.action.label().to_string(),
            entry_confidence: entry_action.confidence.label().to_string(),
            entry_horizon: entry_action.horizon.to_string(),
            entry_why: entry_action.why.to_string(),
            entry_risk: entry_action.risk.to_string(),
            // Entry classification only. Do not reassess_at_h120.
            h120_state: entry_state.label().to_string(),
            h120_action: entry_action.action.label().to_string(),
            h120_confidence: entry_action.confidence.label().to_string(),
            h120_horizon: entry_action.horizon.to_string(),
            h120_why: "As-of IC entry; H120 reassessment not applied.".into(),
            h120_risk: entry_action.risk.to_string(),
            h15_ret: features.h15_ret,
            h30_ret: features.h30_ret,
            h60_ret: features.h60_ret,
            h120_ret: None,
            h180_ret: None,
            h300_ret: None,
            mfe_h60: features.mfe_h60,
            mfe_h120: None,
            outcome: None,
            pnl: None,
            hist_win: None,
            hist_pf: None,
            hist_med: None,
        })
    }
}

/// Assemble from the prefix, then offer. `NotReady` until H60 bars exist.
pub fn offer_asof_from_prefix(
    loop_rt: &mut DecisionLoopRuntime,
    watch: &WatchFixture,
    prefix: &[MarketObservation],
) -> AsOfOffer {
    match AsOfIcAssembler::assemble(watch, prefix) {
        None => AsOfOffer::NotReady,
        Some(brief) => offer_asof_brief(loop_rt, brief),
    }
}

/// Arm a new as-of brief only when the ticker is not already OPEN or armed.
pub fn offer_asof_brief(loop_rt: &mut DecisionLoopRuntime, brief: DecisionBrief) -> AsOfOffer {
    let ticker = brief.ticker.as_str();
    if loop_rt
        .runtime()
        .armed()
        .iter()
        .any(|a| a.ticker == ticker)
    {
        return AsOfOffer::WithheldArmed;
    }
    if loop_rt
        .runtime()
        .ledger()
        .positions
        .iter()
        .any(|p| p.paper.ticker == ticker)
    {
        return AsOfOffer::WithheldOpen;
    }
    if eligible_for_auto_arm(&brief).is_err() {
        return AsOfOffer::NotAct;
    }
    loop_rt.arm(brief);
    AsOfOffer::Armed
}

/// Frozen ACT tickers plus as-of Watch names for a session tape.
pub fn session_tape_tickers(briefs: &[DecisionBrief], date: &str) -> Vec<String> {
    let frozen = select_session_briefs(briefs, date);
    let watches = watches_for_date(briefs, date);
    let mut tickers: Vec<String> = frozen.iter().map(|b| b.ticker.clone()).collect();
    for w in &watches {
        if !tickers.iter().any(|t| t == &w.ticker) {
            tickers.push(w.ticker.clone());
        }
    }
    tickers.sort();
    tickers.dedup();
    tickers
}

/// Shared ingest coordinator. Live and cached observations use the same path:
/// `MarketObservation` → as-of offer at H60 → [`DecisionLoopRuntime::step_sourced`].
/// Does not rewrite [`super::deferred_live::DeferredLiveDriver`].
pub struct AsOfSessionDriver {
    loop_rt: DecisionLoopRuntime,
    watches: HashMap<String, WatchFixture>,
    bars_by_ticker: HashMap<String, Vec<MarketObservation>>,
    offered: HashSet<String>,
    asof_events: Vec<AsOfSessionEvent>,
    observation_count: usize,
    strict_t0_admission: bool,
}

impl AsOfSessionDriver {
    pub fn new(config: DeferredLiveConfig) -> Self {
        Self {
            strict_t0_admission: config.strict_t0_admission,
            loop_rt: DecisionLoopRuntime::new(config),
            watches: HashMap::new(),
            bars_by_ticker: HashMap::new(),
            offered: HashSet::new(),
            asof_events: Vec::new(),
            observation_count: 0,
        }
    }

    /// Frozen auto-arm + as-of Watch fixtures for `date`. Safe to call once at start.
    pub fn install_session(&mut self, briefs: &[DecisionBrief], date: &str) -> Vec<String> {
        self.watches = watches_for_date(briefs, date)
            .into_iter()
            .map(|w| (w.ticker.clone(), w))
            .collect();
        self.loop_rt.begin_session(briefs, date)
    }

    pub fn loop_rt(&self) -> &DecisionLoopRuntime {
        &self.loop_rt
    }

    pub fn arm(&mut self, brief: DecisionBrief) {
        self.loop_rt.arm(brief);
    }

    pub fn begin_session(&mut self, briefs: &[DecisionBrief], date: &str) -> Vec<String> {
        self.loop_rt.begin_session(briefs, date)
    }

    pub fn enable_reassess_experiment(&mut self) {
        self.loop_rt.enable_reassess_experiment();
    }

    pub fn set_feed_snapshot(&mut self, snap: super::live_observation::ObservationFeedSnapshot) {
        self.loop_rt.set_feed_snapshot(snap);
    }

    pub fn mark_tape_exhausted(&mut self) {
        self.loop_rt.mark_tape_exhausted();
    }

    pub fn asof_events(&self) -> &[AsOfSessionEvent] {
        &self.asof_events
    }

    pub fn observation_count(&self) -> usize {
        self.observation_count
    }

    pub fn last_surface(&self) -> Option<&DecisionSurface> {
        self.loop_rt.surfaces().last()
    }

    pub fn ledger(&self) -> &super::deferred_live::LivePaperLedger {
        self.loop_rt.runtime().ledger()
    }

    pub fn feed(&self) -> &super::live_observation::ObservationFeedSnapshot {
        self.loop_rt.runtime().feed()
    }

    pub fn session(&self) -> Option<&super::deferred_live_session::SessionState> {
        self.loop_rt.runtime().session()
    }

    pub fn armed(&self) -> Vec<super::deferred_live_loop::ArmedBrief> {
        self.loop_rt.runtime().armed()
    }

    pub fn performance(
        &self,
        briefs: &[DecisionBrief],
    ) -> super::deferred_live_performance::DeferredLivePerformance {
        self.loop_rt.runtime().performance(briefs)
    }

    pub fn reassess_experiment_report(
        &self,
    ) -> super::live_reassess_experiment::ReassessExperimentReport {
        self.loop_rt.runtime().reassess_experiment_report()
    }

    pub fn watched_tickers(&self) -> Vec<String> {
        let mut t = self.loop_rt.runtime().watched_tickers();
        for w in self.watches.keys() {
            if !t.iter().any(|x| x == w) {
                t.push(w.clone());
            }
        }
        t.sort();
        t
    }

    /// Live HTTP / JSONL ticks. Provenance is `EXTERNAL_LIVE`; ingest is unchanged.
    pub fn ingest(&mut self, obs: MarketObservation) -> DecisionSurface {
        self.ingest_sourced(SourcedObservation::external_live(obs))
    }

    pub fn ingest_sourced(&mut self, sourced: SourcedObservation) -> DecisionSurface {
        self.observation_count += 1;
        let ticker = sourced.observation.ticker.clone();
        let bars = self.bars_by_ticker.entry(ticker.clone()).or_default();
        if bars.len() < H60_BAR_COUNT {
            bars.push(sourced.observation.clone());
            if bars.len() == H60_BAR_COUNT && self.offered.insert(ticker.clone()) {
                if let Some(watch) = self.watches.get(&ticker) {
                    if let Some(mut brief) = AsOfIcAssembler::assemble(watch, bars) {
                        brief.execution.freshness = sourced.source.freshness().to_string();
                        let as_of_unix = brief
                            .execution
                            .snap_unix
                            .unwrap_or(sourced.observation.unix);
                        let decision_id = brief.id.clone();
                        let entry_action = brief.entry_action.clone();
                        
                        let offer = if self.strict_t0_admission && watch.t0_entry_action != "ACT" && brief.entry_action == "ACT" {
                            AsOfOffer::NotAct
                        } else {
                            offer_asof_brief(&mut self.loop_rt, brief)
                        };

                        self.asof_events.push(AsOfSessionEvent {
                            ticker,
                            as_of_unix,
                            offer,
                            decision_id,
                            entry_action,
                        });
                        const KEEP: usize = 1024;
                        if self.asof_events.len() > KEEP {
                            let drop = self.asof_events.len() - KEEP;
                            self.asof_events.drain(0..drop);
                        }
                    }
                }
            }
        }
        self.loop_rt.step_sourced(sourced)
    }

    pub fn snapshot_report(&self, date: &str, frozen_armed_ids: Vec<String>) -> CachedSessionRun {
        let last = self.last_surface();
        let mut ledger_decision_ids: Vec<String> = self
            .loop_rt
            .runtime()
            .ledger()
            .positions
            .iter()
            .filter_map(|p| p.paper.decision_id.clone())
            .collect();
        ledger_decision_ids.sort();
        let asof_armed_ids: Vec<String> = self
            .asof_events
            .iter()
            .filter(|e| e.offer == AsOfOffer::Armed)
            .map(|e| e.decision_id.clone())
            .collect();
        CachedSessionRun {
            date: date.into(),
            observations: self.observation_count,
            frozen_armed_ids,
            asof_events: self.asof_events.clone(),
            asof_armed_ids,
            open: last.map(|s| s.open).unwrap_or(0),
            exited: last.map(|s| s.exited).unwrap_or(0),
            horizon: last.map(|s| s.horizon).unwrap_or(0),
            ledger_decision_ids,
            feed_status: format!("{:?}", self.loop_rt.runtime().feed().status),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AsOfSessionEvent {
    pub ticker: String,
    pub as_of_unix: i64,
    pub offer: AsOfOffer,
    pub decision_id: String,
    pub entry_action: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CachedSessionRun {
    pub date: String,
    pub observations: usize,
    pub frozen_armed_ids: Vec<String>,
    pub asof_events: Vec<AsOfSessionEvent>,
    pub asof_armed_ids: Vec<String>,
    pub open: usize,
    pub exited: usize,
    pub horizon: usize,
    pub ledger_decision_ids: Vec<String>,
    pub feed_status: String,
}

/// Frozen session auto-arm + as-of IC offers on a real cached tape.
/// Same ingest coordinator as live JSONL / HTTP. Driver semantics unchanged.
pub fn run_cached_session(
    briefs: &[DecisionBrief],
    date: &str,
    cache_dir: &Path,
    speed: f64,
) -> Result<CachedSessionRun, String> {
    let frozen_armed_ids: Vec<String> = select_session_briefs(briefs, date)
        .into_iter()
        .map(|b| b.id)
        .collect();
    let tickers = session_tape_tickers(briefs, date);
    let mut tape = cached_session_tape(cache_dir, &tickers, date, speed)?;
    let mut driver = AsOfSessionDriver::new(DeferredLiveConfig::default());
    driver.install_session(briefs, date);
    while let Some(sourced) = tape.next() {
        driver.ingest_sourced(sourced);
    }
    driver.mark_tape_exhausted();
    Ok(driver.snapshot_report(date, frozen_armed_ids))
}

fn watches_for_date(briefs: &[DecisionBrief], date: &str) -> Vec<WatchFixture> {
    let mut candidates: Vec<&DecisionBrief> = briefs.iter().filter(|b| b.date == date).collect();
    candidates.sort_by(|a, b| {
        a.ticker
            .cmp(&b.ticker)
            .then(b.oqs.cmp(&a.oqs))
            .then(a.id.cmp(&b.id))
    });
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for brief in candidates {
        if !seen.insert(brief.ticker.clone()) {
            continue;
        }
        if let Some(watch) = WatchFixture::try_from_brief(brief) {
            out.push(watch);
        }
    }
    out
}

/// Port of `scripts/p4_sep8_replay.py::classify_h60`.
pub fn classify_h60(direction: &str, h60_ret: Option<f64>) -> String {
    let Some(h60_ret) = h60_ret else {
        return "WAIT".into();
    };
    let signed = if direction.eq_ignore_ascii_case("LONG") {
        h60_ret
    } else {
        -h60_ret
    };
    if signed > 0.003 {
        "ENTER".into()
    } else if signed > -0.003 {
        "WAIT".into()
    } else {
        "AVOID".into()
    }
}

/// Port of `scripts/p4_sep8_replay.py::compute_time_safe_oqs`.
pub fn compute_time_safe_oqs(
    direction: &str,
    h15_ret: Option<f64>,
    _h60_ret: Option<f64>,
    mfe_h60: Option<f64>,
    mae_h60: Option<f64>,
    momentum_persistence: Option<f64>,
) -> u32 {
    let mut score = 0i32;
    if let Some(h15_ret) = h15_ret {
        let signed = if direction.eq_ignore_ascii_case("LONG") {
            h15_ret
        } else {
            -h15_ret
        };
        if signed > 0.005 {
            score += 25;
        } else if signed > 0.002 {
            score += 18;
        } else if signed > 0.0 {
            score += 10;
        } else if signed > -0.002 {
            score += 4;
        }
    }
    if let Some(mfe_h60) = mfe_h60 {
        if mfe_h60 > 0.010 {
            score += 25;
        } else if mfe_h60 > 0.005 {
            score += 18;
        } else if mfe_h60 > 0.002 {
            score += 12;
        } else if mfe_h60 > 0.0 {
            score += 6;
        }
    }
    if let Some(mp) = momentum_persistence {
        if mp >= 0.75 {
            score += 25;
        } else if mp >= 0.60 {
            score += 18;
        } else if mp >= 0.45 {
            score += 10;
        } else if mp >= 0.30 {
            score += 4;
        }
    }
    if let Some(mae_h60) = mae_h60 {
        let abs_mae = mae_h60.abs();
        if abs_mae < 0.002 {
            score += 25;
        } else if abs_mae < 0.005 {
            score += 18;
        } else if abs_mae < 0.010 {
            score += 10;
        } else if abs_mae < 0.015 {
            score += 4;
        }
    }
    score.clamp(0, 100) as u32
}

fn ticker_prefix(ticker: &str, prefix: &[MarketObservation]) -> Vec<MarketObservation> {
    let mut bars: Vec<MarketObservation> = prefix
        .iter()
        .filter(|o| o.ticker == ticker)
        .cloned()
        .collect();
    bars.sort_by_key(|o| o.unix);
    bars
}

fn path_features(direction: &str, entry_price: f64, session_bars: &[MarketObservation]) -> AsOfPathFeatures {
    let signed_ret = |close: f64| {
        if direction.eq_ignore_ascii_case("LONG") {
            (close - entry_price) / entry_price
        } else {
            (entry_price - close) / entry_price
        }
    };
    let ret_at = |n: usize| {
        session_bars
            .get(n.saturating_sub(1))
            .filter(|_| session_bars.len() >= n)
            .map(|b| signed_ret(b.price))
    };
    let h60_bars = if session_bars.len() >= H60_BAR_COUNT {
        &session_bars[..H60_BAR_COUNT]
    } else {
        session_bars
    };
    let rets: Vec<f64> = h60_bars.iter().map(|b| signed_ret(b.price)).collect();
    let (mfe_h60, mae_h60, momentum_persistence) = if rets.is_empty() {
        (None, None, None)
    } else {
        let mfe = rets.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let mae = rets.iter().cloned().fold(f64::INFINITY, f64::min);
        let in_dir = rets.iter().filter(|r| **r > 0.0).count() as f64;
        (
            Some(mfe),
            Some(mae),
            Some(in_dir / rets.len() as f64),
        )
    };
    AsOfPathFeatures {
        h15_ret: ret_at(3),
        h30_ret: ret_at(6),
        h60_ret: ret_at(12),
        mfe_h60,
        mae_h60,
        momentum_persistence,
    }
}

fn parse_direction(s: &str) -> Direction {
    if s.eq_ignore_ascii_case("SHORT") {
        Direction::Short
    } else {
        Direction::Long
    }
}

fn parse_h60(s: &str) -> H60Classification {
    match s {
        "ENTER" => H60Classification::Enter,
        "AVOID" => H60Classification::Avoid,
        _ => H60Classification::Wait,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::product::deferred_live::DeferredLiveConfig;
    use crate::product::intraday_decision::ExecutionFacts;
    use crate::product::live_observation::ObservationSourceKind;
    use crate::product::LivePaperStatus;

    fn watch_long() -> WatchFixture {
        WatchFixture {
            ticker: "AAA_NS".into(),
            date: "2026-09-07".into(),
            direction: "LONG".into(),
            reference_price: 100.0,
            entry_price: 100.0,
            adaptive_target: 102.0,
            adaptive_risk: 98.0,
            t0_entry_action: "MONITOR".into(),
        }
    }

    fn rising_prefix(n: usize) -> Vec<MarketObservation> {
        (0..n)
            .map(|i| MarketObservation {
                ticker: "AAA_NS".into(),
                unix: 1_000_000 + (i as i64) * 60,
                price: 100.0 + (i as f64) * 0.2,
                high: None,
                low: None,
            })
            .collect()
    }

    fn frozen_open_brief() -> DecisionBrief {
        watch_brief("LIVE-OPEN-AAA_NS", "ACT")
    }

    fn wait_watch_brief() -> DecisionBrief {
        watch_brief("WATCH-AAA_NS", "WAIT")
    }

    fn watch_brief(id: &str, entry_action: &str) -> DecisionBrief {
        DecisionBrief {
            id: id.into(),
            ticker: "AAA_NS".into(),
            date: "2026-09-07".into(),
            direction: "LONG".into(),
            oqs: 80,
            h60_class: "ENTER".into(),
            reference_price: Some(100.0),
            entry_price: Some(100.0),
            execution: ExecutionFacts {
                snap_unix: Some(1_000_000),
                adaptive_target: Some(102.0),
                adaptive_risk: Some(98.0),
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
            },
            entry_state: "ENTER".into(),
            entry_action: entry_action.into(),
            entry_confidence: "HIGH".into(),
            entry_horizon: "H300".into(),
            entry_why: "frozen".into(),
            entry_risk: "frozen".into(),
            h120_state: "ENTER".into(),
            h120_action: "ACT".into(),
            h120_confidence: "HIGH".into(),
            h120_horizon: "H300".into(),
            h120_why: "frozen".into(),
            h120_risk: "frozen".into(),
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
    fn classify_h60_matches_python() {
        assert_eq!(classify_h60("LONG", None), "WAIT");
        assert_eq!(classify_h60("LONG", Some(0.004)), "ENTER");
        assert_eq!(classify_h60("LONG", Some(0.0)), "WAIT");
        assert_eq!(classify_h60("LONG", Some(-0.004)), "AVOID");
        assert_eq!(classify_h60("SHORT", Some(-0.004)), "ENTER");
        assert_eq!(classify_h60("SHORT", Some(0.004)), "AVOID");
    }

    #[test]
    fn time_safe_oqs_matches_python_components() {
        let oqs = compute_time_safe_oqs(
            "LONG",
            Some(0.006),
            Some(0.004),
            Some(0.011),
            Some(-0.001),
            Some(0.80),
        );
        assert_eq!(oqs, 100);
        let mid = compute_time_safe_oqs(
            "LONG",
            Some(0.003),
            None,
            Some(0.006),
            Some(-0.006),
            Some(0.50),
        );
        assert_eq!(mid, 18 + 18 + 10 + 10);
    }

    #[test]
    fn assembler_waits_until_h60_bar_count() {
        let watch = watch_long();
        assert!(AsOfIcAssembler::assemble(&watch, &rising_prefix(11)).is_none());
        let brief = AsOfIcAssembler::assemble(&watch, &rising_prefix(12)).expect("H60 ready");
        assert_eq!(brief.id, "ASOF-2026-09-07-1000660-AAA_NS");
        assert_eq!(brief.execution.snap_unix, Some(1_000_000 + 11 * 60));
        assert_eq!(brief.h120_state, brief.entry_state);
        assert_eq!(brief.h120_ret, None);
        assert_eq!(brief.reference_price, Some(100.0));
        assert_eq!(brief.entry_price, Some(100.0));
        assert_eq!(brief.h60_class, "ENTER");
        assert_eq!(brief.entry_action, "ACT");
    }

    #[test]
    fn offer_from_prefix_not_ready_before_h60() {
        let mut loop_rt = DecisionLoopRuntime::new(DeferredLiveConfig::default());
        assert_eq!(
            offer_asof_from_prefix(&mut loop_rt, &watch_long(), &rising_prefix(11)),
            AsOfOffer::NotReady
        );
        assert!(loop_rt.runtime().armed().is_empty());
    }

    #[test]
    fn offer_arms_when_ticker_free() {
        let mut loop_rt = DecisionLoopRuntime::new(DeferredLiveConfig::default());
        let brief = AsOfIcAssembler::assemble(&watch_long(), &rising_prefix(12)).unwrap();
        assert_eq!(offer_asof_brief(&mut loop_rt, brief), AsOfOffer::Armed);
        assert_eq!(loop_rt.runtime().armed().len(), 1);
    }

    #[test]
    fn offer_withholds_when_ticker_armed() {
        let mut loop_rt = DecisionLoopRuntime::new(DeferredLiveConfig::default());
        loop_rt.arm(frozen_open_brief());
        let brief = AsOfIcAssembler::assemble(&watch_long(), &rising_prefix(12)).unwrap();
        assert_eq!(offer_asof_brief(&mut loop_rt, brief), AsOfOffer::WithheldArmed);
        assert_eq!(loop_rt.runtime().armed().len(), 1);
        assert_eq!(loop_rt.runtime().armed()[0].decision_id, "LIVE-OPEN-AAA_NS");
    }

    #[test]
    fn offer_withholds_when_ticker_open_no_invert() {
        let mut loop_rt = DecisionLoopRuntime::new(DeferredLiveConfig::default());
        loop_rt.arm(frozen_open_brief());
        loop_rt.step(MarketObservation::last("AAA_NS", 1_000_000, 100.0));
        assert_eq!(loop_rt.runtime().ledger().positions[0].status(), LivePaperStatus::Open);
        assert!(loop_rt.runtime().armed().is_empty());
        let brief = AsOfIcAssembler::assemble(&watch_long(), &rising_prefix(12)).unwrap();
        assert_eq!(offer_asof_brief(&mut loop_rt, brief), AsOfOffer::WithheldOpen);
        assert_eq!(loop_rt.runtime().ledger().positions.len(), 1);
        assert_eq!(
            loop_rt.runtime().ledger().positions[0]
                .paper
                .decision_id
                .as_deref(),
            Some("LIVE-OPEN-AAA_NS")
        );
    }

    #[test]
    fn cached_session_20260907_keeps_frozen_book_and_offers_asof() {
        use std::collections::HashSet;
        use std::path::Path;
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let dataset = root.join("datasets/p4_opportunity_dataset.json");
        let cache = root.join("intraday_capture/yahoo_cache_1m");
        if !dataset.exists() || !cache.join("JUBLFOOD.NS.json").exists() {
            return;
        }
        let briefs = crate::product::load_intraday_briefs(dataset.to_str().unwrap()).unwrap();
        let run = run_cached_session(&briefs, "2026-09-07", &cache, 0.0).expect("cached session");
        assert!(run.observations > 12, "session tape should exceed H60");
        assert!(
            run.feed_status.contains("TapeExhausted"),
            "feed_status={}",
            run.feed_status
        );
        let jubl = "LIVE-005-20260907-1000-JUBLFOOD_NS";
        if run.frozen_armed_ids.iter().any(|id| id == jubl) {
            assert!(
                run.ledger_decision_ids.iter().any(|id| id == jubl),
                "frozen JUBLFOOD must remain on the live book"
            );
            assert!(
                !run.asof_armed_ids.iter().any(|id| id.contains("JUBLFOOD")),
                "as-of must not replace frozen JUBLFOOD"
            );
        }
        for ev in run.asof_events.iter().filter(|e| e.ticker == "JUBLFOOD_NS") {
            assert!(
                matches!(ev.offer, AsOfOffer::WithheldOpen | AsOfOffer::WithheldArmed),
                "JUBLFOOD as-of offer {:?}",
                ev.offer
            );
        }
        let mut frozen_tickers: HashSet<String> = HashSet::new();
        for id in &run.frozen_armed_ids {
            if let Some(t) = id.rsplit('-').next() {
                frozen_tickers.insert(t.to_string());
            }
        }
        for id in &run.asof_armed_ids {
            let ticker = id.rsplit('-').next().unwrap_or(id);
            assert!(
                !frozen_tickers.contains(ticker),
                "as-of {id} must not open a ticker already on the frozen session book"
            );
        }
        for id in &run.ledger_decision_ids {
            if id.starts_with("ASOF-") {
                continue;
            }
            let ticker = id.rsplit('-').next().unwrap_or(id);
            assert!(
                !run.asof_armed_ids.iter().any(|a| a.ends_with(ticker)),
                "ledger still has frozen {id} plus as-of on {ticker}"
            );
        }
    }

    #[test]
    fn live_external_twelve_bars_arm_asof_same_ingest() {
        let mut driver = AsOfSessionDriver::new(DeferredLiveConfig::default());
        let armed = driver.install_session(&[wait_watch_brief()], "2026-09-07");
        assert!(armed.is_empty(), "WAIT watch must not auto-arm");
        let bars = rising_prefix(12);
        let mut last = None;
        for obs in bars {
            last = Some(driver.ingest(obs));
        }
        assert_eq!(driver.observation_count(), 12);
        assert_eq!(driver.asof_events().len(), 1);
        assert_eq!(driver.asof_events()[0].offer, AsOfOffer::Armed);
        assert_eq!(
            driver.feed().last_source_kind,
            Some(ObservationSourceKind::ExternalLive)
        );
        let last = last.expect("12th surface");
        assert!(last.opened);
        assert_eq!(driver.ledger().positions.len(), 1);
        assert_eq!(
            driver.ledger().positions[0].paper.decision_id.as_deref(),
            Some("ASOF-2026-09-07-1000660-AAA_NS")
        );
    }

    #[test]
    fn live_external_withholds_frozen_ticker_already_on_ledger() {
        let mut driver = AsOfSessionDriver::new(DeferredLiveConfig::default());
        driver.install_session(&[frozen_open_brief()], "2026-09-07");
        for obs in rising_prefix(12) {
            driver.ingest(obs);
        }
        assert_eq!(driver.asof_events().len(), 1);
        assert_eq!(driver.asof_events()[0].offer, AsOfOffer::WithheldOpen);
        assert_eq!(driver.ledger().positions.len(), 1);
        assert_eq!(
            driver.ledger().positions[0].paper.decision_id.as_deref(),
            Some("LIVE-OPEN-AAA_NS")
        );
        assert!(!driver
            .ledger()
            .positions
            .iter()
            .any(|p| p.paper.decision_id.as_deref().unwrap_or("").starts_with("ASOF-")));
        assert_eq!(
            driver.feed().last_source_kind,
            Some(ObservationSourceKind::ExternalLive)
        );
    }

    #[test]
    fn strict_mode_not_act_cannot_execute() {
        let mut config = DeferredLiveConfig::default();
        config.strict_t0_admission = true;
        let mut driver = AsOfSessionDriver::new(config);
        
        let not_act_brief = watch_brief("LIVE-NOTACT", "AVOID");
        driver.install_session(&[not_act_brief], "2026-09-07");
        
        // Feed ascending prices so it qualifies as ACT at H60
        for obs in rising_prefix(H60_BAR_COUNT) {
            driver.ingest(obs);
        }
        
        // Asof event must exist, but it must be NotAct because of strict mode
        assert_eq!(driver.asof_events().len(), 1);
        assert_eq!(driver.asof_events()[0].offer, AsOfOffer::NotAct);
        
        // No paper execution
        assert_eq!(driver.ledger().positions.len(), 0);
    }

    #[test]
    fn strict_mode_act_can_execute() {
        let mut config = DeferredLiveConfig::default();
        config.strict_t0_admission = true;
        let mut driver = AsOfSessionDriver::new(config);
        
        let act_brief = watch_brief("LIVE-ACT", "ACT");
        driver.install_session(&[act_brief], "2026-09-07");
        
        // Already arms 1 from T0 ACT auto-arm
        assert_eq!(driver.ledger().positions.len(), 1);
        
        // Feed ascending prices
        for obs in rising_prefix(H60_BAR_COUNT) {
            driver.ingest(obs);
        }
        
        // It's withheld because it's already open from the auto-arm
        assert_eq!(driver.asof_events().len(), 1);
        assert_eq!(driver.asof_events()[0].offer, AsOfOffer::WithheldOpen);
    }

    #[test]
    fn default_mode_regression() {
        let mut config = DeferredLiveConfig::default();
        config.strict_t0_admission = false;
        let mut driver = AsOfSessionDriver::new(config);
        
        let not_act_brief = watch_brief("LIVE-NOTACT", "AVOID");
        driver.install_session(&[not_act_brief], "2026-09-07");
        
        // Feed ascending prices so it qualifies as ACT at H60
        for obs in rising_prefix(H60_BAR_COUNT) {
            driver.ingest(obs);
        }
        
        // Asof event exists and IS ARMED because strict mode is off
        assert_eq!(driver.asof_events().len(), 1);
        assert_eq!(driver.asof_events()[0].offer, AsOfOffer::Armed);
        
        // Paper execution occurred!
        assert_eq!(driver.ledger().positions.len(), 1);
    }
}
