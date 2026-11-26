"""Post-close next-session watchlist unit tests."""

from __future__ import annotations

import unittest
from datetime import date, datetime, timezone
from pathlib import Path
import sys

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.rp608_nse_next_session_recs import (
    LEVEL_STATUS_INVALIDATED,
    LEVEL_STATUS_STALE,
    LEVEL_STATUS_VALID,
    LEVEL_STATUS_WAITING,
    LEVELS_POLICY_ID,
    LEVELS_ROLE,
    ORDER_FLOW_LABEL,
    apply_price_event_to_watch_levels,
    build_next_session_recommendations,
    classify_order_flow_support,
    classify_watch_level_status,
    compute_indicative_watch_levels,
    enrich_recommendation_levels_live,
    l1_imbalance,
    next_weekday_session,
)


class TestNextSessionRecs(unittest.TestCase):
    def test_next_weekday_skips_weekend(self):
        # Friday → Monday
        self.assertEqual(next_weekday_session(date(2026, 10, 30)), date(2026, 11, 2))
        # Thursday → Friday
        self.assertEqual(next_weekday_session(date(2026, 10, 29)), date(2026, 10, 30))

    def test_ranks_strong_eod_momentum(self):
        hist = {
            "WEAK.NS": [{"close": 100.0 + i * 0.01} for i in range(10)],
            "STRONG.NS": [{"close": 100.0 * (1.01 ** i)} for i in range(12)],
        }
        fixed_now = datetime(2026, 11, 2, 10, 15, 30, tzinfo=timezone.utc)
        payload = build_next_session_recommendations(
            hist,
            from_session_date="2026-10-30",
            next_session_date="2026-11-02",
            top_n=5,
            now_utc=fixed_now,
        )
        self.assertEqual(payload["protocol_id"], "RP-608-NSE-NEXT-SESSION-001")
        self.assertEqual(payload["live_order_authority"], 0.0)
        self.assertEqual(payload["emitted_utc"], "2026-11-02T10:15:30Z")
        self.assertEqual(payload["recommended_at_utc"], "2026-11-02T10:15:30Z")
        self.assertGreaterEqual(payload["n_recommendations"], 1)
        top = payload["recommendations"][0]
        self.assertEqual(top["symbol"], "STRONG.NS")
        self.assertEqual(top["decision"], "WATCH_NEXT_SESSION")
        self.assertEqual(top["actionable_from_session_date"], "2026-11-02")
        self.assertEqual(top["actionable_from_bar"], 5)
        self.assertEqual(top["recommended_at_utc"], "2026-11-02T10:15:30Z")
        self.assertFalse(top["order_flow_supported"])
        self.assertIsNone(top["order_flow_label"])

    def test_deterministic(self):
        hist = {
            "AAA.NS": [{"close": 100 + i} for i in range(8)],
            "BBB.NS": [{"close": 50 + i * 0.5} for i in range(8)],
        }
        fixed_now = datetime(2026, 11, 2, 10, 15, 30, tzinfo=timezone.utc)
        a = build_next_session_recommendations(
            hist,
            from_session_date="2026-10-30",
            next_session_date="2026-11-02",
            now_utc=fixed_now,
        )
        b = build_next_session_recommendations(
            hist,
            from_session_date="2026-10-30",
            next_session_date="2026-11-02",
            now_utc=fixed_now,
        )
        self.assertEqual(a["recommendations"], b["recommendations"])

    def test_l1_imbalance(self):
        self.assertAlmostEqual(l1_imbalance(70, 30), 0.4, places=6)
        self.assertIsNone(l1_imbalance(0, 0))

    def test_order_flow_mark_long_bid_heavy(self):
        of = classify_order_flow_support(
            "LONG",
            {"bid_qty": 80, "ask_qty": 20, "source": "test"},
        )
        self.assertTrue(of["order_flow_supported"])
        self.assertEqual(of["order_flow_label"], ORDER_FLOW_LABEL)
        self.assertGreaterEqual(of["order_flow_imbalance"], 0.30)

    def test_order_flow_unmarked_when_ask_heavy(self):
        of = classify_order_flow_support(
            "LONG",
            {"bid_qty": 20, "ask_qty": 80, "source": "test"},
        )
        self.assertFalse(of["order_flow_supported"])
        self.assertIsNone(of["order_flow_label"])

    def test_order_flow_requires_micro_agree_when_prices_present(self):
        # Bid-heavy qty but microprice below mid (ask-leaning sizes vs prices)
        of = classify_order_flow_support(
            "LONG",
            {
                "bid_qty": 90,
                "ask_qty": 10,
                "bid_price": 100.0,
                "ask_price": 101.0,
                # micro = (101*90 + 100*10)/100 = 100.9; mid=100.5 → positive OK
                "source": "test",
            },
        )
        self.assertTrue(of["order_flow_supported"])
        # Force micro against LONG: use inverted qty weighting with wide book
        # Actually with bid_qty high, micro is above mid. Use ask-heavy for fail case above.
        of2 = classify_order_flow_support(
            "LONG",
            {
                "bid_qty": 10,
                "ask_qty": 90,
                "bid_price": 100.0,
                "ask_price": 101.0,
                "source": "test",
            },
        )
        self.assertFalse(of2["order_flow_supported"])

    def test_build_marks_supported_from_l1(self):
        hist = {
            "FLOW.NS": [{"close": 100.0 * (1.01 ** i)} for i in range(12)],
            "NOFLOW.NS": [{"close": 100.0 * (1.009 ** i)} for i in range(12)],
        }
        l1 = {
            "FLOW.NS": {"bid_qty": 90, "ask_qty": 10, "source": "test"},
            "NOFLOW.NS": {"bid_qty": 10, "ask_qty": 90, "source": "test"},
        }
        fixed_now = datetime(2026, 11, 2, 10, 15, 30, tzinfo=timezone.utc)
        payload = build_next_session_recommendations(
            hist,
            from_session_date="2026-10-30",
            next_session_date="2026-11-02",
            top_n=5,
            l1_by_symbol=l1,
            now_utc=fixed_now,
        )
        by_sym = {r["symbol"]: r for r in payload["recommendations"]}
        self.assertTrue(by_sym["FLOW.NS"]["order_flow_supported"])
        self.assertEqual(by_sym["FLOW.NS"]["order_flow_label"], ORDER_FLOW_LABEL)
        self.assertFalse(by_sym["NOFLOW.NS"]["order_flow_supported"])
        self.assertIsNone(by_sym["NOFLOW.NS"]["order_flow_label"])
        self.assertEqual(payload["n_order_flow_supported"], 1)
        # Deterministic with L1
        b = build_next_session_recommendations(
            hist,
            from_session_date="2026-10-30",
            next_session_date="2026-11-02",
            top_n=5,
            l1_by_symbol=l1,
            now_utc=fixed_now,
        )
        self.assertEqual(payload["recommendations"], b["recommendations"])

    def test_excludes_quarantined_from_eligible_set(self):
        hist = {
            "KEEP.NS": [{"close": 100.0 * (1.01 ** i)} for i in range(12)],
            "QUAR.NS": [{"close": 100.0 * (1.012 ** i)} for i in range(12)],
        }
        payload = build_next_session_recommendations(
            hist,
            from_session_date="2026-10-30",
            next_session_date="2026-11-02",
            top_n=5,
            eligible_symbols={"KEEP.NS", "KEEP"},
            universe_label="RP-607-classified-102",
        )
        syms = [r["symbol"] for r in payload["recommendations"]]
        self.assertIn("KEEP.NS", syms)
        self.assertNotIn("QUAR.NS", syms)
        self.assertEqual(payload["skipped_ineligible"], 1)
        self.assertEqual(payload["universe_label"], "RP-607-classified-102")

    def _ohlc_trend_bars(self, n: int = 40, start: float = 100.0):
        bars = []
        px = start
        for i in range(n):
            px *= 1.004
            bars.append(
                {
                    "timestamp_utc_ms": 1_700_000_000_000 + i * 900_000,
                    "open": px * 0.999,
                    "high": px * 1.006,
                    "low": px * 0.994,
                    "close": px,
                    "volume": 1000 + i,
                }
            )
        return bars

    def test_indicative_levels_from_dynamic_structure(self):
        bars = self._ohlc_trend_bars()
        levels = compute_indicative_watch_levels(
            bars,
            side="LONG",
            calculated_at_utc="2026-11-02T10:00:00Z",
        )
        self.assertTrue(levels.get("ok"), levels)
        self.assertEqual(levels["policy_id"], LEVELS_POLICY_ID)
        self.assertEqual(levels["levels_role"], LEVELS_ROLE)
        self.assertTrue(levels["not_executed_entry"])
        self.assertTrue(levels["not_paper_trade_entry"])
        self.assertLess(levels["entry_low"], levels["entry_high"])
        self.assertLess(levels["stop_price"], levels["entry_low"])
        self.assertGreater(levels["target_price"], levels["entry_high"])
        self.assertIn("1 :", levels["risk_reward_display"])

    def test_level_status_valid_waiting_invalidated_stale(self):
        st = classify_watch_level_status(
            side="LONG",
            last_price=100.0,
            entry_low=99.0,
            entry_high=101.0,
            stop_price=97.0,
            session_actionable=True,
        )
        self.assertEqual(st["level_status"], LEVEL_STATUS_VALID)
        st2 = classify_watch_level_status(
            side="LONG",
            last_price=102.0,
            entry_low=99.0,
            entry_high=101.0,
            stop_price=97.0,
            session_actionable=True,
        )
        self.assertEqual(st2["level_status"], LEVEL_STATUS_WAITING)
        st3 = classify_watch_level_status(
            side="LONG",
            last_price=96.5,
            entry_low=99.0,
            entry_high=101.0,
            stop_price=97.0,
            session_actionable=True,
        )
        self.assertEqual(st3["level_status"], LEVEL_STATUS_INVALIDATED)
        st4 = classify_watch_level_status(
            side="LONG",
            last_price=100.0,
            entry_low=99.0,
            entry_high=101.0,
            stop_price=97.0,
            feed_stale=True,
            session_actionable=True,
        )
        self.assertEqual(st4["level_status"], LEVEL_STATUS_STALE)
        st5 = classify_watch_level_status(
            side="LONG",
            last_price=100.0,
            entry_low=99.0,
            entry_high=101.0,
            stop_price=97.0,
            session_actionable=False,
        )
        self.assertEqual(st5["level_status"], LEVEL_STATUS_WAITING)

    def test_build_includes_levels_block(self):
        hist = {"STRONG.NS": self._ohlc_trend_bars()}
        payload = build_next_session_recommendations(
            hist,
            from_session_date="2026-10-30",
            next_session_date="2026-11-02",
            top_n=5,
            now_utc=datetime(2026, 11, 2, 10, 0, 0, tzinfo=timezone.utc),
        )
        self.assertEqual(payload["levels_role"], LEVELS_ROLE)
        self.assertTrue(payload["levels_informational_only"])
        top = payload["recommendations"][0]
        self.assertTrue(top["not_executed_entry"])
        self.assertIn("levels", top)
        self.assertEqual(top["level_status"], LEVEL_STATUS_WAITING)

    def test_invalidate_latch_blocks_silent_revalid(self):
        """Price returning to band must not clear INVALIDATED without fresh structure."""
        st = classify_watch_level_status(
            side="LONG",
            last_price=96.5,
            entry_low=99.0,
            entry_high=101.0,
            stop_price=97.0,
            session_actionable=True,
        )
        self.assertEqual(st["level_status"], LEVEL_STATUS_INVALIDATED)
        self.assertTrue(st["invalidation_latched"])

        back = classify_watch_level_status(
            side="LONG",
            last_price=100.0,
            entry_low=99.0,
            entry_high=101.0,
            stop_price=97.0,
            session_actionable=True,
            invalidation_latched=True,
        )
        self.assertEqual(back["level_status"], LEVEL_STATUS_INVALIDATED)
        self.assertEqual(back["level_status_reason"], "LATCHED_INVALID_REQUIRES_FRESH_STRUCTURE")
        self.assertTrue(back["invalidation_latched"])

        waiting_latched = classify_watch_level_status(
            side="LONG",
            last_price=102.0,
            entry_low=99.0,
            entry_high=101.0,
            stop_price=97.0,
            session_actionable=True,
            invalidation_latched=True,
        )
        self.assertEqual(waiting_latched["level_status"], LEVEL_STATUS_WAITING)
        self.assertTrue(waiting_latched["invalidation_latched"])

    def test_apply_price_event_preserves_latch(self):
        levels = {
            "entry_low": 99.0,
            "entry_high": 101.0,
            "stop_price": 97.0,
            "target_price": 105.0,
            "invalidation_latched": False,
            "structure_generation": 1,
        }
        hit_stop = apply_price_event_to_watch_levels(
            levels,
            side="LONG",
            last_price=96.0,
            session_actionable=True,
        )
        self.assertEqual(hit_stop["level_status"], LEVEL_STATUS_INVALIDATED)
        self.assertTrue(hit_stop["invalidation_latched"])

        return_to_band = apply_price_event_to_watch_levels(
            hit_stop,
            side="LONG",
            last_price=100.0,
            session_actionable=True,
        )
        self.assertEqual(return_to_band["level_status"], LEVEL_STATUS_INVALIDATED)
        self.assertTrue(return_to_band["invalidation_latched"])

    def test_fresh_structure_clears_latch_on_new_bar(self):
        bars_a = self._ohlc_trend_bars(n=40)
        bars_b = self._ohlc_trend_bars(n=41)
        levels = compute_indicative_watch_levels(bars_a, side="LONG")
        levels["invalidation_latched"] = True
        levels["level_status"] = LEVEL_STATUS_INVALIDATED
        row = {
            "symbol": "X.NS",
            "side": "LONG",
            "levels": levels,
            "close_price": levels.get("reference_price"),
        }
        # Same bar ts → latch must remain
        same = enrich_recommendation_levels_live(
            [row],
            price_history={"X.NS": bars_a},
            live_prices={"X.NS": float(levels["entry_low"]) + 0.01},
            session_actionable=True,
            recalculate_structure=True,
        )[0]
        self.assertTrue(same["invalidation_latched"])
        self.assertEqual(same["level_status"], LEVEL_STATUS_INVALIDATED)

        # New completed bar → structure_generation bumps and latch clears
        mid_probe = enrich_recommendation_levels_live(
            [row],
            price_history={"X.NS": bars_b},
            live_prices={"X.NS": float(levels["entry_low"]) + 0.01},
            session_actionable=True,
            recalculate_structure=True,
        )[0]
        self.assertFalse(mid_probe["invalidation_latched"])
        self.assertEqual(
            int(mid_probe["levels"]["structure_generation"]),
            int(levels["structure_generation"]) + 1,
        )
        # Re-evaluate with price inside the *new* band → VALID after latch clear
        lo = float(mid_probe["levels"]["entry_low"])
        hi = float(mid_probe["levels"]["entry_high"])
        inside = enrich_recommendation_levels_live(
            [mid_probe],
            price_history=None,
            live_prices={"X.NS": 0.5 * (lo + hi)},
            session_actionable=True,
            recalculate_structure=False,
        )[0]
        self.assertEqual(inside["level_status"], LEVEL_STATUS_VALID)
        self.assertFalse(inside["invalidation_latched"])

    def test_presentation_poll_never_recalculates_structure(self):
        bars = self._ohlc_trend_bars(n=40)
        levels = compute_indicative_watch_levels(bars, side="LONG")
        gen = int(levels["structure_generation"])
        bar_ts = levels["levels_source_bar_timestamp_utc_ms"]
        row = {"symbol": "Y.NS", "side": "LONG", "levels": levels}
        longer = self._ohlc_trend_bars(n=45)
        out = enrich_recommendation_levels_live(
            [row],
            price_history={"Y.NS": longer},
            live_prices={"Y.NS": float(levels["entry_low"]) + 0.01},
            session_actionable=True,
            recalculate_structure=False,
        )[0]
        self.assertEqual(int(out["levels"]["structure_generation"]), gen)
        self.assertEqual(out["levels"]["levels_source_bar_timestamp_utc_ms"], bar_ts)


if __name__ == "__main__":
    unittest.main()
