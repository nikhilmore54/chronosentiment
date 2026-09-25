//! Protection Strategy Generator Runtime Module (Gate 3 & Gate 4 Shadow Production).
//!
//! **Operational Invariants:**
//! 1. `raw_coralys_action` is IMMUTABLE and preserved verbatim from the Coralys decision record.
//! 2. `ProtectionAction::Suppress` means position management does not execute the Coralys exit at bar t; Coralys itself is unmodified.
//! 3. Zero timers or fixed reassessment intervals exist. Every incoming bar t recomputes S_t and evaluates G(S_t).
//! 4. Invalid or missing state fails closed rather than inventing data.
//! 5. Shadow Production (Gate 4): Records parallel `ProtectionShadowEvent` into `ProtectionShadowLedger` with zero execution side effects.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Action emitted by the Protection Strategy Generator for the position-management layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProtectionAction {
    /// High-confidence structural price deterioration. Respect Coralys exit immediately and close position.
    Execute,
    /// Dynamic continuous bar-by-bar monitoring. Leave position open for current bar; re-evaluate on bar t+1.
    Protect,
    /// High-confidence recoverable pullback. Position-management layer defers Coralys exit for bar t; re-evaluate on bar t+1.
    #[serde(alias = "DEFER", alias = "DEFER_EXIT")]
    Suppress,
}

impl ProtectionAction {
    /// Helper to display operational term DEFER
    pub fn operational_name(&self) -> &'static str {
        match self {
            ProtectionAction::Execute => "EXECUTE",
            ProtectionAction::Protect => "PROTECT",
            ProtectionAction::Suppress => "DEFER",
        }
    }
}

impl fmt::Display for ProtectionAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtectionAction::Execute => write!(f, "EXECUTE"),
            ProtectionAction::Protect => write!(f, "PROTECT"),
            ProtectionAction::Suppress => write!(f, "DEFER"),
        }
    }
}

/// Point-in-time continuous position trajectory state vector S_t.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PositionTrajectoryState {
    pub bars_since_entry: u32,
    pub curr_return: f64,
    pub mfe_to_date: f64,
    pub giveback_level: f64,
    pub giveback_ratio: f64,
    pub aligned_momentum_5: f64,
    pub aligned_pressure_5: f64,
    pub aligned_delta_momentum_5: f64,
    pub aligned_delta_pressure_5: f64,
    pub aligned_delta2_momentum_5: f64,
    pub aligned_delta2_pressure_5: f64,
    pub volatility_5: f64,
}

impl PositionTrajectoryState {
    /// Validate that the state vector contains valid numbers (no NaN or Infinity).
    pub fn is_valid(&self) -> bool {
        !self.curr_return.is_nan()
            && !self.curr_return.is_infinite()
            && !self.mfe_to_date.is_nan()
            && !self.mfe_to_date.is_infinite()
            && !self.giveback_level.is_nan()
            && !self.giveback_level.is_infinite()
            && !self.giveback_ratio.is_nan()
            && !self.giveback_ratio.is_infinite()
            && !self.aligned_momentum_5.is_nan()
            && !self.aligned_momentum_5.is_infinite()
            && !self.aligned_pressure_5.is_nan()
            && !self.aligned_pressure_5.is_infinite()
            && !self.aligned_delta_momentum_5.is_nan()
            && !self.aligned_delta_momentum_5.is_infinite()
            && !self.aligned_delta_pressure_5.is_nan()
            && !self.aligned_delta_pressure_5.is_infinite()
            && !self.aligned_delta2_momentum_5.is_nan()
            && !self.aligned_delta2_momentum_5.is_infinite()
            && !self.aligned_delta2_pressure_5.is_nan()
            && !self.aligned_delta2_pressure_5.is_infinite()
            && !self.volatility_5.is_nan()
            && !self.volatility_5.is_infinite()
    }
}

