#!/usr/bin/env python3
"""Unit tests for NSE A entry+exit trade pattern library/matcher."""

from __future__ import annotations

import os
import unittest

from scripts.rp608_nse_trade_pattern_library import (
    build_entry_pattern_key,
    build_exit_pattern_key,
    classify_entry_label,
    extract_exit_path_feats,
    mine_library,
)
from scripts.rp608_nse_trade_pattern_match import (
    entry_blocks_prediction,
    entry_gate_enabled,
    entry_gate_mode,
    entry_prediction_from_label,
    features_from_coralys_and_candles,
    match_entry,
    match_exit,
    normalize_entry_gate_mode,
)


class TestKeyExcludesSymbol(unittest.TestCase):
    def test_entry_key_has_no_symbol(self):
        feats = {
            "session_phase": "EARLY",
            "traj15_bucket": "0_10_0_15",
            "preopen_status": "OK",
            "gap_bucket": "20_40",
            "jittery": False,
            "turbulent": False,
            "trend_aligned_long": True,
            "prior_support": True,
            "same_session_prior_stop_in_similar_context": False,
            "bar_tertile": "EARLY_BARS",
            "candle_path3": "UUD",
            "signal_body": "STRONG_U",
            "runup_3b": "UP_MED",
            "vol_surge": "MED",
            "symbol": "BANDHANBNK.NS",  # must not leak into key
        }
        key = build_entry_pattern_key(feats)
        self.assertNotIn("BANDHAN", key)
        self.assertNotIn("symbol", key.lower())
        self.assertIn("EARLY", key)
        self.assertIn("CUUD", key)

    def test_exit_key_has_no_symbol(self):
        key = build_exit_pattern_key(
            {
                "session_phase": "MID",
                "traj15_bucket": "0_15_0_30",
                "exit_path3": "UDD",
                "mfe_bucket": "MFE_40_80",
                "first_reverse": "REV_B2",
                "exit_vol_surge": "HIGH",
                "symbol": "RELIANCE.NS",
            }
        )
        self.assertNotIn("RELIANCE", key)
        self.assertIn("REV_B2", key)
        self.assertIn("VHIGH", key)


