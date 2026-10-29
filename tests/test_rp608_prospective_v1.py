#!/usr/bin/env python3
"""Prospective V1: auth gate, namespace isolation, session clock, DQ, extension."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.rp608_prospective_v1 import (
    ARTIFACT_ROOT,
    GATE4_NS,
    INTEGRITY_THROUGH,
    MAX_EXTENSIONS,
    PROSPECTIVE_NS,
    DataQualityGate,
    Decision,
    Market,
    MarketClock,
    Phase,
    assert_write_allowed,
    ensure_namespace,
    integrity_review_actions_forbidden,
    is_authorized,
    load_authorization,
    separate_market_eval_guard,
)


def _dq_ok() -> DataQualityGate:
    return DataQualityGate(
        completed_bars_only=True,
        quote_age_ms=50.0,
        half_spread_bps=1.0,
        half_spread_cap_bps=1.80,
        dedupe_key_present=True,
        journal_complete=True,
        packing_canonical=True,
    )


class TestAuthGate(unittest.TestCase):
    def test_authorization_does_not_imply_counting(self):
        auth = load_authorization()
        # May be authorized on disk after Prospective V1 auth; clock must stay distinct
        # from authorization. After a market commence_market(), observation_counting may
        # be STARTED while session_1_started / unit clock remain independent.
        if auth.get("authorized") is True:
            self.assertTrue(is_authorized())
            self.assertIn(auth.get("observation_counting"), ("NOT_STARTED", "STARTED"))
            # Unit clock / Session-1 marker is not implied by authorize alone.
            if auth.get("observation_counting") == "NOT_STARTED":
                self.assertFalse(auth.get("session_1_started") is True)
        else:
            self.assertFalse(is_authorized())

    def test_counting_blocked_when_unauthorized(self):
        clock = MarketClock(market=Market.NSE)
        res = clock.record_unit(
            "2026-11-01", authorized=False, dq=_dq_ok(), commencement_started=True
        )
        self.assertFalse(res["accepted"])
        self.assertEqual(res["reason"], "authorization_authorized_false")
        self.assertEqual(clock.completed_units, 0)

    def test_counting_blocked_without_commencement(self):
        clock = MarketClock(market=Market.NSE)
        res = clock.record_unit(
            "2026-11-01", authorized=True, dq=_dq_ok(), commencement_started=False
        )
        self.assertFalse(res["accepted"])
        self.assertEqual(res["reason"], "commencement_not_started")


class TestNamespaceIsolation(unittest.TestCase):
    def test_ensure_namespace(self):
        ns = ensure_namespace()
        self.assertTrue(ns.exists())
        self.assertTrue((ns / "README.md").exists())
        self.assertTrue((ns / ".namespace_isolation.json").exists())

    def test_refuse_gate4_write(self):
        ensure_namespace()
        with self.assertRaises(PermissionError):
            assert_write_allowed(GATE4_NS / "historical_abc_summary.json")
        with self.assertRaises(PermissionError):
            assert_write_allowed(
                ARTIFACT_ROOT / "arm_c_qualification_readonly_audit.json"
            )
        # prospective path allowed
        assert_write_allowed(PROSPECTIVE_NS / "session_journal.jsonl")


class TestSessionClock(unittest.TestCase):
    def test_phases_and_integrity_marker(self):
        clock = MarketClock(market=Market.NSE)
        for i in range(INTEGRITY_THROUGH):
            res = clock.record_unit(
                f"S{i+1}", authorized=True, dq=_dq_ok(), commencement_started=True
            )
            self.assertTrue(res["accepted"])
            self.assertFalse(res["tuning_allowed"])
        self.assertEqual(clock.phase(), Phase.INTEGRITY)
        self.assertTrue(clock.integrity_review_done)
        dup = clock.record_unit(
            "S1", authorized=True, dq=_dq_ok(), commencement_started=True
        )
        self.assertFalse(dup["accepted"])
        self.assertEqual(dup["reason"], "duplicate_unit")
        # continue through collection
        for i in range(INTEGRITY_THROUGH + 1, 11):
            clock.record_unit(
                f"S{i}", authorized=True, dq=_dq_ok(), commencement_started=True
            )
        self.assertEqual(clock.phase(), Phase.COLLECTION)
        for i in range(11, 20):
            clock.record_unit(
                f"S{i}", authorized=True, dq=_dq_ok(), commencement_started=True
            )
        self.assertEqual(clock.completed_units, 19)
        self.assertEqual(clock.phase(), Phase.COMPLETION)
        clock.record_unit(
            "S20", authorized=True, dq=_dq_ok(), commencement_started=True
        )
        self.assertEqual(clock.phase(), Phase.DECISION)

    def test_dq_failure_rejects_unit(self):
        clock = MarketClock(market=Market.CRYPTO)
        bad = _dq_ok()
        bad.packing_canonical = False
        res = clock.record_unit(
            "TB1", authorized=True, dq=bad, commencement_started=True
        )
        self.assertFalse(res["accepted"])
        self.assertIn("non_canonical_packing", res["failures"])

    def test_extension_rule(self):
        clock = MarketClock(market=Market.NSE, completed_units=20)
        bad = clock.authorize_extension(decision=Decision.FAIL)
        self.assertFalse(bad["accepted"])
        ok = clock.authorize_extension(decision=Decision.INCONCLUSIVE)
        self.assertTrue(ok["accepted"])
        self.assertFalse(ok["criteria_relaxed"])
        self.assertEqual(ok["new_window_target"], 30)
        clock.completed_units = 30
        clock.authorize_extension(decision=Decision.INCONCLUSIVE)
        clock.completed_units = 40
        done = clock.authorize_extension(decision=Decision.INCONCLUSIVE)
        self.assertFalse(done["accepted"])
        self.assertEqual(done["reason"], "max_extensions_reached")
        self.assertEqual(clock.extensions_used, MAX_EXTENSIONS)


class TestIntegrityAndSeparation(unittest.TestCase):
    def test_integrity_forbids_tuning(self):
        forbidden = set(integrity_review_actions_forbidden())
        self.assertIn("change_delta_buffer", forbidden)
        self.assertIn("retune_on_interim_pnl", forbidden)
        self.assertIn("pool_nse_crypto", forbidden)

    def test_markets_not_pooled(self):
        separate_market_eval_guard({"exp": 1}, {"exp": 2})
        with self.assertRaises(ValueError):
            separate_market_eval_guard({"pooled": True}, {"exp": 2})


class TestAuthorizeApi(unittest.TestCase):
    def test_authorize_requires_confirm(self):
        from scripts.rp608_prospective_v1 import authorize_prospective_v1

        with self.assertRaises(ValueError):
            authorize_prospective_v1(
                authorized_by="Anuj",
                intent="test",
                expected_deployed_rev="abc",
                confirm=False,
            )


if __name__ == "__main__":
    unittest.main()
