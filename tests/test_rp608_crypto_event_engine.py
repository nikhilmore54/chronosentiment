#!/usr/bin/env python3
"""
tests/test_rp608_crypto_event_engine.py

Comprehensive Unit Tests for RP-608-EVENT-001 Parallel Multi-Timescale Engine.
Verifies:
  1. BookTicker quote validation (malformed, out-of-order, stale, excessive spread).
  2. Micro-feature calculations (micro-price, imbalance, 5s velocity, tick volatility).
  3. Four parallel candidate lookbacks (16b, 32b, 64b, 120b) with independent candle buffers.
  4. Macro-trend filtering differentiation across lookbacks on the identical quote.
  5. Refractory rate limiting (1.0s minimum) and deduplication.
  6. Ledger spam suppression: NO_TRADE / warmup does NOT flood the recommendation ledger.
  7. Strictly 0.0% Order Authority across all records.
"""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from scripts.rp608_crypto_event_engine import (
    BookTickerQuote,
    CompletedCandle,
    CandleContextBuffer,
    MicroFeatureBuffer,
    CandidateLookbackEngine,
    MultiLookbackCoordinator,
    POLICY_FAMILY,
    CANDIDATE_CONFIGS,
    MAX_VALID_SPREAD_BPS,
    MAX_QUOTE_AGE_MS,
    MIN_REFRACTORY_MS
)


class TestBookTickerValidation(unittest.TestCase):
    """Test Suite 1: Malformed and out-of-order events, stale quotes, and data quality."""

    def test_01_valid_quote(self):
        q = BookTickerQuote(
            update_id=1000,
            event_time_ms=1700000000000,
            receive_time_ms=1700000000100,
            bid_price=80000.0,
            bid_qty=1.5,
            ask_price=80000.1,
            ask_qty=2.5
        )
        is_valid, reason = q.validate()
        self.assertTrue(is_valid, f"Expected valid quote, got {reason}")
        self.assertAlmostEqual(q.mid_price, 80000.05, places=4)
        self.assertAlmostEqual(q.top_of_book_imbalance, (1.5 - 2.5) / 4.0, places=4)
        expected_micro = (80000.0 * 2.5 + 80000.1 * 1.5) / 4.0
        self.assertAlmostEqual(q.micro_price, expected_micro, places=4)

    def test_02_negative_or_zero_price_and_qty(self):
        q_zero_bid = BookTickerQuote(1, 1000, 1000, 0.0, 1.0, 80000.0, 1.0)
        self.assertFalse(q_zero_bid.validate()[0])

        q_neg_ask = BookTickerQuote(1, 1000, 1000, 80000.0, 1.0, -80000.0, 1.0)
        self.assertFalse(q_neg_ask.validate()[0])

        q_zero_qty = BookTickerQuote(1, 1000, 1000, 80000.0, 0.0, 80000.1, 1.0)
        self.assertFalse(q_zero_qty.validate()[0])

    def test_03_inverted_or_crossed_spread(self):
        q_crossed = BookTickerQuote(1, 1000, 1000, 80000.5, 1.0, 80000.1, 1.0)
        is_valid, reason = q_crossed.validate()
        self.assertFalse(is_valid)
        self.assertEqual(reason, "INVERTED_OR_CROSSED_SPREAD")

    def test_04_excessive_spread(self):
        q_wide = BookTickerQuote(1, 1000, 1000, 80000.0, 1.0, 80080.0, 1.0)
        is_valid, reason = q_wide.validate()
        self.assertFalse(is_valid)
        self.assertIn("EXCESSIVE_SPREAD", reason)

    def test_05_stale_quote_rejection(self):
        q_stale = BookTickerQuote(1, 1700000000000, 1700000002500, 80000.0, 1.0, 80000.1, 1.0)
        is_valid, reason = q_stale.validate()
        self.assertFalse(is_valid)
        self.assertIn("STALE_QUOTE_AGE", reason)

    def test_06_monotonicity_checks(self):
        q1 = BookTickerQuote(100, 1700000001000, 1700000001050, 80000.0, 1.0, 80000.1, 1.0)
        
        q_bad_id = BookTickerQuote(99, 1700000001100, 1700000001150, 80000.0, 1.0, 80000.1, 1.0)
        self.assertFalse(q_bad_id.validate(prev_quote=q1)[0])

        q_bad_time = BookTickerQuote(101, 1700000000900, 1700000001150, 80000.0, 1.0, 80000.1, 1.0)
        self.assertFalse(q_bad_time.validate(prev_quote=q1)[0])