class TestAdverseVsAbstain(unittest.TestCase):
    def test_adverse_stands_aside_when_gate_on(self):
        lib = {
            "entry_patterns": {
                "P1": {
                    "pattern_id": "P1",
                    "label": "ADVERSE",
                    "train": {
                        "n": 40,
                        "wr": 0.25,
                        "payoff": 0.8,
                        "net_mean": -200.0,
                    },
                }
            }
        }
        # Force exact match by mocking lookup via replacing build — use real key
        feats = features_from_coralys_and_candles(
            input_ctx={
                "session_phase": "LATE",
                "preopen_status": "OK",
                "opening_gap_bps": 10.0,
                "preopen_jittery": False,
                "preopen_turbulent": False,
                "trend_aligned_long": True,
                "prior_support": False,
            },
            traj15=0.12,
            bar_idx=20,
            same_session_prior_stop=False,
        )
        exact = build_entry_pattern_key(feats)
        lib["entry_patterns"] = {
            exact: {
                "pattern_id": exact,
                "label": "ADVERSE",
                "train": {"n": 40, "wr": 0.25, "payoff": 0.8, "net_mean": -200.0},
            }
        }
        stamp = match_entry(library=lib, feats=feats, gate_on=True)
        self.assertEqual(stamp["prediction"], "STANDASIDE_ADVERSE_PATTERN")
        self.assertTrue(stamp["blocks_entry"])
        self.assertEqual(stamp["gate_mode"], "adverse")
        self.assertFalse(stamp["symbol_banned"])

    def test_insufficient_abstains_under_adverse(self):
        feats = features_from_coralys_and_candles(
            input_ctx={"session_phase": "MID", "preopen_status": "MISSING"},
            traj15=0.08,
            bar_idx=10,
        )
        stamp = match_entry(library={"entry_patterns": {}}, feats=feats, gate_on=True)
        self.assertEqual(stamp["prediction"], "ABSTAIN_INSUFFICIENT")
        self.assertFalse(stamp["blocks_entry"])
        self.assertEqual(stamp["gate_mode"], "adverse")

    def test_allow_only_blocks_abstain(self):
        feats = features_from_coralys_and_candles(
            input_ctx={"session_phase": "MID", "preopen_status": "MISSING"},
            traj15=0.08,
            bar_idx=10,
        )
        stamp = match_entry(
            library={"entry_patterns": {}}, feats=feats, gate_mode="allow_only"
        )
        self.assertEqual(stamp["prediction"], "ABSTAIN_INSUFFICIENT")
        self.assertTrue(stamp["blocks_entry"])
        self.assertEqual(stamp["gate_mode"], "allow_only")

    def test_allow_only_allows_neutral_positive_net(self):
        feats = features_from_coralys_and_candles(
            input_ctx={
                "session_phase": "LATE",
                "preopen_status": "OK",
                "opening_gap_bps": 10.0,
                "preopen_jittery": False,
                "preopen_turbulent": False,
                "trend_aligned_long": True,
                "prior_support": False,
            },
            traj15=0.12,
            bar_idx=20,
            same_session_prior_stop=False,
        )
        exact = build_entry_pattern_key(feats)
        lib = {
            "entry_patterns": {
                exact: {
                    "pattern_id": exact,
                    "label": "NEUTRAL",
                    "train": {
                        "n": 40,
                        "wr": 0.55,
                        "payoff": 1.2,
                        "net_mean": 50.0,
                    },
                }
            }
        }
        stamp = match_entry(library=lib, feats=feats, gate_mode="allow_only")
        self.assertEqual(stamp["prediction"], "ALLOW")
        self.assertFalse(stamp["blocks_entry"])

    def test_allow_only_blocks_neutral_nonpositive_net(self):
        feats = features_from_coralys_and_candles(
            input_ctx={
                "session_phase": "LATE",
                "preopen_status": "OK",
                "opening_gap_bps": 10.0,
                "preopen_jittery": False,
                "preopen_turbulent": False,
                "trend_aligned_long": True,
                "prior_support": False,
            },
            traj15=0.12,
            bar_idx=20,
            same_session_prior_stop=False,
        )
        exact = build_entry_pattern_key(feats)
        lib_neg = {
            "entry_patterns": {
                exact: {
                    "pattern_id": exact,
                    "label": "NEUTRAL",
                    "train": {
                        "n": 40,
                        "wr": 0.55,
                        "payoff": 1.1,
                        "net_mean": -80.0,
                    },
                }
            }
        }
        stamp = match_entry(library=lib_neg, feats=feats, gate_mode="allow_only")
        self.assertEqual(stamp["prediction"], "ABSTAIN_NEGATIVE_EXPECTANCY")
        self.assertTrue(stamp["blocks_entry"])
        lib0 = {
            "entry_patterns": {
                exact: {
                    "pattern_id": exact,
                    "label": "NEUTRAL",
                    "train": {
                        "n": 40,
                        "wr": 0.55,
                        "payoff": 1.1,
                        "net_mean": 0.0,
                    },
                }
            }
        }
        stamp0 = match_entry(library=lib0, feats=feats, gate_mode="allow_only")
        self.assertEqual(stamp0["prediction"], "ABSTAIN_NEGATIVE_EXPECTANCY")
        self.assertTrue(stamp0["blocks_entry"])

    def test_prediction_from_label(self):
        self.assertEqual(
            entry_prediction_from_label("ADVERSE", {"net_mean": -1}),
            "STANDASIDE_ADVERSE_PATTERN",
        )
        self.assertEqual(
            entry_prediction_from_label("NEUTRAL", {"net_mean": 1.0}),
            "ALLOW",
        )
        self.assertEqual(
            entry_prediction_from_label("NEUTRAL", {"net_mean": -1.0}),
            "ABSTAIN_NEGATIVE_EXPECTANCY",
        )

    def test_classify_entry_label(self):
        self.assertEqual(
            classify_entry_label({"n": 10, "wr": 0.1}),
            "INSUFFICIENT",
        )
        self.assertEqual(
            classify_entry_label({"n": 40, "wr": 0.30}),
            "ADVERSE",
        )
        self.assertEqual(
            classify_entry_label({"n": 40, "wr": 0.50}),
            "NEUTRAL",
        )


class TestExitVolSurge(unittest.TestCase):
    def test_exit_path_includes_vol_surge(self):
        bars = {
            i: {
                "open": 100.0,
                "high": 100.5,
                "low": 99.5,
                "close": 100.2,
                "volume": 1e5,
            }
            for i in range(1, 6)
        }
        # Path: green then strong down on high volume
        bars[5] = {"open": 100.0, "high": 102.0, "low": 99.8, "close": 101.5, "volume": 1e5}
        bars[6] = {"open": 101.5, "high": 101.6, "low": 98.0, "close": 98.5, "volume": 3e5}
        path = extract_exit_path_feats(bars, 5, 6, 100.0)
        self.assertEqual(path["exit_vol_surge"], "HIGH")
        self.assertIn("VHIGH", build_exit_pattern_key({
            "session_phase": "MID",
            "traj15_bucket": "0_10_0_15",
            "exit_path3": path["exit_path3"],
            "mfe_bucket": path["mfe_bucket"],
            "first_reverse": path["first_reverse"],
            "exit_vol_surge": path["exit_vol_surge"],
        }))


