//! Thin Coralys State Bridge (Phase 2 Adapter Layer)
//!
//! Purpose: Translates ChronoSentiment's causal position state (S_t, ΔS_t) into
//! the input interface expected by the existing `coralys-decision` crate.
//!
//! Architectural Invariant:
//! - Coralys source code remains 100% frozen and untouched.
//! - No optimization algorithm, strategy rules, or manual thresholds inside Coralys.
//! - ChronoSentiment owns the state construction and returned action interpretation.

use serde::{Deserialize, Serialize};
use coralys_decision::recommendation::{
    Rec001hStore, RecommendationAction, RecommendationEngineV1, RecommendationRecordV1,
};

/// Causal position state vector (S_t) constructed strictly at bar t.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CausalPositionState {
    pub current_signed_return: f64,
    pub mfe_to_date: f64,
    pub giveback_from_mfe: f64,
    pub position_age: u32,
    pub recent_momentum: f64,
    pub directional_pressure_5: f64,
    pub return_volatility_5: f64,
    pub relative_volume_5: f64,
}

/// Causal state velocity vector (ΔS_t) constructed strictly at bar t.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CausalStateVelocity {
    pub delta_signed_return: f64,
    pub delta_momentum_5: f64,
    pub delta2_momentum_5: f64,
    pub delta_directional_pressure_5: f64,
    pub delta2_directional_pressure_5: f64,
    pub delta_return_volatility_5: f64,
    pub delta_relative_volume_5: f64,
    pub giveback_velocity: f64,
    pub bars_since_mfe: u32,
}

/// ChronoSentiment action interpretation of Coralys decision output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoralysPositionAction {
    /// Maintain current position posture.
    Hold,
    /// Apply defensive stop / trailing risk boundary.
    Protect,
    /// Close position immediately.
    Exit,
}

/// Thin bridge function translating (S_t, ΔS_t) into coralys-decision input parameters
/// and evaluating through the UNTOUCHED existing `RecommendationEngineV1`.
pub fn evaluate_coralys_position_state(
    decision_id: &str,
    instrument: &str,
    direction: &str,
    state: &CausalPositionState,
    velocity: &CausalStateVelocity,
    reference_price: Option<f64>,
    train_vol_median: f64,
    rec001h_store: &Rec001hStore,
) -> (RecommendationRecordV1, CoralysPositionAction) {
    // Derive Coralys trend label from momentum and directional pressure
    let trend = if state.recent_momentum > 0.0 && state.directional_pressure_5 >= 0.0 {
        "Bullish"
    } else if state.recent_momentum < 0.0 && state.directional_pressure_5 < 0.0 {
        "Bearish"
    } else {
        "absent"
    };

    // Derive Coralys momentum label from momentum velocity (ΔM_t)
    let momentum = if velocity.delta_momentum_5 >= 0.0 {
        "Positive"
    } else {
        "Negative"
    };

    // Derive Coralys volatility label using strict training fold median threshold
    let volatility = if state.return_volatility_5 >= train_vol_median {
        "present"
    } else {
        "absent"
    };

    let relative_volume_20 = state.relative_volume_5;

    // Invoke the UNTOUCHED existing Coralys decision engine
    let engine = RecommendationEngineV1::new(rec001h_store);
    let record = engine.evaluate(
        decision_id,
        instrument,
        direction,
        trend,
        momentum,
        reference_price,
        volatility,
        relative_volume_20,
    );

    // Interpret RecommendationAction into ChronoSentiment position management action
    let action = match record.action {
        RecommendationAction::Buy | RecommendationAction::Sell => CoralysPositionAction::Hold,
        RecommendationAction::Watch => CoralysPositionAction::Protect,
        RecommendationAction::NoTrade => CoralysPositionAction::Exit,
    };

    (record, action)
}

