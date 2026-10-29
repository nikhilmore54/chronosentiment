#!/usr/bin/env python3
"""
RP-608-REGIME-BREAKOUT-001 — Arm A invariance + golden fixture reproducibility.

Proves enabling observational regime telemetry does not alter Arm A decisions,
entries, exits, holding periods, or costs (after stripping telemetry-only keys).
"""

from __future__ import annotations

import importlib
import json
import sys
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.generate_rp608_arm_a_golden_fixtures import build_crypto_fixtures, build_nse_fixtures
from scripts.market_regime_classifier import classify_market_regime
from scripts.rp608_arm_a_baseline import (
    ARM_A_FROZEN,
    ARM_A_LIVE_ORDER_AUTHORITY,
    NseBar,
    TELEMETRY_ONLY_KEYS,
    attach_regime_telemetry,
    crypto_arm_a_decide,
    crypto_arm_a_simulate_trade,
    fingerprint,
    nse_arm_a_decide_entry,
    nse_arm_a_simulate_trade,
    strip_telemetry_keys,
)


class TestArmAGoldenFixtures(unittest.TestCase):
    def test_nse_fixture_reproducible(self):
        fresh = build_nse_fixtures()
        path = WORKSPACE / "fixtures" / "rp608_regime_breakout" / "arm_a_nse_golden.json"
        stored = json.loads(path.read_text(encoding="utf-8"))
        self.assertEqual(fresh["fingerprint_sha256"], stored["fingerprint_sha256"])
        self.assertEqual(fingerprint(fresh["cases"]), stored["fingerprint_sha256"])
        self.assertTrue(stored["arm_a_frozen"])
        self.assertEqual(stored["live_order_authority"], 0.0)

    def test_crypto_fixture_reproducible(self):
        fresh = build_crypto_fixtures()
        path = WORKSPACE / "fixtures" / "rp608_regime_breakout" / "arm_a_crypto_golden.json"
        stored = json.loads(path.read_text(encoding="utf-8"))
        self.assertEqual(fresh["fingerprint_sha256"], stored["fingerprint_sha256"])
        self.assertEqual(fingerprint(fresh["cases"]), stored["fingerprint_sha256"])

    def test_fixture_cases_have_required_fields(self):
        nse = json.loads(
            (WORKSPACE / "fixtures/rp608_regime_breakout/arm_a_nse_golden.json").read_text()
        )
        trade_case = next(c for c in nse["cases"] if c["trade"] is not None)
        trade = trade_case["trade"]
        for key in (
            "decision_code",
            "entry_price",
            "exit_price",
            "exit_reason",
            "holding_bars",
            "gross_pnl_inr",
            "net_pnl_inr",
            "friction",
        ):
            if key == "decision_code":
                self.assertIn(key, trade_case["decision"])
            else:
                self.assertIn(key, trade)