/// Point-in-time protection decision record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProtectionDecision {
    pub decision_id: String,
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    /// Verbatim raw Coralys exit decision string (e.g., "EXIT", "HOLD"). Immutable.
    pub raw_coralys_action: String,
    /// Generator action for position execution layer.
    pub generator_action: ProtectionAction,
    /// Point-in-time state snapshot S_t used for evaluation.
    pub state_snapshot: PositionTrajectoryState,
}

/// Immutable shadow audit record emitted for every Coralys decision in Gate 4.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProtectionShadowEvent {
    pub event_id: String,
    pub decision_id: String,
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub raw_coralys_action: String,
    pub generator_action: ProtectionAction,
    pub generator_version: String,
    pub state_snapshot: PositionTrajectoryState,
}

/// Thread-safe in-memory shadow ledger for Gate 4 production audit.
#[derive(Debug, Clone)]
pub struct ProtectionShadowLedger {
    events: Arc<RwLock<Vec<ProtectionShadowEvent>>>,
}

impl Default for ProtectionShadowLedger {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtectionShadowLedger {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record a shadow event into the audit ledger. Zero side effects.
    pub async fn record_event(&self, event: ProtectionShadowEvent) {
        let mut guard = self.events.write().await;
        guard.push(event);
    }

    /// Retrieve all recorded shadow events.
    pub async fn events(&self) -> Vec<ProtectionShadowEvent> {
        let guard = self.events.read().await;
        guard.clone()
    }

    /// Filter events by symbol.
    pub async fn events_for_symbol(&self, symbol: &str) -> Vec<ProtectionShadowEvent> {
        let guard = self.events.read().await;
        guard.iter().filter(|e| e.symbol == symbol).cloned().collect()
    }

    /// Export shadow ledger as pretty JSON.
    pub async fn export_json(&self) -> String {
        let guard = self.events.read().await;
        serde_json::to_string_pretty(&*guard).unwrap_or_else(|_| "[]".to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeneratorError {
    InvalidState(String),
}

impl fmt::Display for GeneratorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeneratorError::InvalidState(msg) => write!(f, "Invalid trajectory state: {}", msg),
        }
    }
}

impl std::error::Error for GeneratorError {}

/// Thin runtime strategy generator wrapping point-in-time trajectory evaluation.
#[derive(Debug, Clone)]
pub struct ProtectionStrategyGenerator {
    pub version: String,
    pub execute_threshold: f64,
    pub suppress_threshold: f64,
}

impl Default for ProtectionStrategyGenerator {
    fn default() -> Self {
        Self {
            version: "v1.0.0".to_string(),
            execute_threshold: 0.70,
            suppress_threshold: 0.30,
        }
    }
}

impl ProtectionStrategyGenerator {
    pub fn new(version: impl Into<String>, execute_threshold: f64, suppress_threshold: f64) -> Self {
        Self {
            version: version.into(),
            execute_threshold,
            suppress_threshold,
        }
    }

    /// Evaluates a raw Coralys exit decision against point-in-time state S_t.
    pub fn evaluate(
        &self,
        decision_id: impl Into<String>,
        timestamp: DateTime<Utc>,
        symbol: impl Into<String>,
        raw_coralys_action: impl Into<String>,
        state: PositionTrajectoryState,
    ) -> Result<ProtectionDecision, GeneratorError> {
        let raw_action_str = raw_coralys_action.into();
        let sym_str = symbol.into();
        let dec_id = decision_id.into();

        // Fail-closed: invalid state returns error
        if !state.is_valid() {
            return Err(GeneratorError::InvalidState(
                "Trajectory state vector contains NaN or Infinite values".to_string(),
            ));
        }

        let is_exit_signal = raw_action_str.to_uppercase().contains("EXIT");

        let action = if !is_exit_signal {
            ProtectionAction::Suppress
        } else {
            let mom = state.aligned_momentum_5;
            let dp = state.aligned_pressure_5;
            let d_mom = state.aligned_delta_momentum_5;
            let d_dp = state.aligned_delta_pressure_5;
            let gb_rat = state.giveback_ratio;

            let score = 0.50 + (mom * 0.30) + (dp * 0.20) + (d_mom * 0.15) + (d_dp * 0.15) - (gb_rat * 0.10);

            if score >= self.execute_threshold {
                ProtectionAction::Execute
            } else if score < self.suppress_threshold {
                ProtectionAction::Suppress
            } else {
                ProtectionAction::Protect
            }
        };

        Ok(ProtectionDecision {
            decision_id: dec_id,
            timestamp,
            symbol: sym_str,
            raw_coralys_action: raw_action_str,
            generator_action: action,
            state_snapshot: state,
        })
    }

