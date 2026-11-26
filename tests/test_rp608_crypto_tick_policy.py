"""
tests/test_rp608_crypto_tick_policy.py

Determinism + refractory tests for RP-608-SHADOW-TICK-001 tick Policy B/C.
"""

from __future__ import annotations

import hashlib
import json
import unittest
from pathlib import Path
import sys

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE_DIR))

from scripts.rp608_crypto_tick_policy import (
    MIN_REFRACTORY_MS,
    PROTOCOL_ID,
    TickEmissionGate,
    TickFeatureBuffer,
    TickQuote,
    build_tick_snapshot,
    evaluate_tick_policies,
    snapshot_to_dict,
    tick_features_to_evidence,
    validate_tick_quote,
)


POLICY_B_PATH = WORKSPACE_DIR / "datasets" / "crypto_tick" / "frozen_policy_b_tick_champion.json"
POLICY_C_PATH = WORKSPACE_DIR / "datasets" / "crypto_tick" / "frozen_policy_c_tick_champion.json"
LOCKED_B = "acf17394caf402e204bd297510c26b67dcc8480ac7ad13130eb25343ccca314a"
LOCKED_C = "be909f57201c3dd67591ec01ace6bbcec607f08b70ba35ca1804a4f7e34c08ff"


def _sha256(path: Path) -> str:
    h = hashlib.sha256()
    h.update(path.read_bytes())
    return h.hexdigest()


def _quote(
    u: int,
    t_ms: int,
    bid: float,
    ask: float,
    bq: float = 1.0,
    aq: float = 1.0,
    recv_skew: int = 10,
) -> TickQuote:
    return TickQuote(
        update_id=u,
        event_time_ms=t_ms,
        receive_time_ms=t_ms + recv_skew,
        bid_price=bid,
        bid_qty=bq,
        ask_price=ask,
        ask_qty=aq,
    )


class TestRp608CryptoTickPolicy(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.assertTrue = unittest.TestCase.assertTrue
        assert POLICY_B_PATH.exists(), "missing Policy B tick champion"
        assert POLICY_C_PATH.exists(), "missing Policy C tick champion"
        with open(POLICY_B_PATH, encoding="utf-8") as f:
            cls.policy_b = json.load(f)
        with open(POLICY_C_PATH, encoding="utf-8") as f:
            cls.policy_c = json.load(f)

    def test_01_champion_sha_locks(self):
        self.assertEqual(_sha256(POLICY_B_PATH), LOCKED_B)
        self.assertEqual(_sha256(POLICY_C_PATH), LOCKED_C)
        self.assertEqual(self.policy_b.get("protocol_id"), PROTOCOL_ID)
        self.assertEqual(self.policy_c.get("protocol_id"), PROTOCOL_ID)
        self.assertEqual(self.policy_b.get("alphabet"), "TickBase")
        self.assertEqual(self.policy_c.get("alphabet"), "TickWithExhaustion")

    def test_02_validate_rejects_stale_and_inverted(self):
        q = _quote(1, 1_000_000, 100.0, 100.01, recv_skew=600)
        ok, reason = validate_tick_quote(q)
        self.assertFalse(ok)
        self.assertEqual(reason, "STALE_QUOTE")

        q2 = _quote(2, 1_000_000, 100.2, 100.1)
        ok2, reason2 = validate_tick_quote(q2)
        self.assertFalse(ok2)
        self.assertEqual(reason2, "INVERTED_BOOK")

        q3 = _quote(3, 1_000_000, 100.0, 100.01)
        ok3, reason3 = validate_tick_quote(q3, prev_update_id=5)
        self.assertFalse(ok3)
        self.assertEqual(reason3, "OUT_OF_ORDER")

    def test_03_same_quote_sequence_same_actions(self):
        buf_a = TickFeatureBuffer()
        buf_b = TickFeatureBuffer()
        closes = [100.0 + i * 0.05 for i in range(70)]  # mild uptrend macro
        t0 = 1_700_000_000_000
        quotes = []
        mid = 100.0
        for i in range(40):
            # Build ~6s of history with rising mid + bid-heavy book
            mid = 100.0 + i * 0.02
            bid = mid - 0.05
            ask = mid + 0.05
            q = _quote(i + 1, t0 + i * 200, bid, ask, bq=3.0, aq=1.0)
            quotes.append(q)

        actions_a = []
        actions_b = []
        for q in quotes:
            buf_a.append(q)
            buf_b.append(q)
            snap_a = build_tick_snapshot(q, buf_a, closes)
            snap_b = build_tick_snapshot(q, buf_b, closes)
            ab, ac, _, _ = evaluate_tick_policies(
                snap_a, self.policy_b["champion_genome"], self.policy_c["champion_genome"]
            )
            bb, bc, _, _ = evaluate_tick_policies(
                snap_b, self.policy_b["champion_genome"], self.policy_c["champion_genome"]
            )
            actions_a.append((ab, ac, snapshot_to_dict(snap_a)))
            actions_b.append((bb, bc, snapshot_to_dict(snap_b)))

        self.assertEqual(actions_a, actions_b)

    def test_04_emission_gate_refractory_and_unchanged(self):
        gate = TickEmissionGate(refractory_ms=MIN_REFRACTORY_MS)
        ok1, reason1 = gate.should_emit(1000, "LONG", "NO_TRADE")
        self.assertTrue(ok1)
        self.assertEqual(reason1, "OK")
        gate.mark_emitted(1000, "LONG", "NO_TRADE")

        ok2, reason2 = gate.should_emit(1200, "LONG", "NO_TRADE")
        self.assertFalse(ok2)
        self.assertEqual(reason2, "UNCHANGED")

        ok3, reason3 = gate.should_emit(1500, "SHORT", "NO_TRADE")
        self.assertFalse(ok3)
        self.assertEqual(reason3, "REFRACTORY")

        ok4, reason4 = gate.should_emit(2100, "SHORT", "NO_TRADE")
        self.assertTrue(ok4)
        self.assertEqual(reason4, "OK")

    def test_05_factors_include_micro_confirm_for_tickbase(self):
        buf = TickFeatureBuffer()
        t0 = 1_700_000_000_000
        for i in range(30):
            mid = 100.0 + i * 0.05
            q = _quote(i + 1, t0 + i * 250, mid - 0.05, mid + 0.05, bq=5.0, aq=1.0)
            buf.append(q)
        snap = build_tick_snapshot(q, buf, [90.0] * 70)
        factors = tick_features_to_evidence(snap, "TickBase")
        names = {f["concept"] for f in factors}
        self.assertIn("Trend", names)
        self.assertIn("Momentum", names)
        self.assertIn("Volatility", names)
        self.assertIn("MicroConfirm", names)
        factors_c = tick_features_to_evidence(snap, "TickWithExhaustion")
        names_c = {f["concept"] for f in factors_c}
        self.assertIn("PriceRangeStretch", names_c)
        self.assertIn("VWAPStretch", names_c)
        self.assertIn("VolumeBurst", names_c)


if __name__ == "__main__":
    unittest.main()