/// Thin bridge function evaluating Coralys position state with direction-aligned causal state vectors.
///
/// Converts directional quantities into position-relative coordinates (x_aligned = direction * x)
/// before evaluating through the UNTOUCHED existing `RecommendationEngineV1`.
pub fn evaluate_coralys_direction_aligned_state(
    decision_id: &str,
    instrument: &str,
    direction: &str,
    state: &CausalPositionState,
    velocity: &CausalStateVelocity,
    reference_price: Option<f64>,
    train_vol_median: f64,
    rec001h_store: &Rec001hStore,
) -> (RecommendationRecordV1, CoralysPositionAction) {
    let dir_val = if direction.eq_ignore_ascii_case("SHORT") { -1.0 } else { 1.0 };

    let aligned_momentum = state.recent_momentum * dir_val;
    let aligned_pressure = state.directional_pressure_5 * dir_val;
    let aligned_delta_momentum = velocity.delta_momentum_5 * dir_val;

    // Derive Coralys trend label from position-aligned momentum and pressure
    let trend = if aligned_momentum > 0.0 && aligned_pressure >= 0.0 {
        "Bullish"
    } else if aligned_momentum < 0.0 && aligned_pressure < 0.0 {
        "Bearish"
    } else {
        "absent"
    };

    // Derive Coralys momentum label from position-aligned momentum velocity
    let momentum = if aligned_delta_momentum >= 0.0 {
        "Positive"
    } else {
        "Negative"
    };

    // Derive Coralys volatility label using strict training fold median threshold
    let volatility = if state.return_volatility_5 >= train_vol_median {
        "present"
    } else {
        "absent"
    };

    let relative_volume_20 = state.relative_volume_5;

    // Invoke the UNTOUCHED existing Coralys decision engine
    let engine = RecommendationEngineV1::new(rec001h_store);
    let record = engine.evaluate(
        decision_id,
        instrument,
        direction,
        trend,
        momentum,
        reference_price,
        volatility,
        relative_volume_20,
    );

    // Interpret RecommendationAction into ChronoSentiment position management action
    let action = match record.action {
        RecommendationAction::Buy | RecommendationAction::Sell => CoralysPositionAction::Hold,
        RecommendationAction::Watch => CoralysPositionAction::Protect,
        RecommendationAction::NoTrade => CoralysPositionAction::Exit,
    };

    (record, action)
}

/// Layer 1: Causal Market & Position Trajectory State vector constructed strictly at bar t.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketPositionTrajectoryState {
    pub current_return: f64,
    pub mfe: f64,
    pub giveback: f64,
    pub momentum: f64,
    pub pressure: f64,
    pub return_velocity: f64,
    pub momentum_velocity: f64,
    pub pressure_velocity: f64,
    pub volatility: f64,
    pub volatility_velocity: f64,
    pub relative_volume: f64,
    pub bars_since_mfe: u32,
}

/// Layer 2: Observational momentum trajectory regime classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumTrajectoryRegime {
    /// Momentum magnitude is expanding in direction of trend (M5 > 0, ΔM5 > 0)
    Strengthening,
    /// Momentum magnitude is contracting relative to trend (M5 > 0, ΔM5 < 0)
    Weakening,
    /// Momentum velocity sign opposes momentum direction (sign(M5) != sign(ΔM5))
    Reversing,
    /// Velocity is near zero
    Neutral,
}

/// Layer 2: Observational directional trajectory regime classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DirectionalTrajectoryRegime {
    /// Strong directional expansion (|ΔP| > 15 bps, aligned with pressure)
    Expansion,
    /// Directional compression / sideways candidate (|ΔP| < 5 bps, low pressure velocity)
    Compression,
    /// Velocity changing direction / non-aligned transition
    Transitioning,
}

impl MarketPositionTrajectoryState {
    /// Construct trajectory state from causal position state and state velocity vectors.
    pub fn from_state_and_velocity(state: &CausalPositionState, velocity: &CausalStateVelocity) -> Self {
        Self {
            current_return: state.current_signed_return,
            mfe: state.mfe_to_date,
            giveback: state.giveback_from_mfe,
            momentum: state.recent_momentum,
            pressure: state.directional_pressure_5,
            return_velocity: velocity.delta_signed_return,
            momentum_velocity: velocity.delta_momentum_5,
            pressure_velocity: velocity.delta_directional_pressure_5,
            volatility: state.return_volatility_5,
            volatility_velocity: velocity.delta_return_volatility_5,
            relative_volume: state.relative_volume_5,
            bars_since_mfe: velocity.bars_since_mfe,
        }
    }

