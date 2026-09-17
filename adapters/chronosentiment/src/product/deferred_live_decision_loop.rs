//! Increment 1 — Deferred Live ledger/clock harness.
//!
//! Observer/orchestrator around [`DeferredLiveRuntime`]. Does not calculate
//! recommendations. Does not change fill eligibility, TARGET / STOP / HORIZON,
//! H, D/E/G, reassessment, or INVERT.
//!
//! ```text
//! MarketObservation
//!       │
//!       ├──> DeferredLiveRuntime.ingest / ingest_sourced
//!       │          └──> existing ledger behavior
//!       └──> DecisionSurface snapshot at obs.unix
//! ```

use serde::{Deserialize, Serialize};

use super::deferred_live::{
    DeferredLiveConfig, LivePaperEvent, LivePaperStatus, MarketObservation,
};
use super::deferred_live_loop::{ArmedBrief, DeferredLiveRuntime, IngestOutcome};
use super::intraday_decision::DecisionBrief;
use super::live_observation::{
    ControlledObservationTape, ObservationFeedStatus, SourcedObservation,
};

/// Frozen actionable identity: the armed session DecisionBrief, not a new IC.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionSurface {
    pub as_of_unix: i64,
    pub last_ingest_unix: Option<i64>,
    pub feed_status: ObservationFeedStatus,
    pub armed: Vec<ArmedBrief>,
    /// Same as `armed` in Increment 1 — frozen session briefs, not a new recommendation.
    pub current_actionable: Vec<ArmedBrief>,
    pub open: usize,
    pub exited: usize,
    pub horizon: usize,
    pub opened: bool,
    pub last_tick_at: Option<i64>,
    pub events: Vec<LivePaperEvent>,
    pub open_error: Option<String>,
}

/// Thin sidecar. The inner [`DeferredLiveRuntime`] remains authoritative.
pub struct DecisionLoopRuntime {
    runtime: DeferredLiveRuntime,
    surfaces: Vec<DecisionSurface>,
}

impl DecisionLoopRuntime {
    pub fn new(config: DeferredLiveConfig) -> Self {
        Self {
            runtime: DeferredLiveRuntime::new(config),
            surfaces: Vec::new(),
        }
    }

    pub fn runtime(&self) -> &DeferredLiveRuntime {
        &self.runtime
    }

    pub fn surfaces(&self) -> &[DecisionSurface] {
        &self.surfaces
    }

    /// Delegate arming. Does not add eligibility rules.
    pub fn arm(&mut self, brief: DecisionBrief) {
        self.runtime.arm(brief);
    }

    /// Delegate session auto-arm. Does not add eligibility rules.
    pub fn begin_session(&mut self, briefs: &[DecisionBrief], date: &str) -> Vec<String> {
        self.runtime.begin_session(briefs, date)
    }

    pub fn enable_reassess_experiment(&mut self) {
        self.runtime.enable_reassess_experiment();
    }

    pub fn set_feed_snapshot(&mut self, snap: super::live_observation::ObservationFeedSnapshot) {
        self.runtime.set_feed_snapshot(snap);
    }

    /// Snapshot without ingesting. Used for the no-observation WAITING case.
    pub fn snapshot_waiting(&self) -> DecisionSurface {
        self.capture(0, &IngestOutcome {
            opened: false,
            open_error: None,
            events: vec![],
        })
    }

    pub fn step(&mut self, obs: MarketObservation) -> DecisionSurface {
        let as_of = obs.unix;
        let outcome = self.runtime.ingest(obs);
        self.push_surface(as_of, &outcome)
    }

    pub fn step_sourced(&mut self, sourced: SourcedObservation) -> DecisionSurface {
        let as_of = sourced.observation.unix;
        let outcome = self.runtime.ingest_sourced(sourced);
        self.push_surface(as_of, &outcome)
    }

    /// Play a controlled tape through existing ingest, then attach the tape's
    /// feed snapshot so `TapeExhausted` is the producer's status, not a new rule.
    pub fn play_tape(&mut self, mut tape: ControlledObservationTape) -> Vec<DecisionSurface> {
        let mut out = Vec::new();
        while let Some(sourced) = tape.next() {
            out.push(self.step_sourced(sourced));
        }
        self.runtime.set_feed_snapshot(tape.snapshot());
        out
    }

    /// Mark the feed exhausted using the existing snapshot field. Not a driver change.
    pub fn mark_tape_exhausted(&mut self) {
        let mut snap = self.runtime.feed().clone();
        snap.status = ObservationFeedStatus::TapeExhausted;
        snap.clock_simulation = true;
        self.runtime.set_feed_snapshot(snap);
    }

    fn push_surface(&mut self, as_of: i64, outcome: &IngestOutcome) -> DecisionSurface {
        let surface = self.capture(as_of, outcome);
        self.surfaces.push(surface.clone());
        const KEEP: usize = 512;
        if self.surfaces.len() > KEEP {
            let drop = self.surfaces.len() - KEEP;
            self.surfaces.drain(0..drop);
        }
        surface
    }

