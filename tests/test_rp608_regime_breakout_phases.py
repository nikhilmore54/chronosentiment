#!/usr/bin/env python3
"""Tests for RP-608-REGIME-BREAKOUT-001 Phases 2–5 modules."""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.breakout_lifecycle import (
    LifecycleState,
    dedupe_key,
    donchian_20,
    evaluate_quote,
    half_spread_bps,
)
from scripts.nse_market_context import apply_market_chop_to_regime, compute_market_context
from scripts.run_rp608_regime_breakout_arms_abc import (
    holm_bonferroni,
    paired_bootstrap_p,
    profit_factor,
    run_synthetic_smoke,
)


def _bars(n: int = 25, start: float = 100.0, drift: float = 0.001):
    bars = []
    px = start
    for _ in range(n):
        o = px
        px = px * (1.0 + drift)
        bars.append({"open": o, "high": max(o, px) * 1.001, "low": min(o, px) * 0.999, "close": px})
    return bars


class TestMarketChop(unittest.TestCase):
    def test_chop_detection(self):
        # Balanced breadth, near-zero mean
        ctx = compute_market_context([5, -4, 3, -2, 1, -1, 2, -3, 4, -5])
        self.assertTrue(ctx["market_chop"])
        self.assertTrue(0.4 <= ctx["session_breadth"] <= 0.6)

    def test_clear_market(self):
        ctx = compute_market_context([50, 40, 30, 20, 10, 5, 8, 12, 15, 18])
        self.assertFalse(ctx["market_chop"])

    def test_er_hurdle_demotes_directional(self):
        stock = {
            "regime": "DIRECTIONAL",
            "er": 0.45,
            "vol_ratio": 1.0,
            "adx": 30.0,
            "reason": "CONFIRMED",
        }
        ctx = {"market_chop": True, "session_breadth": 0.5, "market_return_bps": 1.0, "market_dispersion_bps": 20.0}
        out = apply_market_chop_to_regime(stock, ctx)
        self.assertEqual(out["regime"], "TRANSITIONING")
        self.assertEqual(out["directional_er_hurdle"], 0.50)

    def test_sideways_unchanged_under_chop(self):
        stock = {"regime": "SIDEWAYS", "er": 0.1, "reason": "LOW"}
        ctx = {"market_chop": True}
        out = apply_market_chop_to_regime(stock, ctx)
        self.assertEqual(out["regime"], "SIDEWAYS")


class TestBreakoutLifecycle(unittest.TestCase):
    def test_donchian_excludes_forming(self):
        bars = _bars(25)
        # Inflate last (forming) high — must not affect boundary
        bars[-1]["high"] = 1e9
        b1 = donchian_20(bars, exclude_forming=True)
        bars2 = bars[:-1]
        bars2[-1] = dict(bars2[-1])
        b2 = donchian_20(bars2 + [{"open": 1, "high": 1, "low": 1, "close": 1}], exclude_forming=True)
        self.assertIsNotNone(b1)
        self.assertLess(b1[0], 1e8)

    def test_crossing_before_watch(self):
        bars = _bars(25, drift=0.0)
        upper, lower = donchian_20(bars, exclude_forming=True)
        # Cross above upper with buffer
        ask = upper * 1.01
        bid = ask * 0.9995
        # Also within approach — but crossing must win
        st = evaluate_quote(
            state=LifecycleState(),
            regime="DIRECTIONAL",
            bars_completed_plus_forming=bars,
            ask=ask,
            bid=bid,
            quote_age_ms=50,
            session_date="2026-01-01",
            symbol="X",
            half_spread_cap_bps=5.0,
            target_distance=150,
            stop_distance=100,
            friction=5,
        )
        self.assertIn(st.state, ("TRADE_QUALIFIED", "INVALIDATED", "BREAKOUT_DETECTED"))
        self.assertNotEqual(st.state, "WATCH")

    def test_stale_invalidates_active(self):
        st0 = LifecycleState(state="WATCH", direction="LONG", boundary_price=100.0)
        st = evaluate_quote(
            state=st0,
            regime="DIRECTIONAL",
            bars_completed_plus_forming=_bars(25),
            ask=100.1,
            bid=100.0,
            quote_age_ms=800,
            session_date="2026-01-01",
            symbol="X",
            half_spread_cap_bps=5.0,
            target_distance=150,
            stop_distance=100,
            friction=5,
        )
        self.assertEqual(st.state, "INVALIDATED")
        self.assertEqual(st.reason, "stale")

    def test_expiry(self):
        st0 = LifecycleState(state="WATCH", direction="LONG", boundary_price=100.0)
        st = evaluate_quote(
            state=st0,
            regime="DIRECTIONAL",
            bars_completed_plus_forming=_bars(25),
            ask=100.1,
            bid=100.0,
            quote_age_ms=10,
            session_date="2026-01-01",
            symbol="X",
            half_spread_cap_bps=5.0,
            target_distance=150,
            stop_distance=100,
            friction=5,
            session_expired=True,
        )
        self.assertEqual(st.state, "EXPIRED")

    def test_dedupe_stable(self):
        a = dedupe_key("2026-01-01", "RELIANCE.NS", 100.5, "LONG")
        b = dedupe_key("2026-01-01", "RELIANCE.NS", 100.5, "LONG")
        self.assertEqual(a, b)

    def test_half_spread_definition(self):
        hs = half_spread_bps(100.2, 100.0)
        self.assertIsNotNone(hs)
        # full spread 0.2 on mid 100.1 -> half-spread bps = 0.1/100.1*10000
        self.assertAlmostEqual(hs, (0.1 / 100.1) * 10000, places=4)


