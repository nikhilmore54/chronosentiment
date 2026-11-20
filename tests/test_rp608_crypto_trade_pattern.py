#!/usr/bin/env python3
"""Unit tests for crypto plan entry+exit trade patterns."""

from __future__ import annotations

import os
import unittest

from scripts.rp608_crypto_trade_pattern_library import (
    build_entry_pattern_key,
    build_exit_pattern_key,
    classify_entry_label,
    mine_library,
    vol_surge_bucket,
)
from scripts.rp608_crypto_trade_pattern_match import (
    entry_gate_enabled,
    features_from_card_or_plan,
    match_entry,
    match_exit,
)


class TestVolSurgeBucket(unittest.TestCase):
    def test_buckets(self):
        self.assertEqual(vol_surge_bucket(None), "UNK")
        self.assertEqual(vol_surge_bucket(0.5), "LOW")
        self.assertEqual(vol_surge_bucket(1.0), "MED")
        self.assertEqual(vol_surge_bucket(2.0), "HIGH")


class TestKeyExcludesSymbol(unittest.TestCase):
    def test_entry_key(self):
        feats = {
            "utc_session_phase": "US",
            "direction": "LONG",
            "of_supported": True,
            "imb_bucket": "GE_30",
            "atr_bucket": "ATR_0_3_0_6",
            "stop_dist_bucket": "80_150",
            "target_dist_bucket": "GE_150",
            "gross_rr_bucket": "RR_1_5_2",
            "prior_stop_same_utc_day_dir": False,
            "vol_surge": "HIGH",
            "symbol": "BTCUSDT",
        }
        key = build_entry_pattern_key(feats)
        self.assertNotIn("BTC", key.upper())
        self.assertNotIn("symbol", key.lower())
        self.assertIn("VHIGH", key)

    def test_exit_key(self):
        key = build_exit_pattern_key(
            {
                "utc_session_phase": "EU",
                "direction": "SHORT",
                "mfe_bucket": "MFE_40_80",
                "first_reverse": "REV_AFTER_GREEN",
                "hold_bucket": "H_10_30M",
                "exit_vol_surge": "MED",
                "symbol": "BTCUSDT",
            }
        )
        self.assertNotIn("BTC", key.upper())
        self.assertIn("VMED", key)


class TestAdverseVsAbstain(unittest.TestCase):
    def test_adverse_blocks_when_gated(self):
        feats = features_from_card_or_plan(
            card={
                "policy_b_action": "SHORT",
                "order_flow_supported": True,
                "order_flow_imbalance": -0.4,
                "atr_14": 400.0,
                "reference_price": 80000.0,
                "close_time_ms": 1_700_000_000_000,
                "stop_loss_price": 80400.0,
                "profit_target_price": 79400.0,
                "gross_rr_ratio": 1.5,
            }
        )
        exact = build_entry_pattern_key(feats)
        lib = {
            "entry_patterns": {
                exact: {
                    "pattern_id": exact,
                    "label": "ADVERSE",
                    "train": {"n": 50, "wr": 0.28, "payoff": 0.7, "net_mean": -3.0},
                }
            }
        }
        stamp = match_entry(library=lib, feats=feats, gate_on=True)
        self.assertEqual(stamp["prediction"], "STANDASIDE_ADVERSE_PATTERN")
        self.assertTrue(stamp["blocks_entry"])

    def test_insufficient_abstains(self):
        feats = features_from_card_or_plan(card={"policy_b_action": "LONG"})
        stamp = match_entry(library={"entry_patterns": {}}, feats=feats, gate_on=True)
        self.assertEqual(stamp["prediction"], "ABSTAIN_INSUFFICIENT")
        self.assertFalse(stamp["blocks_entry"])

    def test_classify(self):
        self.assertEqual(classify_entry_label({"n": 5, "wr": 0.1}), "INSUFFICIENT")
        self.assertEqual(classify_entry_label({"n": 40, "wr": 0.3}), "ADVERSE")


class TestExitReverse(unittest.TestCase):
    def test_exit_reverse(self):
        plan = {
            "direction": "LONG",
            "entry_price": 100.0,
            "entry_time_ms": 1_700_000_000_000,
            "mfe_exec_price": 100.5,
            "mae_exec_price": 99.5,
            "holding_duration_seconds": 900,
            "exit_reason": "STOP_LOSS_HIT",
        }
        # Build key that match_exit will use
        from scripts.rp608_crypto_trade_pattern_library import enrich_plan

        feats = enrich_plan(plan, prior_stop=False)
        pid = feats["exit_pattern_id"]
        lib = {
            "exit_patterns": {
                pid: {
                    "pattern_id": pid,
                    "label": "REVERSE_STANDASIDE_HOLD",
                    "train": {"n": 40, "wr": 0.2, "payoff": 0.5, "net_mean": -2.0},
                }
            }
        }
        stamp = match_exit(library=lib, plan=plan, gate_on=True)
        self.assertEqual(stamp["suggestion"], "EXIT_REVERSE_PATTERN")
        self.assertTrue(stamp["apply_early_exit"])


class TestMine(unittest.TestCase):
    def test_mine_no_symbol(self):
        plans = []
        base = 1_700_000_000_000
        for i in range(50):
            plans.append(
                {
                    "plan_id": f"P{i}",
                    "symbol": "BTCUSDT",
                    "direction": "LONG" if i % 2 == 0 else "SHORT",
                    "entry_time_ms": base + i * 86_400_000,
                    "entry_price": 80000.0,
                    "atr_14": 400.0,
                    "stop_loss_price": 79600.0 if i % 2 == 0 else 80400.0,
                    "profit_target_price": 80600.0 if i % 2 == 0 else 79400.0,
                    "gross_rr_ratio": 1.5,
                    "order_flow_supported": True,
                    "order_flow_imbalance": 0.35 if i % 2 == 0 else -0.35,
                    "exit_reason": "STOP_LOSS_HIT" if i % 3 else "PROFIT_TARGET_HIT",
                    "net_pnl_usdt": -5.0 if i % 3 else 4.0,
                    "gross_pnl_usdt": -4.0 if i % 3 else 5.0,
                    "holding_duration_seconds": 1200,
                    "mfe_exec_price": 80100.0 if i % 2 == 0 else 79900.0,
                }
            )
        lib = mine_library(plans)
        for pid in lib["entry_patterns"]:
            self.assertNotIn("BTC", pid.upper())


class TestGateEnv(unittest.TestCase):
    def test_default_off(self):
        os.environ.pop("RP608_CRYPTO_PATTERN_GATE", None)
        self.assertFalse(entry_gate_enabled())


if __name__ == "__main__":
    unittest.main()
