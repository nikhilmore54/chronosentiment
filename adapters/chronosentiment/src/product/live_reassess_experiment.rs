//! Live-paper reassessment **experiment** (v0.2 EXIT-only + adverse-mark gate).
//!
//! Observes the Deferred Live book. Does **not** mutate fills, TARGET / STOP /
//! HORIZON, Stage C marks, or `paper_trader_v2` CSVs. INVERT is disabled.
//!
//! ```text
//! as-of observation
//!       ↓
//! minimum-hold gate
//!       ↓
//! path-shape deterioration   (baseline v0.2)
//!       ↓
//! mark adverse?              (economic gate)
//!       ↓
//! REASSESS_EXIT
//! ```
//!
//! Baseline v0.2 is retained on the same stream for comparison. The experiment
//! under test is path-shape **and** as-of signed mark < 0.
//!
//! See `docs/CS-P-001_DECISION_SUPPORT_PRODUCT_MODE.md` Stage B/C and
//! `docs/CS-P-001-C_DEFERRED_LIVE_REASSESS_SIDECAR_20260907.md` (branch closed).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::deferred_live::{LivePaperLedger, LivePaperStatus, MarketObservation};
use super::paper_lifecycle::signed_return;

pub const DOWNTREND_WINDOW: usize = 5;
pub const MIN_HOLD_BARS: i32 = 6;
pub const LOWER_COUNT_MIN: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HlBar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub unix: i64,
}

/// Adverse structural trend: LONG lower-highs+lower-lows; SHORT higher-highs+higher-lows.
pub fn is_sustained_adverse_trend(direction: &str, history: &[HlBar]) -> bool {
    if history.len() < DOWNTREND_WINDOW + 1 {
        return false;
    }
    let recent = &history[history.len() - (DOWNTREND_WINDOW + 1)..];
    let mut adverse_highs = 0usize;
    let mut adverse_lows = 0usize;
    let long = direction.eq_ignore_ascii_case("LONG");
    for w in recent.windows(2) {
        let prev = w[0];
        let curr = w[1];
        if long {
            if curr.high < prev.high {
                adverse_highs += 1;
            }
            if curr.low < prev.low {
                adverse_lows += 1;
            }
        } else {
            if curr.low > prev.low {
                adverse_highs += 1;
            }
            if curr.high > prev.high {
                adverse_lows += 1;
            }
        }
    }
    adverse_highs >= LOWER_COUNT_MIN && adverse_lows >= LOWER_COUNT_MIN
}