class TestMultiTimescaleLookbackDifferentiation(unittest.TestCase):
    """Test Suite 2: Validates that 16b, 32b, 64b, and 120b genuinely differentiate signals on identical quotes."""

    def setUp(self):
        self.tmp_dir = tempfile.TemporaryDirectory()
        self.base_dir = Path(self.tmp_dir.name)
        self.coordinator = MultiLookbackCoordinator(base_dir=self.base_dir)

    def tearDown(self):
        self.tmp_dir.cleanup()

    def test_01_macro_trend_differentiation(self):
        """
        Simulates market scenario:
        - 120-bar macro trend is DOWN (price dropped from 85,000 to 80,000 = -588 bps).
        - 16-bar short trend is UP (bounced from 79,500 to 80,000 = +62 bps).
        - A bullish micro surge occurs (I = +0.66, V_5s = +2.5 bps).
        Verifies:
        - Candidate A (16b) EMITS RECOMMEND_LONG (concordant with short-term bounce).
        - Candidate D (120b) SUPPRESSES and emits NO_TRADE (filtered by severe macro downtrend).
        """
        base_t = 1700000000000
        # Ingest 120 completed candles
        for idx in range(120):
            # First 104 candles drop from 85000 to 79500
            if idx < 104:
                p = 85000.0 - idx * (5500.0 / 104.0)
            else: # last 16 candles bounce from 79500 to 80000
                p = 79500.0 + (idx - 104) * (500.0 / 16.0)

            c = CompletedCandle(
                bar_idx=idx + 1,
                open_time_ms=base_t + idx * 900000,
                close_time_ms=base_t + (idx + 1) * 900000 - 1,
                open_price=p,
                high_price=p + 20,
                low_price=p - 20,
                close_price=p,
                volume=100.0
            )
            self.coordinator.ingest_completed_candle(c)

        # Confirm buffer status
        self.assertEqual(self.coordinator.candle_buf.count(), 120)
        t16 = self.coordinator.candle_buf.get_trend_return_bps(16)
        t120 = self.coordinator.candle_buf.get_trend_return_bps(120)
        self.assertGreater(t16, 20.0, "Expected 16-bar trend to be bullish")
        self.assertLess(t120, -100.0, "Expected 120-bar trend to be bearish")

        # Now feed 10 seconds of warmup quotes to all candidates
        q_base_t = base_t + 120 * 900000
        for s in range(10):
            raw_q = {
                "u": 1000 + s, "s": "BTCUSDT", "b": "80000.00", "B": "1.0",
                "a": "80000.01", "A": "1.0", "local_wallclock_utc_ms": q_base_t + s * 1000
            }
            self.coordinator.process_quote_event(raw_q, receive_time_ms=q_base_t + s * 1000 + 10)

        # Now feed 5s of strong bullish microstructure surge
        for s in range(10, 15):
            p = 80000.0 + (s - 10) * 4.0
            raw_q = {
                "u": 1000 + s, "s": "BTCUSDT", "b": f"{p:.2f}", "B": "5.0",
                "a": f"{p+0.01:.2f}", "A": "1.0", "local_wallclock_utc_ms": q_base_t + s * 1000
            }
            self.coordinator.process_quote_event(raw_q, receive_time_ms=q_base_t + s * 1000 + 10)

        # Trigger tick at s=15
        p15 = 80020.0
        trigger_q = {
            "u": 1015, "s": "BTCUSDT", "b": f"{p15:.2f}", "B": "5.0",
            "a": f"{p15+0.01:.2f}", "A": "1.0", "local_wallclock_utc_ms": q_base_t + 15000
        }
        res = self.coordinator.process_quote_event(trigger_q, receive_time_ms=q_base_t + 15010)

        # Candidate A (16b): MUST emit RECOMMEND_LONG
        rec_16 = res[16]
        self.assertIsNotNone(rec_16)
        self.assertEqual(rec_16["recommendation"]["direction"], "LONG")
        self.assertIn("K16", rec_16["recommendation"]["rationale"])

        # Candidate D (120b): MUST be filtered out by macro bearish trend
        rec_120 = res[120]
        self.assertIsNone(rec_120, "Candidate D should have filtered out the counter-trend long")
        self.assertGreater(self.coordinator.engines[120].stats["recs_counter_trend_filtered"], 0)


