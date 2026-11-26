#!/usr/bin/env python3
"""Start-rule v1.0: eligibility, gates, idempotency, no auto-start."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from datetime import date, datetime, timezone
from pathlib import Path
from unittest import mock

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

import scripts.rp608_prospective_v1 as pv1
from scripts.rp608_prospective_v1 import (
    CommencementState,
    Market,
    MarketClock,
    START_RULE_VERSION,
    commence_market,
    compute_crypto_eligible_boundary,
    compute_nse_eligible_boundary,
    evaluate_start_eligibility,
    is_market_started,
    load_start_event,
    run_prestart_gates,
)


class TestNoAutoStart(unittest.TestCase):
    def test_auth_and_preflight_do_not_start(self):
        # Without a persisted start event, market commencement is NOT_STARTED
        # (repo may already be authorized — that alone must not count).
        if load_start_event(Market.NSE) is None:
            self.assertFalse(is_market_started(Market.NSE))
        clock = MarketClock(market=Market.NSE)
        res = clock.record_unit(
            "S1",
            authorized=True,
            dq=pv1.DataQualityGate(
                completed_bars_only=True,
                quote_age_ms=10,
                half_spread_bps=1,
                half_spread_cap_bps=2,
                dedupe_key_present=True,
                journal_complete=True,
                packing_canonical=True,
            ),
            commencement_started=False,
        )
        self.assertFalse(res["accepted"])
        self.assertEqual(res["reason"], "commencement_not_started")


class TestNseBoundaries(unittest.TestCase):
    def test_before_open_same_day(self):
        # 2026-10-29 Thursday 02:00 UTC = 07:30 IST — before 09:15
        decision = datetime(2026, 10, 29, 2, 0, tzinfo=timezone.utc)
        el = compute_nse_eligible_boundary(decision)
        self.assertEqual(el["eligible_session_boundary"], "2026-10-29")

    def test_mid_session_rejects_current(self):
        # 2026-10-29 05:00 UTC = 10:30 IST — in session
        decision = datetime(2026, 10, 29, 5, 0, tzinfo=timezone.utc)
        el = compute_nse_eligible_boundary(decision)
        self.assertEqual(el["eligible_session_boundary"], "2026-10-30")

    def test_weekend_skips_to_monday(self):
        # Saturday 2026-10-31 10:00 UTC
        decision = datetime(2026, 10, 31, 10, 0, tzinfo=timezone.utc)
        el = compute_nse_eligible_boundary(decision)
        self.assertEqual(el["eligible_session_boundary"], "2026-11-02")  # Monday

    def test_after_close_next_day(self):
        # Thursday after close: 12:00 UTC = 17:30 IST
        decision = datetime(2026, 10, 29, 12, 0, tzinfo=timezone.utc)
        el = compute_nse_eligible_boundary(decision)
        self.assertEqual(el["eligible_session_boundary"], "2026-10-30")


class TestCryptoBoundaries(unittest.TestCase):
    def test_mid_utc_day_next_midnight(self):
        decision = datetime(2026, 10, 28, 14, 0, tzinfo=timezone.utc)
        el = compute_crypto_eligible_boundary(decision)
        self.assertEqual(el["eligible_session_boundary"], "2026-10-29")
        self.assertTrue(el["eligible_session_open_utc"].startswith("2026-10-29T00:00:00"))

    def test_exact_midnight_strictly_after(self):
        decision = datetime(2026, 10, 29, 0, 0, 0, tzinfo=timezone.utc)
        el = compute_crypto_eligible_boundary(decision)
        self.assertEqual(el["eligible_session_boundary"], "2026-10-30")


class TestMarketsIndependent(unittest.TestCase):
    def test_nse_crypto_boundaries_independent(self):
        # Saturday: NSE jumps to Monday; crypto takes next UTC day (Sunday)
        decision = datetime(2026, 10, 31, 14, 0, tzinfo=timezone.utc)
        nse = evaluate_start_eligibility(Market.NSE, decision_utc=decision)
        crypto = evaluate_start_eligibility(Market.CRYPTO, decision_utc=decision)
        self.assertEqual(nse["eligible_session_boundary"], "2026-11-02")
        self.assertEqual(crypto["eligible_session_boundary"], "2026-11-01")
        self.assertNotEqual(nse["eligible_session_boundary"], crypto["eligible_session_boundary"])


class TestCommenceIsolated(unittest.TestCase):
    def setUp(self):
        self._tmpdir = tempfile.TemporaryDirectory()
        self.root = Path(self._tmpdir.name)
        self.ns = self.root / "prospective_v1"
        self.ns.mkdir()
        (self.ns / "DEPLOYED_REV.txt").write_text("680e18640aba2e30d06c3301005bce9c6e32cbd7\n")
        self.auth = self.root / "auth.json"
        self.auth.write_text(json.dumps({
            "authorized": True,
            "observation_counting": "NOT_STARTED",
            "deployed_revision": "680e18640aba2e30d06c3301005bce9c6e32cbd7",
            "git_rev": "680e18640aba2e30d06c3301005bce9c6e32cbd7",
            "controls": {
                "arm_a": "FROZEN",
                "holdout_sessions_41_68": "SEALED",
                "live_order_authority_pct": 0.0,
            },
        }))
        self.preflight = self.root / "prospective_v1_preflight.json"
        self.preflight.write_text(json.dumps({
            "preflight_pass": True,
            "controls": {"live_order_authority_pct": 0.0},
            "checks": {
                "lifecycle_fixtures": {"passed": True},
                "arm_a_invariance": {"passed": True},
            },
        }))
        self.patches = [
            mock.patch.object(pv1, "PROSPECTIVE_NS", self.ns),
            mock.patch.object(pv1, "AUTH_PATH", self.auth),
            mock.patch.object(pv1, "ARTIFACT_ROOT", self.root),
            mock.patch.object(pv1, "GATE4_NS", self.root / "historical_abc_research"),
            mock.patch.object(
                pv1,
                "PROTECTED_PATHS",
                (self.root / "historical_abc_research", self.root / "legacy_audit.json"),
            ),
        ]
        (self.root / "historical_abc_research").mkdir()
        (self.root / "historical_abc_research" / "historical_abc_summary.json").write_text("{}")
        for p in self.patches:
            p.start()

    def tearDown(self):
        for p in reversed(self.patches):
            p.stop()
        self._tmpdir.cleanup()

    def test_gate_failure_blocks(self):
        self.preflight.write_text(json.dumps({"preflight_pass": False, "checks": {}}))
        decision = datetime(2026, 10, 28, 1, 0, tzinfo=timezone.utc)  # before NSE open next day path
        res = commence_market(
            Market.NSE,
            operator="Anuj",
            confirm=True,
            decision_utc=decision,
            require_preflight_pass=True,
        )
        self.assertFalse(res["accepted"])
        self.assertEqual(res["state"], CommencementState.BLOCKED.value)
        self.assertFalse(is_market_started(Market.NSE))

    def test_confirm_required(self):
        res = commence_market(Market.CRYPTO, operator="Anuj", confirm=False)
        self.assertFalse(res["accepted"])
        self.assertEqual(res["reason"], "confirm_required")

    def test_successful_start_idempotent(self):
        # Friday 2026-10-30 before open IST: 02:00 UTC = 07:30 IST
        decision = datetime(2026, 10, 30, 2, 0, tzinfo=timezone.utc)
        res = commence_market(
            Market.NSE,
            operator="Anuj",
            confirm=True,
            decision_utc=decision,
        )
        self.assertTrue(res["accepted"], res)
        self.assertEqual(res["start_event"]["start_rule_version"], START_RULE_VERSION)
        self.assertEqual(res["start_event"]["eligible_session_boundary"], "2026-10-30")
        self.assertTrue(is_market_started(Market.NSE))
        # Idempotent repeat
        res2 = commence_market(
            Market.NSE,
            operator="Anuj",
            confirm=True,
            decision_utc=decision,
        )
        self.assertTrue(res2["accepted"])
        self.assertTrue(res2.get("idempotent"))
        self.assertEqual(
            res2["start_event"]["eligible_session_boundary"],
            res["start_event"]["eligible_session_boundary"],
        )
        # Crypto still independent / not started
        self.assertFalse(is_market_started(Market.CRYPTO))

    def test_live_authority_nonzero_blocks(self):
        auth = json.loads(self.auth.read_text())
        auth["controls"]["live_order_authority_pct"] = 1.0
        self.auth.write_text(json.dumps(auth))
        gates = run_prestart_gates(market=Market.CRYPTO, require_preflight_pass=True)
        self.assertFalse(gates.ok)
        self.assertIn("live_authority_nonzero", gates.failures)
        res = commence_market(
            Market.CRYPTO,
            operator="Anuj",
            confirm=True,
            decision_utc=datetime(2026, 10, 28, 12, 0, tzinfo=timezone.utc),
        )
        self.assertFalse(res["accepted"])
        self.assertEqual(res["state"], CommencementState.BLOCKED.value)


if __name__ == "__main__":
    unittest.main()