    fn capture(&self, as_of: i64, outcome: &IngestOutcome) -> DecisionSurface {
        let armed = self.runtime.armed();
        let mut open = 0;
        let mut exited = 0;
        let mut horizon = 0;
        let mut last_tick_at = None;
        for pos in &self.runtime.ledger().positions {
            match pos.status() {
                LivePaperStatus::Open => open += 1,
                LivePaperStatus::Exited => exited += 1,
                LivePaperStatus::Horizon => horizon += 1,
            }
            last_tick_at = Some(pos.last_tick_at.max(last_tick_at.unwrap_or(pos.last_tick_at)));
        }
        DecisionSurface {
            as_of_unix: as_of,
            last_ingest_unix: self.runtime.feed().last_ingest_unix,
            feed_status: self.runtime.feed().status.clone(),
            current_actionable: armed.clone(),
            armed,
            open,
            exited,
            horizon,
            opened: outcome.opened,
            last_tick_at,
            events: outcome.events.clone(),
            open_error: outcome.open_error.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::product::deferred_live_loop::lifecycle_scenario_tape;
    use crate::product::intraday_decision::ExecutionFacts;
    use crate::product::{LifecycleScenario, LivePaperStatus};

    const TICKER: &str = "JUBLFOOD_NS";
    const SNAP: i64 = 1_000_000;
    const FILL: f64 = 480.5;

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

    fn act_brief(id: &str, snap: i64, target: f64, risk: f64) -> DecisionBrief {
        DecisionBrief {
            id: id.into(),
            ticker: TICKER.into(),
            date: "2026-09-07".into(),
            direction: "SHORT".into(),
            oqs: 53,
            h60_class: "WAIT".into(),
            reference_price: Some(475.4),
            entry_price: Some(473.0),
            execution: facts(target, risk, snap),
            entry_state: "WAIT-HIGH".into(),
            entry_action: "ACT".into(),
            entry_confidence: "HIGH".into(),
            entry_horizon: "H300".into(),
            entry_why: "frozen session brief".into(),
            entry_risk: "frozen session brief".into(),
            h120_state: "WAIT-HIGH".into(),
            h120_action: "ACT".into(),
            h120_confidence: "HIGH".into(),
            h120_horizon: "H300".into(),
            h120_why: "frozen session brief".into(),
            h120_risk: "frozen session brief".into(),
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

    fn obs(unix: i64, price: f64) -> MarketObservation {
        MarketObservation {
            ticker: TICKER.into(),
            unix,
            price,
            high: Some(price),
            low: Some(price),
        }
    }

    fn armed_loop(horizon_secs: i64) -> DecisionLoopRuntime {
        let mut rt = DecisionLoopRuntime::new(DeferredLiveConfig { horizon_secs, ..Default::default() });
        rt.arm(act_brief("LIVE-HARNESS-JUBLFOOD_NS", SNAP, 450.0, 490.0));
        rt
    }

    #[test]
    fn tick_before_snap_enters_at_that_tick_frozen_a() {
        let mut rt = armed_loop(300 * 60);
        let before = SNAP - 60;
        let surface = rt.step(obs(before, FILL));
        assert!(surface.opened);
        assert_eq!(surface.open, 1);
        assert!(surface.armed.is_empty());
        assert!(surface.current_actionable.is_empty());
        let pos = &rt.runtime().ledger().positions[0];
        assert_eq!(pos.status(), LivePaperStatus::Open);
        assert_eq!(pos.paper_entry_price(), FILL);
        assert_eq!(pos.opened_at, before);
        assert_eq!(pos.last_tick_at, before);
        assert_eq!(surface.last_ingest_unix, Some(before));
        assert_ne!(before, SNAP, "locks current A: fill may precede snap_unix");
    }

    #[test]
    fn tick_exactly_at_snap_enters() {
        let mut rt = armed_loop(300 * 60);
        let surface = rt.step(obs(SNAP, FILL));
        assert!(surface.opened);
        assert_eq!(surface.open, 1);
        let pos = &rt.runtime().ledger().positions[0];
        assert_eq!(pos.opened_at, SNAP);
        assert_eq!(pos.paper_entry_price(), FILL);
        assert_eq!(pos.last_tick_at, SNAP);
    }

    #[test]
    fn first_tick_after_snap_enters_when_no_earlier_observation() {
        let mut rt = armed_loop(300 * 60);
        let after = SNAP + 60;
        let surface = rt.step(obs(after, FILL));
        assert!(surface.opened);
        assert_eq!(rt.runtime().ledger().positions[0].opened_at, after);
        assert_eq!(rt.runtime().ledger().positions[0].last_tick_at, after);
    }

    #[test]
    fn no_observation_remains_armed_waiting() {
        let rt = armed_loop(300 * 60);
        let surface = rt.snapshot_waiting();
        assert!(!surface.opened);
        assert_eq!(surface.open, 0);
        assert_eq!(surface.armed.len(), 1);
        assert_eq!(surface.current_actionable.len(), 1);
        assert_eq!(surface.current_actionable[0].decision_id, "LIVE-HARNESS-JUBLFOOD_NS");
        assert!(rt.runtime().ledger().positions.is_empty());
        assert_eq!(surface.feed_status, ObservationFeedStatus::WaitingForFeed);
        assert!(surface.last_ingest_unix.is_none());
    }

    #[test]
    fn tape_exhaustion_leaves_open_and_tape_exhausted() {
        let mut rt = armed_loop(300 * 60);
        let tape = ControlledObservationTape::new(
            crate::product::ObservationSourceKind::Cached1m,
            vec![obs(SNAP, FILL)],
            0.0,
        );
        rt.play_tape(tape);
        assert_eq!(rt.runtime().ledger().positions[0].status(), LivePaperStatus::Open);
        assert_eq!(rt.runtime().feed().status, ObservationFeedStatus::TapeExhausted);
        let last = rt.surfaces().last().unwrap();
        assert_eq!(last.open, 1);
        assert_eq!(last.horizon, 0);
        assert_eq!(last.exited, 0);
        assert_eq!(last.last_tick_at, Some(SNAP));
    }

    #[test]
    fn tick_at_horizon_unix_exits_horizon_at_last() {
        let mut rt = armed_loop(180);
        rt.step(obs(SNAP, FILL));
        let horizon_tick = SNAP + 180;
        let surface = rt.step(obs(horizon_tick, 475.4));
        assert_eq!(surface.horizon, 1);
        assert_eq!(surface.open, 0);
        let pos = &rt.runtime().ledger().positions[0];
        assert_eq!(pos.status(), LivePaperStatus::Horizon);
        assert_eq!(pos.exit_reason(), Some("HORIZON"));
        assert_eq!(pos.paper_exit_price(), Some(475.4));
        assert_eq!(pos.last_tick_at, horizon_tick);
    }

    #[test]
    fn target_and_stop_on_same_observation_target_wins() {
        let (brief, tape, cfg) = lifecycle_scenario_tape(LifecycleScenario::Target, 0.0);
        let mut rt = DecisionLoopRuntime::new(cfg);
        rt.arm(brief);
        rt.play_tape(tape);
        let pos = &rt.runtime().ledger().positions[0];
        assert_eq!(pos.status(), LivePaperStatus::Exited);
        assert_eq!(pos.exit_reason(), Some("TARGET"));
        assert_eq!(pos.paper_exit_price(), Some(475.0));
        assert_eq!(rt.surfaces().last().unwrap().exited, 1);
    }

    #[test]
    fn two_ticks_same_unix_both_applied() {
        let mut rt = armed_loop(300 * 60);
        rt.step(obs(SNAP, FILL));
        let same = SNAP + 60;
        rt.step(MarketObservation {
            ticker: TICKER.into(),
            unix: same,
            price: 479.0,
            high: Some(479.0),
            low: Some(479.0),
        });
        rt.step(MarketObservation {
            ticker: TICKER.into(),
            unix: same,
            price: 478.0,
            high: Some(478.5),
            low: Some(477.5),
        });
        let pos = &rt.runtime().ledger().positions[0];
        assert_eq!(pos.status(), LivePaperStatus::Open);
        assert_eq!(pos.last_tick_at, same);
        assert_eq!(pos.current_price, 478.0);
        assert_eq!(rt.surfaces().len(), 3);
        assert_eq!(rt.surfaces()[1].as_of_unix, same);
        assert_eq!(rt.surfaces()[2].as_of_unix, same);
        assert_eq!(rt.surfaces()[2].last_ingest_unix, Some(same));
    }

    #[test]
    fn same_tape_twice_identical_surfaces_and_events() {
        fn play() -> (Vec<DecisionSurface>, Vec<LivePaperEvent>) {
            let mut rt = armed_loop(180);
            rt.step(obs(SNAP, FILL));
            rt.step(obs(SNAP + 60, 479.0));
            rt.step(obs(SNAP + 180, 475.4));
            let events: Vec<LivePaperEvent> = rt
                .runtime()
                .ledger()
                .positions
                .iter()
                .flat_map(|p| p.events.clone())
                .collect();
            (rt.surfaces().to_vec(), events)
        }
        let (s1, e1) = play();
        let (s2, e2) = play();
        assert_eq!(s1, s2);
        assert_eq!(e1, e2);
        assert_eq!(e1.first().map(|e| e.kind.as_str()), Some("PAPER_ENTER"));
        assert_eq!(e1.last().map(|e| e.kind.as_str()), Some("HORIZON"));
    }
}