class TestArmATelemetryInvariance(unittest.TestCase):
    def _synthetic_bars(self, n: int = 80) -> list:
        bars = []
        px = 100.0
        for i in range(n):
            # Mild directional drift so classifier can produce non-UNAVAILABLE
            px *= 1.002
            bars.append({"open": px * 0.999, "high": px * 1.001, "low": px * 0.998, "close": px})
        return bars

    def test_attach_telemetry_does_not_change_behavioral_fingerprint(self):
        closes = [1000.0 * (1.002 ** i) for i in range(12)]
        decision = nse_arm_a_decide_entry(closes, bar_idx=10)
        forward = [
            NseBar(i, closes[-1], closes[-1] * 1.01, closes[-1] * 0.99, closes[-1] * (1 + 0.001 * (i - 10)))
            for i in range(11, 20)
        ]
        trade = nse_arm_a_simulate_trade(10, closes[-1], forward)
        card = {"decision": decision, "trade": trade, "symbol": "TEST.NS"}

        regime = classify_market_regime(self._synthetic_bars())
        off = attach_regime_telemetry(card, None, enabled=False)
        on = attach_regime_telemetry(card, regime, enabled=True)

        self.assertIn("regime_state", on)
        self.assertTrue(on.get("regime_telemetry_enabled"))
        # fingerprint() strips telemetry — behavioral hashes must match
        self.assertEqual(fingerprint(off), fingerprint(on))
        self.assertEqual(
            fingerprint(strip_telemetry_keys(off)),
            fingerprint(strip_telemetry_keys(on)),
        )
        # Explicit behavioral fields equal
        self.assertEqual(off["decision"], on["decision"])
        self.assertEqual(off["trade"]["exit_reason"], on["trade"]["exit_reason"])
        self.assertEqual(off["trade"]["holding_bars"], on["trade"]["holding_bars"])
        self.assertEqual(off["trade"]["net_pnl_inr"], on["trade"]["net_pnl_inr"])
        self.assertEqual(off["trade"]["friction"], on["trade"]["friction"])

    def test_crypto_telemetry_invariance(self):
        decision = crypto_arm_a_decide("LONG", True)
        trade = crypto_arm_a_simulate_trade("LONG", 100000.0, [100100, 100200, 100400, 100600])
        card = {"decision": decision, "trade": trade}
        regime = classify_market_regime(self._synthetic_bars())
        off = attach_regime_telemetry(card, None, enabled=False)
        on = attach_regime_telemetry(card, regime, enabled=True)
        self.assertEqual(
            fingerprint(strip_telemetry_keys(off)),
            fingerprint(strip_telemetry_keys(on)),
        )

    def test_telemetry_only_keys_documented(self):
        self.assertIn("regime_state", TELEMETRY_ONLY_KEYS)
        self.assertIn("adx_14", TELEMETRY_ONLY_KEYS)

    def test_arm_a_freeze_constants(self):
        self.assertTrue(ARM_A_FROZEN)
        self.assertEqual(ARM_A_LIVE_ORDER_AUTHORITY, 0.0)

    def test_daemon_freeze_markers_present(self):
        nse = importlib.import_module("scripts.run_rp608_nse_shadow_daemon")
        crypto = importlib.import_module("scripts.run_rp608_shadow_daemon")
        self.assertTrue(nse.ARM_A_FROZEN)
        self.assertTrue(crypto.ARM_A_FROZEN)
        self.assertEqual(nse.LIVE_ORDER_AUTHORITY_PCT, 0.0)
        self.assertEqual(crypto.LIVE_ORDER_AUTHORITY_PCT, 0.0)
        self.assertTrue(nse.REGIME_TELEMETRY_ENABLED)
        self.assertTrue(crypto.REGIME_TELEMETRY_ENABLED)
        # No order placement API on NSE daemon class
        self.assertFalse(hasattr(nse, "place_order"))
        src = Path(nse.__file__).read_text(encoding="utf-8")
        self.assertNotIn("def place_order", src)
        self.assertIn("REGIME_TELEMETRY_ENABLED", src)


class TestHoldoutAndAuthorityInvariants(unittest.TestCase):
    def test_protocol_documents_sealed_holdout(self):
        proto = (WORKSPACE / "docs/research/rp608_regime_breakout_001_protocol.md").read_text()
        self.assertIn("41–68", proto)
        self.assertIn("0.0%", proto)
        self.assertIn("crossing before watch", proto.lower())

    def test_phase0_census_exists_and_does_not_touch_holdout(self):
        census = json.loads(
            (WORKSPACE / "artifacts/rp608_regime_breakout/phase0_is_census.json").read_text()
        )
        self.assertEqual(census["holdout_sessions_41_68"], "SEALED_UNTOUCHED")
        self.assertEqual(census["live_order_authority"], "0.0%")
        self.assertEqual(census["nse"]["holdout_status"], "SEALED_UNTOUCHED")
        self.assertEqual(census["crypto"]["holdout_status"], "SEALED_UNTOUCHED")
        # Floors not relaxed
        self.assertEqual(census["threshold_policy"], "Do not relax floors; short sample => INCONCLUSIVE")


if __name__ == "__main__":
    unittest.main()
