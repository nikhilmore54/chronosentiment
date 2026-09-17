//! Deferred-live driver — live clock over Paper Trader v0.2 lifecycle.
//!
//! Does not reimplement TARGET / STOP / HORIZON. That lives in
//! [`crate::product::paper_lifecycle`].
//!
//! ```text
//! Paper Trader v0.2 semantics  (paper_lifecycle)
//!         │
//!         ├── historical driver  CSV bars → paper_replay (frozen results)
//!         └── deferred-live driver
//!                MarketObservation → PaperObservation
//!                ↓
//!             apply_observation
//!                ↓
//!             deferred-live ledger
//! ```
//!
//! `paper_entry_price` is the observed live fill. It never overwrites
//! `DecisionBrief.entry_price` / `reference_price`.
//!
//! See `docs/CS-P-001_DECISION_SUPPORT_PRODUCT_MODE.md` Stage B.

use serde::{Deserialize, Serialize};

use super::intraday_decision::DecisionBrief;
use super::paper_lifecycle::{
    apply_observation, event_kind, signed_return, ExitReason, HorizonPolicy, PaperObservation,
    PaperWalkState,
};
use super::paper_replay::PaperPosition;

pub const EXECUTION_MODE: &str = "DEFERRED_LIVE";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeferredLiveConfig {
    pub horizon_secs: i64,
    #[serde(default)]
    pub strict_t0_admission: bool,
    #[serde(default)]
    pub max_concurrent_positions: Option<usize>,
}

impl Default for DeferredLiveConfig {
    fn default() -> Self {
        Self {
            horizon_secs: 300 * 60,
            strict_t0_admission: false,
            max_concurrent_positions: None,
        }
    }
}

/// Live market tick/bar. Converted to [`PaperObservation`] before the evaluator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketObservation {
    pub ticker: String,
    pub unix: i64,
    pub price: f64,
    pub high: Option<f64>,
    pub low: Option<f64>,
}

impl MarketObservation {
    pub fn last(ticker: impl Into<String>, unix: i64, price: f64) -> Self {
        Self {
            ticker: ticker.into(),
            unix,
            price,
            high: None,
            low: None,
        }
    }

