#!/usr/bin/env python3
"""
Deterministic fixtures for RP-608 breakout lifecycle (locked thresholds unchanged).

Coverage: non-crossing, long/short crossing, stale, insufficient Net R:R,
boundary inclusion (signal excluded), timestamp/quote alignment with signal,
repeated-event dedupe / terminal stickiness, and harness packing consistency.
"""

from __future__ import annotations

import ast
import sys
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.breakout_lifecycle import (
    LifecycleState,
    dedupe_key,
    delta_buffer,
    donchian_20,
    evaluate_quote,
    pack_bars_for_breakout_eval,
)


def _flat_bars(n: int, px: float = 100.0):
    return [{"open": px, "high": px + 0.5, "low": px - 0.5, "close": px} for _ in range(n)]


class TestBreakoutFixturesLockedThresholds(unittest.TestCase):
    """Buffer / spread caps / R:R remain protocol defaults; only geometry is tested."""

    def setUp(self):
        # 20 prior completed bars with known Donchian upper=105, lower=95
        self.prior = []
        for i in range(20):
            self.prior.append({
                "open": 100.0,
                "high": 105.0 if i == 10 else 101.0,
                "low": 95.0 if i == 5 else 99.0,
                "close": 100.0,
            })
        self.upper, self.lower = donchian_20(
            self.prior + [{"open": 100, "high": 200, "low": 50, "close": 100}],  # forming must not affect
            exclude_forming=True,
        )
        self.assertEqual(self.upper, 105.0)
        self.assertEqual(self.lower, 95.0)

    def _eval(self, ask, bid, age_ms=50.0, target=150.0, stop=100.0, friction=12.0, regime="DIRECTIONAL"):
        # Signal/evaluation candle as LAST (forming for Donchian exclusion)
        signal = {"open": 100.0, "high": 110.0, "low": 99.0, "close": (ask + bid) / 2.0}
        return evaluate_quote(
            state=LifecycleState(),
            regime=regime,
            bars_completed_plus_forming=self.prior + [signal],
            ask=ask,
            bid=bid,
            quote_age_ms=age_ms,
            session_date="FIX",
            symbol="TEST",
            half_spread_cap_bps=1.80,
            target_distance=target,
            stop_distance=stop,
            friction=friction,
        )

    def test_donchian_excludes_forming_even_if_extreme(self):
        bars = self.prior + [{"open": 100, "high": 999, "low": 1, "close": 100}]
        u, l = donchian_20(bars, exclude_forming=True)
        self.assertEqual(u, 105.0)
        self.assertEqual(l, 95.0)

    def test_non_crossing_stays_idle_or_watch(self):
        mid = 100.0
        # Tight book around mid — well inside [95, 105]
        ask, bid = mid + 0.01, mid - 0.01
        st = self._eval(ask, bid)
        self.assertNotEqual(st.state, "TRADE_QUALIFIED")
        self.assertNotEqual(st.state, "BREAKOUT_DETECTED")
        self.assertIn(st.state, ("IDLE", "WATCH"))

    def test_valid_long_crossing_qualifies(self):
        mid = 105.0
        full_spread = mid * 0.0002  # small spread → half-spread ~1 bps < 1.80 cap
        ask = mid + full_spread / 2.0
        bid = mid - full_spread / 2.0
        buf = delta_buffer(mid, ask, bid)
        # Executable ask must clear upper + buffer
        ask = self.upper + buf + 0.01
        bid = ask - full_spread
        st = self._eval(ask, bid, target=150.0, stop=100.0, friction=12.0)
        # Net R:R = (150-12)/(100+12) > 1
        self.assertEqual(st.state, "TRADE_QUALIFIED")
        self.assertEqual(st.direction, "LONG")
        self.assertEqual(st.reason, "net_rr_ok")

    def test_stale_quote_no_new_qualify(self):
        # First establish WATCH/active via approach, then stale invalidates
        ask = self.upper  # at boundary → approach
        bid = ask - 0.02
        st0 = self._eval(ask, bid, age_ms=50.0)
        self.assertEqual(st0.state, "WATCH")
        st1 = evaluate_quote(
            state=st0,
            regime="DIRECTIONAL",
            bars_completed_plus_forming=self.prior
            + [{"open": 100, "high": 110, "low": 99, "close": 105}],
            ask=ask,
            bid=bid,
            quote_age_ms=800.0,  # > 500 ms locked freshness
            session_date="FIX",
            symbol="TEST",
            half_spread_cap_bps=1.80,
            target_distance=150.0,
            stop_distance=100.0,
            friction=12.0,
        )
        self.assertEqual(st1.state, "INVALIDATED")
        self.assertEqual(st1.reason, "stale")

    def test_insufficient_net_rr_invalidates_after_cross(self):
        full_spread = 0.02
        ask = self.upper + delta_buffer(105.0, 105.01, 104.99) + 0.05
        bid = ask - full_spread
        # Force Net R:R < 1: target 50, stop 100, friction 12 → (50-12)/(100+12) < 1
        st = self._eval(ask, bid, target=50.0, stop=100.0, friction=12.0)
        self.assertEqual(st.state, "INVALIDATED")
        self.assertEqual(st.reason, "insufficient_rr")

    def test_signal_bar_must_not_inflate_donchian(self):
        """
        Harness defect regression: if signal bar is treated as completed history,
        ask≈close can never clear max(high) including that bar.
        Correct packing: prior completed + [signal as forming].
        """
        signal_close = 106.0  # above prior upper 105
        signal = {"open": 104.0, "high": 108.0, "low": 103.5, "close": signal_close}
        # WRONG packing (historical Gate 4 bug): signal already in completed hist
        wrong = self.prior + [signal] + [{"open": signal_close, "high": signal_close * 1.002,
                                          "low": signal_close * 0.999, "close": signal_close}]
        u_wrong, _ = donchian_20(wrong, exclude_forming=True)
        self.assertEqual(u_wrong, 108.0)  # inflated by signal high
        ask = signal_close * 1.00015
        self.assertLessEqual(ask, u_wrong)  # cannot cross

        # CORRECT packing: signal is forming
        right = pack_bars_for_breakout_eval(self.prior + [signal])
        u_right, _ = donchian_20(right, exclude_forming=True)
        self.assertEqual(u_right, 105.0)
        self.assertLess(u_right, u_wrong)

    def test_boundary_inclusion_window_is_prior_20_only(self):
        """Bar t-21 and signal bar must not enter the Donchian-20 window."""
        # Insert an extreme at t-21 (index 0 of a 21-prior series) — outside window
        older = [{"open": 100, "high": 200, "low": 10, "close": 100}]
        windowed = older + self.prior  # len 21; donchian uses last 20 of completed = self.prior
        signal = {"open": 100, "high": 150, "low": 50, "close": 106}
        packed = pack_bars_for_breakout_eval(windowed + [signal])
        u, l = donchian_20(packed, exclude_forming=True)
        self.assertEqual(u, 105.0)
        self.assertEqual(l, 95.0)
        # Signal extremes must not appear in the completed window
        completed = packed[:-1]
        self.assertNotIn(150.0, [b["high"] for b in completed[-20:]])
        self.assertNotIn(50.0, [b["low"] for b in completed[-20:]])
        self.assertNotIn(200.0, [b["high"] for b in completed[-20:]])

    def test_timestamp_alignment_quote_with_signal_close(self):
        """Contemporaneous quote mid equals signal close (harness close-proxied L1)."""
        signal_close = 106.5
        signal = {"open": 104.0, "high": 107.0, "low": 103.0, "close": signal_close}
        packed = pack_bars_for_breakout_eval(self.prior + [signal])
        half = signal_close * 0.00015
        ask, bid = signal_close + half, signal_close - half
        mid = (ask + bid) / 2.0
        self.assertAlmostEqual(mid, signal_close, places=10)
        self.assertIs(packed[-1], signal)
        u, _ = donchian_20(packed, exclude_forming=True)
        self.assertEqual(u, 105.0)
        buf = delta_buffer(mid, ask, bid)
        st = evaluate_quote(
            state=LifecycleState(),
            regime="DIRECTIONAL",
            bars_completed_plus_forming=packed,
            ask=ask,
            bid=bid,
            quote_age_ms=50.0,
            session_date="FIX",
            symbol="TEST",
            half_spread_cap_bps=1.80,
            target_distance=150.0,
            stop_distance=100.0,
            friction=12.0,
        )
        # close 106.5 clears prior upper 105 + small buffer → long qualify
        self.assertTrue(ask > u + buf)
        self.assertEqual(st.state, "TRADE_QUALIFIED")
        self.assertEqual(st.direction, "LONG")

    def test_valid_short_crossing_qualifies(self):
        full_spread = 0.02
        bid = self.lower - delta_buffer(95.0, 95.01, 94.99) - 0.05
        ask = bid + full_spread
        st = self._eval(ask, bid, target=150.0, stop=100.0, friction=12.0)
        self.assertEqual(st.state, "TRADE_QUALIFIED")
        self.assertEqual(st.direction, "SHORT")
        self.assertEqual(st.reason, "net_rr_ok")
        self.assertEqual(st.boundary_price, self.lower)

    def test_repeated_event_dedupe_and_terminal_stickiness(self):
        full_spread = 0.02
        ask = self.upper + delta_buffer(105.0, 105.01, 104.99) + 0.05
        bid = ask - full_spread
        st1 = self._eval(ask, bid)
        self.assertEqual(st1.state, "TRADE_QUALIFIED")
        key = dedupe_key("FIX", "TEST", self.upper, "LONG")
        self.assertEqual(st1.dedupe, key)
        # Same identity → same key
        self.assertEqual(dedupe_key("FIX", "TEST", self.upper, "LONG"), key)
        # Opposite side → different key
        self.assertNotEqual(dedupe_key("FIX", "TEST", self.lower, "SHORT"), key)
        # Terminal: further quotes must not transition
        st2 = evaluate_quote(
            state=st1,
            regime="DIRECTIONAL",
            bars_completed_plus_forming=self.prior
            + [{"open": 100, "high": 120, "low": 90, "close": 110}],
            ask=ask + 1.0,
            bid=bid + 1.0,
            quote_age_ms=50.0,
            session_date="FIX",
            symbol="TEST",
            half_spread_cap_bps=1.80,
            target_distance=150.0,
            stop_distance=100.0,
            friction=12.0,
        )
        self.assertEqual(st2.state, "TRADE_QUALIFIED")
        self.assertEqual(st2.dedupe, st1.dedupe)
        self.assertEqual(st2.direction, "LONG")


