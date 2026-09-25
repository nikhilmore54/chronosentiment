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
//! Earliest same-session point is the 12th one-minute session bar
//! (09:26 IST for a 09:15 session open). This is the P4 replay
//! H60 evidence boundary, not 60 elapsed one-minute bars.

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

/// P4 replay H60 bar index (12th session bar, 09:26 IST for a 09:15 open).
/// Not 60 one-minute bars.
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionTrajectoryReport {
    pub decision_id: Option<String>,
    pub ticker: String,
    pub direction: String,

    // Actual paper-entry anchor
    pub entry_price: f64,
    pub opened_at: i64,

    // Signed return from actual paper entry
    pub h15_ret: Option<f64>,
    pub h30_ret: Option<f64>,
    pub h60_ret: Option<f64>,
    pub h120_ret: Option<f64>,
    pub h180_ret: Option<f64>,
    pub h300_ret: Option<f64>,

    // Return between consecutive checkpoints
    pub h60_to_h120_ret: Option<f64>,
    pub h120_to_h180_ret: Option<f64>,
    pub h180_to_h300_ret: Option<f64>,

    // Final realized outcome
    pub realized_return: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CachedPositionReport {
    pub decision_id: Option<String>,
    pub ticker: String,
    pub direction: String,

    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub exit_reason: Option<String>,
    pub realized_return: Option<f64>,
    pub bars_held: i32,
    pub opened_at: i64,

    // ASOF decision metadata
    pub oqs: Option<u32>,
    pub h60_class: Option<String>,
    pub entry_action: Option<String>,
    pub entry_state: Option<String>,
    pub entry_confidence: Option<String>,
    pub entry_horizon: Option<String>,
    pub entry_risk: Option<String>,

    // Time-safe features available at ASOF
    pub h15_ret: Option<f64>,
    pub h30_ret: Option<f64>,
    pub h60_ret: Option<f64>,
    pub mfe_h60: Option<f64>,
    pub mae_h60: Option<f64>,
    pub momentum_persistence: Option<f64>,
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

/// Shadow-only per-tick assessment of an open position.
///
/// `Exit` is defined here to allow the assessment stream to carry the future
/// policy signal, but **it does not mutate the production position today**.
/// The stream is written to `CachedSessionRun::tick_assessments` for
/// off-line analysis and policy calibration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum PositionAssessment {
    Hold,
    /// Exit intent produced by an experimental shadow rule.
    /// Never acted upon until the rule is promoted to production.
    Exit { reason: String },
}

/// One assessment record emitted per open position per incoming bar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TickPositionAssessment {
    pub decision_id: Option<String>,
    pub ticker: String,
    pub bar_unix: i64,
    pub bars_since_entry: usize,

    // Position state
    pub current_signed_return: f64,

    // Path-state variables (MFE / Giveback)
    pub mfe_to_date: f64,
    pub giveback_from_mfe: f64,
    pub bars_since_mfe: usize,

    // Recent price-path state
    pub recent_momentum: f64,
    pub recent_n: usize,
    pub recent_price_change: f64,

    // Rolling return volatility
    pub return_volatility_5: Option<f64>,
    pub return_volatility_10: Option<f64>,
    pub return_volatility_20: Option<f64>,

    // Directional pressure
    pub directional_pressure_5: Option<f64>,
    pub directional_pressure_10: Option<f64>,

    // Volume state
    pub current_volume: Option<f64>,
    pub relative_volume_5: Option<f64>,
    pub relative_volume_20: Option<f64>,

    // Shadow-only assessment. Still Hold for now.
    pub assessment: PositionAssessment,
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
    volumes_by_ticker: HashMap<String, Vec<Option<f64>>>,
    offered: HashSet<String>,
    asof_events: Vec<AsOfSessionEvent>,
    asof_decisions: HashMap<String, DecisionBrief>,
    /// Shadow stream: one record per open position per bar. No production effect.
    tick_assessments: Vec<TickPositionAssessment>,
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
            volumes_by_ticker: HashMap::new(),
            offered: HashSet::new(),
            asof_events: Vec::new(),
            asof_decisions: HashMap::new(),
            tick_assessments: Vec::new(),
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
        // All observations are accumulated for full-session trajectory lookups.
        // The rolling ASOF IC evaluation uses only the last H60_BAR_COUNT bars.
        let bars = self.bars_by_ticker.entry(ticker.clone()).or_default();
        bars.push(sourced.observation.clone());
        let volumes = self.volumes_by_ticker.entry(ticker.clone()).or_default();
        volumes.push(sourced.volume);
        if bars.len() >= H60_BAR_COUNT {
            let rolling = &bars[bars.len() - H60_BAR_COUNT..];

            if let Some(watch) = self.watches.get(&ticker) {
                if let Some(mut brief) = AsOfIcAssembler::assemble(watch, rolling) {
                    brief.execution.freshness = sourced.source.freshness().to_string();

                    let as_of_unix = brief
                        .execution
                        .snap_unix
                        .unwrap_or(sourced.observation.unix);

                    let decision_id = brief.id.clone();
                    let entry_action = brief.entry_action.clone();
                    let oqs = brief.oqs;
                    let h60_class = brief.h60_class.clone();

                    self.asof_decisions
                        .insert(decision_id.clone(), brief.clone());

                    let offer = if self.strict_t0_admission
                        && watch.t0_entry_action != "ACT"
                        && brief.entry_action == "ACT"
                    {
                        AsOfOffer::NotAct
                    } else {
                        offer_asof_brief(&mut self.loop_rt, brief)
                    };

                    if matches!(offer, AsOfOffer::Armed) {
                        self.offered.insert(ticker.clone());
                    }

                    self.asof_events.push(AsOfSessionEvent {
                        ticker: ticker.clone(),
                        as_of_unix,
                        offer,
                        decision_id,
                        entry_action,
                        oqs,
                        h60_class,
                    });
                }
            }
        }

        // --- Per-tick shadow assessment of any open position for this ticker ---
        // Runs after the ASOF evaluation but before the lifecycle step.
        // The assessment is shadow-only: Exit is never acted upon here.
        // --- Per-tick shadow assessment ---
        eprintln!(
            "[ASSESS] ticker={} unix={} ledger_positions={} open_positions={}",
            ticker,
            sourced.observation.unix,
            self.loop_rt.runtime().ledger().positions.len(),
            self.loop_rt
                .runtime()
                .ledger()
                .positions
                .iter()
                .filter(|p| {
                    p.paper.ticker == ticker
                        && p.status() == super::deferred_live::LivePaperStatus::Open
                })
                .count()
        );

        let assessments = self.assess_open_positions_for_ticker(
            &ticker,
            sourced.observation.unix,
        );

        eprintln!(
            "[ASSESS] ticker={} produced={}",
            ticker,
            assessments.len()
        );

        self.tick_assessments.extend(assessments);

        self.loop_rt.step_sourced(sourced)
    }

    fn return_volatility(
        bars: &[MarketObservation],
        n: usize,
    ) -> Option<f64> {
        if bars.len() < n + 1 {
            return None;
        }

        let start = bars.len() - n - 1;
        let window = &bars[start..];

        let returns: Vec<f64> = window
            .windows(2)
            .filter_map(|w| {
                let prev = w[0].price;
                let curr = w[1].price;

                if prev > 0.0 && curr.is_finite() && prev.is_finite() {
                    Some((curr - prev) / prev)
                } else {
                    None
                }
            })
            .collect();

        if returns.len() < 2 {
            return None;
        }

        let mean = returns.iter().sum::<f64>() / returns.len() as f64;

        let variance = returns
            .iter()
            .map(|r| {
                let d = r - mean;
                d * d
            })
            .sum::<f64>()
            / returns.len() as f64;

        Some(variance.sqrt())
    }

    fn directional_pressure(
        bars: &[MarketObservation],
        n: usize,
        is_long: bool,
    ) -> Option<f64> {
        if bars.len() < n + 1 {
            return None;
        }

        let start = bars.len() - n - 1;
        let window = &bars[start..];

        let mut numerator = 0.0;
        let mut denominator = 0.0;
        let mut count = 0usize;

        for pair in window.windows(2) {
            let prev = pair[0].price;
            let curr = pair[1].price;

            if prev <= 0.0 || !prev.is_finite() || !curr.is_finite() {
                continue;
            }

            let signed_return = if is_long {
                (curr - prev) / prev
            } else {
                (prev - curr) / prev
            };

            numerator += signed_return;
            denominator += signed_return.abs();
            count += 1;
        }

        if count == 0 || denominator <= 0.0 {
            return None;
        }

        Some(numerator / denominator)
    }

    fn relative_volume(volumes: &[Option<f64>], n: usize) -> Option<f64> {
        if volumes.len() < n + 1 {
            return None;
        }

        let current_volume = volumes.last().copied().flatten()?;
        if !current_volume.is_finite() || current_volume <= 0.0 {
            return None;
        }

        let prior_slice = &volumes[volumes.len() - 1 - n..volumes.len() - 1];
        let valid_priors: Vec<f64> = prior_slice
            .iter()
            .filter_map(|v| *v)
            .filter(|v| v.is_finite() && *v > 0.0)
            .collect();

        if valid_priors.len() < n {
            return None;
        }

        let mean_prior = valid_priors.iter().sum::<f64>() / valid_priors.len() as f64;
        if !mean_prior.is_finite() || mean_prior <= 0.0 {
            return None;
        }

        Some(current_volume / mean_prior)
    }

    /// Produce a `TickPositionAssessment` for every currently-open paper position
    /// on `ticker`. Called once per incoming bar.
    ///
    /// The assessment is **shadow-only**: the `Exit` variant is defined but
    /// never acted upon until a validated rule is promoted to production.
    fn assess_open_positions_for_ticker(
        &self,
        ticker: &str,
        bar_unix: i64,
    ) -> Vec<TickPositionAssessment> {
        let bars = match self.bars_by_ticker.get(ticker) {
            Some(b) if !b.is_empty() => b,
            _ => return vec![],
        };
        let current_price = bars.last().expect("non-empty").price;

        let volumes = match self.volumes_by_ticker.get(ticker) {
            Some(v) => v.as_slice(),
            None => &[],
        };

        let current_volume = volumes
            .last()
            .copied()
            .flatten()
            .filter(|v| v.is_finite() && *v > 0.0);
        let relative_volume_5 = Self::relative_volume(volumes, 5);
        let relative_volume_20 = Self::relative_volume(volumes, 20);

        // Short recent window for per-tick momentum. Fixed at 5 bars (~5 min).
        // This constant will be calibrated once the assessment stream is analysed.
        const RECENT_N: usize = 5;

        self.loop_rt
            .runtime()
            .ledger()
            .positions
            .iter()
            .filter(|p| p.paper.ticker == ticker && p.status() == super::deferred_live::LivePaperStatus::Open)
            .map(|p| {
                let entry_price = p.paper.paper_entry_price;
                let direction = p.paper.direction.as_str();
                let is_long = !direction.eq_ignore_ascii_case("SHORT");

                // Find the bar index at or after position entry (opened_at).
                let entry_bar_idx = bars
                    .iter()
                    .position(|b| b.unix >= p.opened_at)
                    .unwrap_or(0);
                let bars_since_entry = bars.len().saturating_sub(entry_bar_idx);

                // Signed return from entry to current bar.
                let current_signed_return = if is_long {
                    (current_price - entry_price) / entry_price
                } else {
                    (entry_price - current_price) / entry_price
                };

                // MFE to date, Giveback from MFE, and Bars since MFE.
                let open_bars = &bars[entry_bar_idx..];
                let mut max_mfe: f64 = 0.0;
                let mut mfe_idx: usize = open_bars.len().saturating_sub(1);

                for (idx_rel, b) in open_bars.iter().enumerate() {
                    let ret = if is_long {
                        (b.price - entry_price) / entry_price
                    } else {
                        (entry_price - b.price) / entry_price
                    };
                    if ret >= max_mfe {
                        max_mfe = ret;
                        mfe_idx = idx_rel;
                    }
                }

                let mfe_to_date = max_mfe;
                let giveback_from_mfe = (max_mfe - current_signed_return).max(0.0);
                let bars_since_mfe = open_bars.len().saturating_sub(1).saturating_sub(mfe_idx);

                // Recent-window momentum: fraction of last RECENT_N bars in-direction.
                let recent_window_start = bars.len().saturating_sub(RECENT_N);
                let recent_bars = &bars[recent_window_start..];
                let recent_n = recent_bars.len();

                let recent_momentum = if recent_n >= 2 {
                    let in_dir = recent_bars.windows(2).filter(|w| {
                        let delta = w[1].price - w[0].price;
                        if is_long { delta > 0.0 } else { delta < 0.0 }
                    }).count() as f64;
                    in_dir / (recent_n - 1) as f64
                } else {
                    0.5 // not enough bars — neutral
                };

                // Signed price change over the recent window (last bar vs first bar of window).
                let recent_price_change = if recent_n >= 2 {
                    let first = recent_bars.first().unwrap().price;
                    let last  = recent_bars.last().unwrap().price;
                    if is_long { (last - first) / first } else { (first - last) / first }
                } else {
                    0.0
                };

                let return_volatility_5 =
                    Self::return_volatility(bars, 5);

                let return_volatility_10 =
                    Self::return_volatility(bars, 10);

                let return_volatility_20 =
                    Self::return_volatility(bars, 20);

                let directional_pressure_5 =
                    Self::directional_pressure(bars, 5, is_long);

                let directional_pressure_10 =
                    Self::directional_pressure(bars, 10, is_long);

                // Policy: Hold unconditionally until a rule is validated.
                // When a rule is ready, replace this with condition-based Exit.
                let assessment = PositionAssessment::Hold;

                TickPositionAssessment {
                    decision_id: p.paper.decision_id.clone(),
                    ticker: ticker.to_string(),
                    bar_unix,
                    bars_since_entry,

                    current_signed_return,

                    mfe_to_date,
                    giveback_from_mfe,
                    bars_since_mfe,

                    recent_momentum,
                    recent_n,
                    recent_price_change,

                    return_volatility_5,
                    return_volatility_10,
                    return_volatility_20,

                    directional_pressure_5,
                    directional_pressure_10,

                    current_volume,
                    relative_volume_5,
                    relative_volume_20,

                    assessment,
                }
            })
            .collect()
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

        // Position-level report: decision metadata + actual paper execution.
        let positions: Vec<CachedPositionReport> = self
            .loop_rt
            .runtime()
            .ledger()
            .positions
            .iter()
            .map(|p| {
                let decision_id = p.paper.decision_id.clone();

                let decision = decision_id
                    .as_ref()
                    .and_then(|id| self.asof_decisions.get(id));

                CachedPositionReport {
                    decision_id,
                    ticker: p.paper.ticker.clone(),
                    direction: p.paper.direction.clone(),

                    entry_price: p.paper.paper_entry_price,
                    exit_price: p.paper_exit_price(),
                    exit_reason: p.exit_reason().map(str::to_string),
                    realized_return: p.realized_return(),
                    bars_held: p.paper.bars_held as i32,
                    opened_at: p.opened_at,

                    oqs: decision.map(|d| d.oqs),
                    h60_class: decision.map(|d| d.h60_class.clone()),
                    entry_action: decision.map(|d| d.entry_action.clone()),
                    entry_state: decision.map(|d| d.entry_state.clone()),
                    entry_confidence: decision.map(|d| d.entry_confidence.clone()),
                    entry_horizon: decision.map(|d| d.entry_horizon.clone()),
                    entry_risk: decision.map(|d| d.entry_risk.clone()),

                    h15_ret: decision.and_then(|d| d.h15_ret),
                    h30_ret: decision.and_then(|d| d.h30_ret),
                    h60_ret: decision.and_then(|d| d.h60_ret),
                    mfe_h60: decision.and_then(|d| d.mfe_h60),

                    // These are not currently exposed by DecisionBrief.
                    mae_h60: None,
                    momentum_persistence: None,
                }
            })
            .collect();

        // Retrospective trajectory from the ACTUAL paper fill.
        //
        // This deliberately uses p.opened_at + p.paper.paper_entry_price,
        // rather than the ASOF WatchFixture entry_price.
        let trajectories: Vec<PositionTrajectoryReport> = self
            .loop_rt
            .runtime()
            .ledger()
            .positions
            .iter()
            .map(|p| {
                let entry_price = p.paper.paper_entry_price;
                let opened_at = p.opened_at;
                let direction = p.paper.direction.clone();

                let bars = self
                    .bars_by_ticker
                    .get(&p.paper.ticker)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);

                let price_at = |minutes: i64| {
                    trajectory_price_at_or_after(
                        bars,
                        opened_at.saturating_add(minutes * 60),
                    )
                };

                let h15_price = price_at(15);
                let h30_price = price_at(30);
                let h60_price = price_at(60);
                let h120_price = price_at(120);
                let h180_price = price_at(180);
                let h300_price = price_at(300);

                let h15_ret = h15_price.map(|price| {
                    signed_return(&direction, entry_price, price)
                });

                let h30_ret = h30_price.map(|price| {
                    signed_return(&direction, entry_price, price)
                });

                let h60_ret = h60_price.map(|price| {
                    signed_return(&direction, entry_price, price)
                });

                let h120_ret = h120_price.map(|price| {
                    signed_return(&direction, entry_price, price)
                });

                let h180_ret = h180_price.map(|price| {
                    signed_return(&direction, entry_price, price)
                });

                let h300_ret = h300_price.map(|price| {
                    signed_return(&direction, entry_price, price)
                });

                // Incremental movement between checkpoints.
                let h60_to_h120_ret = match (h60_price, h120_price) {
                    (Some(from), Some(to)) if from > 0.0 => {
                        Some(signed_return(&direction, from, to))
                    }
                    _ => None,
                };

                let h120_to_h180_ret = match (h120_price, h180_price) {
                    (Some(from), Some(to)) if from > 0.0 => {
                        Some(signed_return(&direction, from, to))
                    }
                    _ => None,
                };

                let h180_to_h300_ret = match (h180_price, h300_price) {
                    (Some(from), Some(to)) if from > 0.0 => {
                        Some(signed_return(&direction, from, to))
                    }
                    _ => None,
                };

                PositionTrajectoryReport {
                    decision_id: p.paper.decision_id.clone(),
                    ticker: p.paper.ticker.clone(),
                    direction,

                    entry_price,
                    opened_at,

                    h15_ret,
                    h30_ret,
                    h60_ret,
                    h120_ret,
                    h180_ret,
                    h300_ret,

                    h60_to_h120_ret,
                    h120_to_h180_ret,
                    h180_to_h300_ret,

                    realized_return: p.realized_return(),
                }
            })
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

            positions,
            trajectories,
            tick_assessments: self.tick_assessments.clone(),
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
    pub oqs: u32,
    pub h60_class: String,
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
    pub positions: Vec<CachedPositionReport>,
    pub trajectories: Vec<PositionTrajectoryReport>,
    /// Shadow per-tick assessment stream. One record per open position per bar.
    /// No production effect — for analysis and policy calibration only.
    pub tick_assessments: Vec<TickPositionAssessment>,
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
fn trajectory_price_at_or_after(
    bars: &[MarketObservation],
    target_unix: i64,
) -> Option<f64> {
    bars.iter()
        .filter(|bar| bar.unix >= target_unix)
        .min_by_key(|bar| bar.unix)
        .map(|bar| bar.price)
}

