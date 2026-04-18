use crate::ConsensusDecision;
use crate::pipeline::SignalAction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExitReason {
    TakeProfit,
    TrailingStop,
    ConsensusDecay,
    DisagreementSpike,
    RegimeFlip,
    SignalInversion,
    TimeExpiry,
}

#[derive(Debug, Clone)]
pub struct ExitDecision {
    pub should_exit: bool,
    pub exit_pressure: f64,
    pub reason: Option<ExitReason>,
    pub metadata: ExitMetadata,
}

#[derive(Debug, Clone)]
pub struct ExitMetadata {
    pub decay: f64,
    pub time_factor: f64,
    pub signal_flip: bool,
    pub current_edge: f64,
    pub entry_edge: f64,
}

pub struct ExitEvaluator {
    pub min_hold_bars: usize,
    pub max_hold_bars: usize,
    pub decay_threshold: f64, // e.g. 0.6
}

impl ExitEvaluator {
    pub fn new(min_hold: usize, max_hold: usize) -> Self {
        Self {
            min_hold_bars: min_hold,
            max_hold_bars: max_hold,
            decay_threshold: 0.6,
        }
    }

    /// Evaluates whether to exit a position based on ensemble conviction decay,
    /// signal inversion, and time-based pressure.
    pub fn evaluate_exit(
        &self,
        entry_edge: f64,
        _entry_action: SignalAction,
        current_consensus: &ConsensusDecision,
        holding_time_bars: usize,
    ) -> ExitDecision {
        // 1. SIGNAL INVERSION (Immediate Exit with Strength Filter)
        // Only flip if current consensus is opposite AND strong enough to not be noise.
        let flip_threshold = (entry_edge.abs() * 0.1).max(0.0002);
        let _current_action = current_consensus.combined_action.clone();

        let signal_flipped = (current_consensus.consensus_score * entry_edge < 0.0)
            && (current_consensus.consensus_score.abs() > flip_threshold);

        if signal_flipped {
            return ExitDecision {
                should_exit: true,
                exit_pressure: 1.0,
                reason: Some(ExitReason::SignalInversion),
                metadata: ExitMetadata {
                    decay: 1.0,
                    time_factor: 0.0,
                    signal_flip: true,
                    current_edge: current_consensus.effective_edge,
                    entry_edge,
                },
            };
        }

        // 1.5 HARD MAX-HOLD EXIT
        if holding_time_bars >= self.max_hold_bars {
            return ExitDecision {
                should_exit: true,
                exit_pressure: 1.0,
                reason: Some(ExitReason::TimeExpiry),
                metadata: ExitMetadata {
                    decay: 0.0,
                    time_factor: 1.0,
                    signal_flip: false,
                    current_edge: current_consensus.effective_edge,
                    entry_edge,
                },
            };
        }

        // 2. CONSENSUS DECAY (Institutional Logic with Zero Guard)
        // Edge is relative to entry conviction.
        let decay = if entry_edge.abs() > 1e-6 {
            1.0 - (current_consensus.effective_edge / entry_edge).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // 3. TIME DECAY (Delayed Non-linear activation)
        let time_factor = if holding_time_bars < self.min_hold_bars {
            0.0
        } else {
            ((holding_time_bars - self.min_hold_bars) as f64
                / (self.max_hold_bars - self.min_hold_bars).max(1) as f64)
                .clamp(0.0, 1.0)
        };

        // 4. EXIT PRESSURE GRADIENT (Phase 11.3)
        // 0.7 Decay + 0.3 Time (Disagreement is already embedded in effective_edge)
        let exit_pressure = 0.7 * decay + 0.3 * time_factor;

        // 5. DECISION
        // Adaptive thresholding happens in pipeline (Phase 11.3 Refinements).
        // Here we just provide a baseline 'should_exit' if pressure is very high (> 0.8)
        let should_exit = exit_pressure > 0.8;

        ExitDecision {
            should_exit,
            exit_pressure,
            reason: if should_exit {
                if decay > 0.5 {
                    Some(ExitReason::ConsensusDecay)
                } else {
                    Some(ExitReason::TimeExpiry)
                }
            } else {
                None
            },
            metadata: ExitMetadata {
                decay,
                time_factor,
                signal_flip: false,
                current_edge: current_consensus.effective_edge,
                entry_edge,
            },
        }
    }
}
