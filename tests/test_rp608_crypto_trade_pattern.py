#!/usr/bin/env python3
"""Unit tests for crypto plan entry+exit trade patterns (Gate 2E allow_only / BACKOFF)."""

from __future__ import annotations

import os
import unittest

from scripts.rp608_crypto_trade_pattern_library import (
    build_entry_pattern_key,
    build_exit_pattern_key,
    classify_entry_label,
    enrich_plan,
    mine_library,
    vol_surge_bucket,
)
from scripts.rp608_crypto_trade_pattern_match import (
    SHADOW_ALLOW_IF_N10_WR_GT_035,
    build_gate2e_shadow_allow_stamp,
    entry_blocks_prediction,
    entry_gate_enabled,
    entry_gate_mode,
    features_from_card_or_plan,
    gate2e_shadow_thin_wr_enabled,
    match_entry,
    match_exit,
    matches_shadow_allow_if_n10_wr_gt_035,
    normalize_entry_gate_mode,
    parse_entry_pattern_key,
)


def _short_feats(**overrides):
    base = {
        "utc_session_phase": "US",
        "direction": "SHORT",
        "of_supported": True,
        "imb_bucket": "LE_M30",
        "atr_bucket": "ATR_0_3_0_6",
        "stop_dist_bucket": "LT_40",
        "target_dist_bucket": "40_80",
        "gross_rr_bucket": "RR_1_5_2",
        "prior_stop_same_utc_day_dir": False,
        "vol_surge": "MED",
    }
    base.update(overrides)
    return base


def _cell(feats, *, label: str, n: int, wr: float = 0.2):
    key = build_entry_pattern_key(feats)
    return key, {
        "pattern_id": key,
        "label": label,
        "train": {"n": n, "wr": wr, "payoff": 0.6, "net_mean": -2.0},
    }


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

    def test_parse_entry_key_roundtrip(self):
        feats = _short_feats()
        key = build_entry_pattern_key(feats)
        parsed = parse_entry_pattern_key(key)
        self.assertIsNotNone(parsed)
        assert parsed is not None
        self.assertEqual(parsed["utc_session_phase"], "US")
        self.assertEqual(parsed["direction"], "SHORT")
        self.assertTrue(parsed["of_supported"])
        self.assertEqual(parsed["imb_bucket"], "LE_M30")
        self.assertEqual(parsed["vol_surge"], "MED")
        self.assertFalse(parsed["prior_stop_same_utc_day_dir"])

    def test_parse_entry_key_rejects_short(self):
        self.assertIsNone(parse_entry_pattern_key("US|SHORT|OFT"))


class TestClassifyEntryLabel(unittest.TestCase):
    def test_insufficient_small_n(self):
        self.assertEqual(classify_entry_label({"n": 5, "wr": 0.1}), "INSUFFICIENT")

    def test_adverse(self):
        self.assertEqual(classify_entry_label({"n": 40, "wr": 0.3}), "ADVERSE")

    def test_neutral(self):
        self.assertEqual(classify_entry_label({"n": 40, "wr": 0.5}), "NEUTRAL")


