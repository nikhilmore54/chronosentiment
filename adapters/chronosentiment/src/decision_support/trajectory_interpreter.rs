//! Position Trajectory Interpreter — Config A (Frozen Research Model)
//!
//! Evaluates qualitative lifecycle topology across multi-horizon bar series.
//! Invariant: Parameters (K=3 bars, G_max=0.70, dM_min=-0.10) and Coralys score formulas are 100% FROZEN.

use serde::{Deserialize, Serialize};

/// Config A Trajectory Parameters (FROZEN)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigAParameters {
    pub pullback_window_k: usize,   // K = 3 bars
    pub g_max: f64,                  // G_max = 0.70
    pub delta_m5_min: f64,           // dM_min = -0.10
}

impl Default for ConfigAParameters {
    fn default() -> Self {
        Self {
            pullback_window_k: 3,
            g_max: 0.70,
            delta_m5_min: -0.10,
        }
    }
}

/// Bar OHLCV struct for trajectory evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarObservation {
    pub timestamp: i64,
    pub datetime: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Evaluated Trajectory Output
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrajectoryEvaluation {
    pub label: String,
    pub score: f64,
    pub momentum_5: f64,
}

pub struct ConfigATrajectoryInterpreter {
    pub params: ConfigAParameters,
}

impl ConfigATrajectoryInterpreter {
    pub fn new() -> Self {
        Self {
            params: ConfigAParameters::default(),
        }
    }

    /// Computes frozen Coralys v0.1.0 trajectory score and M_5 trend momentum.
    pub fn compute_coralys_score(&self, bars: &[BarObservation]) -> (f64, f64) {
        if bars.len() < 5 {
            return (0.5000, 0.0);
        }

        let n = bars.len();
        let close_latest = bars[n - 1].close;
        let close_5 = bars[n - 5].close;
        let close_2 = if n >= 2 { bars[n - 2].close } else { close_latest };

        let ret_5 = if close_5 != 0.0 { (close_latest - close_5) / close_5 } else { 0.0 };
        let ret_1 = if close_2 != 0.0 { (close_latest - close_2) / close_2 } else { 0.0 };

        let raw_score = ret_5 * 10.0 + ret_1 * 5.0;
        let score = raw_score.max(-1.0).min(1.0);
        let rounded_score = (score * 10000.0).round() / 10000.0;

        (rounded_score, ret_5)
    }

    /// Evaluates trajectory state for a given resolution stream under Config A rules.
    pub fn evaluate_stream(&self, bars: &[BarObservation]) -> TrajectoryEvaluation {
        if bars.len() < 5 {
            return TrajectoryEvaluation {
                label: "INSUFFICIENT".into(),
                score: 0.5000,
                momentum_5: 0.0,
            };
        }

        let (score, m5) = self.compute_coralys_score(bars);

        let label = if score <= -0.20 {
            "REVERSAL".to_string()
        } else if score >= 0.20 {
            "CONTINUATION".to_string()
        } else {
            "PULLBACK".to_string()
        };

        TrajectoryEvaluation {
            label,
            score,
            momentum_5: m5,
        }
    }
}
