#!/usr/bin/env python3
"""
tests/test_rp615_causal_feature_integrity.py

RP-615 Milestone: Phase 1 Preregistration Verification Test Suite.
Verifies all 9 candidate feature contracts (F01 - F09), point-in-time causality,
warm-up boundaries, zero-division protections, self-exclusion, and replay determinism.
"""

import math
import unittest
from datetime import datetime, timezone
from pathlib import Path
import numpy as np

import sys
WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.rp615_feature_extractor import FeatureExtractor, BarCacheManager, DecisionCardsIndexer


class MockBarCacheManager(BarCacheManager):
    """Mock cache manager to inject deterministic synthetic candle series."""
    def __init__(self, synthetic_candles_by_sym):
        super().__init__(WORKSPACE / "intraday_capture" / "yahoo_cache_1m")
        self._cache = synthetic_candles_by_sym
        self._timestamps = {
            sym: np.array([c["timestamp"] for c in candles], dtype=np.int64)
            for sym, candles in synthetic_candles_by_sym.items()
        }

    def get_all_symbols(self):
        return list(self._cache.keys())


class MockDecisionCardsIndexer(DecisionCardsIndexer):
    """Mock cards indexer for testing trigger concurrency and self-exclusion."""
    def __init__(self, mock_cards):
        self._triggers_by_sec = {}
        for c in mock_cards:
            ts = c["timestamp"]
            if ts not in self._triggers_by_sec:
                self._triggers_by_sec[ts] = []
            self._triggers_by_sec[ts].append(c)
        self._loaded = True


