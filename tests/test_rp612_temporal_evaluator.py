#!/usr/bin/env python3
"""
test_rp612_temporal_evaluator.py

Unit test suite for Milestone RP-612-001 (Specification & Implementation Audit).
Validates:
- G1: Input causality & point-in-time constraints.
- G2: Bit-exact determinism across independent runs.
- Deterministic transition priority resolution.
- Direct 1-to-1 Trade ID pairing invariant.
- Accumulator dwell accumulation and reset semantics.
- IEEE 754 exact floating-point summation without rounding drift.
"""

import json
import math
import sys
from pathlib import Path
import unittest

WORKSPACE_DIR = Path(__file__).parent.parent
sys.path.insert(0, str(WORKSPACE_DIR))
sys.path.insert(0, str(WORKSPACE_DIR / "scripts"))

from scripts.rp612_temporal_trajectory_evaluator import (
    EvaluatorConfig,
    ExitReason,
    RP612EvaluationRecord,
    TemporalTrajectoryStateMachine,
    TrajectoryState,
    simulate_trade_shadow,
)


class TestRP612TemporalEvaluator(unittest.TestCase):
    def setUp(self):
        self.config = EvaluatorConfig()

    def _generate_synthetic_bars(self, base_px=1000.0, trend=0.001, count=30):
        bars = []
        px = base_px
        history = [px * (1.0 - 0.001 * (60 - i)) for i in range(60)]
        for i in range(count):
            epoch = 1790650000 + i * 60
            px += px * trend
            history = history[1:] + [px]
            bars.append({
                "timestamp_epoch": epoch,
                "timestamp_utc": f"2026-09-29T04:{i:02d}:00Z",
                "close": px,
                "history_closes": list(history),
            })
        return bars

    def test_gate_g2_determinism_bit_exact(self):
        """G2: Identical inputs produce bit-identical states and outputs."""
        bars = self._generate_synthetic_bars(base_px=1000.0, trend=0.002, count=25)
        trade = {
            "trade_id": "TRADE_DET_001",
            "symbol": "INFY.NS",
            "side": "LONG",
            "entry_price": 1000.0,
            "quantity": 100,
            "entry_timestamp_epoch": 1790650000,
            "entry_timestamp_utc": "2026-09-29T04:00:00Z",
            "exit_timestamp_epoch": 1790650000 + 24 * 60,
            "exit_timestamp_utc": "2026-09-29T04:24:00Z",
            "exit_price": 1050.0,
            "exit_reason": "RP605_TARGET",
        }
        
        run1 = simulate_trade_shadow(trade, bars, self.config)
        run2 = simulate_trade_shadow(trade, bars, self.config)
        
        self.assertEqual(run1.trade_id, run2.trade_id)
        self.assertEqual(run1.recommendation_emitted, run2.recommendation_emitted)
        self.assertEqual(run1.recommendation_reason, run2.recommendation_reason)
        self.assertEqual(run1.simulated_fill_price, run2.simulated_fill_price)
        self.assertEqual(run1.rp612_net_pnl, run2.rp612_net_pnl)
        self.assertEqual(len(run1.observations), len(run2.observations))
        for o1, o2 in zip(run1.observations, run2.observations):
            self.assertEqual(o1.state, o2.state)
            self.assertEqual(o1.p_severity, o2.p_severity)
            self.assertEqual(o1.s15_tilde, o2.s15_tilde)

    def test_transient_pullback_and_recovery(self):
        """Verifies that transient intraday pullback is tolerated and recovers cleanly."""
        sm = TemporalTrajectoryStateMachine(direction="LONG", entry_price=1000.0, config=self.config)
        
        # 1. Healthy start
        history_supportive = [1000.0 + i * 0.1 for i in range(60)]
        obs1 = sm.evaluate_bar(1790650000, "2026-09-29T04:00:00Z", 1006.0, history_supportive, 0)
        self.assertEqual(obs1.state, TrajectoryState.IMPULSE_CONTINUATION)
        self.assertEqual(sm.tau_adverse, 0)
        
        # 2. Intraday dip: s15 goes negative, but macro 1d remains strongly positive
        # Construct history where recent 15m drops slightly, but 60m overall is up
        history_pullback = [980.0 + i * 0.5 for i in range(45)] + [1002.5 - i * 0.4 for i in range(15)]
        obs2 = sm.evaluate_bar(1790650060, "2026-09-29T04:01:00Z", 996.5, history_pullback, 60)
        self.assertEqual(obs2.state, TrajectoryState.TRANSIENT_PULLBACK)
        self.assertEqual(sm.tau_adverse, 60)
        self.assertFalse(sm.exit_recommended)
        
        # 3. Recovery: s15 rebounds above +0.05
        history_recovery = [980.0 + i * 0.4 for i in range(45)] + [998.0 + i * 0.6 for i in range(15)]
        obs3 = sm.evaluate_bar(1790650120, "2026-09-29T04:02:00Z", 1007.0, history_recovery, 120)
        self.assertEqual(obs3.state, TrajectoryState.IMPULSE_CONTINUATION)
        self.assertEqual(sm.tau_adverse, 0, "Accumulator must reset upon confirmed recovery")
        self.assertFalse(sm.exit_recommended)

    def test_confirmed_structural_reversal_exit(self):
        """Verifies that multi-horizon inversion with macro breakdown triggers structural abort."""
        sm = TemporalTrajectoryStateMachine(direction="LONG", entry_price=1000.0, config=self.config)
        
        # Severe downtrend across all lookbacks
        history_down = [1200.0 - i * 5.0 for i in range(60)]
        
        # Bar 1: Warning armed
        obs1 = sm.evaluate_bar(1790650000, "2026-09-29T04:00:00Z", 900.0, history_down, 0)
        self.assertIn(obs1.state, {TrajectoryState.STRUCTURAL_REVERSAL_WARNING, TrajectoryState.RAPID_REVERSAL_WARNING})
        self.assertFalse(sm.exit_recommended, "Exit must require confirmation; not fire on unconfirmed single tick")
        
        # Bar 2: Persistence confirmed after tau_confirm (60s)
        obs2 = sm.evaluate_bar(1790650060, "2026-09-29T04:01:00Z", 890.0, history_down, 60)
        self.assertEqual(obs2.state, TrajectoryState.EXIT_RECOMMENDED)
        self.assertTrue(sm.exit_recommended)
        self.assertIn(sm.exit_reason, {ExitReason.STRUCTURAL_REVERSAL_ABORT, ExitReason.RAPID_REVERSAL_ABORT})

    def test_direct_trade_id_pairing_integrity(self):
        """Verifies 1-to-1 trade pairing contract."""
        bars = self._generate_synthetic_bars(base_px=500.0, trend=-0.001, count=15)
        trade = {
            "trade_id": "TRADE_CANONICAL_7264",
            "symbol": "TCS.NS",
            "side": "SHORT",
            "entry_price": 500.0,
            "quantity": 200,
            "entry_timestamp_epoch": 1790650000,
            "entry_timestamp_utc": "2026-09-29T04:00:00Z",
            "exit_timestamp_epoch": 1790650000 + 14 * 60,
            "exit_timestamp_utc": "2026-09-29T04:14:00Z",
            "exit_price": 490.0,
            "exit_reason": "RP605_TARGET",
        }
        record = simulate_trade_shadow(trade, bars, self.config)
        self.assertEqual(record.trade_id, "TRADE_CANONICAL_7264")
        self.assertEqual(record.symbol, "TCS.NS")
        self.assertEqual(record.direction, "SHORT")
        self.assertEqual(record.rp605_gross_pnl, (500.0 - 490.0) * 200)

    def test_gate_g1_point_in_time_causality(self):
        """G1: Explicit test verifying zero future-bar or post-exit leakage."""
        bars_clean = self._generate_synthetic_bars(base_px=1000.0, trend=-0.005, count=20)
        trade = {
            "trade_id": "TRADE_CAUSAL_001",
            "symbol": "HDFCBANK.NS",
            "side": "LONG",
            "entry_price": 1000.0,
            "quantity": 100,
            "entry_timestamp_epoch": 1790650000,
            "entry_timestamp_utc": "2026-09-29T04:00:00Z",
            "exit_timestamp_epoch": 1790650000 + 10 * 60,
            "exit_timestamp_utc": "2026-09-29T04:10:00Z",
            "exit_price": 950.0,
            "exit_reason": "RP605_ENGINE_EXIT",
        }
        
        # Run 1: Clean timeline
        record_clean = simulate_trade_shadow(trade, bars_clean, self.config)
        
        # Run 2: Mutate future bars (indices 11 to 19, which occur AFTER the trade's RP-605 exit)
        bars_mutated = []
        for i, b in enumerate(bars_clean):
            b_copy = dict(b)
            if i > 10:  # strictly future bars after trade exit
                b_copy["close"] = 999999.0  # wild price spike
                b_copy["history_closes"] = [999999.0] * 60
            bars_mutated.append(b_copy)
            
        record_mutated = simulate_trade_shadow(trade, bars_mutated, self.config)
        
        # Invariant: Observations, recommendation, and fill must be 100% bit-identical
        self.assertEqual(len(record_clean.observations), len(record_mutated.observations))
        self.assertEqual(record_clean.recommendation_emitted, record_mutated.recommendation_emitted)
        self.assertEqual(record_clean.recommendation_timestamp_epoch, record_mutated.recommendation_timestamp_epoch)
        self.assertEqual(record_clean.simulated_fill_price, record_mutated.simulated_fill_price)
        self.assertEqual(record_clean.rp612_net_pnl, record_mutated.rp612_net_pnl)
        self.assertEqual(record_clean.rp612_net_paise, record_mutated.rp612_net_paise)

    def test_exact_floating_point_summation(self):
        """Verifies exact IEEE 754 float identities, integer paise arithmetic, and display conventions."""
        pnl1 = 55901.9506
        pnl2 = 21509.2450
        total = pnl1 + pnl2
        self.assertTrue(math.isclose(total, 77411.1956, rel_tol=1e-9))
        self.assertEqual(round(total, 2), 77411.20)
        
        # Integer paise calculation (1 rupee = 100 paise)
        paise1 = round(pnl1 * 100)  # 5590195 paise
        paise2 = round(pnl2 * 100)  # 2150924 paise (round-half-to-even)
        total_paise = round(total * 100)  # 7741120 paise
        
        # Demonstrates exact mechanism of the 1-paisa (₹0.01) divergence:
        # Sum of rounded individual components = 7,741,119 paise (₹77,411.19)
        # Rounded sum of exact values = 7,741,120 paise (₹77,411.20)
        self.assertEqual(paise1 + paise2, 7741119)
        self.assertEqual(total_paise, 7741120)
        self.assertEqual(total_paise - (paise1 + paise2), 1, "Proves exact 1-paisa divergence from component-level rounding")
        
        # Note: round(pnl1, 2) + round(pnl2, 2) == 77411.19 != round(total, 2)
        # Demonstrates why unrounded exact float and integer paise arithmetic is required
        self.assertNotEqual(round(pnl1, 2) + round(pnl2, 2), round(total, 2))


if __name__ == "__main__":
    unittest.main()