class TestMatchEntryGates(unittest.TestCase):
    def test_exact_adverse_blocks_when_adverse_gate(self):
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
        stamp = match_entry(library=lib, feats=feats, gate_mode="adverse")
        self.assertEqual(stamp["match_mode"], "EXACT")
        self.assertEqual(stamp["prediction"], "STANDASIDE_ADVERSE_PATTERN")
        self.assertTrue(stamp["blocks_entry"])

    def test_adverse_gate_does_not_block_abstain(self):
        feats = features_from_card_or_plan(card={"policy_b_action": "LONG"})
        stamp = match_entry(library={"entry_patterns": {}}, feats=feats, gate_mode="adverse")
        self.assertEqual(stamp["prediction"], "ABSTAIN_INSUFFICIENT")
        self.assertFalse(stamp["blocks_entry"])

    def test_gate_on_true_maps_to_adverse(self):
        feats = features_from_card_or_plan(card={"policy_b_action": "LONG"})
        stamp = match_entry(library={"entry_patterns": {}}, feats=feats, gate_on=True)
        self.assertEqual(stamp["gate_mode"], "adverse")
        self.assertFalse(stamp["blocks_entry"])

    def test_gate_on_false_maps_to_off(self):
        live = _short_feats()
        key, rec = _cell(live, label="ADVERSE", n=40)
        stamp = match_entry(
            library={"entry_patterns": {key: rec}}, feats=live, gate_on=False
        )
        self.assertEqual(stamp["prediction"], "STANDASIDE_ADVERSE_PATTERN")
        self.assertFalse(stamp["blocks_entry"])
        self.assertEqual(stamp["gate_mode"], "off")

    def test_allow_only_blocks_abstain(self):
        feats = features_from_card_or_plan(card={"policy_b_action": "LONG"})
        stamp = match_entry(
            library={"entry_patterns": {}}, feats=feats, gate_mode="allow_only"
        )
        self.assertEqual(stamp["prediction"], "ABSTAIN_INSUFFICIENT")
        self.assertTrue(stamp["blocks_entry"])
        self.assertEqual(stamp["gate_mode"], "allow_only")

    def test_allow_only_blocks_standaside(self):
        live = _short_feats()
        key, rec = _cell(live, label="ADVERSE", n=40)
        stamp = match_entry(
            library={"entry_patterns": {key: rec}}, feats=live, gate_mode="allow_only"
        )
        self.assertEqual(stamp["prediction"], "STANDASIDE_ADVERSE_PATTERN")
        self.assertTrue(stamp["blocks_entry"])

    def test_allow_only_allows_neutral(self):
        live = _short_feats()
        key, rec = _cell(live, label="NEUTRAL", n=40, wr=0.55)
        stamp = match_entry(
            library={"entry_patterns": {key: rec}}, feats=live, gate_mode="allow_only"
        )
        self.assertEqual(stamp["prediction"], "ALLOW")
        self.assertFalse(stamp["blocks_entry"])

    def test_off_never_blocks_adverse(self):
        live = _short_feats()
        key, rec = _cell(live, label="ADVERSE", n=40)
        stamp = match_entry(
            library={"entry_patterns": {key: rec}}, feats=live, gate_mode="off"
        )
        self.assertEqual(stamp["prediction"], "STANDASIDE_ADVERSE_PATTERN")
        self.assertFalse(stamp["blocks_entry"])