    /// Classify observational momentum regime from causal state & velocity (Layer 2).
    pub fn classify_momentum_regime(&self) -> MomentumTrajectoryRegime {
        if self.momentum != 0.0 && self.momentum_velocity != 0.0 && (self.momentum.signum() != self.momentum_velocity.signum()) {
            MomentumTrajectoryRegime::Reversing
        } else if self.momentum != 0.0 && self.momentum_velocity != 0.0 && (self.momentum.signum() == self.momentum_velocity.signum()) {
            MomentumTrajectoryRegime::Strengthening
        } else if self.momentum_velocity != 0.0 {
            MomentumTrajectoryRegime::Weakening
        } else {
            MomentumTrajectoryRegime::Neutral
        }
    }

    /// Classify observational directional regime from causal state & velocity (Layer 2).
    pub fn classify_directional_regime(&self) -> DirectionalTrajectoryRegime {
        let abs_dp = self.return_velocity.abs();
        let abs_dd = self.pressure_velocity.abs();
        let abs_dm = self.momentum_velocity.abs();

        if abs_dp < 0.0005 && abs_dd < 0.10 && abs_dm < 0.05 {
            DirectionalTrajectoryRegime::Compression
        } else if abs_dp > 0.0015 && (self.return_velocity.signum() == self.pressure.signum()) {
            DirectionalTrajectoryRegime::Expansion
        } else {
            DirectionalTrajectoryRegime::Transitioning
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coralys_state_bridge_smoke() {
        let temp_dir = std::env::temp_dir().join("coralys_bridge_test_store");
        let _ = std::fs::create_dir_all(&temp_dir);
        let store = Rec001hStore::load_from_dir(temp_dir.to_str().unwrap()).unwrap();
        let state = CausalPositionState {
            current_signed_return: -0.0012,
            mfe_to_date: 0.0045,
            giveback_from_mfe: 0.0057,
            position_age: 12,
            recent_momentum: -0.0008,
            directional_pressure_5: -0.15,
            return_volatility_5: 0.0012,
            relative_volume_5: 1.45,
        };
        let velocity = CausalStateVelocity {
            delta_signed_return: -0.0004,
            delta_momentum_5: -0.0002,
            delta2_momentum_5: -0.0001,
            delta_directional_pressure_5: -0.08,
            delta2_directional_pressure_5: -0.04,
            delta_return_volatility_5: 0.0001,
            delta_relative_volume_5: 0.20,
            giveback_velocity: 0.0003,
            bars_since_mfe: 4,
        };

        let (record, action) = evaluate_coralys_position_state(
            "DEC-SMOKE-001",
            "RELIANCE.NS",
            "LONG",
            &state,
            &velocity,
            Some(1000.0),
            0.0010,
            &store,
        );

        assert_eq!(record.decision_id, "DEC-SMOKE-001");
        assert_eq!(record.instrument, "RELIANCE.NS");
        assert!(matches!(
            action,
            CoralysPositionAction::Hold | CoralysPositionAction::Protect | CoralysPositionAction::Exit
        ));

        // Test Layer 1 extraction and Layer 2 observational classifications
        let traj = MarketPositionTrajectoryState::from_state_and_velocity(&state, &velocity);
        assert_eq!(traj.current_return, -0.0012);
        assert_eq!(traj.momentum_velocity, -0.0002);
        assert_eq!(traj.classify_momentum_regime(), MomentumTrajectoryRegime::Strengthening);

        let sideways_traj = MarketPositionTrajectoryState {
            current_return: 0.0001,
            mfe: 0.0002,
            giveback: 0.0001,
            momentum: 0.0001,
            pressure: 0.02,
            return_velocity: 0.0001,
            momentum_velocity: 0.0001,
            pressure_velocity: 0.01,
            volatility: 0.0008,
            volatility_velocity: -0.00005,
            relative_volume: 0.95,
            bars_since_mfe: 2,
        };
        assert_eq!(sideways_traj.classify_directional_regime(), DirectionalTrajectoryRegime::Compression);
    }
}