class TestNoiseControlAndLedgerIntegrity(unittest.TestCase):
    """Test Suite 3: Validates spam suppression, refractory periods, and ledger formatting."""

    def setUp(self):
        self.tmp_dir = tempfile.TemporaryDirectory()
        self.ledger_path = Path(self.tmp_dir.name) / "test_candidate_16b.jsonl"
        self.engine = CandidateLookbackEngine(lookback_bars=16, ledger_path=self.ledger_path)

    def tearDown(self):
        self.tmp_dir.cleanup()

    def test_01_no_trade_does_not_flood_ledger(self):
        """Verifies that 100 neutral quotes do NOT write 100 NO_TRADE records to the ledger."""
        base_t = 1700000000000
        for s in range(50):
            payload = {
                "u": 100 + s, "s": "BTCUSDT", "b": "80000.00", "B": "1.0",
                "a": "80000.01", "A": "1.0", "local_wallclock_utc_ms": base_t + s * 100
            }
            self.engine.process_raw_dict(payload, receive_time_ms=base_t + s * 100 + 10)

        # If ledger exists, it must have 0 records because no active signals occurred
        if self.ledger_path.exists():
            lines = self.ledger_path.read_text().strip().split("\n")
            lines = [l for l in lines if l.strip()]
            self.assertEqual(len(lines), 0, "NO_TRADE ticks must NOT be written to the recommendation ledger")
        self.assertEqual(self.engine.stats["recs_emitted"], 0)

    def test_02_refractory_rate_limiting_and_deduplication(self):
        """Verifies 1000ms refractory period and deduplication on active signals."""
        base_t = 1700000000000
        # Warmup 10s
        for s in range(10):
            payload = {"u": 100 + s, "s": "BTCUSDT", "b": "80000.00", "B": "1.0", "a": "80000.01", "A": "1.0", "local_wallclock_utc_ms": base_t + s * 1000}
            self.engine.process_raw_dict(payload, receive_time_ms=base_t + s * 1000 + 10)

        # Surge quotes across 5s
        for s in range(10, 15):
            p = 80000.0 + (s - 10) * 5.0
            payload = {"u": 100 + s, "s": "BTCUSDT", "b": f"{p:.2f}", "B": "5.0", "a": f"{p+0.01:.2f}", "A": "1.0", "local_wallclock_utc_ms": base_t + s * 1000}
            self.engine.process_raw_dict(payload, receive_time_ms=base_t + s * 1000 + 10)

        # First active signal at s=15
        p15 = 80025.0
        rec1 = self.engine.process_raw_dict({
            "u": 115, "s": "BTCUSDT", "b": f"{p15:.2f}", "B": "5.0", "a": f"{p15+0.01:.2f}", "A": "1.0", "local_wallclock_utc_ms": base_t + 15000
        }, receive_time_ms=base_t + 15010)

        self.assertIsNotNone(rec1)
        self.assertEqual(rec1["recommendation"]["direction"], "LONG")

        # Second signal 400ms later -> MUST be rate-limited
        rec_suppressed = self.engine.process_raw_dict({
            "u": 116, "s": "BTCUSDT", "b": f"{p15:.2f}", "B": "5.0", "a": f"{p15+0.01:.2f}", "A": "1.0", "local_wallclock_utc_ms": base_t + 15400
        }, receive_time_ms=base_t + 15410)
        self.assertIsNone(rec_suppressed)
        self.assertEqual(self.engine.stats["recs_rate_limited"], 1)

        # Third signal 1200ms later (> 1000ms) with unchanged price -> MUST be deduplicated
        rec_dedup = self.engine.process_raw_dict({
            "u": 117, "s": "BTCUSDT", "b": f"{p15:.2f}", "B": "5.0", "a": f"{p15+0.01:.2f}", "A": "1.0", "local_wallclock_utc_ms": base_t + 16300
        }, receive_time_ms=base_t + 16310)
        self.assertIsNone(rec_dedup)
        self.assertEqual(self.engine.stats["recs_deduplicated"], 1)

    def test_03_zero_order_authority_invariant(self):
        """Verifies strictly 0.0 order authority across emitted records."""
        base_t = 1700000000000
        # Warmup and trigger
        for s in range(10):
            payload = {"u": 100 + s, "s": "BTCUSDT", "b": "80000.00", "B": "1.0", "a": "80000.01", "A": "1.0", "local_wallclock_utc_ms": base_t + s * 1000}
            self.engine.process_raw_dict(payload, receive_time_ms=base_t + s * 1000 + 10)
        for s in range(10, 15):
            p = 80000.0 + (s - 10) * 5.0
            payload = {"u": 100 + s, "s": "BTCUSDT", "b": f"{p:.2f}", "B": "5.0", "a": f"{p+0.01:.2f}", "A": "1.0", "local_wallclock_utc_ms": base_t + s * 1000}
            self.engine.process_raw_dict(payload, receive_time_ms=base_t + s * 1000 + 10)

        rec = self.engine.process_raw_dict({
            "u": 115, "s": "BTCUSDT", "b": "80025.00", "B": "5.0", "a": "80025.01", "A": "1.0", "local_wallclock_utc_ms": base_t + 15000
        }, receive_time_ms=base_t + 15010)

        self.assertIsNotNone(rec)
        self.assertEqual(rec["order_authority"], 0.0)


if __name__ == "__main__":
    unittest.main()
