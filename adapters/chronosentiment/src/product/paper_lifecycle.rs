//! Paper Trader v0.2 lifecycle evaluator — one observation at a time.
//!
//! Port of `scripts/run_paper_trader_v2.py::walk_position` / `signed_ret`.
//! The Python runner and frozen CSV replay are not modified.
//!
//! ```text
//! historical bar  ─┐
//!                  ├─► apply_observation ─► TARGET | STOP | HORIZON | hold
//! live observation─┘
//! ```
//!
//! TARGET is checked before STOP on the same observation. HORIZON exits at
//! last/close, not at a barrier. Stop may be `None` (v0.2 no-stop counterfactual).
//!
//! Horizon *policy* is the clock adapter (session bar index vs unix). Exit
//! classification is not duplicated per driver.

use serde::{Deserialize, Serialize};

/// v0.2 session walk starts at bar 12 and horizons at session bar 59.
pub const V02_START_BAR: usize = 12;
pub const V02_HORIZON_BAR: usize = 59;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExitReason {
    Target,
    Stop,
    Horizon,
}

impl ExitReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Target => "TARGET",
            Self::Stop => "STOP",
            Self::Horizon => "HORIZON",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LifecycleExit {
    pub reason: ExitReason,
    pub price: f64,
    pub unix: i64,
}

/// One OHLC (or last-trade) observation. No future bars.
#[derive(Debug, Clone, PartialEq)]
pub struct PaperObservation {
    pub unix: i64,
    pub last: f64,
    pub high: f64,
    pub low: f64,
    /// Session bar index (0 = 09:15 IST). Required for `HorizonPolicy::SessionBarIndex`.
    pub session_bar_index: Option<usize>,
}

impl PaperObservation {
    pub fn bar(unix: i64, last: f64, high: f64, low: f64, session_bar_index: usize) -> Self {
        Self {
            unix,
            last,
            high,
            low,
            session_bar_index: Some(session_bar_index),
        }
    }

    pub fn last_trade(unix: i64, price: f64) -> Self {
        Self {
            unix,
            last: price,
            high: price,
            low: price,
            session_bar_index: None,
        }
    }
}

/// Clock for horizon — not a second exit engine.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HorizonPolicy {
    /// v0.2: after TARGET/STOP, horizon if `session_bar_index >= limit` (59).
    SessionBarIndex { limit: usize },
    /// Deferred live: after TARGET/STOP, horizon if `unix >= horizon_unix`.
    Unix { horizon_unix: i64 },
}