    fn to_paper(&self) -> PaperObservation {
        let last = self.price;
        PaperObservation {
            unix: self.unix,
            last,
            high: self.high.unwrap_or(last),
            low: self.low.unwrap_or(last),
            session_bar_index: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LivePaperStatus {
    Open,
    Exited,
    Horizon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LivePaperAction {
    Hold,
    Exit,
    Horizon,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LivePaperEvent {
    pub unix: i64,
    pub kind: String,
    pub price: f64,
    pub note: Option<String>,
}

/// Live ledger row: shared [`PaperPosition`] plus live clock fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LivePaperPosition {
    pub paper: PaperPosition,
    pub walk: PaperWalkState,
    pub current_price: f64,
    pub opened_at: i64,
    pub last_tick_at: i64,
    pub horizon: HorizonPolicy,
    pub events: Vec<LivePaperEvent>,
}

impl LivePaperPosition {
    pub fn status(&self) -> LivePaperStatus {
        match self.walk.exit.as_ref().map(|e| e.reason) {
            None => LivePaperStatus::Open,
            Some(ExitReason::Horizon) => LivePaperStatus::Horizon,
            Some(_) => LivePaperStatus::Exited,
        }
    }

    pub fn action(&self) -> LivePaperAction {
        match self.status() {
            LivePaperStatus::Open => LivePaperAction::Hold,
            LivePaperStatus::Horizon => LivePaperAction::Horizon,
            LivePaperStatus::Exited => LivePaperAction::Exit,
        }
    }

    pub fn paper_entry_price(&self) -> f64 {
        self.paper.paper_entry_price
    }

    pub fn paper_exit_price(&self) -> Option<f64> {
        self.paper.paper_exit_price
    }

    pub fn exit_reason(&self) -> Option<&str> {
        if self.paper.exit_reason.is_empty() {
            None
        } else {
            Some(self.paper.exit_reason.as_str())
        }
    }

    pub fn realized_return(&self) -> Option<f64> {
        self.paper.realized_return
    }

    pub fn unrealized_return(&self) -> f64 {
        signed_return(
            &self.paper.direction,
            self.paper.paper_entry_price,
            self.current_price,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LivePaperLedger {
    pub mode: String,
    pub positions: Vec<LivePaperPosition>,
}

impl LivePaperLedger {
    pub fn new() -> Self {
        Self {
            mode: EXECUTION_MODE.to_string(),
            positions: Vec::new(),
        }
    }
}

impl Default for LivePaperLedger {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenError {
    MissingDecisionId,
    TickerMismatch,
    MissingTarget,
    MissingRisk,
    InvalidPrice,
    AlreadyOpen,
}

impl std::fmt::Display for OpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingDecisionId => write!(f, "missing decision_id"),
            Self::TickerMismatch => write!(f, "observation ticker does not match DecisionBrief"),
            Self::MissingTarget => write!(f, "DecisionBrief.execution.adaptive_target missing"),
            Self::MissingRisk => write!(f, "DecisionBrief.execution.adaptive_risk missing"),
            Self::InvalidPrice => write!(f, "invalid observation price"),
            Self::AlreadyOpen => write!(f, "paper position already open for this decision"),
        }
    }
}

/// Overlay a live quote onto a DecisionBrief the live runtime owns.
/// Does not change `entry_price` or `reference_price`.
pub fn overlay_live_quote(brief: &mut DecisionBrief, price: f64, unix: i64) {
    brief.execution.current_price = Some(price);
    brief.execution.last_tick_unix = Some(unix);
    brief.execution.freshness = "LIVE".into();
}

fn sync_paper_from_walk(paper: &mut PaperPosition, walk: &PaperWalkState) {
    paper.bars_held = walk.bars_held;
    paper.max_adverse_excursion = Some(walk.mae);
    paper.max_favourable_excursion = Some(walk.mfe);
    if let Some(exit) = &walk.exit {
        paper.paper_exit_price = Some(exit.price);
        paper.exit_reason = exit.reason.as_str().to_string();
        paper.exit_time_ist = Some(super::paper_lifecycle::format_ist_hm(exit.unix));
        paper.realized_return = Some(signed_return(
            &paper.direction,
            paper.paper_entry_price,
            exit.price,
        ));
    }
}

fn empty_paper(
    brief: &DecisionBrief,
    fill: f64,
    target: f64,
    risk: f64,
) -> PaperPosition {
    PaperPosition {
        ticker: brief.ticker.clone(),
        direction: brief.direction.clone(),
        date: brief.date.clone(),
        paper_entry_price: fill,
        paper_target: target,
        paper_risk: risk,
        candidate_stop_pct: None,
        candidate_stop_price: Some(risk),
        stop_confidence: String::new(),
        coverage_quality: String::new(),
        n_obs: 0,
        path_context: String::new(),
        final_tier: String::new(),
        paper_exit_price: None,
        exit_time_ist: None,
        exit_reason: String::new(),
        realized_return: None,
        max_adverse_excursion: Some(0.0),
        max_favourable_excursion: Some(0.0),
        bars_held: 0,
        h300_counterfactual_ret: None,
        stop_consequence_pp: None,
        decision_id: Some(brief.id.clone()),
        entry_action: Some(brief.entry_action.clone()),
        oqs: Some(brief.oqs),
    }
}

pub struct DeferredLiveDriver {
    config: DeferredLiveConfig,
    ledger: LivePaperLedger,
}

impl DeferredLiveDriver {
    pub fn new(config: DeferredLiveConfig) -> Self {
        Self {
            config,
            ledger: LivePaperLedger::new(),
        }
    }

    pub fn ledger(&self) -> &LivePaperLedger {
        &self.ledger
    }

    pub fn open_from_brief(
        &mut self,
        brief: &DecisionBrief,
        obs: &MarketObservation,
    ) -> Result<&LivePaperPosition, OpenError> {
        if brief.id.trim().is_empty() {
            return Err(OpenError::MissingDecisionId);
        }
        if brief.ticker != obs.ticker {
            return Err(OpenError::TickerMismatch);
        }
        if !obs.price.is_finite() || obs.price <= 0.0 {
            return Err(OpenError::InvalidPrice);
        }
        let target = brief
            .execution
            .adaptive_target
            .filter(|v| v.is_finite())
            .ok_or(OpenError::MissingTarget)?;
        let risk = brief
            .execution
            .adaptive_risk
            .filter(|v| v.is_finite())
            .ok_or(OpenError::MissingRisk)?;
        if self
            .ledger
            .positions
            .iter()
            .any(|p| p.paper.decision_id.as_deref() == Some(brief.id.as_str()) && p.walk.is_open())
        {
            return Err(OpenError::AlreadyOpen);
        }

        let t0 = brief.execution.snap_unix.unwrap_or(obs.unix);
        let fill = obs.price;
        let paper = empty_paper(brief, fill, target, risk);
        let walk = PaperWalkState::open(&brief.direction, fill, target, Some(risk));

        self.ledger.positions.push(LivePaperPosition {
            paper,
            walk,
            current_price: fill,
            opened_at: obs.unix,
            last_tick_at: obs.unix,
            horizon: HorizonPolicy::Unix {
                horizon_unix: t0 + self.config.horizon_secs,
            },
            events: vec![LivePaperEvent {
                unix: obs.unix,
                kind: "PAPER_ENTER".into(),
                price: fill,
                note: Some(brief.direction.clone()),
            }],
        });
        Ok(self.ledger.positions.last().expect("just pushed"))
    }

    pub fn on_observation(&mut self, obs: &MarketObservation) -> Vec<LivePaperEvent> {
        let mut emitted = Vec::new();
        if !obs.price.is_finite() || obs.price <= 0.0 {
            return emitted;
        }
        let paper_obs = obs.to_paper();
        for pos in &mut self.ledger.positions {
            if !pos.walk.is_open() || pos.paper.ticker != obs.ticker {
                continue;
            }
            if obs.unix < pos.last_tick_at {
                continue;
            }
            pos.current_price = obs.price;
            pos.last_tick_at = obs.unix;
            if let Some(exit) = apply_observation(&mut pos.walk, &paper_obs, pos.horizon) {
                sync_paper_from_walk(&mut pos.paper, &pos.walk);
                pos.current_price = exit.price;
                let ev = LivePaperEvent {
                    unix: exit.unix,
                    kind: event_kind(exit.reason).into(),
                    price: exit.price,
                    note: Some(exit.reason.as_str().to_string()),
                };
                pos.events.push(ev.clone());
                emitted.push(ev);
            } else {
                sync_paper_from_walk(&mut pos.paper, &pos.walk);
            }
        }
        emitted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::product::intraday_decision::ExecutionFacts;

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
    fn ledger_starts_empty_and_is_not_csv_replay() {
        let d = DeferredLiveDriver::new(DeferredLiveConfig::default());
        assert_eq!(d.ledger().mode, "DEFERRED_LIVE");
        assert!(d.ledger().positions.is_empty());
    }

    #[test]
    fn paper_fill_is_live_observation_not_decision_entry() {
        let mut d = DeferredLiveDriver::new(DeferredLiveConfig::default());
        let b = brief(
            "LIVE-005-20260914-1000-JUBLFOOD_NS",
            "JUBLFOOD_NS",
            "SHORT",
            469.00,
            470.00,
            facts(467.30, 472.10, 1_000),
        );
        let obs = MarketObservation::last("JUBLFOOD_NS", 1_000, 470.04);
        let pos = d.open_from_brief(&b, &obs).unwrap();
        assert_eq!(pos.paper_entry_price(), 470.04);
        assert_eq!(b.entry_price, Some(469.00));
        assert_eq!(b.reference_price, Some(470.00));
        assert_ne!(pos.paper_entry_price(), b.entry_price.unwrap());
        assert_eq!(pos.action(), LivePaperAction::Hold);
        assert_eq!(pos.status(), LivePaperStatus::Open);
        assert_eq!(pos.events[0].kind, "PAPER_ENTER");
    }

    #[test]
    fn overlay_live_quote_does_not_mutate_decision_prices() {
        let mut b = brief("id", "T", "LONG", 100.0, 101.0, facts(102.0, 99.0, 1));
        overlay_live_quote(&mut b, 100.5, 50);
        assert_eq!(b.entry_price, Some(100.0));
        assert_eq!(b.reference_price, Some(101.0));
        assert_eq!(b.execution.current_price, Some(100.5));
        assert_eq!(b.execution.freshness, "LIVE");
    }

    #[test]
    fn long_target_uses_shared_lifecycle() {
        let mut d = DeferredLiveDriver::new(DeferredLiveConfig::default());
        let b = brief("d1", "AAA_NS", "LONG", 99.5, 100.0, facts(102.0, 98.0, 0));
        d.open_from_brief(&b, &MarketObservation::last("AAA_NS", 0, 100.0))
            .unwrap();
        let events = d.on_observation(&MarketObservation {
            ticker: "AAA_NS".into(),
            unix: 10,
            price: 101.9,
            high: Some(102.1),
            low: Some(101.5),
        });
        assert_eq!(events[0].kind, "PAPER_EXIT");
        assert_eq!(events[0].note.as_deref(), Some("TARGET"));
        let pos = &d.ledger().positions[0];
        assert_eq!(pos.paper_exit_price(), Some(102.0));
        assert_eq!(pos.status(), LivePaperStatus::Exited);
        assert!((pos.realized_return().unwrap() - 0.02).abs() < 1e-12);
    }

    #[test]
    fn short_hold_then_unix_horizon() {
        let mut d = DeferredLiveDriver::new(DeferredLiveConfig { horizon_secs: 60, strict_t0_admission: false });
        let b = brief("d1", "JUBLFOOD_NS", "SHORT", 469.0, 470.0, facts(467.30, 472.10, 1_000));
        d.open_from_brief(&b, &MarketObservation::last("JUBLFOOD_NS", 1_000, 470.04))
            .unwrap();
        assert!(d
            .on_observation(&MarketObservation::last("JUBLFOOD_NS", 1_030, 468.82))
            .is_empty());
        let pos = &d.ledger().positions[0];
        assert_eq!(pos.status(), LivePaperStatus::Open);
        assert_eq!(pos.current_price, 468.82);

        let done = d.on_observation(&MarketObservation::last("JUBLFOOD_NS", 1_060, 468.50));
        assert_eq!(done[0].kind, "HORIZON");
        let pos = &d.ledger().positions[0];
        assert_eq!(pos.status(), LivePaperStatus::Horizon);
        assert_eq!(pos.paper_exit_price(), Some(468.50));
        assert_eq!(pos.exit_reason(), Some("HORIZON"));
    }

    #[test]
    fn refuses_duplicate_open_and_out_of_order_ticks() {
        let mut d = DeferredLiveDriver::new(DeferredLiveConfig::default());
        let b = brief("d1", "AAA_NS", "LONG", 100.0, 100.0, facts(102.0, 98.0, 0));
        d.open_from_brief(&b, &MarketObservation::last("AAA_NS", 50, 100.0))
            .unwrap();
        assert_eq!(
            d.open_from_brief(&b, &MarketObservation::last("AAA_NS", 51, 100.1)),
            Err(OpenError::AlreadyOpen)
        );
        d.on_observation(&MarketObservation::last("AAA_NS", 40, 97.0));
        let pos = &d.ledger().positions[0];
        assert_eq!(pos.status(), LivePaperStatus::Open);
        assert_eq!(pos.current_price, 100.0);
        assert_eq!(pos.paper.bars_held, 0);
    }

    #[test]
    fn same_observations_are_deterministic() {
        let run = || {
            let mut d = DeferredLiveDriver::new(DeferredLiveConfig { horizon_secs: 1_000, strict_t0_admission: false });
            let b = brief("d1", "AAA_NS", "LONG", 99.0, 100.0, facts(102.0, 98.0, 0));
            d.open_from_brief(&b, &MarketObservation::last("AAA_NS", 0, 100.0))
                .unwrap();
            d.on_observation(&MarketObservation::last("AAA_NS", 1, 100.2));
            d.on_observation(&MarketObservation {
                ticker: "AAA_NS".into(),
                unix: 2,
                price: 98.1,
                high: Some(98.2),
                low: Some(97.9),
            });
            d.ledger().positions[0].clone()
        };
        assert_eq!(run(), run());
    }
}