    /// Creates a Gate 4 Shadow Production Event from an evaluated decision. Zero side effects.
    pub fn create_shadow_event(&self, decision: &ProtectionDecision, event_id: impl Into<String>) -> ProtectionShadowEvent {
        ProtectionShadowEvent {
            event_id: event_id.into(),
            decision_id: decision.decision_id.clone(),
            timestamp: decision.timestamp,
            symbol: decision.symbol.clone(),
            raw_coralys_action: decision.raw_coralys_action.clone(),
            generator_action: decision.generator_action,
            generator_version: self.version.clone(),
            state_snapshot: decision.state_snapshot.clone(),
        }
    }
}

// ─── UNIT & ACCEPTANCE TESTS ──────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn sample_valid_state() -> PositionTrajectoryState {
        PositionTrajectoryState {
            bars_since_entry: 4,
            curr_return: -0.0015,
            mfe_to_date: 0.0080,
            giveback_level: 0.0095,
            giveback_ratio: 1.1875,
            aligned_momentum_5: 0.80,
            aligned_pressure_5: 0.50,
            aligned_delta_momentum_5: 0.20,
            aligned_delta_pressure_5: 0.10,
            aligned_delta2_momentum_5: 0.05,
            aligned_delta2_pressure_5: 0.02,
            volatility_5: 0.0012,
        }
    }

    fn sample_recovering_state() -> PositionTrajectoryState {
        PositionTrajectoryState {
            bars_since_entry: 4,
            curr_return: 0.0020,
            mfe_to_date: 0.0050,
            giveback_level: 0.0010,
            giveback_ratio: 0.20,
            aligned_momentum_5: -0.80,
            aligned_pressure_5: -0.60,
            aligned_delta_momentum_5: -0.40,
            aligned_delta_pressure_5: -0.30,
            aligned_delta2_momentum_5: -0.10,
            aligned_delta2_pressure_5: -0.05,
            volatility_5: 0.0008,
        }
    }

    fn sample_intermediate_state() -> PositionTrajectoryState {
        PositionTrajectoryState {
            bars_since_entry: 4,
            curr_return: -0.0005,
            mfe_to_date: 0.0030,
            giveback_level: 0.0035,
            giveback_ratio: 1.16,
            aligned_momentum_5: 0.05,
            aligned_pressure_5: 0.02,
            aligned_delta_momentum_5: 0.01,
            aligned_delta_pressure_5: 0.01,
            aligned_delta2_momentum_5: 0.00,
            aligned_delta2_pressure_5: 0.00,
            volatility_5: 0.0010,
        }
    }

    // Gate 3 Test 1: EXECUTE action closes immediately
    #[test]
    fn test_1_execute_closes_immediately() {
        let generator = ProtectionStrategyGenerator::default();
        let now = Utc.timestamp_opt(1758810000, 0).unwrap();
        let state = sample_valid_state();

        let res = generator.evaluate("DEC-001", now, "RELIANCE", "CORALYS_EXIT", state).unwrap();

        assert_eq!(res.generator_action, ProtectionAction::Execute);
        assert_eq!(res.raw_coralys_action, "CORALYS_EXIT");
    }

    // Gate 3 Test 2: PROTECT action leaves position open for dynamic evaluation
    #[test]
    fn test_2_protect_leaves_position_open() {
        let generator = ProtectionStrategyGenerator::default();
        let now = Utc.timestamp_opt(1758810000, 0).unwrap();
        let state = sample_intermediate_state();

        let res = generator.evaluate("DEC-002", now, "RELIANCE", "CORALYS_EXIT", state).unwrap();

        assert_eq!(res.generator_action, ProtectionAction::Protect);
    }

    // Gate 3 Test 3: SUPPRESS action leaves position open and preserves raw Coralys action verbatim
    #[test]
    fn test_3_suppress_leaves_position_open_and_preserves_raw_coralys_action() {
        let generator = ProtectionStrategyGenerator::default();
        let now = Utc.timestamp_opt(1758810000, 0).unwrap();
        let state = sample_recovering_state();

        let res = generator.evaluate("DEC-003", now, "RELIANCE", "CORALYS_EXIT", state).unwrap();

        assert_eq!(res.generator_action, ProtectionAction::Suppress);
        assert_eq!(res.raw_coralys_action, "CORALYS_EXIT");
    }

    // Gate 3 Test 4: PROTECT to next observation re-evaluation
    #[test]
    fn test_4_protect_to_next_observation_reevaluation() {
        let generator = ProtectionStrategyGenerator::default();
        let t0 = Utc.timestamp_opt(1758810000, 0).unwrap();
        let t1 = Utc.timestamp_opt(1758810060, 0).unwrap();

        let state_t0 = sample_intermediate_state();
        let res_t0 = generator.evaluate("DEC-004", t0, "RELIANCE", "CORALYS_EXIT", state_t0).unwrap();
        assert_eq!(res_t0.generator_action, ProtectionAction::Protect);

        let mut state_t1 = sample_valid_state();
        state_t1.bars_since_entry = 5;

        let res_t1 = generator.evaluate("DEC-004", t1, "RELIANCE", "CORALYS_EXIT", state_t1).unwrap();
        assert_eq!(res_t1.generator_action, ProtectionAction::Execute);
    }

    // Gate 3 Test 5: SUPPRESS to next observation re-evaluation
    #[test]
    fn test_5_suppress_to_next_observation_reevaluation() {
        let generator = ProtectionStrategyGenerator::default();
        let t0 = Utc.timestamp_opt(1758810000, 0).unwrap();
        let t1 = Utc.timestamp_opt(1758810060, 0).unwrap();

        let state_t0 = sample_recovering_state();
        let res_t0 = generator.evaluate("DEC-005", t0, "RELIANCE", "CORALYS_EXIT", state_t0).unwrap();
        assert_eq!(res_t0.generator_action, ProtectionAction::Suppress);

        let state_t1 = sample_intermediate_state();
        let res_t1 = generator.evaluate("DEC-005", t1, "RELIANCE", "CORALYS_EXIT", state_t1).unwrap();
        assert_eq!(res_t1.generator_action, ProtectionAction::Protect);
    }

    // Gate 3 Test 6: No fixed timer exists
    #[test]
    fn test_6_no_fixed_timer_exists() {
        let generator = ProtectionStrategyGenerator::default();
        let t0 = Utc.timestamp_opt(1758810000, 0).unwrap();
        let t_arbitrary = Utc.timestamp_opt(1758810005, 0).unwrap();

        let state = sample_recovering_state();
        let r0 = generator.evaluate("DEC-006", t0, "RELIANCE", "CORALYS_EXIT", state.clone()).unwrap();
        let r1 = generator.evaluate("DEC-006", t_arbitrary, "RELIANCE", "CORALYS_EXIT", state).unwrap();

        assert_eq!(r0.generator_action, r1.generator_action);
    }

    // Gate 3 Test 7: Coralys raw decision is preserved verbatim
    #[test]
    fn test_7_coralys_raw_decision_preserved_verbatim() {
        let generator = ProtectionStrategyGenerator::default();
        let now = Utc.timestamp_opt(1758810000, 0).unwrap();
        let raw_strings = vec!["CORALYS_EXIT_R1", "CORALYS_EXIT_VOLATILITY", "RAW_EXIT"];

        for raw_str in raw_strings {
            let res = generator.evaluate("DEC-007", now, "INFY", raw_str, sample_valid_state()).unwrap();
            assert_eq!(res.raw_coralys_action, raw_str);
        }
    }

    // Gate 3 Test 8: State snapshot matches input verbatim
    #[test]
    fn test_8_state_snapshot_matches_input_verbatim() {
        let generator = ProtectionStrategyGenerator::default();
        let now = Utc.timestamp_opt(1758810000, 0).unwrap();
        let input_state = sample_valid_state();

        let res = generator.evaluate("DEC-008", now, "TCS", "CORALYS_EXIT", input_state.clone()).unwrap();
        assert_eq!(res.state_snapshot, input_state);
    }

    // Gate 3 Test 9: Deterministic replay
    #[test]
    fn test_9_deterministic_replay() {
        let generator = ProtectionStrategyGenerator::default();
        let now = Utc.timestamp_opt(1758810000, 0).unwrap();
        let state = sample_valid_state();

        let res_run1 = generator.evaluate("DEC-009", now, "HDFCBANK", "CORALYS_EXIT", state.clone()).unwrap();
        let res_run2 = generator.evaluate("DEC-009", now, "HDFCBANK", "CORALYS_EXIT", state).unwrap();

        assert_eq!(res_run1, res_run2);
    }

    // Gate 3 Test 10: Missing or invalid state fails closed
    #[test]
    fn test_10_missing_or_invalid_state_fails_closed() {
        let generator = ProtectionStrategyGenerator::default();
        let now = Utc.timestamp_opt(1758810000, 0).unwrap();

        let mut invalid_state = sample_valid_state();
        invalid_state.aligned_momentum_5 = f64::NAN;

        let res = generator.evaluate("DEC-010", now, "RELIANCE", "CORALYS_EXIT", invalid_state);

        assert!(res.is_err());
        match res.unwrap_err() {
            GeneratorError::InvalidState(msg) => {
                assert!(msg.contains("NaN or Infinite"));
            }
        }
    }

    // ─── GATE 4 SHADOW PRODUCTION ACCEPTANCE TESTS ─────────────────────────────

    // Gate 4 Acceptance Test 1: Coverage (Every Coralys exit produces a shadow event)
    #[tokio::test]
    async fn test_gate4_coverage() {
        let generator = ProtectionStrategyGenerator::default();
        let shadow_ledger = ProtectionShadowLedger::new();
        let now = Utc.timestamp_opt(1758810000, 0).unwrap();

        let raw_exits = vec!["DEC-SHADOW-1", "DEC-SHADOW-2", "DEC-SHADOW-3"];
        for (idx, dec_id) in raw_exits.iter().enumerate() {
            let state = if idx % 2 == 0 { sample_valid_state() } else { sample_recovering_state() };
            let dec = generator.evaluate(*dec_id, now, "RELIANCE", "CORALYS_EXIT", state).unwrap();
            let shadow_evt = generator.create_shadow_event(&dec, format!("EVT-{}", idx));
            shadow_ledger.record_event(shadow_evt).await;
        }

        let recorded = shadow_ledger.events().await;
        assert_eq!(recorded.len(), 3);
        assert_eq!(recorded[0].decision_id, "DEC-SHADOW-1");
        assert_eq!(recorded[1].decision_id, "DEC-SHADOW-2");
        assert_eq!(recorded[2].decision_id, "DEC-SHADOW-3");
    }

    // Gate 4 Acceptance Test 2: Causality (Point-in-time state contains zero future info)
    #[tokio::test]
    async fn test_gate4_causality() {
        let generator = ProtectionStrategyGenerator::default();
        let shadow_ledger = ProtectionShadowLedger::new();
        let t_point = Utc.timestamp_opt(1758810000, 0).unwrap();

        let state_point = sample_valid_state();
        let dec = generator.evaluate("DEC-CAUSAL", t_point, "INFY", "CORALYS_EXIT", state_point.clone()).unwrap();
        let shadow_evt = generator.create_shadow_event(&dec, "EVT-CAUSAL-1");
        shadow_ledger.record_event(shadow_evt).await;

        let recorded = shadow_ledger.events().await;
        assert_eq!(recorded[0].timestamp, t_point);
        assert_eq!(recorded[0].state_snapshot, state_point);
    }

    // Gate 4 Acceptance Test 3: Determinism (Identical stream produces identical shadow ledger)
    #[tokio::test]
    async fn test_gate4_determinism() {
        let generator = ProtectionStrategyGenerator::default();
        let ledger1 = ProtectionShadowLedger::new();
        let ledger2 = ProtectionShadowLedger::new();
        let now = Utc.timestamp_opt(1758810000, 0).unwrap();

        let state = sample_intermediate_state();
        let dec = generator.evaluate("DEC-DET", now, "TCS", "CORALYS_EXIT", state).unwrap();

        ledger1.record_event(generator.create_shadow_event(&dec, "EVT-1")).await;
        ledger2.record_event(generator.create_shadow_event(&dec, "EVT-1")).await;

        let evts1 = ledger1.events().await;
        let evts2 = ledger2.events().await;
        assert_eq!(evts1, evts2);
    }

    // Gate 4 Acceptance Test 4: Continuous Lifecycle Transitions (PROTECT -> PROTECT -> EXECUTE)
    #[tokio::test]
    async fn test_gate4_continuous_lifecycle() {
        let generator = ProtectionStrategyGenerator::default();
        let shadow_ledger = ProtectionShadowLedger::new();
        let t0 = Utc.timestamp_opt(1758810000, 0).unwrap();
        let t1 = Utc.timestamp_opt(1758810060, 0).unwrap();
        let t2 = Utc.timestamp_opt(1758810120, 0).unwrap();

        // Bar t0: PROTECT
        let dec_t0 = generator.evaluate("DEC-LIFE", t0, "RELIANCE", "CORALYS_EXIT", sample_intermediate_state()).unwrap();
        shadow_ledger.record_event(generator.create_shadow_event(&dec_t0, "EVT-L0")).await;

        // Bar t1: PROTECT
        let dec_t1 = generator.evaluate("DEC-LIFE", t1, "RELIANCE", "CORALYS_EXIT", sample_intermediate_state()).unwrap();
        shadow_ledger.record_event(generator.create_shadow_event(&dec_t1, "EVT-L1")).await;

        // Bar t2: EXECUTE (terminal)
        let dec_t2 = generator.evaluate("DEC-LIFE", t2, "RELIANCE", "CORALYS_EXIT", sample_valid_state()).unwrap();
        shadow_ledger.record_event(generator.create_shadow_event(&dec_t2, "EVT-L2")).await;

        let evts = shadow_ledger.events().await;
        assert_eq!(evts.len(), 3);
        assert_eq!(evts[0].generator_action, ProtectionAction::Protect);
        assert_eq!(evts[1].generator_action, ProtectionAction::Protect);
        assert_eq!(evts[2].generator_action, ProtectionAction::Execute);
    }

    // Gate 4 Acceptance Test 5: Isolation & Verbatim Raw Coralys Preservation
    #[tokio::test]
    async fn test_gate4_isolation_and_raw_preservation() {
        let generator = ProtectionStrategyGenerator::default();
        let shadow_ledger = ProtectionShadowLedger::new();
        let now = Utc.timestamp_opt(1758810000, 0).unwrap();

        let raw_action_original = "CORALYS_IMMUTABLE_EXIT_V1";
        let dec = generator.evaluate("DEC-ISO", now, "HDFCBANK", raw_action_original, sample_recovering_state()).unwrap();
        let shadow_evt = generator.create_shadow_event(&dec, "EVT-ISO-1");

        // Action is SUPPRESS, but raw_coralys_action remains 100% immutable
        assert_eq!(shadow_evt.generator_action, ProtectionAction::Suppress);
        assert_eq!(shadow_evt.raw_coralys_action, raw_action_original);

        shadow_ledger.record_event(shadow_evt).await;
        let recorded = shadow_ledger.events().await;
        assert_eq!(recorded[0].raw_coralys_action, raw_action_original);
    }
}