class TestGate2EShadowThinWR(unittest.TestCase):
    """Observational SHADOW_ALLOW_IF_N10_WR_GT_035 — stamp only, no opens."""

    def test_env_default_on(self):
        prev = os.environ.pop("RP608_CRYPTO_GATE2E_SHADOW_THIN_WR", None)
        try:
            self.assertTrue(gate2e_shadow_thin_wr_enabled())
            self.assertFalse(gate2e_shadow_thin_wr_enabled("0"))
            self.assertTrue(gate2e_shadow_thin_wr_enabled("1"))
        finally:
            if prev is None:
                os.environ.pop("RP608_CRYPTO_GATE2E_SHADOW_THIN_WR", None)
            else:
                os.environ["RP608_CRYPTO_GATE2E_SHADOW_THIN_WR"] = prev

    def test_matches_thin_high_wr_abstain(self):
        self.assertTrue(
            matches_shadow_allow_if_n10_wr_gt_035(
                {
                    "prediction": "ABSTAIN_INSUFFICIENT",
                    "label": "INSUFFICIENT",
                    "cohort_n": 15,
                    "hist_wr": 0.42,
                }
            )
        )

    def test_rejects_n_below_10(self):
        self.assertFalse(
            matches_shadow_allow_if_n10_wr_gt_035(
                {
                    "prediction": "ABSTAIN_INSUFFICIENT",
                    "cohort_n": 9,
                    "hist_wr": 0.50,
                }
            )
        )

    def test_rejects_wr_at_or_below_035(self):
        self.assertFalse(
            matches_shadow_allow_if_n10_wr_gt_035(
                {
                    "prediction": "ABSTAIN_INSUFFICIENT",
                    "cohort_n": 20,
                    "hist_wr": 0.35,
                }
            )
        )

    def test_rejects_allow_prediction(self):
        self.assertFalse(
            matches_shadow_allow_if_n10_wr_gt_035(
                {
                    "prediction": "ALLOW",
                    "label": "NEUTRAL",
                    "cohort_n": 40,
                    "hist_wr": 0.55,
                }
            )
        )

    def test_build_stamp_schema(self):
        stamp = build_gate2e_shadow_allow_stamp(
            pattern_stamp={
                "pattern_id": "US|SHORT|…|VUNK",
                "feature_pattern_id": "US|SHORT|…|VHIGH",
                "prediction": "ABSTAIN_INSUFFICIENT",
                "hist_wr": 0.41,
                "cohort_n": 12,
                "match_mode": "BACKOFF",
                "gate_mode": "allow_only",
                "label": "INSUFFICIENT",
                "live_vol_surge": "HIGH",
                "live_vol_burst_20": 1.9,
            },
            plan_id="PLAN_1",
            direction="SHORT",
            suppressed_ms=1_700_000_000_000,
            book_id="CRYPTO_PLAN",
            plan={"entry_price": 100.0, "vol_burst_20": 1.9, "vol_surge": "HIGH"},
        )
        self.assertEqual(stamp["shadow_rule_id"], SHADOW_ALLOW_IF_N10_WR_GT_035)
        self.assertEqual(stamp["plan_id"], "PLAN_1")
        self.assertEqual(stamp["book_id"], "CRYPTO_PLAN")
        self.assertEqual(stamp["cohort_n"], 12)
        self.assertEqual(stamp["hist_wr"], 0.41)
        self.assertFalse(stamp["opens_trade"])
        self.assertEqual(stamp["authority_pct"], 0.0)
        self.assertEqual(stamp["live_vol_surge"], "HIGH")
        self.assertEqual(stamp["vol_axis"], "VHIGH")
        self.assertEqual(stamp["entry_price"], 100.0)
        self.assertEqual(stamp["feature_pattern_id"], "US|SHORT|…|VHIGH")

    def test_resolve_vol_ignores_unk_placeholder_when_burst_present(self):
        from scripts.rp608_crypto_trade_pattern_match import _resolve_vol_surge

        self.assertEqual(
            _resolve_vol_surge(
                plan={"vol_surge": "UNK"},
                card={"vol_burst_20": 1.8},
            ),
            "HIGH",
        )


