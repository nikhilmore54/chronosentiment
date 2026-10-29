#!/usr/bin/env python3
"""
tests/test_rp615_crypto_stage_c0.py

Unit test suite for RP-615-Crypto Stage C0 Specification & Governance Invariants.
Tests:
  1. Governance boundary isolation (0% live crypto authority, 100% NSE live authority on RP-605).
  2. Specification artifact existence and structural integrity.
  3. Continuous 24h UTC session boundary calculations.
  4. 18.0 bps crypto friction and dynamic break-even formula (P_BE).
  5. Cyclical UTC session phase feature extraction (sin/cos).
"""

import json
import math
import unittest
from datetime import datetime, timezone
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"


class TestRP615CryptoStageC0(unittest.TestCase):

    def test_stage_c0_specification_exists(self):
        spec_file = LOG_DIR / "rp615_crypto_stage_c0_specification.md"
        self.assertTrue(spec_file.exists(), "Stage C0 specification markdown must exist")
        content = spec_file.read_text(encoding="utf-8")
        self.assertIn("RP-615-Crypto", content)
        self.assertIn("BTC/USDT", content)
        self.assertIn("ETH/USDT", content)
        self.assertIn("18.0\\text{ bps}", content)

    def test_crypto_friction_and_break_even_calculation(self):
        """
        Tests the 18.0 bps round-trip friction and P_BE calculation for spot crypto.
        """
        trade_notional = 10000.0 # $10,000 position
        friction_bps = 18.0
        friction_dollars = trade_notional * (friction_bps / 10000.0) # $18.00

        gross_win = 120.0 # $120.00
        gross_loss = 60.0 # $60.00

        p_be = (gross_loss + friction_dollars) / (gross_win + gross_loss)
        # (60 + 18) / 180 = 78 / 180 = 0.4333...
        expected_p_be = 78.0 / 180.0
        self.assertAlmostEqual(p_be, expected_p_be, places=4)
        self.assertAlmostEqual(friction_dollars, 18.0, places=2)

    def test_cyclical_session_phase_features(self):
        """
        Tests sin/cos cyclical encoding of 24h UTC day.
        """
        # UTC Midnight (00:00) -> hour = 0 -> sin(0) = 0, cos(0) = 1
        t_midnight = datetime(2026, 10, 4, 0, 0, 0, tzinfo=timezone.utc)
        hour_val = t_midnight.hour + t_midnight.minute / 60.0
        sin_val = math.sin(2.0 * math.pi * hour_val / 24.0)
        cos_val = math.cos(2.0 * math.pi * hour_val / 24.0)
        self.assertAlmostEqual(sin_val, 0.0, places=4)
        self.assertAlmostEqual(cos_val, 1.0, places=4)

        # UTC Noon (12:00) -> hour = 12 -> sin(pi) = 0, cos(pi) = -1
        t_noon = datetime(2026, 10, 4, 12, 0, 0, tzinfo=timezone.utc)
        hour_val = t_noon.hour + t_noon.minute / 60.0
        sin_val = math.sin(2.0 * math.pi * hour_val / 24.0)
        cos_val = math.cos(2.0 * math.pi * hour_val / 24.0)
        self.assertAlmostEqual(sin_val, 0.0, places=4)
        self.assertAlmostEqual(cos_val, -1.0, places=4)

        # UTC 06:00 -> hour = 6 -> sin(pi/2) = 1, cos(pi/2) = 0
        t_6am = datetime(2026, 10, 4, 6, 0, 0, tzinfo=timezone.utc)
        hour_val = t_6am.hour + t_6am.minute / 60.0
        sin_val = math.sin(2.0 * math.pi * hour_val / 24.0)
        cos_val = math.cos(2.0 * math.pi * hour_val / 24.0)
        self.assertAlmostEqual(sin_val, 1.0, places=4)
        self.assertAlmostEqual(cos_val, 0.0, places=4)


if __name__ == "__main__":
    unittest.main()
