"""OFX aggressor tempo — staggered vs clubbed (research-only)."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from research.rp608_ofx.executed_flow.volume_mark import (
    OFX_CLUBBED_MAX_OVERSHOOT,
    OFX_TEMPO_CLUBBED,
    OFX_TEMPO_MIXED,
    OFX_TEMPO_STAGGERED,
    classify_ofx_aggressor_tempo,
    is_ofx_clubbed_opposing_completion,
    ofx_entry_qty_for_tempo,
    should_suppress_ofx_entry_for_tempo,
)


def _ticks(side_qtys):
    """Build fake trade rows: list of (agg, qty)."""
    rows = []
    ts = 1_700_000_000_000
    for i, (agg, qty) in enumerate(side_qtys):
        rows.append(
            {
                "timestamp_ms": ts + i * 1000,
                "quantity": qty,
                "aggressor": agg,
            }
        )
    return rows


class TestOfxTempo(unittest.TestCase):
    def test_clubbed_sell(self):
        # one dominant sell among sells
        trades = _ticks(
            [("BUY", 0.01)] * 5
            + [("SELL", 0.02), ("SELL", 0.02), ("SELL", 1.0), ("SELL", 0.02)]
        )
        t = classify_ofx_aggressor_tempo(trades, aggressor_side="SELL", lookback=20, min_side_ticks=4)
        self.assertEqual(t["tempo"], OFX_TEMPO_CLUBBED)
        self.assertGreaterEqual(t["top1_share"], 0.45)

    def test_staggered_buy(self):
        trades = _ticks([("BUY", 0.05)] * 10 + [("SELL", 0.2)])
        t = classify_ofx_aggressor_tempo(trades, aggressor_side="BUY", lookback=20, min_side_ticks=4)
        self.assertEqual(t["tempo"], OFX_TEMPO_STAGGERED)
        self.assertLessEqual(t["top1_share"], 0.22)

    def test_staggered_entry_qty_uses_side_sum(self):
        tempo = {"tempo": OFX_TEMPO_STAGGERED, "side_qty_sum": 0.55}
        q = ofx_entry_qty_for_tempo(surge_entry_volume=0.08, entry_side_tempo=tempo)
        self.assertEqual(q["entry_qty_source"], "staggered_same_side_sum")
        self.assertAlmostEqual(q["entry_aggressor_qty"], 0.55, places=5)

    def test_clubbed_entry_qty_uses_surge(self):
        tempo = {"tempo": OFX_TEMPO_CLUBBED, "side_qty_sum": 1.2}
        q = ofx_entry_qty_for_tempo(surge_entry_volume=0.9, entry_side_tempo=tempo)
        self.assertEqual(q["entry_qty_source"], "surge_tick")
        self.assertAlmostEqual(q["entry_aggressor_qty"], 0.9, places=5)

    def test_suppress_when_clubbed_opposite_dominates(self):
        g = should_suppress_ofx_entry_for_tempo(
            direction="LONG",
            entry_side_tempo={"tempo": OFX_TEMPO_STAGGERED, "side_qty_sum": 0.2},
            opposite_side_tempo={"tempo": OFX_TEMPO_CLUBBED, "side_qty_sum": 2.0},
        )
        self.assertTrue(g["suppress"])

    def test_suppress_mixed_entry_tempo(self):
        g = should_suppress_ofx_entry_for_tempo(
            direction="SHORT",
            entry_side_tempo={"tempo": OFX_TEMPO_MIXED, "side_qty_sum": 1.0},
            opposite_side_tempo={"tempo": OFX_TEMPO_STAGGERED, "side_qty_sum": 0.3},
        )
        self.assertTrue(g["suppress"])
        self.assertEqual(g["reason"], "mixed_entry_tempo_suppressed")

    def test_clubbed_completion_exit_on_modest_overshoot(self):
        # entry 1.0, opp 1.40 → +40% overshoot inside default 50% cap
        c = is_ofx_clubbed_opposing_completion(
            entry_aggressor_qty=1.0,
            opposite_aggressor_qty=1.40,
            opposite_tempo=OFX_TEMPO_CLUBBED,
            match_dev_max=0.10,
            max_overshoot=OFX_CLUBBED_MAX_OVERSHOOT,
        )
        self.assertTrue(c["exit"])
        self.assertEqual(c["reason"], "CLUBBED_OPPOSING_COMPLETION")
        self.assertAlmostEqual(c["completion_ceiling_qty"], 1.5, places=5)

    def test_clubbed_completion_rejects_extreme_overshoot(self):
        # entry 0.02, opp 1.5 ≈ 74× — beyond 50% cap (ceiling 0.03)
        c = is_ofx_clubbed_opposing_completion(
            entry_aggressor_qty=0.02,
            opposite_aggressor_qty=1.5,
            opposite_tempo=OFX_TEMPO_CLUBBED,
            match_dev_max=0.10,
            max_overshoot=OFX_CLUBBED_MAX_OVERSHOOT,
        )
        self.assertFalse(c["exit"])
        self.assertEqual(c["reason"], "clubbed_opposite_overshoot_exceeds_cap")

    def test_clubbed_completion_requires_clubbed_tempo(self):
        c = is_ofx_clubbed_opposing_completion(
            entry_aggressor_qty=1.0,
            opposite_aggressor_qty=1.40,
            opposite_tempo=OFX_TEMPO_STAGGERED,
            match_dev_max=0.10,
        )
        self.assertFalse(c["exit"])


if __name__ == "__main__":
    unittest.main()