class TestHarnessPackingConsistency(unittest.TestCase):
    """NSE + crypto historical paths and smoke harness share pack_bars_for_breakout_eval."""

    def test_historical_and_smoke_import_canonical_packer(self):
        paths = [
            WORKSPACE / "scripts/run_rp608_regime_breakout_arms_abc_historical.py",
            WORKSPACE / "scripts/run_rp608_regime_breakout_arms_abc.py",
            WORKSPACE / "scripts/audit_rp608_arm_c_qualification_readonly.py",
        ]
        for path in paths:
            src = path.read_text(encoding="utf-8")
            self.assertIn("pack_bars_for_breakout_eval", src, msg=str(path))
            tree = ast.parse(src)
            calls = [
                n for n in ast.walk(tree)
                if isinstance(n, ast.Call) and (
                    (isinstance(n.func, ast.Name) and n.func.id == "pack_bars_for_breakout_eval")
                    or (isinstance(n.func, ast.Attribute) and n.func.attr == "pack_bars_for_breakout_eval")
                )
            ]
            self.assertGreaterEqual(len(calls), 1, msg=f"no call in {path}")

    def test_historical_nse_and_crypto_both_pack(self):
        src = (WORKSPACE / "scripts/run_rp608_regime_breakout_arms_abc_historical.py").read_text(
            encoding="utf-8"
        )
        # Exactly two evaluate sites (NSE + crypto) both via canonical packer
        self.assertEqual(src.count("pack_bars_for_breakout_eval("), 2)
        self.assertNotIn("mid * 1.002", src)
        self.assertNotIn("mid * 1.001", src)


if __name__ == "__main__":
    unittest.main()
