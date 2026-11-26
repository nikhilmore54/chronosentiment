#!/usr/bin/env python3
"""Unit tests: missing session candle detection for bootstrap fetch-on-gap."""

from __future__ import annotations

import sys
import unittest
from datetime import datetime, timedelta, timezone
from types import SimpleNamespace
from unittest.mock import MagicMock

# Daemon imports optional WS deps; stub for unit tests.
sys.modules.setdefault("websockets", MagicMock())

from scripts.run_rp608_nse_shadow_daemon import (  # noqa: E402
    BarOHLCV,
    RP608NSEShadowDaemon,
)


IST = timezone(timedelta(hours=5, minutes=30))


class TestMissingCandleFetch(unittest.TestCase):
    def _daemon(self) -> RP608NSEShadowDaemon:
        d = RP608NSEShadowDaemon.__new__(RP608NSEShadowDaemon)
        d.price_history = {}
        d.symbols = ["AAA.NS"]
        d._symbol_last_bootstrap_bar = {}
        d._symbol_bootstrap_fail_mono = {}
        d.exchange_holidays = set()
        return d

    def test_expected_starts_count(self):
        d = self._daemon()
        open_ist = datetime(2026, 11, 20, 9, 15, tzinfo=IST).replace(tzinfo=None)
        ist_offset = timedelta(hours=5, minutes=30)
        starts = d._expected_session_candle_starts_ms(
            open_time_ist=open_ist, completed_bars=3, ist_offset=ist_offset
        )
        self.assertEqual(len(starts), 3)
        # 15m apart
        self.assertEqual(starts[1] - starts[0], 15 * 60 * 1000)

    def test_missing_mid_candle_detected(self):
        d = self._daemon()
        open_ist = datetime(2026, 11, 20, 9, 15, tzinfo=IST).replace(tzinfo=None)
        ist_offset = timedelta(hours=5, minutes=30)
        sess_date = open_ist.date()
        starts = d._expected_session_candle_starts_ms(
            open_time_ist=open_ist, completed_bars=3, ist_offset=ist_offset
        )
        # Only bar 1 and 3 present — missing bar 2
        d.price_history["AAA.NS"] = [
            BarOHLCV(
                timestamp_utc_ms=starts[0],
                open=1,
                high=1,
                low=1,
                close=1,
                volume=1,
                is_completed=True,
                source="kite_live",
            ),
            BarOHLCV(
                timestamp_utc_ms=starts[2],
                open=1,
                high=1,
                low=1,
                close=1,
                volume=1,
                is_completed=True,
                source="kite_live",
            ),
        ]
        missing = d._missing_session_candle_starts(
            "AAA.NS",
            open_time_ist=open_ist,
            completed_bars=3,
            session_date=sess_date,
            ist_offset=ist_offset,
        )
        self.assertEqual(missing, [starts[1]])

    def test_bootstrap_fetches_when_gap(self):
        d = self._daemon()
        open_ist = datetime(2026, 11, 20, 9, 15, tzinfo=IST).replace(tzinfo=None)
        ist_offset = timedelta(hours=5, minutes=30)
        starts = d._expected_session_candle_starts_ms(
            open_time_ist=open_ist, completed_bars=3, ist_offset=ist_offset
        )
        d.price_history["AAA.NS"] = [
            BarOHLCV(
                timestamp_utc_ms=starts[0],
                open=10,
                high=11,
                low=9,
                close=10.5,
                volume=100,
                is_completed=True,
                source="kite_live",
            ),
            BarOHLCV(
                timestamp_utc_ms=starts[2],
                open=12,
                high=13,
                low=11,
                close=12.5,
                volume=100,
                is_completed=True,
                source="kite_live",
            ),
        ]
        # Historical returns the missing middle candle (+ the others).
        mid = BarOHLCV(
            timestamp_utc_ms=starts[1],
            open=11,
            high=12,
            low=10,
            close=11.5,
            volume=50,
            is_completed=True,
            source="kite_historical_same_day",
        )
        adapter = MagicMock()
        adapter.fetch_historical_15m_bars = MagicMock(
            return_value=[
                BarOHLCV(
                    timestamp_utc_ms=starts[0],
                    open=10,
                    high=11,
                    low=9,
                    close=10.5,
                    volume=100,
                    is_completed=True,
                    source="kite_historical_same_day",
                ),
                mid,
                BarOHLCV(
                    timestamp_utc_ms=starts[2],
                    open=12,
                    high=13,
                    low=11,
                    close=12.5,
                    volume=100,
                    is_completed=True,
                    source="kite_historical_same_day",
                ),
            ]
        )
        adapter.reload_credentials_if_stale = MagicMock()
        d.feed_adapter = adapter
        d._purge_quote_snapshot_spam = MagicMock(return_value=0)
        d._active_session_window = SimpleNamespace(
            open_ist=SimpleNamespace(hour=9, minute=15),
            warmup_completed_bars=1,
            kind="REGULAR",
            label="regular",
        )

        # 10:00 IST → 3 completed bars (09:15, 09:30, 09:45 starts; 10:00 forming)
        now_utc = datetime(2026, 11, 20, 4, 30, tzinfo=timezone.utc)  # 10:00 IST
        # Patch session window resolution used inside bootstrap
        import scripts.run_rp608_nse_shadow_daemon as mod

        win = SimpleNamespace(
            open_ist=SimpleNamespace(hour=9, minute=15),
            warmup_completed_bars=1,
            kind="REGULAR",
            label="regular",
        )
        orig_resolve = mod.resolve_session_window
        orig_completed = mod.completed_bars_since_open
        mod.resolve_session_window = lambda *a, **k: (win, None)
        mod.completed_bars_since_open = lambda *a, **k: 3
        try:
            out = d.bootstrap_same_day_history(now_utc=now_utc, target_symbols=["AAA.NS"])
        finally:
            mod.resolve_session_window = orig_resolve
            mod.completed_bars_since_open = orig_completed

        self.assertEqual(out.get("status"), "BOOTSTRAP_SUCCESS")
        self.assertGreaterEqual(int(out.get("total_backfilled_bars") or 0), 1)
        missing_after = d._missing_session_candle_starts(
            "AAA.NS",
            open_time_ist=open_ist,
            completed_bars=3,
            session_date=open_ist.date(),
            ist_offset=ist_offset,
        )
        self.assertEqual(missing_after, [])


if __name__ == "__main__":
    unittest.main()
