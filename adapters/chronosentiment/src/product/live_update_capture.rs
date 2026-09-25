//! Live Recommendation Update Capture
//!
//! Observes the Deferred Live book and generates a new `DecisionBrief` update
//! for every completed 1-minute observation (once 12 rolling bars are available).
//! Persists updates to a JSONL file when the management state changes.

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::deferred_live::{LivePaperLedger, MarketObservation};
use super::deferred_live_asof_ic::{
    compute_time_safe_oqs, WatchFixture, H60_BAR_COUNT
};
use super::intraday_decision::{DecisionBrief, ExecutionFacts};
use crate::reasoning::intraday_classification::{
    classify_at_entry, resolve_action, Checkpoint, Direction, H60Classification, EntryInput
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationUpdateEvent {
    pub position_id: String,
    pub timestamp: i64,
    pub event_type: String,
    pub source_decision_id: String,
    pub calculation_window: CalculationWindow,
    pub previous_state: ManagementState,
    pub new_state: ManagementState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationWindow {
    pub bars: usize,
    pub start_unix: i64,
    pub end_unix: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManagementState {
    pub direction: String,
    pub oqs: u32,
    pub h60_action: String,
    pub target: f64,
    pub stop: f64,
}

impl ManagementState {
    pub fn from_brief(b: &DecisionBrief) -> Self {
        Self {
            direction: b.direction.clone(),
            oqs: b.oqs,
            h60_action: b.entry_action.clone(),
            target: b.execution.adaptive_target.unwrap_or(0.0),
            stop: b.execution.adaptive_risk.unwrap_or(0.0),
        }
    }
}

pub struct LiveUpdateCapture {
    pub enabled: bool,
    pub ledger_dir: Option<PathBuf>,
    pub date: String,
    
    // Ticker -> trailing rolling bars
    pub bars_by_ticker: HashMap<String, Vec<MarketObservation>>,
    pub last_state_by_ticker: HashMap<String, ManagementState>,
}

impl LiveUpdateCapture {
    pub fn new() -> Self {
        Self {
            enabled: false,
            ledger_dir: None,
            date: "".into(),
            bars_by_ticker: HashMap::new(),
            last_state_by_ticker: HashMap::new(),
        }
    }

    pub fn enable(&mut self, date: &str, ledger_dir: PathBuf) {
        self.enabled = true;
        self.date = date.to_string();
        self.ledger_dir = Some(ledger_dir);
    }

    pub fn observe(&mut self, ledger: &LivePaperLedger, obs: &MarketObservation) {
        if !self.enabled {
            return;
        }

        // Only track open positions
        let pos = ledger.positions.iter().find(|p| p.paper.ticker == obs.ticker && p.walk.is_open());
        let pos = match pos {
            Some(p) => p,
            None => {
                // If not open, we don't care. Clear any trailing bars if it just closed.
                self.bars_by_ticker.remove(&obs.ticker);
                self.last_state_by_ticker.remove(&obs.ticker);
                return;
            }
        };

        // Initialize state tracker for a newly observed open position
        if !self.last_state_by_ticker.contains_key(&obs.ticker) {
            // First time we see it open, the "previous state" is its T0 state
            let initial_state = ManagementState {
                direction: pos.paper.direction.clone(),
                oqs: pos.paper.oqs.unwrap_or(0),
                h60_action: pos.paper.entry_action.clone().unwrap_or_default(),
                target: pos.paper.paper_target,
                stop: pos.paper.paper_risk,
            };
            self.last_state_by_ticker.insert(obs.ticker.clone(), initial_state);
        }

        // Append the completed bar
        let bars = self.bars_by_ticker.entry(obs.ticker.clone()).or_default();
        bars.push(obs.clone());

        // We need exactly H60_BAR_COUNT (12) trailing bars
        if bars.len() > H60_BAR_COUNT {
            bars.remove(0);
        }

        if bars.len() < H60_BAR_COUNT {
            return;
        }

        // Evaluate the rolling brief
        let watch = WatchFixture {
            ticker: pos.paper.ticker.clone(),
            date: self.date.clone(),
            direction: pos.paper.direction.clone(),
            reference_price: pos.paper.paper_entry_price, // fallback
            entry_price: pos.paper.paper_entry_price,
            adaptive_target: pos.paper.paper_target,
            adaptive_risk: pos.paper.paper_risk,
            t0_entry_action: pos.paper.entry_action.clone().unwrap_or_default(),
        };

        if watch.entry_price <= 0.0 {
            return;
        }

        if let Some(brief) = assemble_rolling_update(&watch, bars) {
            let new_state = ManagementState::from_brief(&brief);
            let prev_state = self.last_state_by_ticker.get(&obs.ticker).unwrap();

            // Emit only if materially changed
            if &new_state != prev_state {
                let event = RecommendationUpdateEvent {
                    position_id: pos.paper.decision_id.clone().unwrap_or_default(),
                    timestamp: obs.unix,
                    event_type: "RECOMMENDATION_UPDATE".into(),
                    source_decision_id: brief.id.clone(),
                    calculation_window: CalculationWindow {
                        bars: H60_BAR_COUNT,
                        start_unix: bars[0].unix,
                        end_unix: bars[H60_BAR_COUNT - 1].unix,
                    },
                    previous_state: prev_state.clone(),
                    new_state: new_state.clone(),
                };

                self.persist_event(&event);
                self.last_state_by_ticker.insert(obs.ticker.clone(), new_state);
            }
        }
    }

    fn persist_event(&self, event: &RecommendationUpdateEvent) {
        if let Some(dir) = &self.ledger_dir {
            let file_path = dir.join(format!("position_management_events_{}.jsonl", self.date.replace("-", "")));
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&file_path) {
                if let Ok(json) = serde_json::to_string(event) {
                    let _ = writeln!(file, "{}", json);
                }
            }
        }
    }
}

/// Helper that calculates features strictly from the provided 12-bar trailing window,
/// keeping the original T0 entry price.
fn assemble_rolling_update(watch: &WatchFixture, rolling_bars: &[MarketObservation]) -> Option<DecisionBrief> {
    if rolling_bars.len() != H60_BAR_COUNT {
        return None;
    }
    
    let as_of = rolling_bars.last().unwrap().unix;
    let entry_price = watch.entry_price;
    let direction = &watch.direction;

    let signed_ret = |close: f64| {
        if direction.eq_ignore_ascii_case("LONG") {
            (close - entry_price) / entry_price
        } else {
            (entry_price - close) / entry_price
        }
    };

    let ret_at = |n: usize| {
        rolling_bars
            .get(n.saturating_sub(1))
            .map(|b| signed_ret(b.price))
    };

    let rets: Vec<f64> = rolling_bars.iter().map(|b| signed_ret(b.price)).collect();
    let mfe = rets.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mae = rets.iter().cloned().fold(f64::INFINITY, f64::min);
    let in_dir = rets.iter().filter(|r| **r > 0.0).count() as f64;
    let momentum_persistence = in_dir / rets.len() as f64;

    let h15_ret = ret_at(3);
    let h30_ret = ret_at(6);
    let h60_ret = ret_at(12);

    let h60_class = classify_h60_rolling(direction, h60_ret);
    let oqs = compute_time_safe_oqs(
        direction,
        h15_ret,
        h60_ret,
        Some(mfe),
        Some(mae),
        Some(momentum_persistence),
    );

    let dir_enum = if direction.eq_ignore_ascii_case("SHORT") {
        Direction::Short
    } else {
        Direction::Long
    };

    let h60_enum = match h60_class.as_str() {
        "ENTER" => H60Classification::Enter,
        "AVOID" => H60Classification::Avoid,
        _ => H60Classification::Wait,
    };

    let entry_state = classify_at_entry(&EntryInput {
        direction: dir_enum,
        opportunity_quality_score: oqs,
        h60_classification: h60_enum,
    });
    
    let entry_action = resolve_action(Checkpoint::Entry, entry_state, None);

    Some(DecisionBrief {
        id: format!("ASOF-UPDATE-{}-{}-{}", watch.date, as_of, watch.ticker),
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
            freshness: "UPDATE_CAPTURE".into(),
            horizon_elapsed: Some(false),
        },
        entry_state: entry_state.label().to_string(),
        entry_action: entry_action.action.label().to_string(),
        entry_confidence: entry_action.confidence.label().to_string(),
        entry_horizon: entry_action.horizon.to_string(),
        entry_why: entry_action.why.to_string(),
        entry_risk: entry_action.risk.to_string(),
        h120_state: entry_state.label().to_string(),
        h120_action: entry_action.action.label().to_string(),
        h120_confidence: entry_action.confidence.label().to_string(),
        h120_horizon: entry_action.horizon.to_string(),
        h120_why: "Rolling update; H120 reassessment not applied.".into(),
        h120_risk: entry_action.risk.to_string(),
        h15_ret,
        h30_ret,
        h60_ret,
        h120_ret: None,
        h180_ret: None,
        h300_ret: None,
        mfe_h60: Some(mfe),
        mfe_h120: None,
        outcome: None,
        pnl: None,
        hist_win: None,
        hist_pf: None,
        hist_med: None,
    })
}

fn classify_h60_rolling(direction: &str, h60_ret: Option<f64>) -> String {
    let ret = h60_ret.unwrap_or(0.0);
    if direction.eq_ignore_ascii_case("LONG") {
        if ret > 0.005 { "ENTER".into() }
        else if ret > -0.002 { "WAIT".into() }
        else { "AVOID".into() }
    } else {
        if ret > 0.005 { "ENTER".into() }
        else if ret > -0.002 { "WAIT".into() }
        else { "AVOID".into() }
    }
}