class TestExitReverse(unittest.TestCase):
    def test_exit_reverse_suggestion(self):
        bars = {
            5: {"open": 100, "high": 101, "low": 99.5, "close": 100.8, "volume": 1e5},
            6: {"open": 100.8, "high": 102, "low": 100.5, "close": 101.5, "volume": 1e5},
            7: {"open": 101.5, "high": 101.6, "low": 99.0, "close": 99.2, "volume": 1.2e5},
        }
        feats_exit = {
            "session_phase": "MID",
            "traj15_bucket": "0_10_0_15",
            "exit_path3": "UUD",
            "mfe_bucket": "MFE_15_40",
            "first_reverse": "REV_B3_4",
        }
        pid = build_exit_pattern_key(feats_exit)
        # Build library keyed to whatever match_exit will produce from bars
        stamp_probe = match_exit(
            library={"exit_patterns": {}},
            session_phase="MID",
            traj15=0.12,
            bars_dict=bars,
            entry_bar_idx=5,
            current_bar_idx=7,
            entry_price=100.0,
            gate_on=False,
        )
        pid = stamp_probe["pattern_id"]
        lib = {
            "exit_patterns": {
                pid: {
                    "pattern_id": pid,
                    "label": "REVERSE_STANDASIDE_HOLD",
                    "train": {"n": 30, "wr": 0.30, "payoff": 0.7, "net_mean": -150.0},
                }
            }
        }
        stamp = match_exit(
            library=lib,
            session_phase="MID",
            traj15=0.12,
            bars_dict=bars,
            entry_bar_idx=5,
            current_bar_idx=7,
            entry_price=100.0,
            gate_on=True,
        )
        self.assertEqual(stamp["suggestion"], "EXIT_REVERSE_PATTERN")
        self.assertTrue(stamp.get("apply_early_exit"))
        self.assertFalse(stamp["symbol_banned"])


class TestMineNoSymbolLeak(unittest.TestCase):
    def test_mine_library_keys(self):
        trades = []
        base_ms = 1_700_000_000_000
        for i in range(40):
            trades.append(
                {
                    "estimand": "ESTIMAND_A",
                    "symbol": f"SYM{i % 5}.NS",
                    "trade_id": f"t{i}",
                    "entry_bar_idx": 5 + (i % 3),
                    "exit_bar_idx": 8,
                    "entry_price": 100.0,
                    "entry_timestamp_utc_ms": base_ms + i * 86_400_000,
                    "exit_reason": "STOP_LOSS" if i % 3 else "TAKE_PROFIT",
                    "win_label": 0 if i % 3 else 1,
                    "net_pnl_inr": -200.0 if i % 3 else 300.0,
                    "gross_pnl_inr": -150.0 if i % 3 else 350.0,
                    "coralys_input_context": {
                        "session_phase": "EARLY",
                        "traj15": 0.12,
                        "preopen_status": "OK",
                        "opening_gap_bps": 25.0,
                        "preopen_jittery": False,
                        "preopen_turbulent": False,
                        "trend_aligned_long": True,
                        "prior_support": True,
                    },
                }
            )
        lib = mine_library(trades, symbol_bars={}, eval_idx={})
        for pid in lib["entry_patterns"]:
            self.assertNotIn(".NS", pid)
            self.assertNotIn("SYM", pid)


class TestGateEnv(unittest.TestCase):
    def test_gate_default_off(self):
        os.environ.pop("RP608_NSE_PATTERN_GATE", None)
        self.assertFalse(entry_gate_enabled())
        self.assertEqual(entry_gate_mode(), "off")

    def test_env_allow_only(self):
        prev = os.environ.get("RP608_NSE_PATTERN_GATE")
        try:
            os.environ["RP608_NSE_PATTERN_GATE"] = "allow_only"
            self.assertEqual(entry_gate_mode(), "allow_only")
            self.assertTrue(entry_gate_enabled())
        finally:
            if prev is None:
                os.environ.pop("RP608_NSE_PATTERN_GATE", None)
            else:
                os.environ["RP608_NSE_PATTERN_GATE"] = prev

    def test_normalize_and_blocks(self):
        self.assertEqual(normalize_entry_gate_mode("allow_only"), "allow_only")
        self.assertEqual(normalize_entry_gate_mode("2"), "allow_only")
        self.assertEqual(normalize_entry_gate_mode("1"), "adverse")
        self.assertTrue(entry_blocks_prediction("ABSTAIN_INSUFFICIENT", "allow_only"))
        self.assertTrue(
            entry_blocks_prediction("ABSTAIN_NEGATIVE_EXPECTANCY", "allow_only")
        )
        self.assertFalse(entry_blocks_prediction("ALLOW", "allow_only"))
        self.assertFalse(entry_blocks_prediction("ABSTAIN_INSUFFICIENT", "adverse"))
        self.assertTrue(
            entry_blocks_prediction("STANDASIDE_ADVERSE_PATTERN", "adverse")
        )


if __name__ == "__main__":
    unittest.main()