fn signed_return(direction: &str, entry_price: f64, price: f64) -> f64 {
    if direction.eq_ignore_ascii_case("LONG") {
        (price - entry_price) / entry_price
    } else {
        (entry_price - price) / entry_price
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
    fn cached_session_20260915_keeps_frozen_book_and_rolling_asof() {
        use std::collections::HashSet;
        use std::path::Path;

        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let dataset = root.join("datasets/live005_20260915.json");
        let cache = root.join("intraday_capture/yahoo_cache_1m");

        if !dataset.exists() || !cache.join("JUBLFOOD.NS.json").exists() {
            return;
        }

        let briefs =
            crate::product::load_intraday_briefs(dataset.to_str().unwrap()).unwrap();

        let run =
            run_cached_session(&briefs, "2026-09-15", &cache, 0.0)
                .expect("cached session");

        // The cached tape must contain a complete session.
        assert!(
            run.observations > 12,
            "session tape should exceed H60"
        );

        assert!(
            run.feed_status.contains("TapeExhausted"),
            "feed_status={}",
            run.feed_status
        );

        // Frozen LIVE-005 book remains intact.
        assert!(
            !run.frozen_armed_ids.is_empty(),
            "frozen session should arm at least one brief"
        );

        // Rolling ASOF must actually produce admissions.
        assert!(
            !run.asof_armed_ids.is_empty(),
            "rolling ASOF should produce at least one admission"
        );

        // No ticker may be admitted twice through ASOF.
        let mut asof_tickers = HashSet::new();

        for id in &run.asof_armed_ids {
            let ticker = id.rsplit('-').next().unwrap_or(id);

            assert!(
                asof_tickers.insert(ticker.to_string()),
                "ticker {ticker} was admitted more than once by ASOF"
            );
        }

        // ASOF must never replace a ticker already present
        // in the frozen session book.
        let frozen_tickers: HashSet<String> = run
            .frozen_armed_ids
            .iter()
            .filter_map(|id| id.rsplit('-').next())
            .map(str::to_string)
            .collect();

        for id in &run.asof_armed_ids {
            let ticker = id.rsplit('-').next().unwrap_or(id);

            assert!(
                !frozen_tickers.contains(ticker),
                "ASOF {id} must not open ticker already on frozen book"
            );
        }

        // Verify that rolling evaluation actually happened more than
        // once for at least one ticker.
        let mut evaluation_counts = std::collections::HashMap::<String, usize>::new();

        for ev in &run.asof_events {
            *evaluation_counts
                .entry(ev.ticker.clone())
                .or_insert(0) += 1;
        }

        assert!(
            evaluation_counts.values().any(|n| *n > 1),
            "rolling ASOF must evaluate at least one ticker more than once"
        );

        // Verify that evaluation timestamps advance chronologically
        // within each ticker.
        for ticker in evaluation_counts.keys() {
            let times: Vec<i64> = run
                .asof_events
                .iter()
                .filter(|e| &e.ticker == ticker)
                .map(|e| e.as_of_unix)
                .collect();

            assert!(
                times.windows(2).all(|w| w[0] < w[1]),
                "ASOF evaluations for {ticker} must advance chronologically"
            );
        }

        // Every ASOF admission must correspond to an ACT evaluation.
        for ev in &run.asof_events {
            if matches!(ev.offer, AsOfOffer::Armed) {
                assert_eq!(
                    ev.entry_action,
                    "ACT",
                    "ASOF admission must only occur for ACT"
                );
            }
        }

        // Frozen and ASOF positions must not coexist for the same ticker.
        for id in &run.ledger_decision_ids {
            if id.starts_with("ASOF-") {
                continue;
            }

            let ticker = id.rsplit('-').next().unwrap_or(id);

            assert!(
                !run
                    .asof_armed_ids
                    .iter()
                    .any(|a| a.ends_with(ticker)),
                "ledger contains frozen {id} plus ASOF admission for {ticker}"
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
        assert_eq!(driver.armed().len(), 1);
        assert_eq!(driver.ledger().positions.len(), 0);

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
