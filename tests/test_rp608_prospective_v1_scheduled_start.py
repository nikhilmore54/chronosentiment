#!/usr/bin/env python3
"""Scheduled start: arm gate, windowing, no late start."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from datetime import datetime, timedelta, timezone
from pathlib import Path
from unittest import mock

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

import scripts.rp608_prospective_v1 as pv1
import scripts.run_rp608_prospective_v1_scheduled_start as sched


class TestScheduledWindow(unittest.TestCase):
    def test_in_window_and_too_early(self):
        # Crypto next midnight
        now = datetime(2026, 10, 28, 23, 0, tzinfo=timezone.utc)
        w = sched.evaluate_market_window(pv1.Market.CRYPTO, now=now, lead_minutes=90)
        self.assertTrue(w["in_prestart_window"])
        self.assertEqual(w["eligibility"]["eligible_session_boundary"], "2026-10-29")

        early = datetime(2026, 10, 28, 12, 0, tzinfo=timezone.utc)
        w2 = sched.evaluate_market_window(pv1.Market.CRYPTO, now=early, lead_minutes=90)
        self.assertTrue(w2["too_early"])
        self.assertFalse(w2["in_prestart_window"])

    def test_missed_day_rolls_forward_not_in_window(self):
        # At/after 00:00 UTC, that day is no longer eligible; next midnight is far → waiting
        now = datetime(2026, 10, 29, 0, 5, tzinfo=timezone.utc)
        w = sched.evaluate_market_window(pv1.Market.CRYPTO, now=now, lead_minutes=90)
        self.assertEqual(w["eligibility"]["eligible_session_boundary"], "2026-10-30")
        self.assertTrue(w["too_early"])
        self.assertFalse(w["in_prestart_window"])


class TestArmRequired(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.ns = self.root / "prospective_v1"
        self.ns.mkdir()
        self.arm = self.ns / "scheduled_start_arm.json"
        self.patches = [
            mock.patch.object(pv1, "PROSPECTIVE_NS", self.ns),
            mock.patch.object(sched, "PROSPECTIVE_NS", self.ns),
            mock.patch.object(sched, "ARM_PATH", self.arm),
            mock.patch.object(sched, "LOG_PATH", self.ns / "scheduled_start.jsonl"),
        ]
        for p in self.patches:
            p.start()

    def tearDown(self):
        for p in reversed(self.patches):
            p.stop()
        self.tmp.cleanup()

    def test_skips_when_not_armed(self):
        out = sched.run_once(dry_run=True)
        self.assertEqual(out["result"], "skipped_not_armed")


if __name__ == "__main__":
    unittest.main()