impl HorizonPolicy {
    fn reached(self, obs: &PaperObservation) -> bool {
        match self {
            Self::SessionBarIndex { limit } => obs
                .session_bar_index
                .map(|i| i >= limit)
                .unwrap_or(false),
            Self::Unix { horizon_unix } => obs.unix >= horizon_unix,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaperWalkState {
    pub direction: String,
    pub entry: f64,
    pub target: f64,
    /// `None` = v0.2 no-stop counterfactual walk.
    pub stop: Option<f64>,
    pub bars_held: i32,
    pub mae: f64,
    pub mfe: f64,
    pub exit: Option<LifecycleExit>,
}

impl PaperWalkState {
    pub fn open(direction: impl Into<String>, entry: f64, target: f64, stop: Option<f64>) -> Self {
        Self {
            direction: direction.into(),
            entry,
            target,
            stop,
            bars_held: 0,
            mae: 0.0,
            mfe: 0.0,
            exit: None,
        }
    }

    pub fn is_open(&self) -> bool {
        self.exit.is_none()
    }
}

/// Direction-adjusted return. Same as `signed_ret` in run_paper_trader_v2.py.
pub fn signed_return(direction: &str, entry: f64, exit: f64) -> f64 {
    if entry == 0.0 {
        return 0.0;
    }
    let r = (exit - entry) / entry;
    if direction.eq_ignore_ascii_case("SHORT") {
        -r
    } else {
        r
    }
}

pub fn format_ist_hm(unix: i64) -> String {
    use chrono::{FixedOffset, TimeZone};
    let ist = FixedOffset::east_opt(5 * 3600 + 30 * 60).expect("IST offset");
    ist.timestamp_opt(unix, 0)
        .single()
        .map(|t| t.format("%H:%M").to_string())
        .unwrap_or_else(|| unix.to_string())
}

fn update_excursions(state: &mut PaperWalkState, high: f64, low: f64) {
    let entry = state.entry;
    if entry == 0.0 {
        return;
    }
    if state.direction.eq_ignore_ascii_case("SHORT") {
        state.mae = state.mae.min((entry - high) / entry);
        state.mfe = state.mfe.max((entry - low) / entry);
    } else {
        state.mae = state.mae.min((low - entry) / entry);
        state.mfe = state.mfe.max((high - entry) / entry);
    }
}

/// Apply one observation. Returns `Some` on TARGET / STOP / HORIZON.
/// Does not look at any later observation.
pub fn apply_observation(
    state: &mut PaperWalkState,
    obs: &PaperObservation,
    horizon: HorizonPolicy,
) -> Option<LifecycleExit> {
    if !state.is_open() {
        return None;
    }
    state.bars_held += 1;
    update_excursions(state, obs.high, obs.low);

    let long = state.direction.eq_ignore_ascii_case("LONG");
    let hit = if long {
        if obs.high >= state.target {
            Some(LifecycleExit {
                reason: ExitReason::Target,
                price: state.target,
                unix: obs.unix,
            })
        } else if state.stop.is_some_and(|s| obs.low <= s) {
            Some(LifecycleExit {
                reason: ExitReason::Stop,
                price: state.stop.unwrap(),
                unix: obs.unix,
            })
        } else {
            None
        }
    } else if obs.low <= state.target {
        Some(LifecycleExit {
            reason: ExitReason::Target,
            price: state.target,
            unix: obs.unix,
        })
    } else if state.stop.is_some_and(|s| obs.high >= s) {
        Some(LifecycleExit {
            reason: ExitReason::Stop,
            price: state.stop.unwrap(),
            unix: obs.unix,
        })
    } else {
        None
    };

    let exit = hit.or_else(|| {
        if horizon.reached(obs) {
            Some(LifecycleExit {
                reason: ExitReason::Horizon,
                price: obs.last,
                unix: obs.unix,
            })
        } else {
            None
        }
    });

    if let Some(exit) = exit.clone() {
        state.exit = Some(exit);
    }
    exit
}

/// Batch walk equivalent to `walk_position(..., start_bar=12)` over a session.
/// If bars end before the horizon index, still HORIZON at `bars[min(limit, last)]`.
pub fn walk_bars(
    direction: &str,
    entry: f64,
    target: f64,
    stop: Option<f64>,
    bars: &[PaperObservation],
    start_bar: usize,
    horizon_index: usize,
) -> PaperWalkState {
    let mut state = PaperWalkState::open(direction, entry, target, stop);
    let horizon = HorizonPolicy::SessionBarIndex {
        limit: horizon_index,
    };
    let from = start_bar.min(bars.len());
    for (i, bar) in bars[from..].iter().enumerate() {
        let mut obs = bar.clone();
        obs.session_bar_index = Some(from + i);
        if apply_observation(&mut state, &obs, horizon).is_some() {
            return state;
        }
    }
    if state.is_open() && !bars.is_empty() {
        let idx = horizon_index.min(bars.len() - 1);
        let h = &bars[idx];
        state.exit = Some(LifecycleExit {
            reason: ExitReason::Horizon,
            price: h.last,
            unix: h.unix,
        });
    }
    state
}

pub fn event_kind(reason: ExitReason) -> &'static str {
    match reason {
        ExitReason::Horizon => "HORIZON",
        ExitReason::Target | ExitReason::Stop => "PAPER_EXIT",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_before_stop_on_same_bar() {
        let mut s = PaperWalkState::open("LONG", 100.0, 102.0, Some(98.0));
        let obs = PaperObservation::bar(1, 100.0, 103.0, 97.0, 12);
        let e = apply_observation(&mut s, &obs, HorizonPolicy::SessionBarIndex { limit: 59 }).unwrap();
        assert_eq!(e.reason, ExitReason::Target);
        assert_eq!(e.price, 102.0);
    }

    #[test]
    fn long_stop_at_barrier_not_last() {
        let mut s = PaperWalkState::open("LONG", 100.0, 102.0, Some(98.0));
        let obs = PaperObservation::bar(1, 98.2, 99.0, 97.9, 12);
        let e = apply_observation(&mut s, &obs, HorizonPolicy::Unix { horizon_unix: 10_000 }).unwrap();
        assert_eq!(e.reason, ExitReason::Stop);
        assert_eq!(e.price, 98.0);
    }

    #[test]
    fn none_stop_skips_stop_and_can_horizon() {
        let mut s = PaperWalkState::open("LONG", 100.0, 110.0, None);
        let obs = PaperObservation::bar(1, 97.0, 99.0, 96.0, 59);
        let e = apply_observation(&mut s, &obs, HorizonPolicy::SessionBarIndex { limit: 59 }).unwrap();
        assert_eq!(e.reason, ExitReason::Horizon);
        assert_eq!(e.price, 97.0);
    }

    #[test]
    fn signed_return_matches_python() {
        assert!((signed_return("LONG", 100.0, 102.0) - 0.02).abs() < 1e-12);
        assert!((signed_return("SHORT", 100.0, 98.0) - 0.02).abs() < 1e-12);
    }

    #[test]
    fn walk_bars_equals_stepwise_apply() {
        let bars: Vec<_> = (0..60)
            .map(|i| PaperObservation::bar(i as i64, 100.0 + i as f64 * 0.01, 100.1, 99.9, i))
            .collect();
        let batch = walk_bars("LONG", 100.0, 102.0, Some(98.0), &bars, V02_START_BAR, V02_HORIZON_BAR);
        let mut step = PaperWalkState::open("LONG", 100.0, 102.0, Some(98.0));
        let h = HorizonPolicy::SessionBarIndex {
            limit: V02_HORIZON_BAR,
        };
        for i in V02_START_BAR..bars.len() {
            if apply_observation(&mut step, &bars[i], h).is_some() {
                break;
            }
        }
        assert_eq!(batch.exit, step.exit);
        assert_eq!(batch.bars_held, step.bars_held);
    }

    #[test]
    fn unix_horizon_does_not_use_session_index() {
        let mut s = PaperWalkState::open("SHORT", 470.04, 467.30, Some(472.10));
        let hold = PaperObservation::last_trade(1_030, 468.82);
        assert!(apply_observation(&mut s, &hold, HorizonPolicy::Unix { horizon_unix: 1_060 }).is_none());
        let done = PaperObservation::last_trade(1_060, 468.50);
        let e = apply_observation(&mut s, &done, HorizonPolicy::Unix { horizon_unix: 1_060 }).unwrap();
        assert_eq!(e.reason, ExitReason::Horizon);
        assert_eq!(e.price, 468.50);
    }
}