/// As-of signed mark is underwater. Flat (0) is not adverse.
pub fn mark_is_adverse(direction: &str, entry: f64, close: f64) -> bool {
    signed_return(direction, entry, close) < 0.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReassessSignal {
    pub unix: i64,
    pub price: f64,
    pub bars_held: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReassessExperimentRow {
    pub decision_id: String,
    pub ticker: String,
    pub direction: String,
    pub paper_entry_price: f64,
    pub frozen_outcome: String,
    pub frozen_ret: f64,
    pub frozen_ret_kind: String,
    /// Baseline v0.2: path-shape only, no economic gate.
    pub v02_triggered: bool,
    pub v02_price: Option<f64>,
    pub v02_unix: Option<i64>,
    pub v02_ret: Option<f64>,
    pub v02_delta_vs_frozen: Option<f64>,
    /// Gated experiment: path-shape **and** as-of mark < 0.
    pub reassess_triggered: bool,
    pub reassess_price: Option<f64>,
    pub reassess_unix: Option<i64>,
    pub experiment_ret: Option<f64>,
    pub delta_vs_frozen: Option<f64>,
    pub suppressed_by_mark_gate: bool,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReassessVariantStats {
    pub n_triggered: usize,
    pub n_helped: usize,
    pub n_hurt: usize,
    pub mean_delta_vs_frozen: Option<f64>,
    pub sum_delta_vs_frozen: f64,
}

impl ReassessVariantStats {
    fn empty() -> Self {
        Self {
            n_triggered: 0,
            n_helped: 0,
            n_hurt: 0,
            mean_delta_vs_frozen: None,
            sum_delta_vs_frozen: 0.0,
        }
    }

    fn from_deltas(deltas: &[f64]) -> Self {
        if deltas.is_empty() {
            return Self::empty();
        }
        let n_helped = deltas.iter().filter(|d| **d > 0.0).count();
        let n_hurt = deltas.iter().filter(|d| **d < 0.0).count();
        let sum = deltas.iter().sum::<f64>();
        Self {
            n_triggered: deltas.len(),
            n_helped,
            n_hurt,
            mean_delta_vs_frozen: Some(sum / deltas.len() as f64),
            sum_delta_vs_frozen: sum,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReassessExperimentReport {
    pub enabled: bool,
    pub invert_enabled: bool,
    /// Pipeline under test. Baseline v0.2 is `baseline`; this report's
    /// `n_triggered` / `rows[].reassess_triggered` are the gated rule.
    pub rule: String,
    pub n_open_watched: usize,
    pub baseline: ReassessVariantStats,
    pub n_triggered: usize,
    pub n_helped: usize,
    pub n_hurt: usize,
    pub mean_delta_vs_frozen: Option<f64>,
    pub sum_delta_vs_frozen: f64,
    pub n_suppressed_by_mark_gate: usize,
    pub rows: Vec<ReassessExperimentRow>,
    pub note: String,
}

impl ReassessExperimentReport {
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            invert_enabled: false,
            rule: "off".into(),
            n_open_watched: 0,
            baseline: ReassessVariantStats::empty(),
            n_triggered: 0,
            n_helped: 0,
            n_hurt: 0,
            mean_delta_vs_frozen: None,
            sum_delta_vs_frozen: 0.0,
            n_suppressed_by_mark_gate: 0,
            rows: vec![],
            note: "Live reassess experiment off. Frozen Deferred Live book and Stage C marks are unchanged. INVERT disabled.".into(),
        }
    }
}

#[derive(Debug, Clone)]
struct Track {
    history: Vec<HlBar>,
    v02_signal: Option<ReassessSignal>,
    gated_signal: Option<ReassessSignal>,
}

#[derive(Debug, Clone)]
pub struct LiveReassessExperiment {
    enabled: bool,
    tracks: HashMap<String, Track>,
}

impl LiveReassessExperiment {
    pub fn off() -> Self {
        Self {
            enabled: false,
            tracks: HashMap::new(),
        }
    }

    pub fn on() -> Self {
        Self {
            enabled: true,
            tracks: HashMap::new(),
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// After frozen ingest. Uses only this observation and prior as-of history.
    pub fn observe(&mut self, ledger: &LivePaperLedger, obs: &MarketObservation) {
        if !self.enabled {
            return;
        }
        let high = obs.high.unwrap_or(obs.price);
        let low = obs.low.unwrap_or(obs.price);
        let bar = HlBar {
            high,
            low,
            close: obs.price,
            unix: obs.unix,
        };
        for pos in &ledger.positions {
            if pos.paper.ticker != obs.ticker {
                continue;
            }
            let id = match pos.paper.decision_id.as_deref() {
                Some(id) if !id.is_empty() => id.to_string(),
                _ => continue,
            };
            let track = self.tracks.entry(id).or_insert_with(|| Track {
                history: Vec::new(),
                v02_signal: None,
                gated_signal: None,
            });
            if pos.status() != LivePaperStatus::Open {
                continue;
            }
            if track.history.last().is_some_and(|prev| prev.unix == bar.unix) {
                continue;
            }
            track.history.push(bar);
            if pos.paper.bars_held < MIN_HOLD_BARS {
                continue;
            }
            if !is_sustained_adverse_trend(&pos.paper.direction, &track.history) {
                continue;
            }
            let sig = ReassessSignal {
                unix: obs.unix,
                price: obs.price,
                bars_held: pos.paper.bars_held,
            };
            if track.v02_signal.is_none() {
                track.v02_signal = Some(sig.clone());
            }
            if track.gated_signal.is_none()
                && mark_is_adverse(&pos.paper.direction, pos.paper_entry_price(), obs.price)
            {
                track.gated_signal = Some(sig);
            }
        }
    }

    pub fn report(&self, ledger: &LivePaperLedger) -> ReassessExperimentReport {
        if !self.enabled {
            return ReassessExperimentReport::disabled();
        }
        let mut rows = Vec::new();
        let mut v02_deltas = Vec::new();
        let mut gated_deltas = Vec::new();
        let mut n_suppressed = 0usize;
        for pos in &ledger.positions {
            let id = pos.paper.decision_id.clone().unwrap_or_default();
            let open = pos.status() == LivePaperStatus::Open;
            let frozen_ret = if open {
                pos.unrealized_return()
            } else {
                pos.realized_return().unwrap_or(0.0)
            };
            let frozen_outcome = pos.exit_reason().unwrap_or("OPEN").to_string();
            let track = self.tracks.get(&id);
            let v02 = track.and_then(|t| t.v02_signal.clone());
            let gated = track.and_then(|t| t.gated_signal.clone());
            let v02_ret = v02.as_ref().map(|s| {
                signed_return(&pos.paper.direction, pos.paper_entry_price(), s.price)
            });
            let v02_delta = v02_ret.map(|r| r - frozen_ret);
            if let Some(d) = v02_delta {
                v02_deltas.push(d);
            }
            let experiment_ret = gated.as_ref().map(|s| {
                signed_return(&pos.paper.direction, pos.paper_entry_price(), s.price)
            });
            let delta = experiment_ret.map(|r| r - frozen_ret);
            if let Some(d) = delta {
                gated_deltas.push(d);
            }
            let suppressed = v02.is_some() && gated.is_none();
            if suppressed {
                n_suppressed += 1;
            }
            let note = if gated.is_some() {
                "Would have REASSESS_EXIT (path-shape + adverse mark). Frozen book was not changed."
                    .to_string()
            } else if suppressed {
                "v0.2 path-shape fired while mark was still favorable; economic gate held the position."
                    .to_string()
            } else {
                "No v0.2 EXIT signal while frozen book was OPEN.".to_string()
            };
            rows.push(ReassessExperimentRow {
                decision_id: id,
                ticker: pos.paper.ticker.clone(),
                direction: pos.paper.direction.clone(),
                paper_entry_price: pos.paper_entry_price(),
                frozen_outcome,
                frozen_ret,
                frozen_ret_kind: if open { "MARK".into() } else { "REALIZED".into() },
                v02_triggered: v02.is_some(),
                v02_price: v02.as_ref().map(|s| s.price),
                v02_unix: v02.as_ref().map(|s| s.unix),
                v02_ret,
                v02_delta_vs_frozen: v02_delta,
                reassess_triggered: gated.is_some(),
                reassess_price: gated.as_ref().map(|s| s.price),
                reassess_unix: gated.as_ref().map(|s| s.unix),
                experiment_ret,
                delta_vs_frozen: delta,
                suppressed_by_mark_gate: suppressed,
                note,
            });
        }
        rows.sort_by(|a, b| a.ticker.cmp(&b.ticker));
        let gated = ReassessVariantStats::from_deltas(&gated_deltas);
        ReassessExperimentReport {
            enabled: true,
            invert_enabled: false,
            rule: "min_hold → path_shape → mark_adverse".into(),
            n_open_watched: ledger.positions.len(),
            baseline: ReassessVariantStats::from_deltas(&v02_deltas),
            n_triggered: gated.n_triggered,
            n_helped: gated.n_helped,
            n_hurt: gated.n_hurt,
            mean_delta_vs_frozen: gated.mean_delta_vs_frozen,
            sum_delta_vs_frozen: gated.sum_delta_vs_frozen,
            n_suppressed_by_mark_gate: n_suppressed,
            rows,
            note: "Live-paper experiment · sidecar only · INVERT=false. \
                   Baseline = v0.2 path-shape. Experiment = path-shape AND as-of signed mark < 0. \
                   Does not mutate Deferred Live fills or Stage C marks. \
                   delta_vs_frozen = experiment_ret − frozen MARK/REALIZED (positive = would have helped). \
                   Not a G-GATE predictive-value claim."
                .into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hl(high: f64, low: f64) -> HlBar {
        HlBar {
            high,
            low,
            close: (high + low) / 2.0,
            unix: 0,
        }
    }

    #[test]
    fn long_sustained_lower_highs_and_lows() {
        let hist = vec![
            hl(10.0, 9.0),
            hl(9.5, 8.8),
            hl(9.2, 8.5),
            hl(9.0, 8.2),
            hl(8.7, 8.0),
            hl(8.4, 7.7),
        ];
        assert!(is_sustained_adverse_trend("LONG", &hist));
    }

    #[test]
    fn short_needs_higher_highs_and_higher_lows() {
        let hist = vec![
            hl(10.0, 9.0),
            hl(10.2, 9.3),
            hl(10.5, 9.6),
            hl(10.8, 9.9),
            hl(11.0, 10.1),
            hl(11.3, 10.4),
        ];
        assert!(is_sustained_adverse_trend("SHORT", &hist));
        assert!(!is_sustained_adverse_trend("LONG", &hist));
    }

    #[test]
    fn min_history_is_window_plus_one() {
        let hist = vec![hl(10.0, 9.0); 5];
        assert!(!is_sustained_adverse_trend("LONG", &hist));
    }

    use crate::product::deferred_live::{DeferredLiveConfig, MarketObservation};
    use crate::product::deferred_live_loop::DeferredLiveRuntime;
    use crate::product::deferred_live_performance::score_deferred_live;
    use crate::product::intraday_decision::ExecutionFacts;
    use crate::product::DecisionBrief;

    fn long_brief() -> DecisionBrief {
        DecisionBrief {
            id: "d-long".into(),
            ticker: "AAA_NS".into(),
            date: "2026-09-07".into(),
            direction: "LONG".into(),
            oqs: 50,
            h60_class: "WAIT".into(),
            reference_price: Some(10.0),
            entry_price: Some(10.0),
            execution: ExecutionFacts {
                snap_unix: Some(1_000),
                adaptive_target: Some(1_000.0),
                adaptive_risk: Some(0.01),
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
            entry_state: "WAIT-HIGH".into(),
            entry_action: "ACT".into(),
            entry_confidence: "HIGH".into(),
            entry_horizon: "H300".into(),
            entry_why: "t".into(),
            entry_risk: "t".into(),
            h120_state: "WAIT-HIGH".into(),
            h120_action: "ACT".into(),
            h120_confidence: "HIGH".into(),
            h120_horizon: "H300".into(),
            h120_why: "t".into(),
            h120_risk: "t".into(),
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

    fn ohlc(unix: i64, close: f64, high: f64, low: f64) -> MarketObservation {
        MarketObservation {
            ticker: "AAA_NS".into(),
            unix,
            price: close,
            high: Some(high),
            low: Some(low),
        }
    }

    /// Six consecutive lower-highs / lower-lows. Target/stop far from path.
    fn adverse_long_tape() -> Vec<MarketObservation> {
        vec![
            ohlc(1_000, 9.5, 10.0, 9.0),
            ohlc(1_060, 9.1, 9.5, 8.8),
            ohlc(1_120, 8.8, 9.2, 8.5),
            ohlc(1_180, 8.5, 9.0, 8.2),
            ohlc(1_240, 8.2, 8.7, 8.0),
            ohlc(1_300, 8.0, 8.4, 7.7),
            ohlc(1_360, 7.6, 8.0, 7.4),
        ]
    }

    fn play(rt: &mut DeferredLiveRuntime, bars: &[MarketObservation]) {
        rt.arm(long_brief());
        for b in bars {
            rt.ingest(b.clone());
        }
    }

    #[test]
    fn min_hold_blocks_exit_before_six_live_bars() {
        let mut rt = DeferredLiveRuntime::new(DeferredLiveConfig { horizon_secs: 10_000, strict_t0_admission: false });
        rt.enable_reassess_experiment();
        play(&mut rt, &adverse_long_tape()[..5]);
        let rep = rt.reassess_experiment_report();
        assert!(rep.enabled);
        assert_eq!(rep.n_triggered, 0);
        assert_eq!(rt.ledger().positions[0].paper.bars_held, 5);
        assert!(!rep.rows[0].reassess_triggered);
    }

    #[test]
    fn long_adverse_path_fires_exit_only_after_min_hold() {
        let mut rt = DeferredLiveRuntime::new(DeferredLiveConfig { horizon_secs: 10_000, strict_t0_admission: false });
        rt.enable_reassess_experiment();
        play(&mut rt, &adverse_long_tape());
        let rep = rt.reassess_experiment_report();
        assert_eq!(rep.n_triggered, 1);
        assert!(rep.rows[0].reassess_triggered);
        assert!(rep.rows[0].v02_triggered);
        assert!(!rep.rows[0].suppressed_by_mark_gate);
        assert_eq!(rep.rows[0].reassess_price, Some(8.0));
        assert_eq!(rep.rows[0].frozen_outcome, "OPEN");
        assert_eq!(rep.invert_enabled, false);
        let frozen = rt.ledger().positions[0].unrealized_return();
        let exp = rep.rows[0].experiment_ret.unwrap();
        assert!((exp - signed_return("LONG", 9.5, 8.0)).abs() < 1e-12);
        assert!(rep.rows[0].delta_vs_frozen.unwrap() > 0.0, "earlier exit should beat a worse open mark");
        assert!(frozen < exp);
        assert_eq!(rep.baseline.n_triggered, 1);
    }

    #[test]
    fn frozen_target_wins_and_blocks_the_experiment() {
        let mut rt = DeferredLiveRuntime::new(DeferredLiveConfig { horizon_secs: 10_000, strict_t0_admission: false });
        rt.enable_reassess_experiment();
        let mut b = long_brief();
        b.execution.adaptive_target = Some(9.55);
        rt.arm(b);
        rt.ingest(ohlc(1_000, 9.5, 10.0, 9.0));
        rt.ingest(ohlc(1_060, 9.4, 9.6, 9.3));
        for (i, bar) in adverse_long_tape().into_iter().skip(2).enumerate() {
            let mut bar = bar;
            bar.unix = 1_120 + (i as i64) * 60;
            rt.ingest(bar);
        }
        assert_eq!(rt.ledger().positions[0].exit_reason(), Some("TARGET"));
        let rep = rt.reassess_experiment_report();
        assert_eq!(rep.n_triggered, 0);
        assert!(!rep.rows[0].reassess_triggered);
    }

    #[test]
    fn experiment_does_not_mutate_stage_c_or_the_frozen_book() {
        let cfg = DeferredLiveConfig { horizon_secs: 10_000, strict_t0_admission: false };
        let tape = adverse_long_tape();
        let mut frozen = DeferredLiveRuntime::new(cfg.clone());
        play(&mut frozen, &tape);
        let mut exp = DeferredLiveRuntime::new(cfg);
        exp.enable_reassess_experiment();
        play(&mut exp, &tape);
        assert_eq!(frozen.ledger(), exp.ledger());
        let briefs = [long_brief()];
        assert_eq!(
            score_deferred_live(frozen.ledger(), &briefs, Some("2026-09-07")),
            score_deferred_live(exp.ledger(), &briefs, Some("2026-09-07"))
        );
        assert!(exp.reassess_experiment_report().n_triggered > 0);
        assert!(!frozen.reassess_experiment_report().enabled);
    }

    #[test]
    fn same_tape_is_deterministic() {
        let cfg = DeferredLiveConfig { horizon_secs: 10_000, strict_t0_admission: false };
        let tape = adverse_long_tape();
        let mut a = DeferredLiveRuntime::new(cfg.clone());
        a.enable_reassess_experiment();
        play(&mut a, &tape);
        let mut b = DeferredLiveRuntime::new(cfg);
        b.enable_reassess_experiment();
        play(&mut b, &tape);
        assert_eq!(a.reassess_experiment_report(), b.reassess_experiment_report());
    }

    /// Path-shape present, but as-of close stays above the LONG fill.
    fn favorable_long_structure_tape() -> Vec<MarketObservation> {
        vec![
            ohlc(1_000, 10.00, 11.00, 9.90),
            ohlc(1_060, 10.20, 10.80, 9.85),
            ohlc(1_120, 10.15, 10.60, 9.80),
            ohlc(1_180, 10.10, 10.40, 9.75),
            ohlc(1_240, 10.05, 10.20, 9.70),
            ohlc(1_300, 10.02, 10.10, 9.65),
        ]
    }

    #[test]
    fn mark_flat_or_positive_is_not_adverse() {
        assert!(mark_is_adverse("LONG", 10.0, 9.9));
        assert!(!mark_is_adverse("LONG", 10.0, 10.0));
        assert!(!mark_is_adverse("LONG", 10.0, 10.1));
        assert!(mark_is_adverse("SHORT", 10.0, 10.1));
        assert!(!mark_is_adverse("SHORT", 10.0, 9.9));
    }

    #[test]
    fn favorable_mark_blocks_gated_exit_but_baseline_still_fires() {
        let mut rt = DeferredLiveRuntime::new(DeferredLiveConfig { horizon_secs: 10_000, strict_t0_admission: false });
        rt.enable_reassess_experiment();
        play(&mut rt, &favorable_long_structure_tape());
        let rep = rt.reassess_experiment_report();
        assert!(rep.rows[0].v02_triggered, "path-shape baseline should still fire");
        assert!(!rep.rows[0].reassess_triggered, "economic gate must hold a favorable mark");
        assert!(rep.rows[0].suppressed_by_mark_gate);
        assert_eq!(rep.n_triggered, 0);
        assert_eq!(rep.baseline.n_triggered, 1);
        assert_eq!(rep.n_suppressed_by_mark_gate, 1);
        assert!(rt.ledger().positions[0].unrealized_return() > 0.0);
    }

    #[test]
    fn gated_exit_can_fire_later_once_the_mark_turns_adverse() {
        let mut tape = favorable_long_structure_tape();
        tape.push(ohlc(1_360, 9.80, 10.00, 9.50));
        let mut rt = DeferredLiveRuntime::new(DeferredLiveConfig { horizon_secs: 10_000, strict_t0_admission: false });
        rt.enable_reassess_experiment();
        play(&mut rt, &tape);
        let rep = rt.reassess_experiment_report();
        assert!(rep.rows[0].v02_triggered);
        assert!(rep.rows[0].reassess_triggered);
        assert!(!rep.rows[0].suppressed_by_mark_gate);
        assert_eq!(rep.rows[0].reassess_price, Some(9.80));
        assert!(rep.rows[0].v02_price.unwrap() > 10.0);
    }
}