class TestArmsABC(unittest.TestCase):
    def test_smoke_report_labels(self):
        report = run_synthetic_smoke()
        self.assertEqual(report["label"], "diagnostic_replay")
        self.assertEqual(report["live_order_authority"], "0.0%")
        self.assertEqual(report["holdout"], "SEALED_UNTOUCHED")
        self.assertEqual(report["promotion"], "INCONCLUSIVE")
        for opp in report["opportunities"]:
            self.assertIn(opp["arm_b"]["outcome_source"], ("taken", "arm_a_counterfactual"))
            self.assertIn(opp["arm_c"]["outcome_source"], ("taken", "arm_a_counterfactual"))

    def test_pf_edge_cases(self):
        self.assertEqual(profit_factor([1, 2, 3]), float("inf"))
        self.assertIsNone(profit_factor([]))
        self.assertLess(profit_factor([1, -2]), 1.0)

    def test_holm_and_bootstrap_deterministic(self):
        p1 = paired_bootstrap_p([0.1, 0.2, 0.15, -0.05])
        p2 = paired_bootstrap_p([0.1, 0.2, 0.15, -0.05])
        self.assertEqual(p1, p2)
        holm = holm_bonferroni([0.01, 0.04, 0.10])
        self.assertEqual(len(holm), 3)


class TestOpsIntegrity(unittest.TestCase):
    def test_phase_artifacts_exist(self):
        base = WORKSPACE / "artifacts" / "rp608_regime_breakout"
        for name in (
            "phase0_is_census.json",
            "phase0_gate_result.json",
            "phase1_arm_a_invariance_gate.json",
        ):
            self.assertTrue((base / name).exists(), name)

    def test_census_holdout_sealed(self):
        census = json.loads((WORKSPACE / "artifacts/rp608_regime_breakout/phase0_is_census.json").read_text())
        self.assertEqual(census["nse"]["holdout_status"], "SEALED_UNTOUCHED")
        self.assertEqual(census["crypto"]["holdout_status"], "SEALED_UNTOUCHED")
        self.assertEqual(census["live_order_authority"], "0.0%")

    def test_protocol_and_authority(self):
        proto = (WORKSPACE / "docs/research/rp608_regime_breakout_001_protocol.md").read_text()
        self.assertIn("0.0%", proto)
        self.assertIn("41–68", proto)
        self.assertIn("arm_a_counterfactual", proto)

    def test_no_place_order_in_daemons(self):
        for rel in (
            "scripts/run_rp608_nse_shadow_daemon.py",
            "scripts/run_rp608_shadow_daemon.py",
        ):
            src = (WORKSPACE / rel).read_text()
            self.assertNotIn("def place_order", src)
            self.assertIn("LIVE_ORDER_AUTHORITY_PCT = 0.0", src)


if __name__ == "__main__":
    unittest.main()