class TestParentCellBackoff(unittest.TestCase):
    def test_parent_backoff_hits_adverse_after_vol_drop(self):
        live = _short_feats(vol_surge="MED")
        cell = dict(live)
        cell["vol_surge"] = "HIGH"
        cell_key, rec = _cell(cell, label="ADVERSE", n=48)
        stamp = match_entry(
            library={"entry_patterns": {cell_key: rec}},
            feats=live,
            gate_mode="allow_only",
        )
        self.assertEqual(stamp["match_mode"], "BACKOFF")
        self.assertEqual(stamp["prediction"], "STANDASIDE_ADVERSE_PATTERN")
        self.assertGreaterEqual(stamp["cohort_n"], 10)
        self.assertTrue(stamp["blocks_entry"])
        self.assertIn("vol_surge", stamp["dropped_axes"])

    def test_parent_backoff_prefers_highest_n(self):
        live = _short_feats(vol_surge="MED")
        small = dict(live)
        small["vol_surge"] = "HIGH"
        large = dict(live)
        large["vol_surge"] = "LOW"
        k_small, r_small = _cell(small, label="ADVERSE", n=12, wr=0.1)
        k_large, r_large = _cell(large, label="NEUTRAL", n=90, wr=0.55)
        stamp = match_entry(
            library={"entry_patterns": {k_small: r_small, k_large: r_large}},
            feats=live,
            gate_mode="off",
        )
        self.assertEqual(stamp["match_mode"], "BACKOFF")
        self.assertEqual(stamp["cohort_n"], 90)
        self.assertEqual(stamp["prediction"], "ALLOW")

    def test_parent_ignores_cells_below_min_n(self):
        live = _short_feats(vol_surge="MED")
        cell = dict(live)
        cell["vol_surge"] = "HIGH"
        key, rec = _cell(cell, label="ADVERSE", n=5)
        stamp = match_entry(
            library={"entry_patterns": {key: rec}}, feats=live, gate_mode="off"
        )
        self.assertEqual(stamp["match_mode"], "NONE")
        self.assertEqual(stamp["prediction"], "ABSTAIN_INSUFFICIENT")
        self.assertEqual(stamp["cohort_n"], 0)

    def test_session_mismatch_never_matches(self):
        live = _short_feats(utc_session_phase="ASIA")
        cell = _short_feats(utc_session_phase="US", vol_surge="HIGH")
        key, rec = _cell(cell, label="ADVERSE", n=80)
        stamp = match_entry(
            library={"entry_patterns": {key: rec}}, feats=live, gate_mode="off"
        )
        self.assertEqual(stamp["match_mode"], "NONE")

    def test_include_volume_false_skips_vol_axis(self):
        live = _short_feats(vol_surge="MED")
        # Cell without requiring vol in key (include_volume=False)
        cell_feats = dict(live)
        key = build_entry_pattern_key(cell_feats, include_volume=False)
        # Differ only on vol in live feats — key has no V component
        lib = {
            "entry_patterns": {
                key: {
                    "pattern_id": key,
                    "label": "NEUTRAL",
                    "train": {"n": 20, "wr": 0.5, "payoff": 1.0, "net_mean": 0.0},
                }
            }
        }
        stamp = match_entry(
            library=lib, feats=live, gate_mode="off", include_volume=False
        )
        self.assertEqual(stamp["match_mode"], "EXACT")
        self.assertEqual(stamp["prediction"], "ALLOW")


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
    def tearDown(self):
        os.environ.pop("RP608_CRYPTO_PATTERN_GATE", None)

    def test_default_off(self):
        os.environ.pop("RP608_CRYPTO_PATTERN_GATE", None)
        self.assertFalse(entry_gate_enabled())
        self.assertEqual(entry_gate_mode(), "off")

    def test_env_allow_only(self):
        os.environ["RP608_CRYPTO_PATTERN_GATE"] = "allow_only"
        self.assertTrue(entry_gate_enabled())
        self.assertEqual(entry_gate_mode(), "allow_only")

    def test_env_adverse_aliases(self):
        os.environ["RP608_CRYPTO_PATTERN_GATE"] = "1"
        self.assertEqual(entry_gate_mode(), "adverse")
        os.environ["RP608_CRYPTO_PATTERN_GATE"] = "adverse"
        self.assertEqual(entry_gate_mode(), "adverse")

    def test_normalize_and_blocks_matrix(self):
        self.assertEqual(normalize_entry_gate_mode("allow_only"), "allow_only")
        self.assertEqual(normalize_entry_gate_mode("2"), "allow_only")
        self.assertEqual(normalize_entry_gate_mode("strict"), "allow_only")
        self.assertEqual(normalize_entry_gate_mode("1"), "adverse")
        self.assertEqual(normalize_entry_gate_mode("garbage"), "off")
        self.assertTrue(entry_blocks_prediction("ABSTAIN_INSUFFICIENT", "allow_only"))
        self.assertTrue(entry_blocks_prediction("STANDASIDE_ADVERSE_PATTERN", "allow_only"))
        self.assertFalse(entry_blocks_prediction("ALLOW", "allow_only"))
        self.assertFalse(entry_blocks_prediction("ABSTAIN_INSUFFICIENT", "adverse"))
        self.assertTrue(entry_blocks_prediction("STANDASIDE_ADVERSE_PATTERN", "adverse"))
        self.assertFalse(entry_blocks_prediction("STANDASIDE_ADVERSE_PATTERN", "off"))


if __name__ == "__main__":
    unittest.main()