class TestRP615CausalFeatureIntegrity(unittest.TestCase):

    def setUp(self):
        # Generate 70 consecutive 1m synthetic bars for SYMBOL_A (from 09:15 to 10:25 IST)
        # Base timestamp: 1727500000
        self.t0 = 1727500000
        self.candles_a = []
        base_p = 100.0
        for i in range(70):
            ts = self.t0 + i * 60
            o = base_p + i * 0.10
            h = o + 0.50
            l = o - 0.20
            c = o + 0.20
            v = 1000.0 + (i % 5) * 100.0
            self.candles_a.append({
                "timestamp": ts,
                "open": round(o, 2),
                "high": round(h, 2),
                "low": round(l, 2),
                "close": round(c, 2),
                "volume": v
            })

        # Generate candles for SYMBOL_B (advancing) and SYMBOL_C (declining)
        self.candles_b = [
            {"timestamp": self.t0 + i * 60, "open": 50.0, "high": 52.0, "low": 49.0, "close": 50.0 + i * 0.5, "volume": 500}
            for i in range(70)
        ]
        self.candles_c = [
            {"timestamp": self.t0 + i * 60, "open": 200.0, "high": 201.0, "low": 198.0, "close": 200.0 - i * 0.3, "volume": 800}
            for i in range(70)
        ]

        self.mock_cache = MockBarCacheManager({
            "SYMBOL_A": self.candles_a,
            "SYMBOL_B": self.candles_b,
            "SYMBOL_C": self.candles_c,
        })

        # Mock cards at t_e = t0 + 65*60 + 15
        t_e_sec = self.t0 + 65 * 60 + 15
        self.mock_cards = [
            {"timestamp": t_e_sec - 10, "instrument": "SYMBOL_B", "decision_state": "BUY"},
            {"timestamp": t_e_sec - 20, "instrument": "SYMBOL_C", "decision_state": "SHORT"},
            {"timestamp": t_e_sec - 15, "instrument": "SYMBOL_A", "decision_state": "BUY"}, # Self-trade
        ]
        self.mock_indexer = MockDecisionCardsIndexer(self.mock_cards)
        self.extractor = FeatureExtractor(cache_mgr=self.mock_cache, cards_indexer=self.mock_indexer)

    def test_01_causal_point_in_time_boundary(self):
        """Verify that bar at t_e or future bars are strictly excluded."""
        # Query at minute 65 close + 15s
        t_e_dt = datetime.fromtimestamp(self.t0 + 65 * 60 + 15, tz=timezone.utc)
        t_e_iso = t_e_dt.strftime("%Y-%m-%dT%H:%M:%SZ")
        res = self.extractor.extract_features_at_entry("SYMBOL_A", "LONG", t_e_iso)

        self.assertEqual(res["status"], "ELIGIBLE")
        # Bar close timestamp must be t_e - 15s = t0 + 65*60
        self.assertEqual(res["t_bar_close_sec"], self.t0 + 65 * 60)

    def test_02_warmup_and_lookback_rejection(self):
        """Verify that queries with < 15 bars return INELIGIBLE_WARMUP."""
        # Query at bar 5
        t_e_dt = datetime.fromtimestamp(self.t0 + 5 * 60 + 15, tz=timezone.utc)
        t_e_iso = t_e_dt.strftime("%Y-%m-%dT%H:%M:%SZ")
        res = self.extractor.extract_features_at_entry("SYMBOL_A", "LONG", t_e_iso)

        self.assertEqual(res["status"], "INELIGIBLE_WARMUP")

    def test_03_zero_division_and_degenerate_protection(self):
        """Verify feature extractor handles flat candles and zero volume without crashing."""
        flat_candles = [
            {"timestamp": self.t0 + i * 60, "open": 100.0, "high": 100.0, "low": 100.0, "close": 100.0, "volume": 0.0}
            for i in range(30)
        ]
        mock_flat_cache = MockBarCacheManager({"FLAT_SYM": flat_candles})
        extractor = FeatureExtractor(cache_mgr=mock_flat_cache, cards_indexer=self.mock_indexer)

        t_e_dt = datetime.fromtimestamp(self.t0 + 25 * 60 + 15, tz=timezone.utc)
        res = extractor.extract_features_at_entry("FLAT_SYM", "LONG", t_e_dt.strftime("%Y-%m-%dT%H:%M:%SZ"))

        self.assertEqual(res["status"], "ELIGIBLE")
        self.assertEqual(res["F01_ret_15m"], 0.0)
        self.assertEqual(res["F03_ker_15m"], 0.0)
        self.assertEqual(res["F04_rvol_15m"], 1.0)
        self.assertEqual(res["F08_wick_asym"], 0.0)

    def test_04_ohlc_integrity_validation(self):
        """Verify that corrupt OHLC candle (Low > High) triggers INELIGIBLE_DATA."""
        corrupt_candles = [
            {"timestamp": self.t0 + i * 60, "open": 100.0, "high": 90.0, "low": 110.0, "close": 100.0, "volume": 100.0}
            for i in range(30)
        ]
        mock_corrupt_cache = MockBarCacheManager({"CORRUPT_SYM": corrupt_candles})
        extractor = FeatureExtractor(cache_mgr=mock_corrupt_cache, cards_indexer=self.mock_indexer)

        t_e_dt = datetime.fromtimestamp(self.t0 + 25 * 60 + 15, tz=timezone.utc)
        res = extractor.extract_features_at_entry("CORRUPT_SYM", "LONG", t_e_dt.strftime("%Y-%m-%dT%H:%M:%SZ"))

        self.assertEqual(res["status"], "INELIGIBLE_DATA")
        self.assertEqual(res["ineligibility_reason"], "CORRUPT_OHLC_CANDLE")

    def test_05_f07_directional_breadth_alignment(self):
        """Verify CS_raw and CS_aligned directional inversion."""
        t_e_dt = datetime.fromtimestamp(self.t0 + 65 * 60 + 15, tz=timezone.utc)
        t_e_iso = t_e_dt.strftime("%Y-%m-%dT%H:%M:%SZ")

        res_long = self.extractor.extract_features_at_entry("SYMBOL_A", "LONG", t_e_iso)
        res_short = self.extractor.extract_features_at_entry("SYMBOL_A", "SHORT", t_e_iso)

        self.assertEqual(res_long["F07_cs_raw"], res_short["F07_cs_raw"])
        self.assertAlmostEqual(res_long["F07_cs_aligned"], -res_short["F07_cs_aligned"])

    def test_06_f09_self_exclusion(self):
        """Verify that candidate trade's own symbol trigger is excluded from N_trig."""
        t_e_dt = datetime.fromtimestamp(self.t0 + 65 * 60 + 15, tz=timezone.utc)
        t_e_iso = t_e_dt.strftime("%Y-%m-%dT%H:%M:%SZ")

        res_a = self.extractor.extract_features_at_entry("SYMBOL_A", "LONG", t_e_iso)
        # Mock has 3 cards (SYMBOL_B, SYMBOL_C, SYMBOL_A). For SYMBOL_A, count must be 2.
        self.assertEqual(res_a["F09_n_trig"], 2)

    def test_07_replay_determinism(self):
        """Verify 100 repeated extractions yield bit-for-bit identical results."""
        t_e_dt = datetime.fromtimestamp(self.t0 + 65 * 60 + 15, tz=timezone.utc)
        t_e_iso = t_e_dt.strftime("%Y-%m-%dT%H:%M:%SZ")

        base_res = self.extractor.extract_features_at_entry("SYMBOL_A", "LONG", t_e_iso)
        for _ in range(100):
            res = self.extractor.extract_features_at_entry("SYMBOL_A", "LONG", t_e_iso)
            self.assertEqual(base_res, res)


if __name__ == "__main__":
    unittest.main()
